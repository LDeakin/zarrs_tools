#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![doc(hidden)]

use std::{
    sync::Mutex,
    time::{Duration, SystemTime},
};

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon_iter_concurrent_limit::iter_concurrent_limit;
use zarrs::{
    array::{
        codec::{
            array_to_bytes::sharding, ArrayCodecTraits, ArrayToBytesCodecTraits, BytesCodec, Codec,
            CodecOptionsBuilder, Crc32cCodec, ShardingCodec,
        },
        concurrency::RecommendedConcurrency,
        Array, ArrayBuilder, CodecChain, DataType, DimensionName, FillValueMetadata,
    },
    array_subset::ArraySubset,
    config::global_config,
    metadata::Metadata,
    storage::{store::FilesystemStore, ReadableWritableStorageTraits},
};

#[derive(Parser)]
#[allow(rustdoc::bare_urls)]
pub struct ZarrEncodingArgs {
    /// Fill value. See https://zarr-specs.readthedocs.io/en/latest/v3/core/v3.0.html#fill-value
    ///
    /// The fill value must be compatible with the data type.
    ///
    /// Examples:
    ///   int/uint: 0
    ///   float: 0.0 "NaN" "Infinity" "-Infinity"
    ///   r*: "[0, 255]"
    #[arg(short, long, verbatim_doc_comment, allow_hyphen_values(true))]
    pub fill_value: String,

    /// The chunk key encoding separator. Either `/`. or `.`.
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
    /// A JSON string holding array to array codec metadata.
    ///
    /// Examples:
    ///   '[ { "name": "bitround", "configuration": { "keepbits": 9 } } ]'
    #[arg(long, verbatim_doc_comment)]
    pub array_to_array_codecs: Option<String>,

    /// Array to bytes codec (optional).
    ///
    /// A JSON string holding array to array codec metadata.
    ///
    /// Examples:
    ///   '{ "name": "zfp", "configuration": { "mode": "fixedprecision", "precision": 19 } }'
    #[arg(long, verbatim_doc_comment)]
    pub array_to_bytes_codec: Option<String>,

    /// Bytes to bytes codecs (optional).
    ///
    /// A JSON string holding bytes to bytes codec configurations.
    ///
    /// Examples:
    ///   '[ { "name": "blosc", "configuration": { "cname": "blosclz", "clevel": 9, "shuffle": "bitshuffle", "typesize": 2, "blocksize": 0 } } ]'
    ///   '[ { "name": "gzip", "configuration": { "level": 3 } } ]'
    #[arg(long, verbatim_doc_comment)]
    pub bytes_to_bytes_codecs: Option<String>,
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
    let fill_value_metadata =
        FillValueMetadata::try_from(encoding_args.fill_value.as_str()).unwrap();
    let fill_value = data_type
        .fill_value_from_metadata(&fill_value_metadata)
        .unwrap();

    // Create array
    let mut array_builder = ArrayBuilder::new(
        array_shape.to_vec(),
        data_type,
        block_shape.clone().try_into().unwrap(),
        fill_value,
    );
    array_builder.dimension_names(dimension_names);
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

#[derive(Parser, Debug)]
pub struct ZarrReEncodingArgs {
    /// Fill value. See <https://zarr-specs.readthedocs.io/en/latest/v3/core/v3.0.html#fill-value>
    ///
    /// The fill value must be compatible with the data type.
    ///
    /// Examples:
    ///   int/uint: 0
    ///   float: 0.0 "NaN" "Infinity" "-Infinity"
    ///   r*: "[0, 255]"
    #[arg(short, long, verbatim_doc_comment, allow_hyphen_values(true))]
    pub fill_value: Option<String>,

    /// The chunk key encoding separator. Either `/`. or `.`.
    #[arg(long)]
    pub separator: Option<char>,

    /// Chunk shape. A comma separated list of the chunk size along each array dimension.
    ///
    /// If any dimension has size zero, it will be set to match the array shape.
    #[arg(short, long, value_delimiter = ',')]
    pub chunk_shape: Option<Vec<u64>>,

    /// Shard shape. A comma separated list of the shard size along each array dimension.
    ///
    /// If specified, the array is encoded using the sharding codec.
    /// If any dimension has size zero, it will be set to match the array shape.
    #[arg(short, long, verbatim_doc_comment, value_delimiter = ',')]
    pub shard_shape: Option<Vec<u64>>,

    /// Array to array codecs.
    ///
    /// A JSON string holding array to array codec metadata.
    ///
    /// Examples:
    ///   '[ { "name": "bitround", "configuration": { "keepbits": 9 } } ]'
    #[arg(long, verbatim_doc_comment)]
    pub array_to_array_codecs: Option<String>,

    /// Array to bytes codec.
    ///
    /// A JSON string holding array to array codec metadata.
    ///
    /// Examples:
    ///   '{ "name": "zfp", "configuration": { "mode": "fixedprecision", "precision": 19 } }'
    #[arg(long, verbatim_doc_comment)]
    pub array_to_bytes_codec: Option<String>,

