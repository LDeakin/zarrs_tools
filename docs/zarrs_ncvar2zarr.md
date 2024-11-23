# zarrs_ncvar2zarr

Convert a NetCDF variable to a Zarr v3 array.
Multi-file variables are supported.


## Installation
`zarrs_ncvar2zarr` is installed with the `ncvar2zarr` feature of `zarrs_tools`.

### Prebuilt Binaries
```shell
# Requires cargo-binstall https://github.com/cargo-bins/cargo-binstall
cargo binstall zarrs_tools
```

### From Source
```shell
cargo install --features=ncvar2zarr zarrs_tools
```

## Usage
<details>
<summary>zarrs_ncvar2zarr --help</summary>

```text
Convert a netCDF variable to a Zarr V3 array

Usage: zarrs_ncvar2zarr [OPTIONS] --fill-value <FILL_VALUE> --chunk-shape <CHUNK_SHAPE> <INPUT> <VARIABLE> <OUT>

Arguments:
  <INPUT>
          The path to a netCDF file or a directory of netcdf files

  <VARIABLE>
          The name of the netCDF variable

  <OUT>
          The output directory for the zarr array

Options:
  -f, --fill-value <FILL_VALUE>
          Fill value. See https://zarr-specs.readthedocs.io/en/latest/v3/core/v3.0.html#fill-value
          
          The fill value must be compatible with the data type.
          
          Examples:
            int/uint: 0 100 -100
            float: 0.0 "NaN" "Infinity" "-Infinity"
            r*: "[0, 255]"

      --separator <SEPARATOR>
          The chunk key encoding separator. Either . or /
          
          [default: /]

  -c, --chunk-shape <CHUNK_SHAPE>
          Chunk shape. A comma separated list of the chunk size along each array dimension.
          
          If any dimension has size zero, it will be set to match the array shape.

  -s, --shard-shape <SHARD_SHAPE>
          Shard shape (optional). A comma separated list of the shard size along each array dimension.
          
          If specified, the array is encoded using the sharding codec.
          If any dimension has size zero, it will be set to match the array shape.

      --array-to-array-codecs <ARRAY_TO_ARRAY_CODECS>
          Array to array codecs (optional).
          
          JSON holding an array of array to array codec metadata.
          
          Examples:
            '[ { "name": "transpose", "configuration": { "order": [0, 2, 1] } } ]'
            '[ { "name": "bitround", "configuration": { "keepbits": 9 } } ]'

      --array-to-bytes-codec <ARRAY_TO_BYTES_CODEC>
          Array to bytes codec (optional).
          
          JSON holding array to bytes codec metadata.
          If unspecified, this defaults to the `bytes` codec.
          
          The sharding codec can be used by setting `shard_shape`, but this can also be done explicitly here.
          
          Examples:
            '{ "name": "bytes", "configuration": { "endian": "little" } }'
            '{ "name": "pcodec", "configuration": { "level": 12 } }'
            '{ "name": "zfp", "configuration": { "mode": "fixedprecision", "precision": 19 } }'

      --bytes-to-bytes-codecs <BYTES_TO_BYTES_CODECS>
          Bytes to bytes codecs (optional).
          
          JSON holding an array of bytes to bytes codec configurations.
          
          Examples:
            '[ { "name": "blosc", "configuration": { "cname": "blosclz", "clevel": 9, "shuffle": "bitshuffle", "typesize": 2, "blocksize": 0 } } ]'
            '[ { "name": "bz2", "configuration": { "level": 9 } } ]'
            '[ { "name": "crc32c" ]'
            '[ { "name": "gzip", "configuration": { "level": 9 } } ]'
            '[ { "name": "zstd", "configuration": { "level": 22, "checksum": false } } ]'

      --attributes <ATTRIBUTES>
          Attributes (optional).
          
          JSON holding array attributes.

      --concurrent-chunks <CONCURRENT_CHUNKS>
          Number of concurrent chunks

      --memory-test
          Write to memory

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

</details>

## Example

`tomoLoRes_nc` is a directory of netCDF files, each containing a "tomo" 3D variable, which has been split along dimension 0
 - (depth, height, width) = `(1209, 480, 480)`
 - data type = `uint16`

```bash
tree --du -h tomoLoRes_nc
```
```text
[532M]  tomoLoRes_nc
├── [528M]  block00000000.nc
└── [4.0M]  block00000001.nc
```

With the following command, the image is encoded as a zarr array with the `sharding` codec with a *shard shape* of `(128, 480, 480)`
- inner chunks in each shard have a *chunk shape* of `(32, 32, 32)`
- inner chunks are compressed using the `blosc` codec

```bash
zarrs_ncvar2zarr \
--fill-value -32768 \
--separator '.' \
--chunk-shape 32,32,32 \
--shard-shape 128,0,0 \
--bytes-to-bytes-codecs '[ { "name": "blosc", "configuration": { "cname": "blosclz", "clevel": 9, "shuffle": "bitshuffle", "typesize": 2, "blocksize": 0 } } ]' \
tomoLoRes_nc \
tomo \
tomoLoRes_nc.zarr
```

```bash
tree --du -h tomoLoRes_nc.zarr
[329M]  tomoLoRes_nc.zarr
├── [ 30M]  c.0.0.0
├── [ 35M]  c.1.0.0
├── [ 36M]  c.2.0.0
├── [ 36M]  c.3.0.0
├── [ 36M]  c.4.0.0
├── [ 36M]  c.5.0.0
├── [ 36M]  c.6.0.0
├── [ 36M]  c.7.0.0
├── [ 35M]  c.8.0.0
├── [ 14M]  c.9.0.0
└── [1.5K]  zarr.json
```
