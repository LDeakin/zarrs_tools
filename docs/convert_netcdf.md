# Convert a Multi-File NetCDF Variable to a Zarr v3 Array
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

With the following command, the image is encoded as a zarr array with the `sharding` codec with a *shard shape* of `(128, 1024, 1024)`
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
