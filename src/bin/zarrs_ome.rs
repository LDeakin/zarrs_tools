use std::{
    error::Error,
    hash::Hash,
    num::NonZeroU64,
    path::{Path, PathBuf},
};

use clap::Parser;
use half::{bf16, f16};
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use num_traits::AsPrimitive;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::Serialize;
use zarrs::{
    array::{Array, ArrayCodecTraits, ChunkRepresentation},
    array_subset::ArraySubset,
    bytemuck::Pod,
    group::Group,
    storage::{store::FilesystemStore, StorePrefix, WritableStorageTraits},
};
use zarrs_tools::{
    filter::{
        filters::{downsample::Downsample, gaussian::Gaussian},
        ArraySubsetOverlap, FilterError, FilterTraits,
    },
    progress::{Progress, ProgressCallback, ProgressStats},
    ZarrReEncodingChangeType, ZarrReencodingArgs,
};

/// Convert a Zarr V3 array to OME-Zarr (0.5-dev).
#[derive(Parser, Debug)]
#[command(author, version)]
struct Cli {
    /// The input array path.
    input: PathBuf,
    /// The output group path.
    output: PathBuf,

    /// The downsample factor.
    ///
    /// Defaults to 2 on each axis.
    #[arg(value_delimiter = ',')]
    downsample_factor: Option<Vec<u64>>,

    /// Maximum number of downsample levels.
    #[arg(long, default_value_t = 10)]
    max_levels: usize,

    /// Physical size (per axis).
    #[arg(long, value_delimiter = ',')]
    physical_size: Option<Vec<f32>>,

    /// Physical units (per axis).
    #[arg(long, value_delimiter = ',')]
    physical_units: Option<Vec<String>>,

    /// OME Zarr dataset name.
    #[arg(long)]
    name: Option<String>,

    /// Disable gaussian smoothing of continuous data.
    #[arg(long)]
    no_gaussian: bool,

    /// Do majority downsampling and do not apply gaussian smoothing.
    #[arg(long)]
    discrete: bool,

    /// Exit instead of overwriting an existing array.
    #[arg(long)]
    no_overwrite: bool,

    /// Attributes (optional).
    ///
    /// JSON holding group attributes.
    #[arg(long)]
    group_attributes: Option<String>,

    #[command(flatten)]
    reencoding: ZarrReencodingArgs,

    /// The maximum number of chunks concurrently processed.
    ///
    /// By default, this is set to the number of CPUs.
    /// Consider reducing this for images with large chunk sizes or on systems with low memory availability.
    #[arg(long)]
    chunk_limit: Option<usize>,
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

fn count_dir(src: impl AsRef<Path>) -> std::io::Result<usize> {
    let mut count = 0;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            count += 1;
        } else {
            count += count_dir(entry.path())?;
        }
    }
    Ok(count)
}

fn copy_dir(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    progress: &Progress,
) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            progress.write(|| std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name())))?;
            progress.next();
        } else {
            copy_dir(entry.path(), dst.as_ref().join(entry.file_name()), progress)?;
        }
    }
    Ok(())
}

fn apply_chunk_discrete<T>(
    array_input: &Array<FilesystemStore>,
    array_output: &Array<FilesystemStore>,
    chunk_indices: &[u64],
    downsample_filter: &Downsample,
    progress: &Progress,
) -> Result<(), FilterError>
where
    T: Pod + Copy + Send + Sync + Eq + PartialEq + Hash + AsPrimitive<T>,
{
    let output_subset = array_output.chunk_subset_bounded(chunk_indices).unwrap();
    let downsample_input_subset =
        downsample_filter.input_subset(array_input.shape(), &output_subset);
    let output_chunk = {
        let input_chunk = progress
            .read(|| array_input.retrieve_array_subset_ndarray::<T>(&downsample_input_subset))?;
        downsample_filter.apply_ndarray_discrete(input_chunk, progress)
    };
    progress.write(|| {
        array_output.store_array_subset_ndarray::<T, _, _>(output_subset.start(), output_chunk)
    })?;
    Ok(())
}

