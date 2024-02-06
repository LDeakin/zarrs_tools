# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
 - Bump `cargo-dist` to to 0.9.0 and use GitHub `macos-14` runners for `aarch64-apple-darwin`

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

[unreleased]: https://github.com/LDeakin/zarrs_tools/compare/v0.2.3...HEAD
[0.2.3]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.2.2
[0.2.2]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.2.2
[0.2.1]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.2.1
[0.2.0]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.2.0
[0.1.1]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.1.1
[0.1.0]: https://github.com/LDeakin/zarrs_tools/releases/tag/v0.1.0
