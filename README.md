# zarrs_tools &emsp; [![build](https://github.com/LDeakin/zarrs_tools/actions/workflows/ci.yml/badge.svg)](https://github.com/LDeakin/zarrs_tools/actions/workflows/ci.yml) [![Latest Version](https://img.shields.io/crates/v/zarrs_tools.svg)](https://crates.io/crates/zarrs_tools)

Various tools for creating and manipulating [Zarr v3](https://zarr.dev) data with the [zarrs](https://github.com/LDeakin/zarrs) rust crate.

**zarrs is experimental and in limited production use. Correctness issues with zarrs affecting past versions of zarrs_tools are [detailed here](https://docs.rs/zarrs/latest/zarrs/#correctness-issues-with-past-versions).**

[Changelog (CHANGELOG.md)](https://github.com/LDeakin/zarrs_tools/blob/main/CHANGELOG.md)

## Tools
- `zarrs_reencode`: reencode a Zarr v3 array. [Example](https://github.com/LDeakin/zarrs_tools/blob/main/docs/reencode_rechunk.md).
  - Can change the chunk size, shard size, compression, etc.
  - Suitable for round trip benchmarking
- `zarrs_binary2zarr`: create a Zarr v3 array from piped binary data. [Example](https://github.com/LDeakin/zarrs_tools/blob/main/docs/convert_binary.md).
- `zarrs_ncvar2zarr` (requires `ncvar2zarr` feature): convert a netCDF variable to a zarr V3 array. [Example](https://github.com/LDeakin/zarrs_tools/blob/main/docs/convert_netcdf.md).
- `zarrs_benchmark_read_sync`: Measure the time to read (decode) each chunk of an array using the zarrs sync API.
- `zarrs_benchmark_read_async`: Measure the time to read (decode) each chunk of an array using the zarrs async API.

See [docs/benchmarks.md](https://github.com/LDeakin/zarrs_tools/blob/main/docs/benchmarks.md) for some benchmark measurements.

## Install

### From [crates.io](https://crates.io/crates/zarrs_tools)
```bash
cargo install zarrs_tools
```

### From [source](https://github.com/LDeakin/zarrs_tools)
```bash
cargo install --path .
```

### Enabling SIMD intrinsics
Encoding and decoding performance may be improved with `avx2`/`sse2` enabled (if supported).

This can be enabled by compiling with either of:
 - `RUSTFLAGS="-C target-cpu=native"`
 - `RUSTFLAGS="-C target-feature=+avx2,+sse2"`

### Enabling non-default `zarrs` codecs
Non-default `zarrs` codecs (see [`zarrs` crate features](https://docs.rs/zarrs/latest/zarrs/#crate-features)) can be enabled by passing them as feature flags.

For example:
```bash
cargo install zarrs_tools --features zarrs/bitround,zarrs/zfp
```

## Licence
`zarrs_tools` is licensed under either of
 - the Apache License, Version 2.0 [LICENSE-APACHE](./LICENCE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0> or
 - the MIT license [LICENSE-MIT](./LICENCE-MIT) or <http://opensource.org/licenses/MIT>, at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
