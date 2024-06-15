// TODO: Use an io_uring filesystem store
// https://github.com/apache/opendal/issues/4520

use std::{cell::UnsafeCell, sync::Arc, time::SystemTime};

use clap::Parser;
use futures::{FutureExt, StreamExt};
use zarrs::{
    array::{
        codec::{ArrayCodecTraits, CodecOptionsBuilder},
        concurrency::RecommendedConcurrency,
    },
    array_subset::ArraySubset,
    config::global_config,
    storage::{store::AsyncOpendalStore, AsyncReadableStorageTraits},
};

/// Benchmark zarrs read throughput with the async API.
#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    /// The zarr array directory.
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

pub struct TokioSpawner {
    handle: tokio::runtime::Handle,
}

impl TokioSpawner {
    #[must_use]
    pub fn new(handle: tokio::runtime::Handle) -> Self {
        Self { handle }
    }
}

impl futures::task::Spawn for TokioSpawner {
    fn spawn_obj(
        &self,
        future: futures::task::FutureObj<'static, ()>,
    ) -> Result<(), futures::task::SpawnError> {
        self.handle.spawn(future);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let spawner = TokioSpawner::new(tokio::runtime::Handle::current());

    zarrs::config::global_config_mut().set_validate_checksums(!args.ignore_checksums);

    let mut builder = opendal::services::Fs::default();
    builder.root(&args.path);
    let operator = opendal::Operator::new(builder)?.finish();
    let storage = Arc::new(AsyncOpendalStore::new(operator));
    let array = Arc::new(zarrs::array::Array::async_new(storage.clone(), "/").await?);
    // println!("{:#?}", array.metadata());

    let chunks = ArraySubset::new_with_shape(array.chunk_grid_shape().unwrap());

    let start = SystemTime::now();
    let mut bytes_decoded = 0;
    let chunk_indices = chunks.indices().into_iter().collect::<Vec<_>>();
    if args.read_all {
        let array_shape = array.shape().to_vec();
        let array_subset = ArraySubset::new_with_shape(array_shape.to_vec());
        let array_data = array
            .async_retrieve_array_subset(&spawner, &array_subset)
            .await?;
        bytes_decoded += array_data.len();
    } else {
        // Calculate chunk/codec concurrency
        let chunk_representation =
            array.chunk_array_representation(&vec![0; array.chunk_grid().dimensionality()])?;
        let concurrent_target = std::thread::available_parallelism().unwrap().get();
        let (chunk_concurrent_limit, codec_concurrent_target) =
            zarrs::array::concurrency::calc_concurrency_outer_inner(
                concurrent_target,
                &if let Some(concurrent_chunks) = args.concurrent_chunks {
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
                    .recommended_concurrency(&chunk_representation)?,
            );
        let codec_options = CodecOptionsBuilder::new()
            .concurrent_target(codec_concurrent_target)
            .build();

        let futures = chunk_indices
            .into_iter()
            .map(|chunk_indices| {
                // println!("Chunk/shard: {:?}", chunk_indices);
                let array = array.clone();
                let codec_options = codec_options.clone();
                async move {
                    array
                        .async_retrieve_chunk_opt(&chunk_indices, &codec_options)
                        .map(|bytes| bytes.map(|bytes| bytes.len()))
                        .await
                }
            })
            .map(tokio::task::spawn);
        let mut stream = futures::stream::iter(futures).buffer_unordered(chunk_concurrent_limit);
        while let Some(item) = stream.next().await {
            bytes_decoded += item.unwrap()?;
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
