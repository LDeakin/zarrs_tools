use clap::Parser;
use indicatif::{DecimalBytes, ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon_iter_concurrent_limit::iter_concurrent_limit;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use zarrs_tools::{
    get_array_builder,
    progress::{Progress, ProgressCallback, ProgressStats},
    ZarrEncodingArgs,
};

use zarrs::{
    array::{
        codec::CodecOptionsBuilder, Array, ArrayCodecTraits, DataType, DimensionName,
        RecommendedConcurrency,
    },
    array_subset::ArraySubset,
    config::global_config,
    filesystem::FilesystemStore,
    storage::{
        store::MemoryStore, ReadableWritableListableStorage, ReadableWritableStorageTraits,
        StorePrefix,
    },
};

/// Convert a netCDF variable to a Zarr V3 array.
#[derive(Parser)]
#[command(author, version)]
struct Cli {
    #[command(flatten)]
    encoding: ZarrEncodingArgs,

    /// Number of concurrent chunks.
    #[arg(long)]
    concurrent_chunks: Option<usize>,

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

fn progress_callback(stats: ProgressStats, bar: &ProgressBar) {
    bar.set_length(stats.num_steps as u64);
    bar.set_position(stats.step as u64);
    if stats.process_steps.is_empty() {
        bar.set_message(format!(
            "rw:{:.2}/{:.2} p:{:.2}",
            stats.read.as_secs_f32(),
            stats.write.as_secs_f32(),
            stats.process.as_secs_f32(),
        ));
    } else {
        bar.set_message(format!(
            "rw:{:.2}/{:.2} p:{:.2} {:.2?}",
            stats.read.as_secs_f32(),
            stats.write.as_secs_f32(),
            stats.process.as_secs_f32(),
            stats
                .process_steps
                .iter()
                .map(|t| t.as_secs_f32())
                .collect::<Vec<_>>(),
        ));
    }
}

fn bar_style_run() -> ProgressStyle {
    ProgressStyle::with_template(
        "[{elapsed_precise}/{duration_precise}] {bar:40.black/bold} {pos}/{len} ({percent}%) {prefix} {msg}",
    )
    .unwrap_or(ProgressStyle::default_bar())
}

fn bar_style_finish() -> ProgressStyle {
    ProgressStyle::with_template("[{elapsed_precise}/{elapsed_precise}] {prefix} {msg}")
        .unwrap_or(ProgressStyle::default_bar())
}

#[allow(clippy::too_many_arguments)]
fn ncfiles_to_array<TStore: ReadableWritableStorageTraits + ?Sized + 'static>(
    nc_paths: &[PathBuf],
    offsets: &[u64],
    variable: &str,
    array: &Array<TStore>,
    concurrent_chunks: Option<usize>,
    progress_callback: &ProgressCallback,
) -> anyhow::Result<usize> {
    let bytes_read: AtomicUsize = 0.into();

    let chunk_representation = array
        .chunk_array_representation(&vec![0; array.chunk_grid().dimensionality()])
        .unwrap();
    let chunks = ArraySubset::new_with_shape(array.chunk_grid_shape().unwrap());

    let concurrent_target = std::thread::available_parallelism().unwrap().get();
    let (chunks_concurrent_limit, codec_concurrent_target) =
        zarrs::array::concurrency::calc_concurrency_outer_inner(
            concurrent_target,
            &if let Some(concurrent_chunks) = concurrent_chunks {
                let concurrent_chunks =
                    std::cmp::min(chunks.num_elements_usize(), concurrent_chunks);
                RecommendedConcurrency::new(concurrent_chunks..concurrent_chunks)
            } else {
                let concurrent_chunks = std::cmp::min(
                    chunks.num_elements_usize(),
                    global_config().chunk_concurrent_minimum(),
                );
                RecommendedConcurrency::new_minimum(concurrent_chunks)
            },
            &array
                .codecs()
                .recommended_concurrency(&chunk_representation)
                .unwrap(),
        );
    let codec_options = CodecOptionsBuilder::new()
        .concurrent_target(codec_concurrent_target)
        .build();

    let progress = Progress::new(chunks.num_elements_usize(), progress_callback);
    let write_chunk = |chunk_indices: Vec<u64>| -> anyhow::Result<()> {
        let chunk_subset = array.chunk_subset_bounded(&chunk_indices).unwrap();
        let bytes: Vec<u8> = progress.read(|| {
            // Get all netCDF blocks intersecting the chunk subset
            let nc_idx0 = offsets
                .iter()
                .rposition(|&offset| offset <= chunk_subset.start()[0])
                .expect("No valid offset found");
            let nc_idx1_exc = offsets
                .iter()
                .position(|&offset| offset >= chunk_subset.end_exc()[0])
                .unwrap_or(nc_paths.len());

            // Read them and concatenate them into a single buffer
            let bytes: Vec<u8> = (nc_idx0..nc_idx1_exc)
                .map(|nc_idx: usize| {
                    // Open the NetCDF file
                    let nc_path = &nc_paths[nc_idx];
                    let nc_file = netcdf::open(nc_path).expect("Could not open netCDF file");
                    let nc_var = nc_file
                        .variable(variable)
                        .expect("Could not find variable in netCDF file");
                    let dims = nc_var.dimensions();
                    let dim_sizes: Vec<_> = dims.iter().map(|dim| dim.len() as u64).collect();

                    // Get the overlapping region of the chunk subset with the netCDF block
                    let mut start = Vec::with_capacity(dim_sizes.len());
                    start.push(offsets[nc_idx]);
                    start.extend(vec![0u64; dim_sizes.len() - 1]);
                    let nc_subset =
                        ArraySubset::new_with_start_shape(start, dim_sizes.clone()).unwrap();
                    let nc_subset_overlap_relative = nc_subset
                        .overlap(&chunk_subset)?
                        .relative_to(nc_subset.start())?;

                    // Read the netCDF block subset
                    let extents: Vec<_> = nc_subset_overlap_relative
                        .to_ranges()
                        .iter()
                        .map(|range| {
                            usize::try_from(range.start).unwrap()
                                ..usize::try_from(range.end).unwrap()
                        })
                        .collect();
                    Ok(nc_var.get_raw_values(extents)?)
                })
                .collect::<anyhow::Result<Vec<_>>>()?
                .concat();

            anyhow::Ok(bytes)
        })?;
        bytes_read.fetch_add(bytes.len(), Ordering::Relaxed);

        progress
            .write(|| array.store_array_subset_opt(&chunk_subset, bytes.clone(), &codec_options))?;
        progress.next();
        Ok(())
    };

    let indices = chunks.indices();
    iter_concurrent_limit!(chunks_concurrent_limit, indices, try_for_each, write_chunk).unwrap();

    Ok(bytes_read.load(Ordering::Relaxed))
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

fn main() -> anyhow::Result<()> {
    // Parse and validate arguments
    let cli = Cli::parse();
    if let Some(shard_shape) = &cli.encoding.shard_shape {
        assert_eq!(cli.encoding.chunk_shape.len(), shard_shape.len());
    }
    println!("Input {:?}", cli.input);

    let bar = ProgressBar::new(0);
    bar.set_style(bar_style_run());
    let progress_callback = |stats: ProgressStats| progress_callback(stats, &bar);
    let progress_callback = ProgressCallback::new(&progress_callback);

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
    const CONCAT_DIM: usize = 0;
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
        offset += dim_sizes[CONCAT_DIM];

        // println!("{dim_names:?}, {dim_sizes:?}");
        if let Some(dimension_names) = &dimension_names {
            assert_eq!(dimension_names, &dim_names);
        } else {
            dimension_names = Some(dim_names);
        }
        if let Some(array_shape) = &mut array_shape {
            // FIXME: Validate dims which aren't concatenated are the same shape
            array_shape[CONCAT_DIM] += dim_sizes[CONCAT_DIM];
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
        &array,
        cli.concurrent_chunks,
        &progress_callback,
    )?;
    bar.set_style(bar_style_finish());
    bar.finish_and_clear();
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

    Ok(())
}
