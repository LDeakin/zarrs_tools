use clap::Parser;
use indicatif::{DecimalBytes, ProgressBar, ProgressStyle};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use zarrs_tools::{get_array_builder, ZarrEncodingArgs};

use zarrs::{
    array::{codec::CodecOptionsBuilder, Array, DataType, DimensionName},
    array_subset::ArraySubset,
    storage::{
        store::{FilesystemStore, MemoryStore},
        ReadableWritableListableStorage, ReadableWritableStorageTraits, StorePrefix,
    },
};

/// Convert a netCDF variable to a Zarr V3 array.
#[derive(Parser)]
#[command(author, version)]
struct Cli {
    #[command(flatten)]
    encoding: ZarrEncodingArgs,

    /// Validate written data.
    #[arg(long, default_value_t = false)]
    validate: bool,

    /// Sets the number of netCDF blocks processed concurrently. This parameter is currently ignored.
    #[arg(long)]
    concurrent_blocks: Option<usize>,

    // /// Array shape. A comma separated list of the sizes of each array dimension.
    // #[arg(short, long, required = true, value_delimiter = ',')]
    // array_shape: Vec<u64>,
    /// The dimension to concatenate the variable if it is spread across multiple files.
    ///
    /// Dimension 0 is the outermost (slowest varying) dimension.
    #[arg(long, default_value_t = 0)]
    concat_dim: usize,

    /// Write to memory.
    #[arg(long, default_value_t = false)]
    memory_test: bool,

    /// The path to a netCDF file or a directory of netcdf files.
    input: PathBuf,

    /// The name of the netCDF variable.
    variable: String,

    /// The output directory for the zarr array.
    out: PathBuf,
}

#[allow(clippy::too_many_arguments)]
fn ncfiles_to_array<TStore: ReadableWritableStorageTraits + ?Sized + 'static>(
    nc_paths: &[PathBuf],
    offsets: &[u64],
    variable: &str,
    concat_dim: usize,
    array: &Array<TStore>,
    // num_concurrent_blocks: Option<usize>,
    validate: bool,
) -> usize {
    let style_all =
        ProgressStyle::with_template("[{bar}] ({pos}/{len} blocks, {elapsed_precise}, ETA {eta})")
            .unwrap();

    let bytes_read: AtomicUsize = 0.into();

    let concurrent_target = std::thread::available_parallelism().unwrap().get();
    let codec_options = CodecOptionsBuilder::new()
        .concurrent_target(concurrent_target)
        .build();

    let process_path = |idx: usize, nc_path: &PathBuf| {
        // println!("{nc_path:?}");
        // println!("Read netCDF");
        let nc_file = netcdf::open(nc_path).expect("Could not open netCDF file");
        let nc_var = nc_file
            .variable(variable)
            .expect("Could not find variable in netCDF file");

        let dims = nc_var.dimensions();
        // let dim_sizes: Vec<_> = dims.iter().map(|dim| dim.len()).collect();
        let dim_sizes_u64: Vec<_> = dims.iter().map(|dim| dim.len() as u64).collect();
        // println!("{dim_sizes:?}");

        let mut start = vec![0u64; array.chunk_grid().dimensionality()];
        start[concat_dim] = offsets[idx];
        let array_subset = ArraySubset::new_with_start_shape(start, dim_sizes_u64.clone()).unwrap();
        // println!("{array_subset:?} {dim_sizes:?} {}", buf.len());
        let buf = nc_var.get_raw_values(..).unwrap();
        assert_eq!(
            buf.len(),
            array
                .data_type()
                .fixed_size()
                .expect("data type should be fixed size")
                * array_subset.num_elements_usize(),
            "Size mismatch"
        );
        // println!("Read netCDF done");
        bytes_read.fetch_add(buf.len(), Ordering::Relaxed);

        if validate {
            array
                .store_array_subset_opt(&array_subset, buf.clone(), &codec_options)
                .unwrap();
            let buf_validate = array
                .retrieve_array_subset_opt(&array_subset, &codec_options)
                .unwrap();
            assert!(buf == buf_validate.into_fixed().unwrap().into_owned());
        } else {
            array
                .store_array_subset_opt(&array_subset, buf, &codec_options)
                .unwrap();
        }
    };

    let pb = ProgressBar::new(nc_paths.len() as u64);
    pb.set_style(style_all);
    pb.set_position(0);
    for (idx, nc_path) in nc_paths.iter().enumerate() {
        process_path(idx, nc_path);
        pb.inc(1);
    }
    pb.abandon();
    bytes_read.load(Ordering::Relaxed)
}

fn get_netcdf_paths(path: &std::path::Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let mut nc_files;
    if path.is_dir() {
        // list contents of directory
        nc_files = path
            .read_dir()?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;
        nc_files.sort();
    } else {
        nc_files = vec![path.to_path_buf()]
    }
    Ok(nc_files)
}

