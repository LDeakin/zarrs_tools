# zarrs_filter

Apply simple image filters (transformations) to an array.

> [!WARNING]
> `zarrs_filter` is highly experimental, has had limited production testing, and is sparsely documented.

The filters currently supported are:
 - **reencode**:            Reencode (change encoding, data type, etc.).
 - **crop**:                Crop given an offset and shape.
 - **rescale**:             Rescale values given a multiplier and offset.
 - **clamp**:               Clamp values between a minimum and maximum.
 - **equal**:               Return a binary image where the input is equal to some value.
 - **downsample**:          Downsample given a stride.
 - **gradient-magnitude**:  Compute the gradient magnitude.
 - **gaussian**:            Apply a Gaussian kernel.
 - **summed area table**:   Compute the summed area table.
 - **guided filter**:       Apply a guided filter (edge-preserving noise filter).

## Installation
`zarrs_filter` is installed with the `filter` feature of `zarrs_tools`.

### Prebuilt Binaries
```bash
# Requires cargo-binstall https://github.com/cargo-bins/cargo-binstall
cargo binstall zarrs_tools
```

### From Source
```bash
cargo install --features=filter zarrs_tools
```

## Usage

<details>
<summary>zarrs_filter --help</summary>

```text
Apply simple image filters (transformations) to a Zarr array

Usage: zarrs_filter [OPTIONS] [RUN_CONFIG] [COMMAND]

Commands:
  reencode            Reencode an array
  crop                Crop an array given an offset and shape
  rescale             Rescale array values given a multiplier and offset
  clamp               Clamp values between a minimum and maximum
  equal               Return a binary image where the input is equal to some value
  downsample          Downsample an image given a stride
  gradient-magnitude  Compute the gradient magnitude
  gaussian            Apply a Gaussian kernel
  summed-area-table   Compute a summed area table (integral image)
  guided-filter       Apply a guided filter (edge-preserving noise filter)
  replace-value       Replace a value with another value
  help                Print this message or the help of the given subcommand(s)

Arguments:
  [RUN_CONFIG]
          Path to a JSON run configuration

Options:
      --exists <EXISTS>
          Behaviour if the output exists
          
          [default: erase]

          Possible values:
          - erase: Erase the output
          - exit:  Exit if the output already exists

      --tmp <TMP>
          Directory for temporary arrays.
          
          If omitted, defaults to the platform-specific temporary directory (e.g. ${TMPDIR}, /tmp, etc.)

      --chunk-limit <CHUNK_LIMIT>
          The maximum number of chunks concurrently processed.
          
          By default, this is set to the number of CPUs. Consider reducing this for images with large chunk sizes or on systems with low memory availability.

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

</details>

Run `zarrs_filter <COMMAND> --help` for more information on a specific command.

## Examples (CLI)
```bash
export ENCODE_ARGS="--shard-shape 256,256,256 --chunk-shape 32,32,32"
zarrs_filter reencode           array.zarr       array_reenc.zarr               ${ENCODE_ARGS}
zarrs_filter reencode           array_reenc.zarr array_reenc_int32.zarr         ${ENCODE_ARGS} --data-type int32
zarrs_filter reencode           array_reenc.zarr array_reenc_float32.zarr       ${ENCODE_ARGS} --data-type float32
zarrs_filter crop               array_reenc.zarr array_crop.zarr                ${ENCODE_ARGS} --data-type float32 256,256,256 768,768,768
zarrs_filter rescale            array_reenc.zarr array_rescale.zarr             ${ENCODE_ARGS} --data-type float32 2.0 1.0 --fill-value 1.0
zarrs_filter clamp              array_reenc.zarr array_clamp.zarr               ${ENCODE_ARGS} --data-type float32 5 255 --fill-value 5.0
# zarrs_filter equal              array_reenc.zarr array_eq_bool.zarr             ${ENCODE_ARGS} --data-type bool 1 --fill-value true
zarrs_filter equal              array_reenc.zarr array_eq_u8.zarr               ${ENCODE_ARGS} --data-type uint8 1 --fill-value 1
zarrs_filter downsample         array_reenc.zarr array_downsample.zarr          ${ENCODE_ARGS} --data-type float32 2,2,2
zarrs_filter downsample         array_eq_u8.zarr array_downsample_discrete.zarr ${ENCODE_ARGS} --data-type uint8 2,2,2 --discrete
zarrs_filter gradient-magnitude array_reenc.zarr array_gradient_magnitude.zarr  ${ENCODE_ARGS} --data-type float32
zarrs_filter gaussian           array_reenc.zarr array_gaussian.zarr            ${ENCODE_ARGS} --data-type float32 1.0,1.0,1.0 3,3,3
zarrs_filter summed-area-table  array_reenc.zarr array_sat.zarr                 ${ENCODE_ARGS} --data-type int64
zarrs_filter guided-filter      array_reenc.zarr array_guided_filter.zarr       ${ENCODE_ARGS} --data-type float32 40000 3
zarrs_filter replace-value      array_reenc.zarr array_replace.zarr             ${ENCODE_ARGS} 65535 0 --fill-value 0
```

## Examples (Config)

```bash
zarrs_filter <RUN.json>
```

<details>
<summary>run.json</summary>

```json
[
    {
        "_comment": "Rechunk the input",
        "filter": "reencode",
        "input": "array.zarr",
        "output": "$reencode0",
        "shard_shape": [256, 256, 256],
        "chunk_shape": [32, 32, 32]
    },
    {
        "_comment": "Reencode the previous output as float32, automatically cast the fill value",
        "filter": "reencode",
        "output": "array_float32.zarr",
        "data_type": "float32"
    },
    {
        "filter": "crop",
        "input": "$reencode0",
        "output": "array_crop.zarr",
        "offset": [256, 256, 256],
        "shape": [768, 768, 768]
    },
    {
        "filter": "replace_value",
        "input": "$reencode0",
        "output": "array_replace.zarr",
        "value": 65535,
        "replace": 0
    },
    {
        "_comment": "Multiply by 7.0/20000.0, casting most values in the image between 0 and 7, store in 8-bit (saturate cast)",
        "filter": "rescale",
        "input": "$reencode0",
        "output": "array_3bit.zarr",
        "multiply": 0.00035,
        "add": 0.0,
        "data_type": "uint8",
        "fill_value": 0
    },
    {
        "_comment": "Multiply by 255.0/20000.0, casting most values in the image between 0 and 7, store in 8-bit (saturate cast)",
        "filter": "rescale",
        "input": "$reencode0",
        "output": "array_8bit.zarr",
        "multiply": 0.01275,
        "add": 0.0,
        "data_type": "uint8",
        "fill_value": 0
    },
    {
        "_comment": "Clamp the 3-bit output between 2 and 5 and set the fill value to 2",
        "filter": "clamp",
        "output": "array_3bit_clamp.zarr",
        "min": 2,
        "max": 5,
        "fill_value": 2
    },
    {
        "_comment": "Calculate a binary image where the input is equal to 5 (the max from the clamp). Store as bool",
        "filter": "equal",
        "input": "array_3bit_clamp.zarr", 
        "output": "array_clamp_equal_bool.zarr",
        "value": 5
    },
    {
        "_comment": "Calculate a binary image where the input is equal to 5 (the max from the clamp). Store as uint8",
        "filter": "equal",
        "input": "array_3bit_clamp.zarr",
        "output": "array_3bit_max.zarr",
        "value": 5,
        "data_type": "uint8",
        "fill_value": 0
    },
    {
        "_comment": "Downsample clamped image by a factor of 2 with mean operator.",
        "filter": "downsample",
        "input": "array_3bit_clamp.zarr",
        "output": "array_3bit_clamp_by2_continuous.zarr",
        "stride": [2, 2, 2],
        "discrete": false,
        "data_type": "float32",
        "shard_shape": [128, 128, 128],
        "chunk_shape": [32, 32, 32]
    },
    {
        "_comment": "Downsample clamped image by a factor of 2 with mode operator.",
        "filter": "downsample",
        "input": "array_3bit_clamp.zarr",
        "output": "array_3bit_clamp_by2_discrete.zarr",
        "stride": [2, 2, 2],
        "discrete": true,
        "shard_shape": [128, 128, 128],
        "chunk_shape": [32, 32, 32]
    },
    {
        "filter": "gradient_magnitude",
        "input": "$reencode0",
        "output": "array_gradient.zarr"
    },
    {
        "filter": "gaussian",
        "input": "$reencode0",
        "output": "array_gaussian.zarr",
        "sigma": [1.0, 1.0, 1.0],
        "kernel_half_size": [3, 3, 3]
    },
    {
        "filter": "summed_area_table",
        "input": "$reencode0",
        "output": "array_sat.zarr",
        "data_type": "float32"
    },
    {
        "filter": "guided_filter",
        "input": "$reencode0",
        "output": "array_guided_filter.zarr",
        "epsilon": 40000.0,
        "radius": 3,
        "data_type": "float32"
    }
]
```
</details>

<details>
<summary>output</summary>

```text
0 reencode
        args:   {}
        encode: {"chunk_shape":[32,32,32],"shard_shape":[256,256,256]}
        input:  uint16 [1243, 1403, 1510] "array.zarr"
        output: uint16 [1243, 1403, 1510] "/tmp/.tmpCbeEcJ/$reencode0bxiFEM"
