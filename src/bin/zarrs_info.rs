use std::{error::Error, sync::Arc};

use clap::{Parser, Subcommand};
use rayon::current_num_threads;
use serde::Serialize;
use serde_json::Number;
use zarrs::{
    array::{Array, ArrayMetadataOptions, DimensionName, FillValueMetadata},
    metadata::Metadata,
    storage::store::FilesystemStore,
};

/// Get information about a Zarr V3 array as JSON.
#[derive(Parser)]
#[command(author, version)]
struct Cli {
    /// The maximum number of chunks concurrently processed.
    ///
    /// Defaults to the RAYON_NUM_THREADS environment variable or the number of logical CPUs.
    /// Consider reducing this for images with large chunk sizes or on systems with low memory availability.
    #[arg(long, default_value_t = current_num_threads())]
    chunk_limit: usize,

    #[arg(long, default_value_t = false)]
    time: bool,

    /// Path to zarr input array.
    path: std::path::PathBuf,

    #[command(subcommand)]
    command: InfoCommand,
}

#[derive(Parser)]
struct HistogramParams {
    n_bins: usize,
    min: f64,
    max: f64,
}

#[derive(Subcommand)]
enum InfoCommand {
    /// The metadata.
    Metadata,
    /// The metadata (interpreted as V3).
    MetadataV3,
    /// The array shape.
    Shape,
    /// The array data type.
    DataType,
    /// The array fill value.
    FillValue,
    /// The array attributes.
    Attributes,
    /// The dimension names.
    DimensionNames,
    /// Range.
    Range,
    /// Histogram.
    Histogram(HistogramParams),
}

fn main() -> std::process::ExitCode {
    if let Err(err) = run() {
        println!("{}", err);
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::SUCCESS
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let start = std::time::Instant::now();

    let storage = Arc::new(FilesystemStore::new(&cli.path).unwrap());
    let array = Array::open(storage, "/").unwrap();

    match cli.command {
        InfoCommand::Metadata => {
            println!(
                "{}",
                serde_json::to_string_pretty(array.metadata()).unwrap()
            );
        }
        InfoCommand::MetadataV3 => {
            let mut metadata_options = ArrayMetadataOptions::default();
            metadata_options
                .set_metadata_convert_version(zarrs::metadata::MetadataConvertVersion::V3);
            metadata_options.set_include_zarrs_metadata(false);
            let metadata = array.metadata_opt(&metadata_options);
            println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
        }
        InfoCommand::Shape => {
            #[derive(Serialize)]
            struct Shape {
                shape: Vec<u64>,
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&Shape {
                    shape: array.shape().to_vec()
                })
                .unwrap()
            );
        }
        InfoCommand::DataType => {
            #[derive(Serialize)]
            struct DataType {
                data_type: Metadata,
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&DataType {
                    data_type: array.data_type().metadata()
                })
                .unwrap()
            );
        }
        InfoCommand::FillValue => {
            #[derive(Serialize)]
            struct FillValue {
                fill_value: FillValueMetadata,
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&FillValue {
                    fill_value: array.data_type().metadata_fill_value(array.fill_value())
                })
                .unwrap()
            );
        }
        InfoCommand::DimensionNames => {
            #[derive(Serialize)]
            struct DimensionNames {
                dimension_names: Option<Vec<DimensionName>>,
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&DimensionNames {
                    dimension_names: array.dimension_names().clone()
                })
                .unwrap()
            );
        }
        InfoCommand::Attributes => {
            #[derive(Serialize)]
            struct Attributes {
                attributes: serde_json::Map<String, serde_json::Value>,
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&Attributes {
                    attributes: array.attributes().clone()
                })
                .unwrap()
            );
        }
        InfoCommand::Range => {
            let (min, max) = zarrs_tools::info::calculate_range(&array, cli.chunk_limit)?;
            #[derive(Serialize)]
            struct MinMax {
                min: Number,
                max: Number,
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&MinMax { min, max }).unwrap()
            );
        }
        InfoCommand::Histogram(histogram_params) => {
            let (bin_edges, hist) = zarrs_tools::info::calculate_histogram(
                &array,
                histogram_params.n_bins,
                histogram_params.min,
                histogram_params.max,
                cli.chunk_limit,
            )?;
            #[derive(Serialize)]
            struct Histogram {
                bin_edges: Vec<f64>,
                hist: Vec<u64>,
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&Histogram { bin_edges, hist }).unwrap()
            );
        }
    }

    if cli.time {
        let duration_s = start.elapsed().as_secs_f32();
        eprintln!("Completed in {duration_s:.2}s");
    }

    Ok(())
}
