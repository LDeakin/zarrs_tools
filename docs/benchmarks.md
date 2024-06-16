
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
This benchmark measures the minimum time and memory used to read an entire dataset into memory.
 - These are best of 3 measurements
 - The disk cache is cleared between each measurement

```bash
python3 ./scripts/run_benchmark_read_all.py
```

| Image                              |   Time (s)<br>zarrs<br>rust |   <br>tensorstore<br>python |   <br>zarr<br>python |   Memory (GB)<br>zarrs<br>rust |   <br>tensorstore<br>python |   <br>zarr<br>python |
|:-----------------------------------|----------------------------:|----------------------------:|---------------------:|-------------------------------:|----------------------------:|---------------------:|
| data/benchmark.zarr                |                        3.06 |                        3.11 |                52.99 |                           8.42 |                        8.59 |                15.49 |
| data/benchmark_compress.zarr       |                        2.87 |                        2.94 |                74.58 |                           8.44 |                        8.55 |                19.17 |
| data/benchmark_compress_shard.zarr |                        1.51 |                        2.17 |                39.97 |                           8.63 |                        8.88 |                27.22 |

### Chunk-By-Chunk
<details>
<summary>TODO: Need to review benchmark scripts for tensorstore/zarr-python, performance is not improving much with concurrency</summary>

This benchmark measures the the minimum time and memory to read a dataset chunk-by-chunk into memory.
 - These are best of 1 measurements
 - The disk cache is cleared between each measurement

```bash
python3 ./scripts/run_benchmark_read_chunks.py
```

| Image                              |   Concurrency |   Time (s)<br>zarrs<br>rust |   <br>tensorstore<br>python |   <br>zarr<br>python |   Memory (GB)<br>zarrs<br>rust |   <br>tensorstore<br>python |   <br>zarr<br>python |
|:-----------------------------------|--------------:|----------------------------:|----------------------------:|---------------------:|-------------------------------:|----------------------------:|---------------------:|
| data/benchmark.zarr                |             1 |                       27.8  |                       54.49 |                71.26 |                           0.03 |                        0.31 |                 0.31 |
| data/benchmark.zarr                |             2 |                       15.43 |                       31.12 |                69.41 |                           0.03 |                        0.3  |                 0.32 |
| data/benchmark.zarr                |             4 |                        8.27 |                       24.18 |                70.76 |                           0.02 |                        0.31 |                 0.32 |
| data/benchmark.zarr                |             8 |                        4.75 |                       21.51 |                67.68 |                           0.02 |                        0.32 |                 0.31 |
| data/benchmark.zarr                |            16 |                        2.76 |                       20.17 |                62.85 |                           0.02 |                        0.33 |                 0.32 |
| data/benchmark.zarr                |            32 |                        2.77 |                       17.86 |                56.93 |                           0.02 |                        0.34 |                 0.32 |
| data/benchmark_compress.zarr       |             1 |                       21.88 |                       49.23 |                83.36 |                           0.02 |                        0.31 |                 0.33 |
| data/benchmark_compress.zarr       |             2 |                       13.39 |                       27.82 |                88.61 |                           0.03 |                        0.3  |                 0.33 |
| data/benchmark_compress.zarr       |             4 |                        7.54 |                       23.41 |                90.04 |                           0.03 |                        0.32 |                 0.34 |
| data/benchmark_compress.zarr       |             8 |                        4.07 |                       20.48 |                82.76 |                           0.03 |                        0.31 |                 0.34 |
| data/benchmark_compress.zarr       |            16 |                        2.45 |                       19.37 |                74.4  |                           0.03 |                        0.34 |                 0.34 |
| data/benchmark_compress.zarr       |            32 |                        2.42 |                       17.46 |                68.15 |                           0.04 |                        0.35 |                 0.34 |
| data/benchmark_compress_shard.zarr |             1 |                        2.61 |                        3.14 |                25.93 |                           0.36 |                        0.58 |                 1.22 |
| data/benchmark_compress_shard.zarr |             2 |                        1.53 |                        2.29 |                30.23 |                           0.69 |                        0.89 |                 2.03 |
| data/benchmark_compress_shard.zarr |             4 |                        1.34 |                        1.98 |                32.54 |                           1.29 |                        1.12 |                 3.61 |
| data/benchmark_compress_shard.zarr |             8 |                        1.31 |                        1.85 |                34.62 |                           2.28 |                        1.2  |                 7.01 |
| data/benchmark_compress_shard.zarr |            16 |                        1.36 |                        1.77 |                36.18 |                           4.27 |                        2.13 |                13.77 |
| data/benchmark_compress_shard.zarr |            32 |                        1.38 |                        2.12 |                45.37 |                           6.91 |                        2.49 |                27.29 |

</details>

## Round Trip Benchmarks
TODO