1 reencode
        args:   {}
        encode: {"data_type":"float32"}
        input:  uint16 [1243, 1403, 1510] "/tmp/.tmpCbeEcJ/$reencode0bxiFEM"
        output: float32 [1243, 1403, 1510] "array_float32.zarr" (overwrite)
2 crop
        args:   {"offset":[256,256,256],"shape":[768,768,768]}
        encode: {}
        input:  uint16 [1243, 1403, 1510] "/tmp/.tmpCbeEcJ/$reencode0bxiFEM"
        output: uint16 [768, 768, 768] "array_crop.zarr" (overwrite)
3 replace_value
        args:   {"value":65535,"replace":0}
        encode: {}
        input:  uint16 [1243, 1403, 1510] "/tmp/.tmpCbeEcJ/$reencode0bxiFEM"
        output: uint16 [1243, 1403, 1510] "array_replace.zarr" (overwrite)
4 rescale
        args:   {"multiply":0.00035,"add":0.0,"add_first":false}
        encode: {"data_type":"uint8","fill_value":0}
        input:  uint16 [1243, 1403, 1510] "/tmp/.tmpCbeEcJ/$reencode0bxiFEM"
        output: uint8 [1243, 1403, 1510] "array_3bit.zarr" (overwrite)
5 rescale
        args:   {"multiply":0.01275,"add":0.0,"add_first":false}
        encode: {"data_type":"uint8","fill_value":0}
        input:  uint16 [1243, 1403, 1510] "/tmp/.tmpCbeEcJ/$reencode0bxiFEM"
        output: uint8 [1243, 1403, 1510] "array_8bit.zarr" (overwrite)
