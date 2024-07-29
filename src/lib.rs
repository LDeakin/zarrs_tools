#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![doc(hidden)]

use std::{sync::Mutex, time::SystemTime};

use clap::Parser;
use progress::{Progress, ProgressCallback};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon_iter_concurrent_limit::iter_concurrent_limit;
use serde::{Deserialize, Serialize};
use zarrs::{
    array::{
        codec::{
            array_to_bytes::sharding, ArrayCodecTraits, ArrayToBytesCodecTraits, BytesCodec, Codec,
            CodecOptionsBuilder, Crc32cCodec, ShardingCodec,
        },
        concurrency::RecommendedConcurrency,
        Array, ArrayBuilder, ArrayError, CodecChain, DataType, DimensionName, FillValue,
        FillValueMetadata,
    },
    array_subset::ArraySubset,
    config::global_config,
    metadata::Metadata,
    storage::{store::FilesystemStore, ReadableWritableStorageTraits},
};

pub mod filter;
pub mod info;
pub mod progress;

#[derive(Parser)]
#[allow(rustdoc::bare_urls)]
pub struct ZarrEncodingArgs {
    /// Fill value. See https://zarr-specs.readthedocs.io/en/latest/v3/core/v3.0.html#fill-value
    ///
    /// The fill value must be compatible with the data type.
    ///
    /// Examples:
    ///   int/uint: 0 100 -100
    ///   float: 0.0 "NaN" "Infinity" "-Infinity"
    ///   r*: "[0, 255]"
    #[arg(short, long, verbatim_doc_comment, allow_hyphen_values(true), value_parser = parse_fill_value)]
    pub fill_value: FillValueMetadata,

    /// The chunk key encoding separator. Either . or /.
    #[arg(long, default_value_t = '/')]
    pub separator: char,

    /// Chunk shape. A comma separated list of the chunk size along each array dimension.
    ///
    /// If any dimension has size zero, it will be set to match the array shape.
    #[arg(short, long, required = true, value_delimiter = ',')]
    pub chunk_shape: Vec<u64>,

    /// Shard shape (optional). A comma separated list of the shard size along each array dimension.
    ///
    /// If specified, the array is encoded using the sharding codec.
    /// If any dimension has size zero, it will be set to match the array shape.
    #[arg(short, long, verbatim_doc_comment, value_delimiter = ',')]
    pub shard_shape: Option<Vec<u64>>,

    /// Array to array codecs (optional).
    ///
    /// JSON holding an array of array to array codec metadata.
    ///
    /// Examples:
    ///   '[ { "name": "transpose", "configuration": { "order": [0, 2, 1] } } ]'
    ///   '[ { "name": "bitround", "configuration": { "keepbits": 9 } } ]'
    #[arg(long, verbatim_doc_comment)]
    pub array_to_array_codecs: Option<String>,

    /// Array to bytes codec (optional).
    ///
    /// JSON holding array to bytes codec metadata.
    /// If unspecified, this defaults to the `bytes` codec.
    ///
    /// The sharding codec can be used by setting `shard_shape`, but this can also be done explicitly here.
    ///
    /// Examples:
    ///   '{ "name": "bytes", "configuration": { "endian": "little" } }'
    ///   '{ "name": "pcodec", "configuration": { "level": 12 } }'
    ///   '{ "name": "zfp", "configuration": { "mode": "fixedprecision", "precision": 19 } }'
    #[arg(long, verbatim_doc_comment)]
    pub array_to_bytes_codec: Option<String>,

    /// Bytes to bytes codecs (optional).
    ///
    /// JSON holding an array of bytes to bytes codec configurations.
    ///
    /// Examples:
    ///   '[ { "name": "blosc", "configuration": { "cname": "blosclz", "clevel": 9, "shuffle": "bitshuffle", "typesize": 2, "blocksize": 0 } } ]'
    ///   '[ { "name": "bz2", "configuration": { "level": 9 } } ]'
    ///   '[ { "name": "crc32c" ]'
    ///   '[ { "name": "gzip", "configuration": { "level": 9 } } ]'
    ///   '[ { "name": "zstd", "configuration": { "level": 22, "checksum": false } } ]'
    #[arg(long, verbatim_doc_comment)]
    pub bytes_to_bytes_codecs: Option<String>,

