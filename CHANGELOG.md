# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
 - [#12](https://github.com/LDeakin/zarrs_tools/pull/12) Bump netcdf to 0.10.2 by [@magnusuMET]
 - **Breaking**: Bump MSRV to 1.80
 - Bump `sysinfo` to 0.31
 - Bump `zarrs` to 0.17.0
 - `ncvar2zarr`:
   - Switch to output concurrency
   - **Breaking**: Rename `concurrent-blocks` argument to `concurrent-chunks`
   - **Breaking**: Removed `validate` argument
   - **Breaking**: Removed `concat-dim` argument. Concatenation is now only supported along the first (slowest varying) dimension
   - This tool is intended to be replaced by chunk manifests when the specification and `virtualizarr` matures
 - Move benchmark scripts and measurements to https://github.com/LDeakin/zarr_benchmarks

### Fixed
 - `zarrs_ome` fix axis unit parsing (broken in 0.5.3)

## [0.5.5] - 2024-07-31

### Added
 - Add http read support to `zarrs_reencode`

### Changed
 - Bump `opendal` to 0.48

## [0.5.4] - 2024-07-30

### Changed
 - Bump `zarrs` to 0.16.0

### Fixed
 - Fixed `zarrs_reencode` with `--shard-shape` applying existing array-to-array and bytes-to-bytes codecs as both inner and outer codecs

## [0.5.3] - 2024-07-24

### Added
 - Add `ome_zarr_metadata` dependency for OME-Zarr metadata serialisation

### Changed
 - `zarrs_reencode`: revise output and update docs
 - Update benchmarks and add plots
 - Make the help clearer for valid chunk key encoding separators in various tools

## [0.5.2] - 2024-07-10

### Changed
 - Add `--dimension-names` arg to `zarrs_filter`, `zarrs_ome`, `zarrs_reencode` to change dimension names

### Fixed
 - Remove unused `http` feature from `zarrs` dependency

## [0.5.1] - 2024-07-07

### Added
 - `zarrs_info`: add group metadata support

### Changed
 - Bump `zarrs` to 0.15.0
 - Add `HTTP` store support to most benchmark binaries/scripts

## [0.5.0] - 2024-07-02

### Added
 - Add `zarrs_info` (requires `info` feature): returns information about a Zarr array to stdout (JSON encoded)
    - Metadata
    - Array shape, data type, fill value, dimension names, attributes, etc.
    - Value range
    - Histogram

### Changed
 - Bump `zarrs` to 0.15.0-beta.1
 - Bump `rayon` to 1.10.0
 - Bump `itertools` to 0.13
 - `zarrs_ome`: add `--version` arg, supporting `0.5-dev` or `0.5-dev1`
 - Change `zarrs_ome` default output behaviour to erase
 - Update benchmarks to use `zarrs` 0.15.0-beta.1

## [0.4.2] - 2024-05-16

### Changed
 - Bump zarrs to 0.14.0
 - Disable concurrent netCDF block processing in `zarrs_ncvar2zarr`

## [0.4.1] - 2024-05-06

### Changed
 - Replace `--no_overwrite` with `--exists` in `zarrs_filter` and `zarrs_ome`
   - Both support `erase` and `exit` options
   - `zarrs_ome` also supports an `overwrite` option
 - `zarrs_{ncvar,binary}2zarr` output size change in human readable bytes
 - Change `zarrs_{filter,ome}` to multi progress bars
 - Print input path with `zarrs_{ncva2zarr,ome}`
 - Add bytes codec to encoding/reencoding help

### Removed
 - Remove `--hide-progress` from `zarrs_filter`

## [0.4.0] - 2024-04-20

### Added
 - `zarrs_filter`: apply simple image filters (transformations) to an array
 - `zarrs_ome`: convert an array to OME-Zarr
 - Add `--endianness` option to `zarrs_binary2zarr`

### Changed
 - **Breaking**: put various tools behind feature flags
 - Bump `rayon_iter_concurrent_limit` to 0.2.0
 - Various minor fixes to clap help
 - `zarrs_reencode`: add `--attributes-append` to re-encoding options
 - Bump `zarrs` to 0.13.0
 - **Breaking**: Bump MSRV to 1.75

## [0.3.0] - 2024-02-22

### Added
 - Add benchmark data generator: `scripts/generate_benchmark_array.py`
 - Add benchmark runners: `scripts/run_benchmark_read_{all,chunks}.py`
 - Add tensorstore benchmark script

### Changed
 - Bump `cargo-dist` to to 0.10.0
 - `zarrs_benchmark_read_{sync,async}`, `zarrs_binary2zarr`, `zarrs_reencode`
   - Make `--concurrent-chunks` optional, choosing optimal by default
 - `zarrs_ncvar2zarr`
   - Change `--num-parallel-blocks` to `--concurrent-blocks` and make it optional
   - Remove `--no-parallel-codecs`
 - Improve internal concurrency
 - Update benchmarks with `zarrs` v0.12.0
 - Update dependencies

## [0.2.3] - 2024-02-06

### Added
 - `--ignore_checksums` argument to `zarrs_reencode` and `zarrs_benchmark_read_{sync,async}`
   - See the [relevant zarrs docs](https://docs.rs/zarrs/latest/zarrs/#correctness-issues-with-past-versions) on fixing errant arrays encoded with old `zarrs`/`zarrs_tools` versions 

### Changed
 - Bump `zarrs` to 0.11.6

## [0.2.2] - 2024-02-02

### Added
 - Use `cargo-dist` for releases

### Changed
 - Bump dependencies
   - `zarrs` to 0.11.3

### Fixed
 - Fix typos/errors in various docs files

## [0.2.1] - 2024-01-26

### Changes
- Bump `zarrs` to 0.11

## [0.2.0] - 2023-12-26

### Changes
 - Bump `zarrs` to 0.8.0
 - Increase MSRV to 1.71

## [0.1.1] - 2023-12-11

### Changes
 - Bump `zarrs` to 0.7.1 to fix build with `rust<1.74`

## [0.1.0] - 2023-12-05

### Added
 - Initial public release

[unreleased]: https://github.com/LDeakin/zarrs_tools/compare/v0.5.5...HEAD
[0.5.5]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.5.5
[0.5.4]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.5.4
[0.5.3]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.5.3
[0.5.2]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.5.2
[0.5.1]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.5.1
[0.5.0]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.5.0
[0.4.2]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.4.2
[0.4.1]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.4.1
[0.4.0]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.4.0
[0.3.0]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.3.0
[0.2.3]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.2.2
[0.2.2]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.2.2
[0.2.1]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.2.1
[0.2.0]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.2.0
[0.1.1]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.1.1
[0.1.0]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.1.0

[@magnusuMET]: https://github.com/magnusuMET
