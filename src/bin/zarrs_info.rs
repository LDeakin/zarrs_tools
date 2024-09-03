use std::{error::Error, sync::Arc};

use clap::{Parser, Subcommand};
use rayon::current_num_threads;
use serde::Serialize;
use serde_json::Number;
use zarrs::{
    array::{Array, ArrayMetadataOptions, DimensionName, FillValueMetadataV3},
    group::{Group, GroupMetadataOptions},
    metadata::v3::array::data_type::DataTypeMetadataV3,
    node::{Node, NodeMetadata},
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

#[derive(Parser, Debug)]
struct HistogramParams {
    n_bins: usize,
    min: f64,
    max: f64,
}

#[derive(Subcommand, Debug)]
enum InfoCommand {
    /// The array/group metadata.
    Metadata,
    /// The array/group metadata (interpreted as V3).
    MetadataV3,
    /// The array/group attributes.
    Attributes,
    /// The array shape.
    Shape,
    /// The array data type.
    DataType,
    /// The array fill value.
    FillValue,
    /// The array dimension names.
    DimensionNames,
    /// The array range.
    Range,
    /// The array histogram.
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

fn group_metadata_options_v3() -> GroupMetadataOptions {
    let mut metadata_options = GroupMetadataOptions::default();
    metadata_options.set_metadata_convert_version(zarrs::config::MetadataConvertVersion::V3);
    metadata_options
}

fn array_metadata_options_v3() -> ArrayMetadataOptions {
    let mut metadata_options = ArrayMetadataOptions::default();
    metadata_options.set_metadata_convert_version(zarrs::config::MetadataConvertVersion::V3);
    metadata_options.set_include_zarrs_metadata(false);
    metadata_options
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let start = std::time::Instant::now();

    let storage = Arc::new(FilesystemStore::new(&cli.path)?);

    let node = Node::open(&storage, "/")?;
    if let NodeMetadata::Group(_) = node.metadata() {
        // Group handling
        let group = Group::open(storage.clone(), "/")?;
        match cli.command {
            InfoCommand::Metadata => {
                println!("{}", serde_json::to_string_pretty(group.metadata())?);
            }
            InfoCommand::MetadataV3 => {
                let metadata = group.metadata_opt(&group_metadata_options_v3());
                println!("{}", serde_json::to_string_pretty(&metadata)?);
            }
            InfoCommand::Attributes => {
                println!("{}", serde_json::to_string_pretty(group.attributes())?);
            }
            _ => {
                println!("The {:?} command is not supported for a group", cli.command)
            }
        }
    } else {
        // Array handling
        let array = Array::open(storage.clone(), "/")?;
        match cli.command {
            InfoCommand::Metadata => {
                println!("{}", serde_json::to_string_pretty(array.metadata())?);
            }
            InfoCommand::MetadataV3 => {
                let metadata = array.metadata_opt(&array_metadata_options_v3());
                println!("{}", serde_json::to_string_pretty(&metadata)?);
            }
            InfoCommand::Attributes => {
                println!("{}", serde_json::to_string_pretty(array.attributes())?);
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
                    })?
                );
            }
            InfoCommand::DataType => {
                #[derive(Serialize)]
                struct DataType {
                    data_type: DataTypeMetadataV3,
                }
                println!(
                    "{}",
                    serde_json::to_string_pretty(&DataType {
                        data_type: array.data_type().metadata()
                    })?
                );
            }
            InfoCommand::FillValue => {
                #[derive(Serialize)]
                struct FillValue {
                    fill_value: FillValueMetadataV3,
                }
                println!(
                    "{}",
                    serde_json::to_string_pretty(&FillValue {
                        fill_value: array.data_type().metadata_fill_value(array.fill_value())
                    })?
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
                    })?
                );
            }
            InfoCommand::Range => {
                let (min, max) = zarrs_tools::info::calculate_range(&array, cli.chunk_limit)?;
                #[derive(Serialize)]
                struct MinMax {
                    min: Number,
                    max: Number,
                }
                println!("{}", serde_json::to_string_pretty(&MinMax { min, max })?);
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
                    serde_json::to_string_pretty(&Histogram { bin_edges, hist })?
                );
            }
        }
    }

    if cli.time {
        let duration_s = start.elapsed().as_secs_f32();
        eprintln!("Completed in {duration_s:.2}s");
    }

    Ok(())
}
