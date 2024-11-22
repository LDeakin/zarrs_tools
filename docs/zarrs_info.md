# zarrs_info

Get information about a Zarr array or group.

## Installation
`zarrs_info` is installed with the `info` feature of `zarrs_tools`

### Prebuilt Binaries
```shell
# Requires cargo-binstall https://github.com/cargo-bins/cargo-binstall
cargo binstall zarrs_tools
```

### From Source
```shell
cargo install --features=ncvar2zarr zarrs_tools
```

## Usage

```text
Usage: zarrs_info [OPTIONS] <PATH> <COMMAND>
```

`zarrs_info` supports the following commands:

 - `metadata`         : Get the array/group metadata
 - `metadata-v3`      : Get the array/group metadata (interpreted as V3)
 - `attributes`       : Get the array/group attributes
 - `shape`            : Get the array shape
 - `data-type`        : Get the array data type
 - `fill-value`       : Get the array fill value
 - `dimension-names`  : Get the array dimension names
 - `range`            : Get the array data range
 - `histogram`        : Get the array data histogram

Most commands only read array / group metadata.
Range and histogram will read the array data.

## Examples

#### Data Type
```shell
zarrs_info array.zarr data-type
```
```text
{
  "data_type": "uint16"
}
```

### Array Shape
```shell
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
```shell
zarrs_info array.zarr range
```
```text
{
  "min": 0,
  "max": 65535
}
```
