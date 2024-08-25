# zarrs_reencode

Reencode/rechunk a Zarr V2/V3 to a Zarr v3 array.

## Installation
`zarrs_reencode` packaged by default with `zarrs_tools` and requires no extra features.

```
cargo install zarrs_tools
```

## Example
Reencode `array.zarr` (`uint16`) with:
 - a chunk shape of [32, 32, 32],
 - a shard shape of [128, 128, 0]
   - the last dimension of the shard shape will match the array shape to the nearest multiple of the chunk shape
 - level 9 blosclz compression with bitshuffling
 - an input chunk cache with a size of 1GB

```bash
zarrs_reencode \
--cache-size 1000000000 \
--chunk-shape 32,32,32 \
--shard-shape 128,128,0 \
--bytes-to-bytes-codecs '[ { "name": "blosc", "configuration": { "cname": "blosclz", "clevel": 9, "shuffle": "bitshuffle", "typesize": 2, "blocksize": 0 } } ]' \
array.zarr array_reencode.zarr
```