fn apply_chunk_continuous<T>(
    array_input: &Array<FilesystemStore>,
    array_output: &Array<FilesystemStore>,
    chunk_indices: &[u64],
    downsample_filter: &Downsample,
    progress: &Progress,
) -> Result<(), FilterError>
where
    T: Pod + Copy + Send + Sync + AsPrimitive<f64> + std::iter::Sum,
    f64: AsPrimitive<T>,
{
    let output_subset = array_output.chunk_subset_bounded(chunk_indices).unwrap();
    let downsample_input_subset =
        downsample_filter.input_subset(array_input.shape(), &output_subset);
    let output_chunk = {
        let input_chunk = progress
            .read(|| array_input.retrieve_array_subset_ndarray::<T>(&downsample_input_subset))?;
        downsample_filter.apply_ndarray_continuous(input_chunk, progress)
    };
    progress.write(|| {
        array_output.store_array_subset_ndarray::<T, _, _>(output_subset.start(), output_chunk)
    })?;
    Ok(())
}

fn apply_chunk_continuous_gaussian<T>(
    array_input: &Array<FilesystemStore>,
    array_output: &Array<FilesystemStore>,
    chunk_indices: &[u64],
    downsample_filter: &Downsample,
    gaussian_filter: &Gaussian,
    progress: &Progress,
) -> Result<(), FilterError>
where
    T: Pod + Copy + Send + Sync + AsPrimitive<f32> + std::iter::Sum,
    f64: AsPrimitive<T>,
{
    let output_subset = array_output.chunk_subset_bounded(chunk_indices).unwrap();
    let downsample_input_subset =
        downsample_filter.input_subset(array_input.shape(), &output_subset);
    let gaussian_subset_overlap = ArraySubsetOverlap::new(
        array_input.shape(),
        &downsample_input_subset,
        gaussian_filter.kernel_half_size(),
    );
    let gaussian_chunk = {
        let input_chunk = progress.read(|| {
            array_input.retrieve_array_subset_ndarray::<T>(gaussian_subset_overlap.subset_input())
        })?;
        progress.process(|| {
            let input_chunk: ndarray::ArrayD<f32> = input_chunk.map(|x| x.as_()); // par?
            let output_chunk = gaussian_filter.apply_ndarray(input_chunk);
            gaussian_subset_overlap.extract_subset(&output_chunk)
        })
    };
    let output_chunk = downsample_filter.apply_ndarray_continuous(gaussian_chunk, progress);
    progress.write(|| {
        array_output.store_array_subset_ndarray::<T, _, _>(output_subset.start(), output_chunk)
    })?;
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let cli = Cli::parse();

    let bar = ProgressBar::new(0);
    let progress_callback = |stats: ProgressStats| {
        bar.set_length(stats.num_steps as u64);
        bar.set_position(stats.step as u64);
        bar.set_message(format!(
            "rw:{:.2}/{:.2} p:{:.2}",
            stats.read.as_secs_f32(),
            stats.write.as_secs_f32(),
            stats.process.as_secs_f32(),
        ));
    };
    let progress_callback = ProgressCallback::new(&progress_callback);

    let finish_step = |path: &Path| {
        bar.set_style(bar_style_finish());
        bar.set_prefix(format!("{} {}", bar.prefix(), path.to_string_lossy()));
        bar.finish();
        println!();
        bar.reset();
    };

    // Create group
    let store = std::sync::Arc::new(FilesystemStore::new(&cli.output)?);
    let mut group = Group::new(store.clone(), "/")?;
    if let Some(attributes) = &cli.group_attributes {
        let mut group_attributes: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(attributes).expect("Group attributes are invalid.");
        group.attributes_mut().append(&mut group_attributes);
    }

    // Check for overwrite
    if cli.no_overwrite && cli.output.exists() {
        Err(FilterError::Other(
            "Run without --no-overwrite to overwrite".to_string(),
        ))?;
    }
    store.erase_prefix(&StorePrefix::root()).unwrap();

    {
        let output_0_path = cli.output.join("0");
        let store_in = FilesystemStore::new(&cli.input)?;
        let array_in = Array::new(store_in.into(), "/")?;
        bar.set_style(bar_style_run());
        bar.set_prefix(format!("0 {:?}", array_in.shape()));
        if let ZarrReEncodingChangeType::None = cli.reencoding.change_type() {
            // Copy full res input to output
            let dir_count = count_dir(&cli.input)?;
            let progress = Progress::new(dir_count, &progress_callback);
            copy_dir(&cli.input, &output_0_path, &progress)?;
        } else {
            // Reencode the input
            let reencode = zarrs_tools::filter::filters::reencode::Reencode::new(cli.chunk_limit);
            let store_out = FilesystemStore::new(&cli.output)?;
            let mut array_out = reencode
                .output_array_builder(&array_in, &cli.reencoding)
                .build(store_out.into(), "/0")?;
            reencode.apply(&array_in, &mut array_out, &progress_callback)?;
            array_out.store_metadata()?;
        }
        finish_step(&output_0_path);
    }

    // Setup attributes
    let store = std::sync::Arc::new(FilesystemStore::new(&cli.output)?);
    // store.erase_prefix(&StorePrefix::root()).unwrap();
    let mut array0 = Array::new(store.clone(), "/0")?;
    {
        // Move array0 attributes to group
        group.attributes_mut().append(array0.attributes_mut()); // this clears array0 attributes
        group.attributes_mut().remove_entry("_zarrs");
        array0.store_metadata()?;
    }

    // Initialise multiscales metadata
    let mut axes: Vec<Axis> = Vec::with_capacity(array0.dimensionality());
    let physical_units = cli
        .physical_units
        .map(|physical_units| physical_units.into_iter().map(Some).collect_vec())
        .unwrap_or_else(|| vec![None; array0.dimensionality()]);
    if let Some(dimension_names) = array0.dimension_names() {
        for (i, (dimension_name, unit)) in
            std::iter::zip(dimension_names.iter(), physical_units).enumerate()
        {
            axes.push(Axis {
                name: dimension_name
                    .as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| i.to_string()),
                r#type: Some("space".to_string()),
                unit,
            })
        }
    } else {
        for (i, unit) in physical_units.into_iter().enumerate() {
            axes.push(Axis {
                name: i.to_string(),
                r#type: Some("space".to_string()),
                unit,
            })
        }
    }

    let base_transform = cli.physical_size.map(|physical_size| {
        vec![CoordinateTransform::Scale {
            scale: physical_size,
        }]
    });

    let multiscales_metadata = Metadata {
        description: "Created with zarrs_ome".to_string(),
        repository: env!("CARGO_PKG_REPOSITORY").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let mut multiscales = Multiscales {
        version: "0.5-dev".to_string(),
        name: cli.name,
        axes,
        datasets: Vec::with_capacity(cli.max_levels),
        coordinate_transformations: base_transform,
        r#type: Some(if cli.discrete { "mode" } else { "gaussian" }.to_string()),
        metadata: Some(multiscales_metadata),
    };

    let mut relative_scale = vec![1.0; array0.dimensionality()];
    {
        let dataset = Dataset {
            path: "0".to_string(),
            coordinate_transformations: vec![CoordinateTransform::Scale {
                scale: relative_scale.clone(),
            }],
        };
        multiscales.datasets.push(dataset);
    }

    // Calculate gaussian sigma/kernel size for each axis
    let downsample_factor: Vec<u64> = cli
        .downsample_factor
        .unwrap_or_else(|| vec![2; array0.dimensionality()]);
    let sigma: Vec<f32> = downsample_factor
        .iter()
        .map(|downsample_factor| 2.0 * *downsample_factor as f32 / 6.0)
        .collect_vec();
    let kernel_half_size = sigma
        .iter()
        .map(|sigma| (sigma * 4.0).ceil() as u64)
        .collect_vec();
    // println!("sigma:{sigma} kernel_half_size:{kernel_half_size}");

    for i in 1..=cli.max_levels {
        bar.set_style(bar_style_run());

        // Input
        let store = FilesystemStore::new(&cli.output)?;
        let array_input = Array::new(store.into(), &format!("/{}", i - 1))?;

        // Filters
        let gaussian_filter = Gaussian::new(sigma.clone(), kernel_half_size.clone(), None);
        let downsample_filter = Downsample::new(downsample_factor.clone(), cli.discrete, None);

        // Setup reencoding (this is a bit hacky)
        let chunk_representation =
            array_input.chunk_array_representation(&vec![0; array_input.dimensionality()])?;
        let output_shape = downsample_filter.output_shape(&array_input).unwrap();
        let mut reencoding = ZarrReencodingArgs::default();
        if array_input
            .codecs()
            .array_to_bytes_codec()
            .create_metadata()
            .unwrap()
            .name()
            == "sharding_indexed"
        {
            reencoding.shard_shape = Some(
                std::iter::zip(chunk_representation.shape(), &output_shape)
                    .map(|(c, s)| std::cmp::min(c.get(), *s))
                    .collect_vec(),
            );
            let decode_granularity = array_input
                .codecs()
                .partial_decode_granularity(&chunk_representation);
            reencoding.chunk_shape = Some(
                std::iter::zip(decode_granularity.as_slice(), &output_shape)
                    .map(|(g, s)| std::cmp::min(g.get(), *s))
                    .collect_vec(),
            );
        } else {
            reencoding.chunk_shape = Some(
                std::iter::zip(
                    array_input
                        .chunk_shape(&vec![0; array_input.dimensionality()])?
                        .as_slice(),
                    &output_shape,
                )
                .map(|(g, s)| std::cmp::min(g.get(), *s))
                .collect_vec(),
            );
        }
        // println!("{:?} {:?}", reencoding.chunk_shape, reencoding.shard_shape);
        let output_builder = downsample_filter.output_array_builder(&array_input, &reencoding);

        // Output
        let output_path = cli.output.join(i.to_string());
        let output_store = FilesystemStore::new(&cli.output)?;
        let array_output = output_builder.build(output_store.into(), &format!("/{}", i))?;
        bar.set_prefix(format!("{i} {:?}", array_output.shape()));

        // Scale factor (inverse of downsample factor, accounting for actual changes)
        let real_downsample_factor = std::iter::zip(array_input.shape(), array_output.shape())
            .map(|(i, o)| i / o)
            .collect_vec();
        std::iter::zip(&mut relative_scale, &real_downsample_factor).for_each(
            |(scale, downsample_factor)| {
                *scale *= *downsample_factor as f32;
            },
        );
        // println!("{downsample_factor:?} -> {scale:?}");

        // Chunks
        let chunks = ArraySubset::new_with_shape(array_output.chunk_grid_shape().unwrap());
        let progress = Progress::new(chunks.num_elements_usize(), &progress_callback);

        let chunk_limit = if let Some(chunk_limit) = cli.chunk_limit {
            chunk_limit
        } else {
            // Get memory usage
            let output_chunk =
                array_output.chunk_array_representation(&vec![0; array_input.dimensionality()])?;
            let downsample_memory =
                downsample_filter.memory_per_chunk(&output_chunk /* unused */, &output_chunk);
            let memory_per_chunk = downsample_memory
                + if cli.no_gaussian {
                    0
                } else {
                    let downsample_input_subset = downsample_filter.input_subset(
                        array_input.shape(),
                        &ArraySubset::new_with_shape(output_chunk.shape_u64()),
                    );
                    let downsample_input = ChunkRepresentation::new(
                        downsample_input_subset
                            .shape()
                            .iter()
                            .map(|s| NonZeroU64::new(*s).unwrap())
                            .collect_vec(),
                        array_input.data_type().clone(),
                        array_input.fill_value().clone(),
                    )?;
                    gaussian_filter.memory_per_chunk(&downsample_input, &downsample_input)
                };
            // let system = sysinfo::System::new_with_specifics(
            //     sysinfo::RefreshKind::new()
            //         .with_memory(sysinfo::MemoryRefreshKind::new().with_ram()),
            // );
            // let available_memory = usize::try_from(system.available_memory()).unwrap();
            // let free_memory = usize::try_from(system.free_memory()).unwrap();
            // let chunk_limit = zarrs_tools::filter::calculate_chunk_limit(memory_per_chunk)?;
            // println!(
            //     "{memory_per_chunk} -> {chunk_limit}/{} = {:.2}GB of {:.2}GB/{:.2}GB",
            //     chunks.num_elements(),
            //     (memory_per_chunk * chunk_limit) as f32 / 1e9,
            //     available_memory as f32 / 1e9,
            //     free_memory as f32 / 1e9,
            // );
            zarrs_tools::filter::calculate_chunk_limit(memory_per_chunk)?
        };

        // Apply
        let indices = chunks.indices();
        rayon_iter_concurrent_limit::iter_concurrent_limit!(
            chunk_limit,
            indices,
            try_for_each,
            |chunk_indices: Vec<u64>| {
                macro_rules! discrete_or_continuous {
                    ( $t:ty ) => {{
                        if cli.discrete {
                            apply_chunk_discrete::<$t>(
                                &array_input,
                                &array_output,
                                &chunk_indices,
                                &downsample_filter,
                                &progress,
                            )?
                        } else if cli.no_gaussian {
                            apply_chunk_continuous::<$t>(
                                &array_input,
                                &array_output,
                                &chunk_indices,
                                &downsample_filter,
                                &progress,
                            )?
                        } else {
                            apply_chunk_continuous_gaussian::<$t>(
                                &array_input,
                                &array_output,
                                &chunk_indices,
                                &downsample_filter,
                                &gaussian_filter,
                                &progress,
                            )?
                        }
                    }};
                }
                macro_rules! continuous {
                    ( $t:ty ) => {{
                        if cli.no_gaussian {
                            apply_chunk_continuous::<$t>(
                                &array_input,
                                &array_output,
                                &chunk_indices,
                                &downsample_filter,
                                &progress,
                            )?
                        } else {
                            apply_chunk_continuous_gaussian::<$t>(
                                &array_input,
                                &array_output,
                                &chunk_indices,
                                &downsample_filter,
                                &gaussian_filter,
                                &progress,
                            )?
                        }
                    }};
                }
                macro_rules! apply {
                    ( [$( ( $data_type_out:ident, $t:ty,  $inner:ident ) ),* ]) => {
                        match array_input.data_type() {
                            $(zarrs::array::DataType::$data_type_out => { $inner!($t) } ,)*
                            _ => panic!("unsupported data type")
                        }
                    };
                }
                apply!([
                    (Int8, i8, discrete_or_continuous),
                    (Int16, i16, discrete_or_continuous),
                    (Int32, i32, discrete_or_continuous),
                    (Int64, i64, discrete_or_continuous),
                    (UInt8, u8, discrete_or_continuous),
                    (UInt16, u16, discrete_or_continuous),
                    (UInt32, u32, discrete_or_continuous),
                    (UInt64, u64, discrete_or_continuous),
                    (BFloat16, bf16, continuous),
                    (Float16, f16, continuous),
                    (Float32, f32, continuous),
                    (Float64, f64, continuous)
                ]);

                progress.next();
                Ok::<_, FilterError>(())
            }
        )?;

        // Append multiscales dataset metadata
        let dataset = Dataset {
            path: format!("{i}"),
            coordinate_transformations: vec![
                CoordinateTransform::Scale {
                    scale: relative_scale.clone(),
                },
                CoordinateTransform::Translation {
                    translation: relative_scale.iter().map(|s| (s - 1.0) * 0.5).collect_vec(),
                },
            ],
        };
        multiscales.datasets.push(dataset);

        array_output.store_metadata()?;
        finish_step(&output_path);

        // Stop when for all axis the output shape is 1 or stride is 1
        if std::iter::zip(&downsample_factor, &output_shape).all(|(df, s)| *df == 1 || *s == 1) {
            break;
        }
    }

    // Add multiscales metadata
    let multiscales = vec![serde_json::from_str(&serde_json::to_string(&multiscales)?)?];
    group.attributes_mut().insert(
        "multiscales".to_string(),
        serde_json::Value::Array(multiscales),
    );
    group.store_metadata()?;

    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Multiscales {
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    axes: Vec<Axis>,
    datasets: Vec<Dataset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    coordinate_transformations: Option<Vec<CoordinateTransform>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<Metadata>,
}

#[derive(Serialize)]
struct Metadata {
    description: String,
    repository: String,
    version: String,
}

#[derive(Serialize)]
struct Axis {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unit: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Dataset {
    path: String,
    coordinate_transformations: Vec<CoordinateTransform>,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
enum CoordinateTransform {
    #[allow(dead_code)]
    Identity,
    Translation {
        translation: Vec<f32>,
    },
    Scale {
        scale: Vec<f32>,
    },
}

fn main() -> std::process::ExitCode {
    if let Err(err) = run() {
        println!("{}", err);
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::SUCCESS
    }
}
