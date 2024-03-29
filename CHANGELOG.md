# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
 - `zarrs_filter`: apply simple image filters (transformations) to an array
 - `zarrs_ome`: convert an array to OME-Zarr
 - Add `--endianness` option to `zarrs_binary2zarr`

### Changed
 - **Breaking**: put various tools behind feature flags
 - Bump `rayon_iter_concurrent_limit` to 0.2.0
 - Various minor fixes to clap help
 - `zarrs_reencode`: add `--attributes-append` to re-encoding options

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

[unreleased]: https://github.com/LDeakin/zarrs_tools/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.3.0
[0.2.3]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.2.2
[0.2.2]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.2.2
[0.2.1]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.2.1
[0.2.0]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.2.0
[0.1.1]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.1.1
[0.1.0]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.1.0