6 clamp
        args:   {"min":2.0,"max":5.0}
        encode: {"fill_value":2}
        input:  uint8 [1243, 1403, 1510] "array_8bit.zarr"
        output: uint8 [1243, 1403, 1510] "array_3bit_clamp.zarr" (overwrite)
7 equal
        args:   {"value":5}
        encode: {}
        input:  uint8 [1243, 1403, 1510] "array_3bit_clamp.zarr"
        output: bool [1243, 1403, 1510] "array_clamp_equal_bool.zarr" (overwrite)
8 equal
        args:   {"value":5}
        encode: {"data_type":"uint8","fill_value":0}
        input:  uint8 [1243, 1403, 1510] "array_3bit_clamp.zarr"
        output: uint8 [1243, 1403, 1510] "array_3bit_max.zarr" (overwrite)
9 downsample
        args:   {"stride":[2,2,2],"discrete":false}
        encode: {"data_type":"float32","chunk_shape":[32,32,32],"shard_shape":[128,128,128]}
        input:  uint8 [1243, 1403, 1510] "array_3bit_clamp.zarr"
        output: float32 [621, 701, 755] "array_3bit_clamp_by2_continuous.zarr" (overwrite)
10 downsample
        args:   {"stride":[2,2,2],"discrete":true}
        encode: {"chunk_shape":[32,32,32],"shard_shape":[128,128,128]}
        input:  uint8 [1243, 1403, 1510] "array_3bit_clamp.zarr"
        output: uint8 [621, 701, 755] "array_3bit_clamp_by2_discrete.zarr" (overwrite)
