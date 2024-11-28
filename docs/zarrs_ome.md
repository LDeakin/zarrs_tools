# zarrs_ome

Convert a Zarr array to an [OME-Zarr](https://ngff.openmicroscopy.org/0.5/index.html) 0.5 multiscales hierarchy.

> [!WARNING]>
> Conformance with the OME-Zarr 0.5 specification is not guaranteed, and input validation is limited.
> For example, it is possible to create multiscale arrays with nonconformant axis ordering.

`zarrs_ome` creates a multi-resolution Zarr V3 array through various methods:
 - Gaussian image pyramid
 - Mean downsampling
 - Mode downsampling (for discrete data)

The downsample factor defaults to 2 on all axes (careful if data includes channels!).
The physical size and units of the array elements can be set explicitly.
The array can be reencoded when output to OME-Zarr.

## Installation
`zarrs_ome` is installed with the `ome` feature of `zarrs_tools`.

### Prebuilt Binaries
```shell
# Requires cargo-binstall https://github.com/cargo-bins/cargo-binstall
cargo binstall zarrs_tools
```

### From Source
```shell
cargo install --features=ome zarrs_tools
```

## Usage

<details>
<summary>zarrs_ome --help</summary>

```text
Convert a Zarr array to an OME-Zarr multiscales hierarchy

Usage: zarrs_ome [OPTIONS] <INPUT> <OUTPUT> [DOWNSAMPLE_FACTOR]...

Arguments:
  <INPUT>
          The input array path

  <OUTPUT>
          The output group path

  [DOWNSAMPLE_FACTOR]...
          The downsample factor per axis, comma separated.
          
          Defaults to 2 on each axis.

Options:
      --ome-zarr-version <OME_ZARR_VERSION>
          [default: 0.5]

          Possible values:
          - 0.5: https://ngff.openmicroscopy.org/0.5/

      --max-levels <MAX_LEVELS>
          Maximum number of downsample levels
          
          [default: 10]

      --physical-size <PHYSICAL_SIZE>
          Physical size per axis, comma separated

      --physical-units <PHYSICAL_UNITS>
          Physical units per axis, comma separated.
          
          Set to "channel" for a channel axis.

      --name <NAME>
          OME Zarr dataset name

      --discrete
          Set to true for discrete data.
          
          Performs majority downsampling instead of creating a Gaussian image pyramid or mean downsampling.

      --gaussian-sigma <GAUSSIAN_SIGMA>
          The Gaussian "sigma" to apply when creating a Gaussian image pyramid per axis, comma separated.
          
          This is typically set to 0.5 times the downsample factor for each axis. If omitted, then mean downsampling is applied.
          
          Ignored for discrete data.

      --gaussian-kernel-half-size <GAUSSIAN_KERNEL_HALF_SIZE>
          The Gaussian kernel half size per axis, comma separated.
          
          If omitted, defaults to ceil(3 * sigma).
          
          Ignored for discrete data or if --gaussian-sigma is not set.

      --exists <EXISTS>
          Behaviour if the output exists
          
          [default: erase]

          Possible values:
          - erase:     Erase the output
          - overwrite: Overwrite existing files. Useful if the output includes additional non-zarr files to be preserved. May fail if changing the encoding
          - exit:      Exit if the output already exists

      --group-attributes <GROUP_ATTRIBUTES>
          Attributes (optional).
          
          JSON holding group attributes.

  -d, --data-type <DATA_TYPE>
          The data type as a string
          
          Valid data types:
            - bool
            - int8, int16, int32, int64
            - uint8, uint16, uint32, uint64
            - float16, float32, float64, bfloat16
            - complex64, complex 128
            - r* (raw bits, where * is a multiple of 8)

  -f, --fill-value <FILL_VALUE>
          Fill value. See <https://zarr-specs.readthedocs.io/en/latest/v3/core/v3.0.html#fill-value>
          
          The fill value must be compatible with the data type.
          
          Examples:
            int/uint: 0 100 -100
            float: 0.0 "NaN" "Infinity" "-Infinity"
            r*: "[0, 255]"

      --separator <SEPARATOR>
          The chunk key encoding separator. Either . or /

  -c, --chunk-shape <CHUNK_SHAPE>
          Chunk shape. A comma separated list of the chunk size along each array dimension.
          
          If any dimension has size zero, it will be set to match the array shape.

  -s, --shard-shape <SHARD_SHAPE>
          Shard shape. A comma separated list of the shard size along each array dimension.
          
          If specified, the array is encoded using the sharding codec.
          If any dimension has size zero, it will be set to match the array shape.

      --array-to-array-codecs <ARRAY_TO_ARRAY_CODECS>
          Array to array codecs.
          
          JSON holding an array of array to array codec metadata.
          
          Examples:
            '[ { "name": "transpose", "configuration": { "order": [0, 2, 1] } } ]'
            '[ { "name": "bitround", "configuration": { "keepbits": 9 } } ]'

      --array-to-bytes-codec <ARRAY_TO_BYTES_CODEC>
          Array to bytes codec.
          
          JSON holding array to bytes codec metadata.
          
          Examples:
            '{ "name": "bytes", "configuration": { "endian": "little" } }'
            '{ "name": "pcodec", "configuration": { "level": 12 } }'
            '{ "name": "zfp", "configuration": { "mode": "fixedprecision", "precision": 19 } }'

      --bytes-to-bytes-codecs <BYTES_TO_BYTES_CODECS>
          Bytes to bytes codecs.
          
          JSON holding an array bytes to bytes codec configurations.
          
          Examples:
            '[ { "name": "blosc", "configuration": { "cname": "blosclz", "clevel": 9, "shuffle": "bitshuffle", "typesize": 2, "blocksize": 0 } } ]'
            '[ { "name": "bz2", "configuration": { "level": 9 } } ]'
            '[ { "name": "crc32c" } ]'
            '[ { "name": "gzip", "configuration": { "level": 9 } } ]'
            '[ { "name": "zstd", "configuration": { "level": 22, "checksum": false } } ]'

      --dimension-names <DIMENSION_NAMES>
          Dimension names (optional). Comma separated.

      --attributes <ATTRIBUTES>
          Attributes (optional).
          
          JSON holding array attributes.

      --attributes-append <ATTRIBUTES_APPEND>
          Attributes to append (optional).
          
          JSON holding array attributes.

      --chunk-limit <CHUNK_LIMIT>
          The maximum number of chunks concurrently processed.
          
          By default, this is set to the number of CPUs. Consider reducing this for images with large chunk sizes or on systems with low memory availability.

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

</details>

## Examples

### Match Input Encoding
```bash
zarrs_ome \
    --name "ABC-123" \
    --physical-size 2.0,2.0,2.0 \
    --physical-units micrometer,micrometer,micrometer \
    array.zarr array.ome.zarr
```

```text
[00:00:00/00:00:00] 0 [1243, 1403, 1510] array.ome.zarr/0 rw:0.00/0.76 p:0.00
[00:00:14/00:00:14] 1 [621, 701, 755] array.ome.zarr/1 rw:1.95/0.51 p:12.24
[00:00:01/00:00:01] 2 [310, 350, 377] array.ome.zarr/2 rw:0.62/0.13 p:3.58
[00:00:00/00:00:00] 3 [155, 175, 188] array.ome.zarr/3 rw:0.06/0.01 p:0.26
[00:00:00/00:00:00] 4 [77, 87, 94] array.ome.zarr/4 rw:0.00/0.00 p:0.03
[00:00:00/00:00:00] 5 [38, 43, 47] array.ome.zarr/5 rw:0.00/0.00 p:0.01
[00:00:00/00:00:00] 6 [19, 21, 23] array.ome.zarr/6 rw:0.00/0.00 p:0.01
[00:00:00/00:00:00] 7 [9, 10, 11] array.ome.zarr/7 rw:0.00/0.00 p:0.00
[00:00:00/00:00:00] 8 [4, 5, 5] array.ome.zarr/8 rw:0.00/0.00 p:0.00
[00:00:00/00:00:00] 9 [2, 2, 2] array.ome.zarr/9 rw:0.00/0.00 p:0.00
[00:00:00/00:00:00] 10 [1, 1, 1] array.ome.zarr/10 rw:0.00/0.00 p:0.00
```

### Change Encoding and Downsampling Factor
```bash
zarrs_ome \
    --name "ABC-123" \
    --physical-size 2.0,2.0,2.0 \
    --physical-units micrometer,micrometer,micrometer \
    --shard-shape 256,256,256 \
    --chunk-shape 32,32,32 \
    array.zarr array.ome.zarr 1,4,4
```

```text
[00:00:01/00:00:01] 0 [1243, 1403, 1510] array.ome.zarr/0 rw:25.09/24.50 p:0.00
[00:00:12/00:00:12] 1 [1243, 350, 377] array.ome.zarr/1 rw:5.51/1.21 p:26.79
[00:00:00/00:00:00] 2 [1243, 87, 94] array.ome.zarr/2 rw:0.47/0.13 p:2.97
[00:00:00/00:00:00] 3 [1243, 21, 23] array.ome.zarr/3 rw:0.07/0.00 p:0.16
[00:00:00/00:00:00] 4 [1243, 5, 5] array.ome.zarr/4 rw:0.01/0.00 p:0.02
[00:00:00/00:00:00] 5 [1243, 1, 1] array.ome.zarr/5 rw:0.01/0.00 p:0.00
```
