#!/bin/bash

set -e

for d in $@; do
    echo $d
    zarrs_info $d metadata-v3 # print metadata (as V3)
    zarrs_reencode --validate $d ${d}_tmp_reencode
    rm -r ${d}_tmp_reencode
done
