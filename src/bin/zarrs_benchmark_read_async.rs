use std::{sync::Arc, time::SystemTime};

use clap::Parser;
use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
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
    let mut futures = chunk_indices
        .iter()
        .map(|chunk_indices| {
            println!("Chunk/shard: {:?}", chunk_indices);
            array
                .async_retrieve_chunk(chunk_indices)
                .map(|bytes| bytes.map(|bytes| bytes.len()))
        })
        .collect::<FuturesUnordered<_>>();
    while let Some(item) = futures.next().await {
        bytes_decoded += item?;
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
