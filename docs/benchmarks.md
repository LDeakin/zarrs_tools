
# Benchmarks
`array.zarr` is the same as used in the rechunk/reencode example.

## Benchmark System
- Ubuntu 22.04 (in Windows 11 WSL2)
- Ryzen 5900X
- 64GB DDR4 3600MHz (16-19-19-39)
- 2TB Samsung 990 Pro
- `zarrs_tools` installed with `RUSTFLAGS="-C target-feature=+avx2,+sse2" cargo install --path .`

## Read
```bash
zarrs_benchmark_read_sync -p 4 array.zarr
```

```bash
Decoded array.zarr (1252.56MB) in 1086.55ms (5709.50MB decoded @ 5.25GB/s)
```

## Round trip
```bash
zarrs_reencode -p 4 array.zarr array_copy.zarr
```

```bash
Reencode array.zarr (1252.5632MB) to array_copy.zarr (1252.5634MB) in 1786.47ms
        read in ~878.29ms (5709.50MB decoded @ 6.50GB/s)
        write in ~908.17ms (5709.50MB encoded @ 6.29GB/s)
```

## Round trip (uncompressed output)
```bash
zarrs_reencode -p 4  --bytes-to-bytes-codecs '[]' array.zarr array_copy.zarr
```

```bash
Reencode array.zarr (1252.5632MB) to array_copy.zarr (1956.4475MB) in 1971.09ms
        read in ~956.72ms (5709.50MB decoded @ 5.97GB/s)
        write in ~1014.37ms (5709.50MB encoded @ 5.63GB/s
```

## Rechunk
```bash
zarrs_reencode -p 4 --chunk-shape 32,32,32 --shard-shape 64,0,0 array.zarr array_rechunk.zarr
```

```bash
Reencode array.zarr (1252.5632MB) to array_rechunk.zarr (1252.3507MB) in 3768.89ms
        read in ~2802.99ms (5536.48MB decoded @ 1.98GB/s)
        write in ~965.90ms (5536.48MB encoded @ 5.73GB/s)
```

## Rechunk (uncompressed output)
```bash
zarrs_reencode -p 4  --chunk-shape 32,32,32 --shard-shape 64,0,0 --bytes-to-bytes-codecs '[]' array.zarr array_rechunk.zarr
```

```bash
Reencode array.zarr (1252.5632MB) to array_rechunk.zarr (1963.5664MB) in 3796.30ms
        read in ~2571.03ms (5536.48MB decoded @ 2.15GB/s)
        write in ~1225.27ms (5536.48MB encoded @ 4.52GB/s)
```
