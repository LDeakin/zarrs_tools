# zarrs_binary2zarr

Create a Zarr V3 array from piped binary data.

## Installation
`zarrs_binary2zarr` is installed with the `binary2zarr` feature of `zarrs_tools`.

### Prebuilt Binaries
```shell
# Requires cargo-binstall https://github.com/cargo-bins/cargo-binstall
cargo binstall zarrs_tools
```

### From Source
```shell
cargo install --features=binary2zarr zarrs_tools
```

## Example
`chameleon_1024x1024x1080.uint16` is an uncompressed binary 3D image split into multiple files with
 - (depth, height, width) = `(1080, 1024, 1024)`
 - data type = `uint16`

```bash
tree --du -h chameleon_1024x1024x1080.uint16
```
```text
[2.1G]  chameleon_1024x1024x1080.uint16
├── [512M]  xaa
├── [512M]  xab
├── [512M]  xac
├── [512M]  xad
└── [112M]  xae
```
With the following command, the image is encoded as a zarr array with the `sharding` codec with a *shard shape* of `(128, 1024, 1024)`
- inner chunks in each shard have a *chunk shape* of `(32, 32, 32)`
- inner chunks are compressed using the `blosc` codec

```bash
cat chameleon_1024x1024x1080.uint16/* | \
zarrs_binary2zarr \
--data-type uint16 \
--fill-value 0 \
--separator '.' \
--array-shape 1080,1024,1024 \
--chunk-shape 32,32,32 \
--shard-shape 128,1024,1024 \
--bytes-to-bytes-codecs '[ { "name": "blosc", "configuration": { "cname": "blosclz", "clevel": 9, "shuffle": "bitshuffle", "typesize": 2, "blocksize": 0 } } ]' \
chameleon_1024x1024x1080.zarr
```

```bash
tree --du -h chameleon_1024x1024x1080.zarr
```
```text
[1.3G]  chameleon_1024x1024x1080.zarr
├── [152M]  c.0.0.0
├── [157M]  c.1.0.0
├── [157M]  c.2.0.0
├── [156M]  c.3.0.0
├── [152M]  c.4.0.0
├── [150M]  c.5.0.0
├── [152M]  c.6.0.0
├── [152M]  c.7.0.0
├── [ 67M]  c.8.0.0
└── [1.2K]  zarr.json
```