    /// Attributes (optional).
    ///
    /// JSON holding array attributes.
    #[arg(long)]
    pub attributes: Option<String>,
}

fn parse_data_type(data_type: &str) -> std::io::Result<Metadata> {
    Ok(Metadata::new(data_type))
}

fn parse_fill_value(fill_value: &str) -> std::io::Result<FillValueMetadata> {
    serde_json::from_str(fill_value)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))
}

#[must_use]
pub fn get_array_builder(
    encoding_args: &ZarrEncodingArgs,
    array_shape: &[u64],
    data_type: DataType,
    dimension_names: Option<Vec<DimensionName>>,
) -> zarrs::array::ArrayBuilder {
    // Set the chunk/shard shape to the array shape where it is 0, otherwise make it <= array shape
    let shard_shape = encoding_args.shard_shape.as_ref().map(|shard_shape| {
        std::iter::zip(shard_shape, array_shape)
            .map(|(&s, &a)| if s == 0 { a } else { std::cmp::min(s, a) })
            .collect::<Vec<_>>()
    });

    // Also ensure shard shape is a multiple of chunk shape
    let chunk_shape = std::iter::zip(&encoding_args.chunk_shape, array_shape)
        .map(|(&c, &a)| if c == 0 { a } else { c })
        .collect::<Vec<_>>();
    let shard_shape: Option<Vec<u64>> = shard_shape.map(|shard_shape| {
        std::iter::zip(&shard_shape, &chunk_shape)
            .map(|(s, c)| {
                // the shard shape must be a multiple of the chunk shape
                c * ((s + c - 1) / c)
            })
            .collect()
    });

    // Get the "block shape", which is the shard shape if sharding, otherwise the chunk shape
    let block_shape = shard_shape
        .as_ref()
        .map_or(&chunk_shape, |shard_shape| shard_shape);

    // Get array to array codecs
    let array_to_array_codecs = encoding_args.array_to_array_codecs.as_ref().map_or_else(
        Vec::new,
        |array_to_array_codecs| {
            let metadatas: Vec<Metadata> =
                serde_json::from_str(array_to_array_codecs.as_str()).unwrap();
            let mut codecs = Vec::with_capacity(metadatas.len());
            for metadata in metadatas {
                codecs.push(match Codec::from_metadata(&metadata).unwrap() {
                    Codec::ArrayToArray(codec) => codec,
                    _ => panic!("Must be a bytes to bytes codec"),
                });
            }
            codecs
        },
    );

    // Get array to bytes codec
    let array_to_bytes_codec = encoding_args.array_to_bytes_codec.as_ref().map_or_else(
        || {
            let codec: Box<dyn ArrayToBytesCodecTraits> = Box::<BytesCodec>::default();
            codec
        },
        |array_codec| {
            let metadata = Metadata::try_from(array_codec.as_str()).unwrap();
            match Codec::from_metadata(&metadata).unwrap() {
                Codec::ArrayToBytes(codec) => codec,
                _ => panic!("Must be a arrayc to array codec"),
            }
        },
    );

    // Get bytes to bytes codecs
    let bytes_to_bytes_codecs = encoding_args.bytes_to_bytes_codecs.as_ref().map_or_else(
        Vec::new,
        |bytes_to_bytes_codecs| {
            let metadatas: Vec<Metadata> =
                serde_json::from_str(bytes_to_bytes_codecs.as_str()).unwrap();
            let mut codecs = Vec::with_capacity(metadatas.len());
            for metadata in metadatas {
                codecs.push(match Codec::from_metadata(&metadata).unwrap() {
                    Codec::BytesToBytes(codec) => codec,
                    _ => panic!("Must be a bytes to bytes codec"),
                });
            }
            codecs
        },
    );

    // Get data type / fill value
    let fill_value = data_type
        .fill_value_from_metadata(&encoding_args.fill_value)
        .unwrap();

    // Create array
    let mut array_builder = ArrayBuilder::new(
        array_shape.to_vec(),
        data_type,
        block_shape.clone().try_into().unwrap(),
        fill_value,
    );
    array_builder.dimension_names(dimension_names);
    if let Some(attributes) = &encoding_args.attributes {
        let attributes: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(attributes).expect("Attributes are invalid.");
        array_builder.attributes(attributes);
    }
    array_builder.chunk_key_encoding_default_separator(encoding_args.separator.try_into().unwrap());
    if shard_shape.is_some() {
        let index_codecs = CodecChain::new(
            vec![],
            Box::<BytesCodec>::default(),
            vec![Box::new(Crc32cCodec::new())],
        );
        let inner_codecs = CodecChain::new(
            array_to_array_codecs,
            array_to_bytes_codec,
            bytes_to_bytes_codecs,
        );
        array_builder.array_to_bytes_codec(Box::new(ShardingCodec::new(
            chunk_shape.try_into().unwrap(),
            inner_codecs,
            index_codecs,
            sharding::ShardingIndexLocation::End,
        )));
    } else {
        array_builder.array_to_array_codecs(array_to_array_codecs);
        array_builder.array_to_bytes_codec(array_to_bytes_codec);
        array_builder.bytes_to_bytes_codecs(bytes_to_bytes_codecs);
    }

    array_builder
}

