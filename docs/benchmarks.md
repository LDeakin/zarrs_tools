
# Benchmarks

## Benchmark Data
Benchmark data is generated with `scripts/generate_benchmark_array.py` as follows
```bash
./scripts/generate_benchmark_array.py data/benchmark.zarr
./scripts/generate_benchmark_array.py --compress data/benchmark_compress.zarr
./scripts/generate_benchmark_array.py --compress --shard data/benchmark_compress_shard.zarr
```
- Data type: `uint16`
- Array shape: $1024^3$
- Chunk/shard shape:
  - ~~`--shard`~~: $32^3$
  - `--shard`: $128\times1024\times1024$
    - inner chunk shape: $32^3$
- Bytes to bytes codec for chunks/inner chunks:
  - ~~`--compress`~~: none
  - `--compress`: `blosclz` level 9 with bitshuffling
    - compresses from `2.147GB` to `0.279GB`.

## Benchmark System
- Ubuntu 22.04 (in Windows 11 WSL2)
- Ryzen 5900X
- 64GB DDR4 3600MHz (16-19-19-39)
- 2TB Samsung 990 Pro
- Rust 1.75.0

## Implementation Versions Benchmarked
- zarrs_tools v0.3.0 installed with `RUSTFLAGS="-C target-cpu=native" cargo install --path .`
- tensorstore v0.1.53 installed with `pip install tensorstore`

## Comparative Benchmarks
Measurements are from a best of 3 run. The disk cache is not cleared between runs.
 > TODO: Check benchmark equivalence between implementations, evaluate more implementations, automate running/table creation, switch to plots?

### Read (entire array)
```bash
zarrs_benchmark_read_sync --read-all data/<array>.zarr
```
```bash
zarrs_benchmark_read_async --read-all data/<array>.zarr
```
```bash
./scripts/tensorstore_benchmark_read_async.py --read_all data/<array>.zarr
```

| Array                         | Zarrs (sync) | Zarrs (async) | Tensorstore |
|----------------------------   |--------------|---------------|-------------|
| benchmark.zarr                | 457.84ms     | 3835.24ms     | 611.25ms    |
| benchmark_compress.zarr       | 427.76ms     | 3810.88ms     | 545.48ms    |
| benchmark_compress_shard.zarr | 855.44ms     | 3594.28ms     | 563.19ms    |

### Read (chunk-by-chunk)
```bash
zarrs_benchmark_read_sync -c <concurrency> data/<array>.zarr
```
```bash
zarrs_benchmark_read_async -c <concurrency> data/<array>.zarr
```
```bash
./scripts/tensorstore_benchmark_read_async.py --concurrent_chunks <concurrency> data/<array>.zarr
```

#### benchmark.zarr
| Concurrent chunks | Zarrs (sync) | Zarrs (async) | Tensorstore |
|-------------------|--------------|---------------|-------------|
| 1                 | 225.52ms     | 2338.34ms     | 5763.46ms   |
| 2                 | 122.65ms     | 1403.08ms     | 4911.54ms   |
| 4                 |  80.71ms     | 1045.89ms     | 4456.76ms   |
| 6                 |  65.98ms     | 1121.15ms     | 4369.20ms   |
| 8                 |  63.22ms     | 1167.65ms     | 4295.48ms   |

#### benchmark_compress.zarr
| Concurrent chunks | Zarrs (sync) | Zarrs (async) | Tensorstore |
|-------------------|--------------|---------------|-------------|
| 1                 | 1423.97ms    | 3405.30ms     | 7027.45ms   |
| 2                 |  730.95ms    | 2352.12ms     | 4487.39ms   |
| 4                 |  407.31ms    | 2089.68ms     | 4637.75ms   |
| 6                 |  272.29ms    | 2037.68ms     | 4477.85ms   |
| 8                 |  231.69ms    | 2057.57ms     | 4659.14ms   |

#### benchmark_compress_shard.zarr
| Concurrent shards | Zarrs (sync) | Zarrs (async) | Tensorstore (async) |
|-------------------|--------------|---------------|---------------------|
| 1                 | 532.84ms     | 1584.01ms     | 573.81ms            |
| 2                 | 386.09ms     | 1532.21ms     | 573.21ms            |
| 4                 | 384.95ms     | 1493.33ms     | 601.14ms            |
| 6                 | 383.70ms     | 1551.53ms     | 669.18ms            |
| 8                 | 425.26ms     | 1512.71ms     | 689.08ms            |

### Write (entire array)
 > TODO

### Write (chunk-by-chunk)
 > TODO

## Zarrs Benchmarks

### Round Trip
```bash
zarrs_reencode --concurrent-chunks 8 data/benchmark.zarr data/benchmark_copy0.zarr
zarrs_reencode --concurrent-chunks 8 data/benchmark_compress.zarr data/benchmark_compress_copy0.zarr
zarrs_reencode --concurrent-chunks 4 data/benchmark_compress_shard.zarr data/benchmark_compress_shard_copy0.zarr
```

| Array                         | Concurrency | Read                 | Write               | Total    |
|-------------------------------|-------------|----------------------|---------------------|----------|
| benchmark.zarr                | 8           |  69.03ms (31.11GB/s) | 461.53ms (4.65GB/s) | 513.78ms |
| benchmark_compress.zarr       | 8           | 256.17ms ( 8.38GB/s) | 516.23ms (4.16GB/s) | 772.39ms |
| benchmark_compress_shard.zarr | 4           | 403.72ms ( 5.32GB/s) | 485.06ms (4.43GB/s) | 888.77ms |

### Rechunk
```bash
zarrs_reencode --concurrent-chunks 8 --chunk-shape 32,32,32 --shard-shape 64,0,0 data/benchmark.zarr data/benchmark_copy1.zarr
zarrs_reencode --concurrent-chunks 8 --chunk-shape 32,32,32 --shard-shape 64,0,0 data/benchmark_compress.zarr data/benchmark_compress_copy1.zarr
zarrs_reencode --concurrent-chunks 4 --chunk-shape 32,32,32 --shard-shape 64,0,0 data/benchmark_compress_shard.zarr data/benchmark_compress_shard_copy1.zarr
```

| Array                         | Concurrency | Read                | Write               | Total     |
|-------------------------------|-------------|---------------------|---------------------|-----------|
| benchmark.zarr                | 8           | 455.04ms (4.72GB/s) | 594.05ms (3.61GB/s) | 1049.09ms |
| benchmark_compress.zarr       | 8           | 521.12ms (4.12GB/s) | 476.53ms (4.51GB/s) |  977.65ms |
| benchmark_compress_shard.zarr | 4           | 761.75ms (2.82GB/s) | 373.88ms (5.74GB/s) | 1135.63ms |
