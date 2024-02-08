use std::{sync::Arc, time::SystemTime};

use clap::Parser;
use futures::{FutureExt, StreamExt};
use zarrs::{
    array_subset::ArraySubset, storage::store::AsyncObjectStore,
    storage::AsyncReadableStorageTraits,
};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Benchmark zarrs read throughput with the async API."
)]
struct Args {
    /// The zarr array directory.
    path: String,

    /// Number of concurrent chunks.
    #[arg(long, default_value_t = 4)]
    concurrent_chunks: usize,

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    zarrs::config::global_config_mut().set_validate_checksums(!args.ignore_checksums);

    let storage = Arc::new(AsyncObjectStore::new(
        object_store::local::LocalFileSystem::new_with_prefix(args.path.clone())?,
    ));
    let array = zarrs::array::Array::async_new(storage.clone(), "/").await?;
    println!("{:#?}", array.metadata());

    let chunks = ArraySubset::new_with_shape(array.chunk_grid_shape().unwrap());
    let chunks_shape = chunks.shape();

    let start = SystemTime::now();
    let mut bytes_decoded = 0;
    let chunk_indices = (0..chunks.shape().iter().product())
        .map(|chunk_index| zarrs::array::unravel_index(chunk_index, chunks_shape))
        .collect::<Vec<_>>();
    if args.read_all {
        let subset = ArraySubset::new_with_shape(array.shape().to_vec());
        bytes_decoded += array.async_retrieve_array_subset(&subset).await?.len();
    } else {
        let futures = chunk_indices.iter().map(|chunk_indices| {
            // println!("Chunk/shard: {:?}", chunk_indices);
            array
                .async_retrieve_chunk(chunk_indices)
                .map(|bytes| bytes.map(|bytes| bytes.len()))
        });
        let stream = futures::stream::iter(futures).buffer_unordered(args.concurrent_chunks);
        let results = stream.collect::<Vec<_>>().await;
        for result in results {
            bytes_decoded += result?;
        }
    }
    let duration = SystemTime::now().duration_since(start)?.as_secs_f32();
    println!(
        "Decoded {} ({:.2}MB) in {:.2}ms ({:.2}MB decoded @ {:.2}GB/s)",
        args.path,
        storage.size().await? as f32 / 1e6,
        duration * 1e3,
        bytes_decoded as f32 / 1e6,
        (/* GB */bytes_decoded as f32 * 1e-9) / duration,
    );
    Ok(())
}
