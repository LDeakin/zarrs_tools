# zarrs_validate

Compare the data in two Zarr arrays.

## Installation
`zarrs_validate` is installed with the `validate` feature of `zarrs_tools`.

### Prebuilt Binaries
```bash
# Requires cargo-binstall https://github.com/cargo-bins/cargo-binstall
cargo binstall zarrs_tools
```

### From Source
```bash
cargo install --features=validate zarrs_tools
```

## Usage
<details>
<summary>zarrs_validate --help</summary>

```text
Compare the data in two Zarr arrays.

Equality of the arrays is determined by comparing the shape, data type, and data.

Differences in encoding (e.g codecs, chunk key encoding) and attributes are ignored.

Usage: zarrs_validate [OPTIONS] <FIRST> <SECOND>

Arguments:
  <FIRST>
          The path to the first zarr array

  <SECOND>
          The path to the second zarr array

Options:
      --concurrent-chunks <CONCURRENT_CHUNKS>
          Number of concurrent chunks to compare

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

</details>
