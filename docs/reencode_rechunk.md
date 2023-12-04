# Reencode/Rechunk a Zarr v3 Array
`array.zarr` is a zarr array with the following encoding parameters:
 - data type `int16`
 - shape `[1243, 1403, 1510]`
 - chunk (shard) shape of `[120, 1408, 1536]`
 - array to bytes codec: `sharding_indexed` with `crc32c` on index
   - inner chunk shape `[30, 32, 32]`
   - `blosclz` compression (level 9) with shuffling on linux, without on windows

```bash
# Reencode with a chunk shape of [32, 32, 32], shard shape of [128, 1408, 1536], and zlib compression.
zarrs_reencode \
--chunk-shape 32,32,32 `
--shard-shape 128,0,0 `
--bytes-to-bytes-codecs '[ { "name": "blosc", "configuration": { "cname": "zlib", "clevel": 9, "shuffle": "bitshuffle", "typesize": 2, "blocksize": 0 } } ]' \
array.zarr array_reencode.zarr
```
