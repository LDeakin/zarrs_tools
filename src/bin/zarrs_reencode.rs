use std::sync::Arc;

use clap::Parser;
use zarrs::storage::{ReadableStorageTraits, StorePrefix, WritableStorageTraits};
use zarrs_tools::{do_reencode, get_array_builder_reencode, ZarrReEncodingArgs};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "Reencode a zarr array.")]
struct Args {
    #[command(flatten)]
    encoding: ZarrReEncodingArgs,

    /// The zarr array input directory.
    path_in: String,

    /// The zarr array output directory. If unspecified, data is written to memory.
    path_out: String,

    /// Number of concurrent chunks.
    #[arg(long)]
    concurrent_chunks: Option<usize>,

    /// Ignore checksums.
    ///
    /// If set, checksum validation in codecs (e.g. crc32c) is skipped.
    #[arg(long, default_value_t = false)]
    ignore_checksums: bool,

    /// Validate written data.
    #[arg(long, default_value_t = false)]
    validate: bool,

    /// Print verbose information, such as the array header.
    #[arg(long, short, default_value_t = false)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    zarrs::config::global_config_mut().set_validate_checksums(!args.ignore_checksums);

    let storage_in =
        Arc::new(zarrs::storage::store::FilesystemStore::new(args.path_in.clone()).unwrap());
    let array_in = zarrs::array::Array::new(storage_in.clone(), "/").unwrap();
    if args.verbose {
        println!(
            "{}",
            serde_json::to_string_pretty(&array_in.metadata()).unwrap()
        );
    }

    let storage_out =
        Arc::new(zarrs::storage::store::FilesystemStore::new(args.path_out.clone()).unwrap());
    storage_out.erase_prefix(&StorePrefix::root()).unwrap();
    let builder = get_array_builder_reencode(&args.encoding, &array_in);
    let array_out = builder.build(storage_out.clone(), "/").unwrap();
    array_out.store_metadata().unwrap();

    let (duration, duration_read, duration_write, bytes_decoded) =
        do_reencode(&array_in, &array_out, args.validate, args.concurrent_chunks);
    let bytes_decoded_gb = /* GB */bytes_decoded as f32 * 1e-9;
    println!(
        "Reencode {} ({:2}MB) to {} ({:2}MB) in {:.2}ms\n\tread in ~{:.2}ms ({:.2}MB decoded @ {:.2}GB/s)\n\twrite in ~{:.2}ms ({:.2}MB encoded @ {:.2}GB/s)",
        args.path_in,
        storage_in.size().unwrap() as f32 / 1e6,
        args.path_out,
        storage_out.size().unwrap() as f32 / 1e6,
        duration * 1e3,
        duration_read * 1e3,
        bytes_decoded as f32 / 1e6,
        bytes_decoded_gb / duration_read,
        duration_write * 1e3,
        bytes_decoded as f32 / 1e6,
        bytes_decoded_gb / duration_write,
    );
}