#[derive(Parser, Debug, Clone, Default, Serialize, Deserialize)]

pub struct ZarrReencodingArgs {
    /// The data type as a string
    ///
    /// Valid data types:
    ///   - bool
    ///   - int8, int16, int32, int64
    ///   - uint8, uint16, uint32, uint64
    ///   - float16, float32, float64, bfloat16
    ///   - complex64, complex 128
    ///   - r* (raw bits, where * is a multiple of 8)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(short, long, verbatim_doc_comment, value_parser = parse_data_type)]
    pub data_type: Option<Metadata>,

    /// Fill value. See <https://zarr-specs.readthedocs.io/en/latest/v3/core/v3.0.html#fill-value>
    ///
    /// The fill value must be compatible with the data type.
    ///
    /// Examples:
    ///   int/uint: 0 100 -100
    ///   float: 0.0 "NaN" "Infinity" "-Infinity"
    ///   r*: "[0, 255]"
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(short, long, verbatim_doc_comment, allow_hyphen_values(true), value_parser = parse_fill_value)]
    pub fill_value: Option<FillValueMetadata>,

    /// The chunk key encoding separator. Either . or /.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub separator: Option<char>,

    /// Chunk shape. A comma separated list of the chunk size along each array dimension.
    ///
    /// If any dimension has size zero, it will be set to match the array shape.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(short, long, value_delimiter = ',')]
    pub chunk_shape: Option<Vec<u64>>,

    /// Shard shape. A comma separated list of the shard size along each array dimension.
    ///
    /// If specified, the array is encoded using the sharding codec.
    /// If any dimension has size zero, it will be set to match the array shape.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(short, long, verbatim_doc_comment, value_delimiter = ',')]
    pub shard_shape: Option<Vec<u64>>,

    /// Array to array codecs.
    ///
    /// JSON holding an array of array to array codec metadata.
    ///
    /// Examples:
    ///   '[ { "name": "transpose", "configuration": { "order": [0, 2, 1] } } ]'
    ///   '[ { "name": "bitround", "configuration": { "keepbits": 9 } } ]'
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long, verbatim_doc_comment)]
    pub array_to_array_codecs: Option<String>,

    /// Array to bytes codec.
    ///
    /// JSON holding array to bytes codec metadata.
    ///
    /// Examples:
    ///   '{ "name": "bytes", "configuration": { "endian": "little" } }'
    ///   '{ "name": "pcodec", "configuration": { "level": 12 } }'
    ///   '{ "name": "zfp", "configuration": { "mode": "fixedprecision", "precision": 19 } }'
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long, verbatim_doc_comment)]
    pub array_to_bytes_codec: Option<String>,

    /// Bytes to bytes codecs.
    ///
    /// JSON holding an array bytes to bytes codec configurations.
    ///
    /// Examples:
    ///   '[ { "name": "blosc", "configuration": { "cname": "blosclz", "clevel": 9, "shuffle": "bitshuffle", "typesize": 2, "blocksize": 0 } } ]'
    ///   '[ { "name": "bz2", "configuration": { "level": 9 } } ]'
    ///   '[ { "name": "crc32c" } ]'
    ///   '[ { "name": "gzip", "configuration": { "level": 9 } } ]'
    ///   '[ { "name": "zstd", "configuration": { "level": 22, "checksum": false } } ]'
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long, verbatim_doc_comment)]
    pub bytes_to_bytes_codecs: Option<String>,

    /// Dimension names (optional). Comma separated.
    #[arg(long, verbatim_doc_comment, value_delimiter = ',')]
    pub dimension_names: Option<Vec<String>>,

    /// Attributes (optional).
    ///
    /// JSON holding array attributes.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub attributes: Option<String>,

    /// Attributes to append (optional).
    ///
    /// JSON holding array attributes.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub attributes_append: Option<String>,
}