fn nc_vartype_to_zarr_datatype(nc_vartype: netcdf::types::NcVariableType) -> Option<DataType> {
    use netcdf::types::{FloatType, IntType, NcVariableType};
    let data_type = match nc_vartype {
        NcVariableType::Char => DataType::UInt8,
        NcVariableType::Int(x) => match x {
            IntType::I8 => DataType::Int8,
            IntType::U8 => DataType::UInt8,
            IntType::I16 => DataType::Int16,
            IntType::U16 => DataType::UInt16,
            IntType::I32 => DataType::Int32,
            IntType::U32 => DataType::UInt32,
            IntType::I64 => DataType::Int64,
            IntType::U64 => DataType::UInt64,
        },
        NcVariableType::Float(x) => match x {
            FloatType::F32 => DataType::Float32,
            FloatType::F64 => DataType::Float64,
        },
        _ => return None,
    };
    Some(data_type)
}

fn main() {
    // Parse and validate arguments
    let cli = Cli::parse();
    if let Some(shard_shape) = &cli.encoding.shard_shape {
        assert_eq!(cli.encoding.chunk_shape.len(), shard_shape.len());
    }
    println!("Input {:?}", cli.input);

    let start = std::time::Instant::now();

    // Sort the files
    let nc_paths = get_netcdf_paths(&cli.input).expect("cannot retrieve netCDF filenames");
    // println!("{nc_paths:?}");

    // Inspect the variable for each netCDF file, and get the
    //  - data type
    //  - array shape taking into the concat dimension
    //  - dimension names
    let mut array_shape: Option<Vec<u64>> = None;
    let mut dimension_names: Option<Vec<String>> = None;
    let mut datatype: Option<DataType> = None;
    let mut offset: u64 = 0;
    let mut offsets = Vec::with_capacity(nc_paths.len());
    for nc_path in &nc_paths {
        let nc_file = netcdf::open(nc_path).expect("Could not open netCDF file");
        let nc_var = nc_file
            .variable(&cli.variable)
            .expect("Could not find variable in netCDF file");
        // println!("{:?} {:?}", nc_var.vartype(), nc_var.endian_value());

        let datatype_i = nc_vartype_to_zarr_datatype(nc_var.vartype())
            .expect("Unsupported netcdf variable type");
        if let Some(datatype) = &datatype {
            assert_eq!(datatype, &datatype_i)
        } else {
            datatype = Some(datatype_i);
        }

        let dims = nc_var.dimensions();
        let dim_names: Vec<_> = dims.iter().map(|dim| dim.name()).collect();
        let dim_sizes: Vec<_> = dims.iter().map(|dim| dim.len() as u64).collect();

        offsets.push(offset);
        offset += dim_sizes[cli.concat_dim];

        // println!("{dim_names:?}, {dim_sizes:?}");
        if let Some(dimension_names) = &dimension_names {
            assert_eq!(dimension_names, &dim_names);
        } else {
            dimension_names = Some(dim_names);
        }
        if let Some(array_shape) = &mut array_shape {
            // FIXME: Validate dims which aren't concatenated are the same shape
            array_shape[cli.concat_dim] += dim_sizes[cli.concat_dim];
        } else {
            array_shape = Some(dim_sizes);
        }
    }
    let array_shape = array_shape.unwrap();
    let dimension_names = dimension_names.unwrap();
    let dimension_names: Option<Vec<DimensionName>> =
        Some(dimension_names.iter().map(DimensionName::new).collect());
    let datatype = datatype.unwrap();
    // println!("Shape: {array_shape:?}");
    // println!("Datatype: {datatype}");
    // println!(
    //     "Dimension names: {:?}",
    //     dimension_names
    //         .as_ref()
    //         .unwrap()
    //         .iter()
    //         .map(|d| d.as_str().unwrap_or_default().to_string())
    //         .collect_vec()
    // );

    // Create storage
    let path_out = cli.out.as_path();
    let store: ReadableWritableListableStorage = if cli.memory_test {
        Arc::new(MemoryStore::default())
    } else {
        Arc::new(FilesystemStore::new(path_out).unwrap())
    };

    // Create array
    let array_builder = get_array_builder(&cli.encoding, &array_shape, datatype, dimension_names);
    let array = array_builder.build(store.clone(), "/").unwrap();

    // Erase existing data/metadata
    store.erase_prefix(&StorePrefix::new("").unwrap()).unwrap();

    // Store array metadata
    array.store_metadata().unwrap();

    // Read stdin to the array and write chunks/shards
    let bytes_read: usize = ncfiles_to_array(
        &nc_paths,
        &offsets,
        &cli.variable,
        cli.concat_dim,
        &array,
        cli.validate,
    );
    let duration_s = start.elapsed().as_secs_f32();

    // Output stats
    let duration_ms = duration_s * 1.0e3;
    let size_out = store.size().unwrap();
    // let space_saving = 100.0 * (1.0 - (size_out as f32 / bytes_read as f32));
    let relative_size = 100.0 * (size_out as f32 / bytes_read as f32);
    println!("Output {path_out:?} in {duration_ms:.2}ms ({gbs:.2} GB/s) [{bytes_read} -> {size_out} ({relative_size:.2}%)]",
        gbs = (bytes_read as f32 * 1e-9) / duration_s,
        bytes_read = DecimalBytes(bytes_read as u64),
        size_out = DecimalBytes(size_out as u64),
    );
}
