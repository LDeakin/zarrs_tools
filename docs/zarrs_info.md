# zarrs_info

Get information about a Zarr array or group.

## Installation
`zarrs_info` is installed with the `info` feature of `zarrs_tools`.

### Prebuilt Binaries
```bash
# Requires cargo-binstall https://github.com/cargo-bins/cargo-binstall
cargo binstall zarrs_tools
```

### From Source
```bash
cargo install --features=info zarrs_tools
```

## Usage
<details>
<summary>zarrs_info --help</summary>

```text
Get information about a Zarr array or group.

Outputs are JSON encoded.

Usage: zarrs_info [OPTIONS] <PATH> <COMMAND>

Commands:
  metadata         Get the array/group metadata
  metadata-v3      Get the array/group metadata (interpreted as V3)
  attributes       Get the array/group attributes
  shape            Get the array shape
  data-type        Get the array data type
  fill-value       Get the array fill value
  dimension-names  Get the array dimension names
  range            Get the array data range
  histogram        Get the array data histogram
  help             Print this message or the help of the given subcommand(s)

Arguments:
  <PATH>
          Path to the Zarr input array or group

Options:
      --chunk-limit <CHUNK_LIMIT>
          The maximum number of chunks concurrently processed.
          
          Defaults to the RAYON_NUM_THREADS environment variable or the number of logical CPUs. Consider reducing this for images with large chunk sizes or on systems with low memory availability.
          
          [default: 24]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

</details>

## Examples

#### Data Type
```bash
zarrs_info array.zarr data-type
```
```text
{
  "data_type": "uint16"
}
```

### Array Shape
```bash
zarrs_info array.zarr shape
```
```text
{
  "shape": [
    1243,
    1403,
    1510
  ]
}
```

### Data Range
```bash
zarrs_info array.zarr range
```
```text
{
  "min": 0,
  "max": 65535
}
```