pub enum ZarrReEncodingChangeType {
    None,
    Metadata,
    MetadataAndChunks,
}

impl ZarrReencodingArgs {
    pub fn change_type(&self) -> ZarrReEncodingChangeType {
        if self.data_type.is_some()
            || self.fill_value.is_some()
            || self.separator.is_some()
            || self.chunk_shape.is_some()
            || self.shard_shape.is_some()
            || self.array_to_array_codecs.is_some()
            || self.array_to_bytes_codec.is_some()
            || self.bytes_to_bytes_codecs.is_some()
        {
            ZarrReEncodingChangeType::MetadataAndChunks
        } else if self.dimension_names.is_some()
            || self.attributes.is_some()
            || self.attributes_append.is_some()
        {
            ZarrReEncodingChangeType::Metadata
        } else {
            ZarrReEncodingChangeType::None
        }
    }
}

#[must_use]
pub fn get_array_builder_reencode<TStorage: ?Sized>(
    encoding_args: &ZarrReencodingArgs,
    array: &Array<TStorage>,
    array_shape: Option<Vec<u64>>,
) -> zarrs::array::ArrayBuilder {
    let array_to_bytes_metadata = array
        .codecs()
        .array_to_bytes_codec()
        .create_metadata()
        .unwrap();
    let (
        chunk_shape,
        shard_shape,
        array_to_array_codecs,
        array_array_to_bytes_codec,
        bytes_to_bytes_codecs,
    ) = if array_to_bytes_metadata.name() == "sharding_indexed" {
        let sharding_configuration = array_to_bytes_metadata.configuration().unwrap();
        // println!("{sharding_configuration:#?}");
        let chunk_shape: Vec<u64> =
            serde_json::from_value(sharding_configuration["chunk_shape"].clone()).unwrap();
        let shard_shape = array
            .chunk_shape(&vec![0; chunk_shape.len()])
            .unwrap()
            .iter()
            .map(|i| i.get())
            .collect::<Vec<_>>();
        let codecs: Vec<Metadata> =
            serde_json::from_value(sharding_configuration["codecs"].clone()).unwrap();
        let codec_chain = CodecChain::from_metadata(&codecs).unwrap();
        let array_to_array_codecs = codec_chain.array_to_array_codecs().to_vec();
        let array_to_bytes_codec = codec_chain.array_to_bytes_codec().clone();
        let bytes_to_bytes_codecs = codec_chain.bytes_to_bytes_codecs().to_vec();
        (
            chunk_shape,
            Some(shard_shape),
            array_to_array_codecs,
            array_to_bytes_codec,
            bytes_to_bytes_codecs,
        )
    } else {
        let chunk_shape = array.chunk_grid_shape().unwrap().to_vec();
        let shard_shape = None;
        let array_to_array_codecs = array.codecs().array_to_array_codecs().to_vec();
        let array_to_bytes_codec = array.codecs().array_to_bytes_codec().clone();
        let bytes_to_bytes_codecs = array.codecs().bytes_to_bytes_codecs().to_vec();
        (
            chunk_shape,
            shard_shape,
            array_to_array_codecs,
            array_to_bytes_codec,
            bytes_to_bytes_codecs,
        )
    };

    // Chunk shape override
    let chunk_shape = encoding_args
        .chunk_shape
        .as_ref()
        .map(|chunk_shape| {
            std::iter::zip(chunk_shape.as_slice(), array.shape())
                .map(|(&c, &a)| if c == 0 { a } else { c })
                .collect::<Vec<_>>()
        })
        .unwrap_or(chunk_shape);

    // Shard shape override
    // Set the shard shape to the array shape where it is 0, otherwise make it <= array shape
    let shard_shape: Option<Vec<u64>> =
        encoding_args
            .shard_shape
            .as_ref()
            .map_or(shard_shape, |shard_shape| {
                let shard_shape = std::iter::zip(shard_shape, array.shape())
                    .map(|(&s, &a)| if s == 0 { a } else { std::cmp::min(s, a) })
                    .collect::<Vec<_>>();
                Some(shard_shape)
            });

    // Ensure shard shape is a multiple of the chunk shape
    let shard_shape: Option<Vec<u64>> = shard_shape.clone().map_or(shard_shape, |shard_shape| {
        let shard_shape = std::iter::zip(shard_shape.as_slice(), chunk_shape.as_slice())
            .map(|(s, c)| {
                // the shard shape must be a multiple of the chunk shape
                c * ((s + c - 1) / c)
            })
            .collect::<Vec<_>>();
        Some(shard_shape)
    });

    // println!("{chunk_shape:?} {shard_shape:?}");

    // Get array to array codecs
    let array_to_array_codecs = encoding_args.array_to_array_codecs.clone().map_or(
        array_to_array_codecs,
        |array_to_array_codecs| {
            let metadatas: Vec<Metadata> =
                serde_json::from_str(array_to_array_codecs.as_str()).unwrap();
            let mut codecs = Vec::with_capacity(metadatas.len());
            for metadata in metadatas {
                codecs.push(match Codec::from_metadata(&metadata).unwrap() {
                    Codec::ArrayToArray(codec) => codec,
                    _ => panic!("Must be a bytes to bytes codec"),
                });
            }
            codecs
        },
    );

    // Get array to bytes codec
    let array_to_bytes_codec = encoding_args.array_to_bytes_codec.as_ref().map_or(
        array_array_to_bytes_codec,
        |array_codec| {
            let metadata = Metadata::try_from(array_codec.as_str()).unwrap();
            match Codec::from_metadata(&metadata).unwrap() {
                Codec::ArrayToBytes(codec) => codec,
                _ => panic!("Must be a arrayc to array codec"),
            }
        },
    );

    // Get bytes to bytes codecs
    let bytes_to_bytes_codecs = encoding_args.bytes_to_bytes_codecs.as_ref().map_or(
        bytes_to_bytes_codecs,
        |bytes_to_bytes_codecs| {
            let metadatas: Vec<Metadata> =
                serde_json::from_str(bytes_to_bytes_codecs.as_str()).unwrap();
            let mut codecs = Vec::with_capacity(metadatas.len());
            for metadata in metadatas {
                codecs.push(match Codec::from_metadata(&metadata).unwrap() {
                    Codec::BytesToBytes(codec) => codec,
                    _ => panic!("Must be a bytes to bytes codec"),
                });
            }
            codecs
        },
    );

    // Create array
    let mut array_builder = array.builder();

    if let Some(attributes) = &encoding_args.attributes {
        let attributes: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(attributes).expect("Attributes are invalid.");
        array_builder.attributes(attributes);
    }

    if let Some(attributes_append) = &encoding_args.attributes_append {
        let mut attributes_append: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(attributes_append).expect("Attributes append are invalid.");
        array_builder.attributes.append(&mut attributes_append);
    }

    if let Some(separator) = encoding_args.separator {
        array_builder.chunk_key_encoding_default_separator(separator.try_into().unwrap());
    }

    if let Some(array_shape) = array_shape {
        array_builder.shape(array_shape);
    }

    if let Some(data_type) = &encoding_args.data_type {
        let data_type = DataType::from_metadata(data_type).unwrap();
        array_builder.data_type(data_type.clone());
    }

    if let Some(dimension_names) = encoding_args.dimension_names.clone() {
        // TODO: Remove clone with zarrs 0.15.1+
        array_builder.dimension_names(dimension_names.into());
    }

    if let Some(fill_value) = &encoding_args.fill_value {
        // An explicit fill value was supplied
        let fill_value = array_builder
            .data_type
            .fill_value_from_metadata(fill_value)
            .unwrap();
        array_builder.fill_value(fill_value);
    } else if let Some(data_type) = &encoding_args.data_type {
        // The data type was changed, but no fill value supplied, so just cast it
        let data_type = DataType::from_metadata(data_type).unwrap();
        let fill_value = convert_fill_value(array.data_type(), array.fill_value(), &data_type);
        array_builder.fill_value(fill_value);
    }

    if let Some(shard_shape) = shard_shape {
        array_builder.chunk_grid(shard_shape.try_into().unwrap());
        let index_codecs = CodecChain::new(
            vec![],
            Box::<BytesCodec>::default(),
            vec![Box::new(Crc32cCodec::new())],
        );
        let inner_codecs = CodecChain::new(
            array_to_array_codecs,
            array_to_bytes_codec,
            bytes_to_bytes_codecs,
        );
        array_builder.array_to_array_codecs(vec![]);
        array_builder.array_to_bytes_codec(Box::new(ShardingCodec::new(
            chunk_shape.try_into().unwrap(),
            inner_codecs,
            index_codecs,
            sharding::ShardingIndexLocation::End,
        )));
        array_builder.bytes_to_bytes_codecs(vec![]);
    } else {
        array_builder.array_to_array_codecs(array_to_array_codecs);
        array_builder.array_to_bytes_codec(array_to_bytes_codec);
        array_builder.bytes_to_bytes_codecs(bytes_to_bytes_codecs);
    }

    array_builder
}

