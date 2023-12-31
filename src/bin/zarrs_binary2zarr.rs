use clap::Parser;
use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::{iter::ParallelIterator, prelude::IntoParallelIterator};
use std::{io::Read, path::PathBuf, sync::atomic::AtomicUsize};
use zarrs_tools::{get_array_builder, ZarrEncodingArgs};

use zarrs::{
    array::{Array, DimensionName},
    array_subset::ArraySubset,
    metadata::Metadata,
    storage::store::FilesystemStore,
    storage::ReadableStorageTraits,
};

/// Convert an N-dimensional binary array from standard input to the Zarr V3 storage format.
#[derive(Parser)]
#[command(author, version)]
#[allow(rustdoc::bare_urls)]
struct Cli {
    #[command(flatten)]
    encoding: ZarrEncodingArgs,

    /// Zarr data type. See https://zarr-specs.readthedocs.io/en/latest/v3/core/v3.0.html#id11
    ///
    /// Examples:
    ///   bool
    ///   int8 int16 int32 int64
    ///   uint8 uint16 uint32 uint64
    ///   float32 float64 float16 bfloat16
    ///   complex64 complex128
    ///   r8 r16 r24 r32 r64 (r* where * is a multiple of 8)
    #[arg(short, long, verbatim_doc_comment)]
    data_type: String,

    /// Array shape. A comma separated list of the sizes of each array dimension.
    #[arg(short, long, required = true, value_delimiter = ',')]
    array_shape: Vec<u64>,

    /// Dimension names. A comma separated list of the names of each array dimension.
    #[arg(short, long, value_delimiter = ',')]
    dimension_names: Option<Vec<String>>,

    /// The output directory for the zarr array.
    out: PathBuf,
    // /// The path to a binary file or a directory of binary files.
    // #[arg(short, long, num_args = 1..)]
    // file: Vec<PathBuf>,
}

fn stdin_to_array(array: &Array<FilesystemStore>) -> usize {
    let data_type_size = array.data_type().size();
    let dimensionality = array.chunk_grid().dimensionality();
    let array_shape = array.shape();
    let array_shape_n = *array_shape.first().unwrap();
    let chunk_shape = array
        .chunk_grid()
        .chunk_shape(&vec![0; array.chunk_grid().dimensionality()], array.shape())
        .unwrap()
        .expect("lowest indices should have a chunk shape");
    let block_shape_n = *chunk_shape.first().unwrap();
    let n_blocks = (array_shape_n + block_shape_n - 1) / block_shape_n;

    let style =
        ProgressStyle::with_template("[{elapsed_precise}] [{bar}] ({pos}/{len} blocks, ETA {eta})")
            .unwrap();

    #[allow(clippy::mutex_integer)]
    let idxm = std::sync::Mutex::new(0u64);
    let bytes_read: AtomicUsize = 0.into();
    let idxs = (0..n_blocks as usize).collect::<Vec<_>>();
    idxs.into_par_iter()
        .progress_with_style(style)
        .map(|_| {
            #[allow(clippy::mutex_integer)]
            let mut idxm = idxm.lock().unwrap();
            let idx = *idxm;
            *idxm += 1;

            let start = idx * block_shape_n;
            let end = std::cmp::min((idx + 1) * block_shape_n, array_shape_n);

            let mut startn: Vec<u64> = vec![start];
            startn.resize(dimensionality, 0);
            let mut endn = vec![end];
            endn.extend(array_shape.iter().skip(1));
            let array_subset =
                unsafe { ArraySubset::new_with_start_end_exc_unchecked(startn, endn) };

            let bytes_len =
                usize::try_from(array_subset.num_elements() * data_type_size as u64).unwrap();
            let mut subset_bytes = vec![0; bytes_len];
            std::io::stdin().read_exact(&mut subset_bytes).unwrap();
            bytes_read.fetch_add(bytes_len, std::sync::atomic::Ordering::Relaxed);

            drop(idxm);

            array
                .store_array_subset(&array_subset, subset_bytes)
                .unwrap();

            // let subset_bytes_test = array.retrieve_array_subset(&array_subset).unwrap();
            // assert_eq!(subset_bytes, subset_bytes_test);
        })
        .collect::<Vec<_>>();
    bytes_read.load(std::sync::atomic::Ordering::Relaxed)
}

fn main() {
    // Parse and validate arguments
    let cli = Cli::parse();

    // Get data type
    let data_type = zarrs::array::DataType::from_metadata(&Metadata::new(&cli.data_type)).unwrap();

    // Create storage
    let path_out = cli.out.as_path();
    let store = std::sync::Arc::new(FilesystemStore::new(path_out).unwrap());

    // Create array
    let dimension_names = cli.dimension_names.map(|f| {
        f.iter()
            .map(|dimension_name| DimensionName::new(dimension_name))
            .collect()
    });
    let array_builder =
        get_array_builder(&cli.encoding, &cli.array_shape, data_type, dimension_names);
    let array = array_builder.build(store.clone(), "/").unwrap();

    // Store array metadata
    array.store_metadata().unwrap();

    // Read stdin to the array and write chunks/shards
    let start = std::time::Instant::now();
    // array.set_parallel_codecs(cli.shard_shape.is_some());
    let bytes_read: usize = stdin_to_array(&array);
    let duration_s = start.elapsed().as_secs_f32();

    // Output stats
    let duration_ms = duration_s * 1.0e3;
    let gbs = (bytes_read as f32 * 1e-9) / duration_s;
    let size_out = store.size().unwrap();
    let space_saving = 100.0 * (1.0 - (size_out as f32 / bytes_read as f32));
    println!("Output {path_out:?} in {duration_ms:.2}ms ({gbs:.2} GB/s) [{bytes_read}B -> {size_out}B ({space_saving:.2}%)]");
}
