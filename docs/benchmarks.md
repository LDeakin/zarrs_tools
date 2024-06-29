
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
- [`LDeakin/zarrs`](https://github.com/LDeakin/zarrs) v0.15.0-beta.1 (Rust 1.79.0) via [`LDeakin/zarrs_tools`](https://github.com/LDeakin/zarrs_tools) 0.4.3 (81c580e)
  - Benchmark executable: [zarrs_benchmark_read_sync](https://github.com/LDeakin/zarrs_tools/blob/main/src/bin/zarrs_benchmark_read_sync.rs)
  - Benchmark executable: [zarrs_benchmark_read_async](https://github.com/LDeakin/zarrs_tools/blob/main/src/bin/zarrs_benchmark_read_async.rs)
- [`google/tensorstore`](https://github.com/google/tensorstore) v0.1.61 (Python 3.12.3)
  - Benchmark script: <https://github.com/LDeakin/zarrs_tools/blob/main/scripts/tensorstore_python_benchmark_read_async.py>
- [`zarr-developers/zarr-python`](https://github.com/zarr-developers/zarr-python) 3.0.0a0 (Python 3.12.3)
  - Benchmark script: <https://github.com/LDeakin/zarrs_tools/blob/main/scripts/zarr_python_benchmark_read_async.py>

> [!CAUTION]
> Python benchmarks (tensorstore and zarr-python) are subject to the overheads of Python and may not be using an optimal API for each zarr implementation.

## Read Benchmarks

### Entire Array
This benchmark measures the minimum time and memory used to read an entire dataset into memory.
 - These are best of 3 measurements
 - The disk cache is cleared between each measurement

```bash
python3 ./scripts/run_benchmark_read_all.py
```

| Image                              |   Time (s)<br>zarrs<br>rust |   <br>zarrs<br>rust<br>async |   <br>tensorstore<br>python |   <br>zarr<br>python |   Memory (GB)<br>zarrs<br>rust |   <br>zarrs<br>rust<br>async |   <br>tensorstore<br>python |   <br>zarr<br>python |
|:-----------------------------------|----------------------------:|-----------------------------:|----------------------------:|---------------------:|-------------------------------:|-----------------------------:|----------------------------:|---------------------:|
| data/benchmark.zarr                |                        2.84 |                         9.79 |                        3.10 |                49.70 |                           8.42 |                         8.40 |                        8.60 |                15.52 |
| data/benchmark_compress.zarr       |                        2.70 |                        13.86 |                        2.69 |                70.55 |                           8.44 |                         8.41 |                        8.54 |                19.14 |
| data/benchmark_compress_shard.zarr |                        1.37 |                         2.21 |                        2.11 |                35.09 |                           8.64 |                         8.57 |                        8.94 |                27.33 |

### Chunk-By-Chunk

This benchmark measures the the minimum time and memory to read a dataset chunk-by-chunk into memory.
 - These are best of 1 measurements
 - The disk cache is cleared between each measurement

```bash
python3 ./scripts/run_benchmark_read_chunks.py
```

| Image                              |   Concurrency |   Time (s)<br>zarrs<br>rust |   <br>zarrs<br>rust<br>async |   <br>tensorstore<br>python |   <br>zarr<br>python |   Memory (GB)<br>zarrs<br>rust |   <br>zarrs<br>rust<br>async |   <br>tensorstore<br>python |   <br>zarr<br>python |
|:-----------------------------------|--------------:|----------------------------:|-----------------------------:|----------------------------:|---------------------:|-------------------------------:|-----------------------------:|----------------------------:|---------------------:|
| data/benchmark.zarr                |             1 |                       28.55 |                        55.48 |                       51.17 |                81.21 |                           0.02 |                         0.01 |                        0.10 |                 0.10 |
| data/benchmark.zarr                |             2 |                       16.08 |                        32.79 |                       29.85 |                66.76 |                           0.03 |                         0.02 |                        0.31 |                 0.32 |
| data/benchmark.zarr                |             4 |                        8.49 |                        18.02 |                       23.13 |                66.23 |                           0.03 |                         0.02 |                        0.31 |                 0.31 |
| data/benchmark.zarr                |             8 |                        4.59 |                        10.49 |                       20.36 |                64.06 |                           0.03 |                         0.02 |                        0.31 |                 0.31 |
| data/benchmark.zarr                |            16 |                        2.84 |                         8.42 |                       18.84 |                57.60 |                           0.03 |                         0.02 |                        0.33 |                 0.32 |
| data/benchmark.zarr                |            32 |                        2.83 |                         7.94 |                       16.80 |                53.58 |                           0.03 |                         0.03 |                        0.33 |                 0.32 |
| data/benchmark_compress.zarr       |             1 |                       20.53 |                        36.31 |                       45.32 |                93.08 |                           0.03 |                         0.02 |                        0.10 |                 0.14 |
| data/benchmark_compress.zarr       |             2 |                       12.37 |                        19.87 |                       26.66 |                86.26 |                           0.03 |                         0.02 |                        0.31 |                 0.34 |
| data/benchmark_compress.zarr       |             4 |                        7.21 |                        11.16 |                       22.19 |                85.91 |                           0.03 |                         0.02 |                        0.31 |                 0.33 |
| data/benchmark_compress.zarr       |             8 |                        3.92 |                         7.34 |                       19.56 |                79.25 |                           0.03 |                         0.03 |                        0.33 |                 0.33 |
| data/benchmark_compress.zarr       |            16 |                        2.33 |                         7.13 |                       18.29 |                70.87 |                           0.03 |                         0.04 |                        0.32 |                 0.33 |
| data/benchmark_compress.zarr       |            32 |                        2.29 |                         6.90 |                       16.41 |                65.28 |                           0.04 |                         0.07 |                        0.34 |                 0.33 |
| data/benchmark_compress_shard.zarr |             1 |                        2.06 |                         2.08 |                        3.24 |                25.45 |                           0.37 |                         0.37 |                        0.63 |                 1.17 |
| data/benchmark_compress_shard.zarr |             2 |                        1.49 |                         1.54 |                        2.29 |                29.03 |                           0.70 |                         0.70 |                        0.88 |                 1.95 |
| data/benchmark_compress_shard.zarr |             4 |                        1.37 |                         1.35 |                        1.96 |                31.36 |                           1.30 |                         1.30 |                        1.12 |                 3.61 |
| data/benchmark_compress_shard.zarr |             8 |                        1.27 |                         1.32 |                        1.91 |                33.57 |                           2.30 |                         2.17 |                        1.97 |                 6.99 |
| data/benchmark_compress_shard.zarr |            16 |                        1.28 |                         1.32 |                        1.90 |                34.94 |                           4.55 |                         3.73 |                        1.82 |                13.78 |
| data/benchmark_compress_shard.zarr |            32 |                        1.34 |                         1.52 |                        2.22 |                35.25 |                           6.93 |                         6.71 |                        2.82 |                27.38 |

## Round Trip Benchmarks
TODO
