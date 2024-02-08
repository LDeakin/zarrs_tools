use std::{sync::Arc, sync::Mutex, time::SystemTime};

use clap::Parser;
use rayon::iter::{ParallelBridge, ParallelIterator};
use zarrs::{array_subset::ArraySubset, storage::ReadableStorageTraits};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Benchmark zarrs read throughput with the sync API."
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let storage = Arc::new(zarrs::storage::store::FilesystemStore::new(
        args.path.clone(),
    )?);
    let array = zarrs::array::Array::new(storage.clone(), "/")?;
    println!("{:#?}", array.metadata());

    zarrs::config::global_config_mut().set_validate_checksums(!args.ignore_checksums);

    let chunks = ArraySubset::new_with_shape(array.chunk_grid_shape().unwrap());

    let start = SystemTime::now();
    let bytes_decoded = Mutex::new(0);
    if args.read_all {
        let subset = ArraySubset::new_with_shape(array.shape().to_vec());
        *bytes_decoded.lock().unwrap() += array.par_retrieve_array_subset(&subset)?.len();
    } else {
        (0..chunks.shape().iter().product())
            .collect::<Vec<_>>()
            .as_slice()
            .chunks((chunks.num_elements_usize() + args.concurrent_chunks - 1) / args.concurrent_chunks)
            .par_bridge()
            .for_each(|chunk_index_chunk| {
                for chunk_index in chunk_index_chunk {
                    let chunk_indices = zarrs::array::unravel_index(*chunk_index, chunks.shape());
                    // println!("Chunk/shard: {:?}", chunk_indices);
                    let bytes = array.retrieve_chunk(&chunk_indices).unwrap();
                    *bytes_decoded.lock().unwrap() += bytes.len();
                }
            });
    }
    let bytes_decoded = bytes_decoded.into_inner()?;
    let duration = SystemTime::now().duration_since(start)?.as_secs_f32();
    println!(
        "Decoded {} ({:.2}MB) in {:.2}ms ({:.2}MB decoded @ {:.2}GB/s)",
        args.path,
        storage.size()? as f32 / 1e6,
        duration * 1e3,
        bytes_decoded as f32 / 1e6,
        (/* GB */bytes_decoded as f32 * 1e-9) / duration,
    );
    Ok(())
}