    /// Bytes to bytes codecs.
    ///
    /// A JSON string holding bytes to bytes codec configurations.
    ///
    /// Examples:
    ///   '[ { "name": "blosc", "configuration": { "cname": "blosclz", "clevel": 9, "shuffle": "bitshuffle", "typesize": 2, "blocksize": 0 } } ]'
    ///   '[ { "name": "gzip", "configuration": { "level": 3 } } ]'
    #[arg(long, verbatim_doc_comment)]
    pub bytes_to_bytes_codecs: Option<String>,
}

#[must_use]
pub fn get_array_builder_reencode<TStorage>(
    encoding_args: &ZarrReEncodingArgs,
    array: &Array<TStorage>,
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

    // let shard_shape = encoding_args.shard_shape.map(|shard_shape| {
    //     std::iter::zip(shard_shape, array.shape())
    //         .map(|(&s, &a)| if s == 0 { a } else { std::cmp::min(s, a) })
    //         .zip(&chunk_shape)
    //         .map(|(s, c)| {
    //             // the shard shape must be a multiple of the chunk shape
    //             c * ((s + c - 1) / c)
    //         })
    //         .collect::<Vec<u64>>()
    // });

    // // Set the chunk/shard shape to the array shape where it is 0, otherwise make it <= array shape
    // // Also ensure shard shape is a multiple of chunk shape
    // let chunk_shape: Vec<u64> = std::iter::zip(&encoding_args.chunk_shape, array.shape())
    //     .map(|(&c, &a)| if c == 0 { a } else { c })
    //     .collect();
    // let shard_shape: Option<Vec<u64>> = if let Some(shard_shape) = &encoding_args.shard_shape {
    //     let shard_shape = std::iter::zip(shard_shape, array.shape())
    //         .map(|(&s, &a)| if s == 0 { a } else { std::cmp::min(s, a) })
    //         .zip(&chunk_shape)
    //         .map(|(s, c)| {
    //             // the shard shape must be a multiple of the chunk shape
    //             c * ((s + c - 1) / c)
    //         })
    //         .collect();
    //     Some(shard_shape)
    // } else {
    //     None
    // };

    // // Get the "block shape", which is the shard shape if sharding, otherwise the chunk shape
    // let block_shape = if let Some(shard_shape) = &shard_shape {
    //     shard_shape
    // } else {
    //     &chunk_shape
    // };

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

    if let Some(separator) = encoding_args.separator {
        array_builder.chunk_key_encoding_default_separator(separator.try_into().unwrap());
    }

    if let Some(fill_value) = &encoding_args.fill_value {
        let fill_value_metadata = FillValueMetadata::try_from(fill_value.as_str()).unwrap();
        let fill_value = array
            .data_type()
            .fill_value_from_metadata(&fill_value_metadata)
            .unwrap();
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

pub fn do_reencode<TStorageOut: ReadableWritableStorageTraits + 'static>(
    array_in: &Array<FilesystemStore>,
    array_out: &Array<TStorageOut>,
    validate: bool,
    concurrent_chunks: Option<usize>,
) -> (f32, f32, f32, usize) {
    let start = SystemTime::now();
    let bytes_decoded = Mutex::new(0);
    let duration_read = Mutex::new(Duration::from_secs(0));
    let duration_write = Mutex::new(Duration::from_secs(0));
    let chunks = ArraySubset::new_with_shape(array_out.chunk_grid_shape().unwrap());
    let style =
        ProgressStyle::with_template("[{elapsed_precise}] [{bar}] ({pos}/{len}, ETA {eta})")
            .unwrap();
    let pb = ProgressBar::new(chunks.num_elements());
    pb.set_style(style);
    pb.set_position(0);

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

    let indices = chunks.indices();
    iter_concurrent_limit!(
        chunks_concurrent_limit,
        indices.into_par_iter(),
        for_each,
        |chunk_indices| {
            let chunk_subset = array_out.chunk_subset(&chunk_indices).unwrap();

            let start_read = SystemTime::now();
            let bytes = array_in.retrieve_array_subset(&chunk_subset).unwrap(); // NOTE: Max concurrency
            *duration_read.lock().unwrap() += start_read.elapsed().unwrap();
            *bytes_decoded.lock().unwrap() += bytes.len();

            if validate {
                let bytes_clone = bytes.clone();
                let start_write = SystemTime::now();
                array_out
                    .store_chunk_opt(&chunk_indices, bytes_clone, &codec_options)
                    .unwrap();
                *duration_write.lock().unwrap() += start_write.elapsed().unwrap();
                let bytes_out = array_out
                    .retrieve_chunk_opt(&chunk_indices, &codec_options)
                    .unwrap();
                assert!(bytes == bytes_out);
            } else {
                let start_write = SystemTime::now();
                array_out
                    .store_chunk_opt(&chunk_indices, bytes, &codec_options)
                    .unwrap();
                *duration_write.lock().unwrap() += start_write.elapsed().unwrap();
            }
            pb.inc(1);
        }
    );
    pb.finish_and_clear();

    if validate {
        println!("Validation successful");
    }

    let duration = start.elapsed().unwrap().as_secs_f32();
    let duration_read = duration_read.into_inner().unwrap().as_secs_f32();
    let duration_write = duration_write.into_inner().unwrap().as_secs_f32();
    let duration_read_write = duration_read + duration_write;
    let duration_read = duration_read * duration / duration_read_write;
    let duration_write = duration_write * duration / duration_read_write;

    (
        duration,
        duration_read,
        duration_write,
        bytes_decoded.into_inner().unwrap(),
    )
}
