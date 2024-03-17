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
 - **gradient-magnitude**:  Compute the gradient magnitude (sobel).
 - **gaussian**:            Apply a Gaussian kernel.
 - **summed area table**:   Compute the summed area table.
 - **guided filter**:       Apply a guided filter (edge-preserving noise filter).

## Installation
`zarrs_filter` is installed with the `filter` feature of `zarrs_tools`

```
cargo install --features=filter zarrs_tools
```

## Help
```bash
zarrs_filter --help
zarrs_filter <COMMAND> --help
```

## Examples (CLI)
```bash
export ENCODE_ARGS="--shard-shape 256,256,256 --chunk-shape 32,32,32"
zarrs_filter reencode           array.zarr       array_reenc.zarr               ${ENCODE_ARGS}
zarrs_filter reencode           array_reenc.zarr array_reenc_int32.zarr         ${ENCODE_ARGS} --data-type int32
zarrs_filter reencode           array_reenc.zarr array_reenc_float32.zarr       ${ENCODE_ARGS} --data-type float32
zarrs_filter crop               array_reenc.zarr array_crop.zarr                ${ENCODE_ARGS} --data-type float32 256,256,256 768,768,768
zarrs_filter rescale            array_reenc.zarr array_rescale.zarr             ${ENCODE_ARGS} --data-type float32 2.0 1.0 --fill-value 1.0
zarrs_filter clamp              array_reenc.zarr array_clamp.zarr               ${ENCODE_ARGS} --data-type float32 5 255 --fill-value 5.0
zarrs_filter equal              array_reenc.zarr array_equal_bool.zarr          ${ENCODE_ARGS} --data-type bool 1 --fill-value true
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
zarrs_filter <RUNFILE.json>
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
        input:  int16 [1243, 1403, 1510] "array.zarr"
        output: int16 [1243, 1403, 1510] "/tmp/.tmpCZFyRP/$reencode-0UN9BaQ"
1 crop
        args:   {"offset":[256,256,256],"shape":[768,768,768]}
        encode: {}
        input:  int16 [1243, 1403, 1510] "/tmp/.tmpCZFyRP/$reencode-0UN9BaQ"
        output: int16 [768, 768, 768] "/tmp/.tmpCZFyRP/.tmpVdcTSm"
2 reencode
        args:   {}
        encode: {"data_type":"float32"}
        input:  int16 [768, 768, 768] "/tmp/.tmpCZFyRP/.tmpVdcTSm"
        output: float32 [768, 768, 768] "/tmp/.tmpCZFyRP/.tmpVWBgf7"
3 rescale
        args:   {"multiply":2.0,"add":1.0,"add_first":false}
        encode: {"fill_value":1.0}
        input:  float32 [768, 768, 768] "/tmp/.tmpCZFyRP/.tmpVWBgf7"
        output: float32 [768, 768, 768] "filter/array_crop_convert_rescale.zarr"
4 clamp
        args:   {"min":5.0,"max":255.0}
        encode: {"fill_value":5}
        input:  int16 [1243, 1403, 1510] "/tmp/.tmpCZFyRP/$reencode-0UN9BaQ"
        output: int16 [1243, 1403, 1510] "/tmp/.tmpCZFyRP/$clamp-0kAPqI5"
5 equal
        args:   {"value":5}
        encode: {}
        input:  int16 [1243, 1403, 1510] "/tmp/.tmpCZFyRP/$clamp-0kAPqI5"
        output: bool [1243, 1403, 1510] "filter/array_clamp_equal_bool.zarr"
6 equal
        args:   {"value":5}
        encode: {"data_type":"uint8","fill_value":1}
        input:  int16 [1243, 1403, 1510] "/tmp/.tmpCZFyRP/$clamp-0kAPqI5"
        output: uint8 [1243, 1403, 1510] "filter/array_clamp_equal_uint8.zarr"
7 downsample
        args:   {"stride":[2,2,2],"discrete":false}
        encode: {"chunk_shape":[32,32,32],"shard_shape":[128,128,128]}
        input:  int16 [1243, 1403, 1510] "/tmp/.tmpCZFyRP/$reencode-0UN9BaQ"
        output: int16 [621, 701, 755] "filter/array_downsample.zarr"
8 gradient magnitude
        args:   {}
        encode: {}
        input:  int16 [1243, 1403, 1510] "/tmp/.tmpCZFyRP/$reencode-0UN9BaQ"
        output: int16 [1243, 1403, 1510] "filter/array_gradient_magnitude.zarr"
9 gaussian
        args:   {"sigma":[1.0,1.0,1.0],"kernel_half_size":[3,3,3]}
        encode: {}
        input:  int16 [1243, 1403, 1510] "/tmp/.tmpCZFyRP/$reencode-0UN9BaQ"
        output: int16 [1243, 1403, 1510] "filter/array_gaussian.zarr"
10 summed area table
        args:   {}
        encode: {"data_type":"float32"}
        input:  int16 [1243, 1403, 1510] "filter/array_gradient_magnitude.zarr"
        output: float32 [1243, 1403, 1510] "filter/array_sat.zarr"
[00:00:01/00:00:01] reencode /tmp/.tmpCZFyRP/$reencode-0UN9BaQ rw:28.84/32.41 p:0.00
[00:00:00/00:00:00] crop /tmp/.tmpCZFyRP/.tmpVdcTSm rw:3.07/3.56 p:0.00
[00:00:00/00:00:00] reencode /tmp/.tmpCZFyRP/.tmpVWBgf7 rw:5.53/11.19 p:2.42
[00:00:01/00:00:01] rescale filter/array_crop_convert_rescale.zarr rw:12.20/12.19 p:4.23
[00:00:02/00:00:02] clamp /tmp/.tmpCZFyRP/$clamp-0kAPqI5 rw:28.65/30.70 p:3.25
[00:00:01/00:00:01] equal filter/array_clamp_equal_bool.zarr rw:22.89/14.02 p:3.51
[00:00:01/00:00:01] equal filter/array_clamp_equal_uint8.zarr rw:21.26/9.90 p:4.32
[00:00:03/00:00:03] downsample filter/array_downsample.zarr rw:10.46/2.66 p:93.33
[00:00:20/00:00:20] gradient magnitude filter/array_gradient_magnitude.zarr rw:73.93/17.15 p:235.75
[00:00:10/00:00:10] gaussian filter/array_gaussian.zarr rw:46.85/15.63 p:161.30
[00:00:22/00:00:22] summed area table filter/array_sat.zarr rw:198.70/198.81 p:52.61
```
</details>