pub fn do_reencode<TStorageOut: ReadableWritableStorageTraits + 'static>(
    array_in: &Array<FilesystemStore>,
    array_out: &Array<TStorageOut>,
    validate: bool,
    concurrent_chunks: Option<usize>,
    progress_callback: &ProgressCallback,
) -> Result<(f32, f32, f32, usize), ArrayError> {
    let start = SystemTime::now();
    let bytes_decoded = Mutex::new(0);

    let chunk_representation = array_out
        .chunk_array_representation(&vec![0; array_out.chunk_grid().dimensionality()])
        .unwrap();
    let chunks = ArraySubset::new_with_shape(array_out.chunk_grid_shape().unwrap());

    let concurrent_target = std::thread::available_parallelism().unwrap().get();
    let (chunks_concurrent_limit, codec_concurrent_target) =
        zarrs::array::concurrency::calc_concurrency_outer_inner(
            concurrent_target,
            &if let Some(concurrent_chunks) = concurrent_chunks {
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
            &array_out
                .codecs()
                .recommended_concurrency(&chunk_representation)
                .unwrap(),
        );
    let codec_options = CodecOptionsBuilder::new()
        .concurrent_target(codec_concurrent_target)
        .build();

    let progress = Progress::new(chunks.num_elements_usize(), progress_callback);
    let indices = chunks.indices();
    if array_in.data_type() == array_out.data_type() {
        iter_concurrent_limit!(
            chunks_concurrent_limit,
            indices,
            try_for_each,
            |chunk_indices: Vec<u64>| {
                let chunk_subset = array_out.chunk_subset(&chunk_indices).unwrap();
                let bytes = progress
                    .read(|| array_in.retrieve_array_subset_opt(&chunk_subset, &codec_options))?;
                *bytes_decoded.lock().unwrap() += bytes.size();

                if validate {
                    progress.write(|| {
                        array_out.store_chunk_opt(&chunk_indices, bytes.clone(), &codec_options)
                    })?;
                    let bytes_out = array_out
                        .retrieve_chunk_opt(&chunk_indices, &codec_options)
                        .unwrap();
                    assert!(bytes == bytes_out);
                } else {
                    progress.write(|| {
                        array_out.store_chunk_opt(&chunk_indices, bytes, &codec_options)
                    })?;
                }
                progress.next();
                Ok::<_, ArrayError>(())
            }
        )?;
    } else {
        // FIXME
        todo!("zarrs_reencode does not yet support data type conversion!")
    }

    let duration = start.elapsed().unwrap().as_secs_f32();
    let stats = progress.stats();
    let duration_read = stats.read.as_secs_f32();
    let duration_write = stats.write.as_secs_f32();
    let duration_read_write = duration_read + duration_write;
    let duration_read = duration_read * duration / duration_read_write;
    let duration_write = duration_write * duration / duration_read_write;

    Ok((
        duration,
        duration_read,
        duration_write,
        bytes_decoded.into_inner().unwrap(),
    ))
}

/// Convert an arrays fill value to a new data type
fn convert_fill_value(
    data_type_in: &DataType,
    fill_value_in: &FillValue,
    data_type_out: &DataType,
) -> FillValue {
    macro_rules! convert {
        ( $t_in:ty, $t_out:ty) => {{
            let input_fill_value =
                <$t_in>::from_ne_bytes(fill_value_in.as_ne_bytes().try_into().unwrap());
            use num_traits::AsPrimitive;
            let output_fill_value: $t_out = input_fill_value.as_();
            FillValue::from(output_fill_value)
        }};
    }
    macro_rules! apply_inner {
        ( $type_in:ty, [$( ( $data_type_out:ident, $type_out:ty ) ),* ]) => {
            match data_type_out {
                $(DataType::$data_type_out => { convert!($type_in, $type_out) } ,)*
                _ => panic!()
            }
        };
    }
    macro_rules! apply_outer {
    ([$( ( $data_type_in:ident, $type_in:ty ) ),* ]) => {
            match data_type_in {
                $(
                    DataType::$data_type_in => {
                        apply_inner!($type_in, [
                            (Bool, u8),
                            (Int8, i8),
                            (Int16, i16),
                            (Int32, i32),
                            (Int64, i64),
                            (UInt8, u8),
                            (UInt16, u16),
                            (UInt32, u32),
                            (UInt64, u64),
                            (BFloat16, half::bf16),
                            (Float16, half::f16),
                            (Float32, f32),
                            (Float64, f64)
                        ]
                    )}
                ,)*
                _ => panic!()
            }
        };
    }
    apply_outer!([
        (Bool, u8),
        (Int8, i8),
        (Int16, i16),
        (Int32, i32),
        (Int64, i64),
        (UInt8, u8),
        (UInt16, u16),
        (UInt32, u32),
        (UInt64, u64),
        (BFloat16, half::bf16),
        (Float16, half::f16),
        (Float32, f32),
        (Float64, f64)
    ])
}
