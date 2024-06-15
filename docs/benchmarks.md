
# Benchmarks

## Benchmark Data
Benchmark data is generated with `scripts/generate_benchmark_array.py` as follows
```bash
./scripts/generate_benchmark_array.py data/benchmark.zarr
./scripts/generate_benchmark_array.py --compress data/benchmark_compress.zarr
./scripts/generate_benchmark_array.py --compress --shard data/benchmark_compress_shard.zarr
```
- Data type: `uint16`
- Array shape: $1024\times2048\times2048$
- Chunk/shard shape:
  - Default: $32^3$
  - `--shard`: $512^3$ with $32^3$ inner chunk shape
- Bytes to bytes codec for chunks/inner chunks:
  - Default: none
  - `--compress`: `blosclz` level 9 with bitshuffling
- Size on disk
  - `data/benchmark.zarr`: 8.0G
  - `data/benchmark_compress.zarr`: 1.4G
  - `data/benchmark_compress_shard.zarr`: 1.1G

## Benchmark System
- AMD Ryzen 5900X
- 64GB DDR4 3600MHz (16-19-19-39)
- 2TB Samsung 990 Pro
- Ubuntu 22.04 (in Windows 11 WSL2, swap disabled, 32GB available memory)

## Implementations Benchmarked
- [`LDeakin/zarrs`](https://github.com/LDeakin/zarrs) v0.14 (Rust 1.79.0) via [`LDeakin/zarrs_tools`](https://github.com/LDeakin/zarrs_tools) 0.4.2
  - Benchmark executable: [zarrs_benchmark_read_sync](https://github.com/LDeakin/zarrs_tools/blob/main/src/bin/zarrs_benchmark_read_sync.rs)
  - ~~Benchmark executable: [zarrs_benchmark_read_async](https://github.com/LDeakin/zarrs_tools/blob/main/src/bin/zarrs_benchmark_read_async.rs)~~
- [`google/tensorstore`](https://github.com/google/tensorstore) v0.1.61 (Python 3.12.3)
  - Benchmark script: <https://github.com/LDeakin/zarrs_tools/blob/main/scripts/tensorstore_python_benchmark_read_async.py>
- [`zarr-developers/zarr-python`](https://github.com/zarr-developers/zarr-python) 3.0.0a0 (Python 3.12.3)
  - Benchmark script: <https://github.com/LDeakin/zarrs_tools/blob/main/scripts/zarr_python_benchmark_read_async.py>

> [!CAUTION]
> Python benchmarks are subject to the overheads of Python and may not be using an optimal API for each zarr implementation.

## Read Benchmarks

### Entire Array
This benchmark measures the time and maximum memory used to read an entire dataset into memory.
 - These are best of 3 measurements
 - The disk cache is cleared between each measurement

```bash
python3 ./scripts/run_benchmark_read_all.py
```

| Image                              |   Time (s)<br>zarrs<br>rust |   <br>tensorstore<br>python |   <br>zarr<br>python |   Memory (GB)<br>zarrs<br>rust |   <br>tensorstore<br>python |   <br>zarr<br>python |
|:-----------------------------------|----------------------------:|----------------------------:|---------------------:|-------------------------------:|----------------------------:|---------------------:|
| data/benchmark.zarr                |                        2.95 |                        3.17 |                51.53 |                           8.42 |                        8.59 |                15.28 |
| data/benchmark_compress.zarr       |                        3    |                        2.83 |                74.82 |                           8.44 |                        8.53 |                19.14 |
| data/benchmark_compress_shard.zarr |                        1.47 |                        2.18 |                36.37 |                           8.63 |                        8.94 |                27.42 |

### Chunk-By-Chunk
This benchmark measures the time to read a dataset chunk-by-chunk into memory.
 - These are best of 1 measurements
 - The disk cache is cleared between each measurement
 - TODO: Need to review scripts for tensorstore/zarr-python, performance is not improving much with concurrency

```bash
python3 ./scripts/run_benchmark_read_chunks.py
```

| Image                              |   Concurrency |   Time (s)<br>zarrs<br>rust |   Memory (GB)<br>zarrs<br>rust |
|:-----------------------------------|--------------:|----------------------------:|-------------------------------:|
| data/benchmark.zarr                |             1 |                       27.12 |                           0.03 |
| data/benchmark.zarr                |             2 |                       15.15 |                           0.03 |
| data/benchmark.zarr                |             4 |                        8.58 |                           0.02 |
| data/benchmark.zarr                |             8 |                        4.74 |                           0.03 |
| data/benchmark.zarr                |            16 |                        2.84 |                           0.02 |
| data/benchmark.zarr                |            32 |                        2.8  |                           0.02 |
| data/benchmark_compress.zarr       |             1 |                       22.15 |                           0.02 |
| data/benchmark_compress.zarr       |             2 |                       13.47 |                           0.03 |
| data/benchmark_compress.zarr       |             4 |                        7.68 |                           0.03 |
| data/benchmark_compress.zarr       |             8 |                        4.16 |                           0.03 |
| data/benchmark_compress.zarr       |            16 |                        2.44 |                           0.03 |
| data/benchmark_compress.zarr       |            32 |                        2.42 |                           0.04 |
| data/benchmark_compress_shard.zarr |             1 |                        2.53 |                           0.36 |
| data/benchmark_compress_shard.zarr |             2 |                        1.58 |                           0.7  |
| data/benchmark_compress_shard.zarr |             4 |                        1.42 |                           1.29 |
| data/benchmark_compress_shard.zarr |             8 |                        1.5  |                           2.21 |
| data/benchmark_compress_shard.zarr |            16 |                        1.38 |                           4.46 |
| data/benchmark_compress_shard.zarr |            32 |                        1.5  |                           6.69 |

## Round Trip Benchmarks
TODO
