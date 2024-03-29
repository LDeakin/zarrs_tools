# zarrs_tools 

[![Latest Version](https://img.shields.io/crates/v/zarrs_tools.svg)](https://crates.io/crates/zarrs_tools)
![msrv](https://img.shields.io/crates/msrv/zarrs_tools)
[![build](https://github.com/LDeakin/zarrs_tools/actions/workflows/ci.yml/badge.svg)](https://github.com/LDeakin/zarrs_tools/actions/workflows/ci.yml)

Various tools for creating and manipulating [Zarr v3](https://zarr.dev) data with the [zarrs](https://github.com/LDeakin/zarrs) rust crate.

**zarrs_tools is experimental and in limited production use. Correctness issues with zarrs affecting past versions of zarrs_tools are [detailed here](https://docs.rs/zarrs/latest/zarrs/#correctness-issues-with-past-versions).**

[Changelog (CHANGELOG.md)](https://github.com/LDeakin/zarrs_tools/blob/main/CHANGELOG.md)

## Tools
- [zarrs_reencode](https://github.com/LDeakin/zarrs_tools/blob/main/docs/zarrs_reencode.md): reencode an array.
  - Can change the chunk size, shard size, codecs, fill value, chunk key encoding separator, and attributes.
- [zarrs_binary2zarr](https://github.com/LDeakin/zarrs_tools/blob/main/docs/zarrs_binary2zarr.md) (feature `binary2zarr`): create an array from piped binary data.
- [zarrs_ncvar2zarr](https://github.com/LDeakin/zarrs_tools/blob/main/docs/zarrs_ncvar2zarr.md) (feature `ncvar2zarr`): convert a netCDF variable to an array.
  - Supports multi-file datasets where a variable has been split along a single dimension.

The following tools are highly experimental and have had limited production testing:

- [zarrs_filter](https://github.com/LDeakin/zarrs_tools/blob/main/docs/zarrs_filter.md) (feature `filter`): apply simple image filters (transformations) to an array.
- [zarrs_ome](https://github.com/LDeakin/zarrs_tools/blob/main/docs/zarrs_ome.md) (feature `ome`): convert an array to [OME-Zarr](https://ngff.openmicroscopy.org/latest/index.html).
  - Outputs OME-Zarr `0.5-dev`. This revision is currently recognised by [Neuroglancer](https://github.com/google/neuroglancer) for Zarr V3.

See [docs/](https://github.com/LDeakin/zarrs_tools/blob/main/docs/) for tool documentation.

## `zarrs` Benchmarking
- `zarrs_reencode`: suitable for round trip benchmarking.
- `zarrs_benchmark_read_sync` (feature `benchmark`): benchmark the zarrs sync API.
- `zarrs_benchmark_read_async` (feature `benchmark`): benchmark the zarrs async API.

See [docs/benchmarks.md](https://github.com/LDeakin/zarrs_tools/blob/main/docs/benchmarks.md) for some benchmark measurements.

## Install

### From [crates.io](https://crates.io/crates/zarrs_tools)
```bash
cargo install --all-features zarrs_tools
```

### From [source](https://github.com/LDeakin/zarrs_tools)
```bash
cargo install --all-features --path .
# cargo install --all-features --git https://github.com/LDeakin/zarrs_tools
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
cargo install zarrs_tools --all-features --features zarrs/bitround,zarrs/zfp,zarrs/bz2,zarrs/pcodec
```

## Licence
`zarrs_tools` is licensed under either of
 - the Apache License, Version 2.0 [LICENSE-APACHE](./LICENCE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0> or
 - the MIT license [LICENSE-MIT](./LICENCE-MIT) or <http://opensource.org/licenses/MIT>, at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