11 gradient_magnitude
        args:   {}
        encode: {}
        input:  uint16 [1243, 1403, 1510] "/tmp/.tmpCbeEcJ/$reencode0bxiFEM"
        output: uint16 [1243, 1403, 1510] "array_gradient.zarr" (overwrite)
12 gaussian
        args:   {"sigma":[1.0,1.0,1.0],"kernel_half_size":[3,3,3]}
        encode: {}
        input:  uint16 [1243, 1403, 1510] "/tmp/.tmpCbeEcJ/$reencode0bxiFEM"
        output: uint16 [1243, 1403, 1510] "array_gaussian.zarr" (overwrite)
13 summed area table
        args:   {}
        encode: {"data_type":"float32"}
        input:  uint16 [1243, 1403, 1510] "/tmp/.tmpCbeEcJ/$reencode0bxiFEM"
        output: float32 [1243, 1403, 1510] "array_sat.zarr" (overwrite)
14 guided_filter
        args:   {"epsilon":40000.0,"radius":3}
        encode: {"data_type":"float32"}
        input:  uint16 [1243, 1403, 1510] "/tmp/.tmpCbeEcJ/$reencode0bxiFEM"
        output: float32 [1243, 1403, 1510] "array_guided_filter.zarr" (overwrite)
[00:00:02/00:00:02] reencode /tmp/.tmpCbeEcJ/$reencode0bxiFEM rw:34.78/28.90 p:0.00
[00:00:04/00:00:04] reencode array_float32.zarr rw:30.06/76.57 p:14.16
[00:00:00/00:00:00] crop array_crop.zarr rw:3.46/3.34 p:0.00
[00:00:02/00:00:02] replace_value array_replace.zarr rw:26.73/47.32 p:7.25
[00:00:01/00:00:01] rescale array_3bit.zarr rw:18.11/14.43 p:11.55
[00:00:01/00:00:01] rescale array_8bit.zarr rw:23.54/21.99 p:11.08
[00:00:00/00:00:00] clamp array_3bit_clamp.zarr rw:9.70/10.34 p:0.96
[00:00:00/00:00:00] equal array_clamp_equal_bool.zarr rw:10.61/9.32 p:4.56
[00:00:00/00:00:00] equal array_3bit_max.zarr rw:10.29/9.49 p:3.61
[00:00:02/00:00:02] downsample array_3bit_clamp_by2_continuous.zarr rw:7.01/1.95 p:71.76
[00:00:06/00:00:06] downsample array_3bit_clamp_by2_discrete.zarr rw:16.08/1.01 p:168.86
[00:00:20/00:00:20] gradient_magnitude array_gradient.zarr rw:147.16/14.38 p:289.05
[00:00:10/00:00:10] gaussian array_gaussian.zarr rw:36.19/22.01 p:181.06
[00:00:23/00:00:23] summed area table array_sat.zarr rw:190.51/215.68 p:54.39
[00:01:51/00:01:51] guided_filter array_guided_filter.zarr rw:29.57/59.96 p:2427.96
```
</details>
