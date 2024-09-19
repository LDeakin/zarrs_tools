
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
- [`LDeakin/zarrs`](https://github.com/LDeakin/zarrs) v0.17.0-beta.2 (26b90dd6) (Rust 1.81.0) via [`LDeakin/zarrs_tools`](https://github.com/LDeakin/zarrs_tools) 0.6.0 (343d978)
  - Benchmark executable: [zarrs_benchmark_read_sync](https://github.com/LDeakin/zarrs_tools/blob/main/src/bin/zarrs_benchmark_read_sync.rs)
  - ~~Benchmark executable: [zarrs_benchmark_read_async](https://github.com/LDeakin/zarrs_tools/blob/main/src/bin/zarrs_benchmark_read_async.rs)~~
- [`google/tensorstore`](https://github.com/google/tensorstore) v0.1.65 (Python 3.12.3)
  - Benchmark script: <https://github.com/LDeakin/zarrs_tools/blob/main/scripts/tensorstore_python_benchmark_read_async.py>
- [`zarr-developers/zarr-python`](https://github.com/zarr-developers/zarr-python) 3.0.0a4 (Python 3.12.3)
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

 > [!IMPORTANT]  
 > `zarr-python` does not use multiple cores with an `array[:]` call.
 > This benchmark needs to be reevaluated when `zarr-python` v3 supports multithreading directly or through a supporting library (e.g. `dask` via `xarray`).

![read all benchmark image](./benchmark_read_all.svg)

[Table of raw measurements (benchmarks_read_all.md)](./benchmark_read_all.md)

### Chunk-By-Chunk

This benchmark measures the the minimum time and memory to read a dataset chunk-by-chunk into memory.
 - These are best of 1 measurements
 - The disk cache is cleared between each measurement

```bash
python3 ./scripts/run_benchmark_read_chunks.py
```

 > [!IMPORTANT]  
 > `zarr-python` does not seem to scale up with multiple concurrent chunk requests and uses far more memory than competing implementations.
 > This needs to be reviewed.

![read chunks benchmark image](./benchmark_read_chunks.svg)

[Table of raw measurements (benchmarks_read_chunks.md)](./benchmark_read_chunks.md)

## Round Trip Benchmarks
TODO
