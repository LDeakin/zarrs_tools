use std::{
    sync::{Arc, Mutex},
    time::SystemTime,
};

use clap::Parser;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use zarrs::{
    array::codec::CodecOptionsBuilder, array_subset::ArraySubset, filesystem::FilesystemStore,
    storage::ReadableStorage,
};
use zarrs_tools::calculate_chunk_and_codec_concurrency;

/// Benchmark zarrs read throughput with the sync API.
#[derive(Parser, Debug)]
#[command(author, version=zarrs_tools::ZARRS_TOOLS_VERSION_WITH_ZARRS)]
struct Args {
    /// The path or URL of a zarr array.
    path: String,

    /// Number of concurrent chunks.
    #[arg(long)]
    concurrent_chunks: Option<usize>,

    /// Read the entire array in one operation.
    ///
    /// If set, `concurrent_chunks` is ignored.
    #[arg(long, default_value_t = false)]
    read_all: bool,

    /// Ignore checksums.
    ///
    /// If set, checksum validation in codecs (e.g. crc32c) is skipped.
    #[arg(long, default_value_t = false)]
    ignore_checksums: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // opendal
    // let mut builder = opendal::services::Fs::default();
    // builder.root(&args.path);
    // let operator = opendal::Operator::new(builder)?.finish().blocking();
    // let storage: ReadableStorage = Arc::new(store::OpendalStore::new(operator));

    // Default filesystem store
    let storage: ReadableStorage = Arc::new(FilesystemStore::new(args.path.clone())?);

    let array = zarrs::array::Array::open(storage.clone(), "/")?;
    // println!("{:#?}", array.metadata());

    zarrs::config::global_config_mut().set_validate_checksums(!args.ignore_checksums);

    let chunks = ArraySubset::new_with_shape(array.chunk_grid_shape().unwrap());

    let start = SystemTime::now();
    let bytes_decoded = Mutex::new(0);
    if args.read_all {
        *bytes_decoded.lock().unwrap() += array.retrieve_array_subset(&array.subset_all())?.size();
    } else {
        let chunk_representation =
            array.chunk_array_representation(&vec![0; array.chunk_grid().dimensionality()])?;
        let concurrent_target = std::thread::available_parallelism().unwrap().get();
        let (chunks_concurrent_limit, codec_concurrent_target) =
            calculate_chunk_and_codec_concurrency(
                concurrent_target,
                args.concurrent_chunks,
                array.codecs(),
                chunks.num_elements_usize(),
                &chunk_representation,
            );
        let codec_options = CodecOptionsBuilder::new()
            .concurrent_target(codec_concurrent_target)
            .build();

        // println!("chunks_concurrent_limit {chunks_concurrent_limit:?} codec_concurrent_target {codec_concurrent_target:?}");
        let n_chunks = usize::try_from(chunks.shape().iter().product::<u64>()).unwrap();
        // NOTE: Could init memory per split with for_each_init and then reuse it with retrieve_chunk_into_array_view_opt.
        //       But that might be cheating against tensorstore.
        rayon_iter_concurrent_limit::iter_concurrent_limit!(
            chunks_concurrent_limit,
            0..n_chunks,
            for_each,
            |chunk_index: usize| {
                let chunk_indices = zarrs::array::unravel_index(chunk_index as u64, chunks.shape());
                // println!("Chunk/shard: {:?}", chunk_indices);
                let bytes = array
                    .retrieve_chunk_opt(&chunk_indices, &codec_options)
                    .unwrap();
                *bytes_decoded.lock().unwrap() += bytes.size();
            }
        );
    }
    let bytes_decoded = bytes_decoded.into_inner()?;
    let duration = SystemTime::now().duration_since(start)?.as_secs_f32();
    println!(
        "Decoded {} in {:.2}ms ({:.2}MB decoded @ {:.2}GB/s)",
        args.path,
        duration * 1e3,
        bytes_decoded as f32 / 1e6,
        (/* GB */bytes_decoded as f32 * 1e-9) / duration,
    );
    Ok(())
}
