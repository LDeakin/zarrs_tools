
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
    - compresses from `2.147GB` to `0.279GB`
- Size on disk
  - `data/benchmark.zarr`: 8.0G
  - `data/benchmark_compress.zarr`: 1.4G
  - `data/benchmark_compress_shard.zarr`: 1.1G

## Benchmark System
- Ryzen 5900X
- 64GB DDR4 3600MHz (16-19-19-39)
- 2TB Samsung 990 Pro
- Ubuntu 22.04 (in Windows 11 WSL2, swap disabled, 24GB available memory)
- Rust 1.76.0

## Implementation Versions Benchmarked
- zarrs_tools v0.3.0 installed with `RUSTFLAGS="-C target-cpu=native" cargo install --path .`
- tensorstore v0.1.53 installed with `pip install tensorstore`

## Comparative Benchmarks
 > TODO: Check benchmark equivalence between implementations, evaluate more implementations

### Read Entire Array
```bash
python3 ./scripts/run_benchmark_read_all.py
```

| Image                              |   Wall time (s)<br>zarrs<br>sync |   <br><br>async |   <br>tensorstore<br>async |   Memory usage (GB)<br>zarrs<br>sync |   <br><br>async |   <br>tensorstore<br>async |
|:-----------------------------------|---------------------------------:|----------------:|---------------------------:|-------------------------------------:|----------------:|---------------------------:|
| data/benchmark.zarr                |                             3    |           14.88 |                       3.29 |                                 8.41 |            8.4  |                       8.6  |
| data/benchmark_compress.zarr       |                             2.84 |           17.36 |                       2.76 |                                 8.44 |            8.41 |                       8.53 |
| data/benchmark_compress_shard.zarr |                             1.67 |            2.93 |                       2.66 |                                 8.63 |            8.63 |                       8.6  |


### Read Chunk-By-Chunk
```bash
python3 ./scripts/run_benchmark_read_chunks.py
```

| Image                              |   Concurrency |   Wall time (s)<br>zarrs<br>sync |   <br><br>async |   <br>tensorstore<br>async |   Memory usage (GB)<br>zarrs<br>sync |   <br><br>async |   <br>tensorstore<br>async |
|:-----------------------------------|--------------:|---------------------------------:|----------------:|---------------------------:|-------------------------------------:|----------------:|---------------------------:|
| data/benchmark.zarr                |             1 |                            25.23 |           55.17 |                      52.57 |                                 0.03 |            0.01 |                       0.51 |
| data/benchmark.zarr                |             2 |                            14.45 |           32.84 |                      30.98 |                                 0.03 |            0.01 |                       0.52 |
| data/benchmark.zarr                |             4 |                             7.87 |           18.28 |                      23.71 |                                 0.03 |            0.01 |                       0.51 |
| data/benchmark.zarr                |             8 |                             4.32 |           10.67 |                      20.98 |                                 0.03 |            0.02 |                       0.52 |
| data/benchmark.zarr                |            16 |                             2.71 |            8.03 |                      19.39 |                                 0.03 |            0.02 |                       0.52 |
| data/benchmark.zarr                |            32 |                             2.52 |            8.22 |                      18.58 |                                 0.03 |            0.03 |                       0.53 |
| data/benchmark_compress.zarr       |             1 |                            20.78 |           36.4  |                      46.78 |                                 0.03 |            0.02 |                       0.51 |
| data/benchmark_compress.zarr       |             2 |                            12.47 |           19.71 |                      27.16 |                                 0.03 |            0.02 |                       0.52 |
| data/benchmark_compress.zarr       |             4 |                             7.11 |           11.06 |                      22.32 |                                 0.03 |            0.02 |                       0.51 |
| data/benchmark_compress.zarr       |             8 |                             3.82 |            7.29 |                      20.01 |                                 0.03 |            0.03 |                       0.52 |
| data/benchmark_compress.zarr       |            16 |                             2.22 |            7.09 |                      18.72 |                                 0.04 |            0.04 |                       0.54 |
| data/benchmark_compress.zarr       |            32 |                             2.18 |            6.82 |                      17.72 |                                 0.04 |            0.07 |                       0.54 |
| data/benchmark_compress_shard.zarr |             1 |                             2.59 |            2.63 |                       2.71 |                                 0.37 |            0.4  |                       0.42 |
| data/benchmark_compress_shard.zarr |             2 |                             1.76 |            1.77 |                       2.31 |                                 0.7  |            0.76 |                       0.56 |
| data/benchmark_compress_shard.zarr |             4 |                             1.48 |            1.46 |                       2.31 |                                 1.29 |            1.24 |                       1.05 |
| data/benchmark_compress_shard.zarr |             8 |                             1.41 |            1.47 |                       2.57 |                                 2.37 |            2.29 |                       1.41 |
| data/benchmark_compress_shard.zarr |            16 |                             1.57 |            1.56 |                       2.85 |                                 4.34 |            3.99 |                       2.13 |
| data/benchmark_compress_shard.zarr |            32 |                             1.54 |            1.76 |                       3.15 |                                 6.54 |            6.9  |                       3.46 
