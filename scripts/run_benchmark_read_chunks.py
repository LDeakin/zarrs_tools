#!/usr/bin/env python3

import subprocess
import re
import pandas as pd
import math
import os

def clear_cache():
    subprocess.call(['sudo', 'sh', '-c', "sync; echo 3 > /proc/sys/vm/drop_caches"])

implementation_to_args = {
    "zarrs_sync": ["/usr/bin/time", "-v", "zarrs_benchmark_read_sync", "--concurrent-chunks"],
    "zarrs_async": ["/usr/bin/time", "-v", "zarrs_benchmark_read_async", "--concurrent-chunks"],
    "tensorstore": ["/usr/bin/time", "-v", "./scripts/tensorstore_benchmark_read_async.py", "--concurrent_chunks"],
}

images = [
    "data/benchmark.zarr",
    "data/benchmark_compress.zarr",
    "data/benchmark_compress_shard.zarr",
]
concurrent_chunks_list = [1, 2, 4, 8, 16, 32]

index = []
for image in images:
    for concurrent_chunks in concurrent_chunks_list:
        index.append((image, concurrent_chunks))

rows = []
for image in [
    "data/benchmark.zarr",
    "data/benchmark_compress.zarr",
    "data/benchmark_compress_shard.zarr",
]:
    for concurrent_chunks in concurrent_chunks_list:
        wall_times = []
        memory_usages = []
        for implementation in ["zarrs_sync", "zarrs_async", "tensorstore"]:
            print(implementation, concurrent_chunks)
            args = implementation_to_args[implementation] + [str(concurrent_chunks)] + [image]
            clear_cache()
            pipes = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            std_out, std_err = pipes.communicate()
            # print(std_err)
            wall_time = re.search(
                r"Elapsed \(wall clock\) time \(h:mm:ss or m:ss\): (\d+?):([\d\.]+?)\\n",
                str(std_err),
            )
            memory_usage = re.search(
                r"Maximum resident set size \(kbytes\): (\d+?)\\n", str(std_err)
            )
            if wall_time and memory_usage:
                m = int(wall_time.group(1))
                s = float(wall_time.group(2))
                wall_time_s = m * 60 + s
                # print(wall_time_s)
                memory_usage_kb = int(memory_usage.group(1))
                memory_usage_gb = float(memory_usage_kb) / 1.0e6
                # print(memory_usage_gb)
                wall_times.append(f"{wall_time_s:.02f}")
                memory_usages.append(f"{memory_usage_gb:.02f}")
                print(wall_time_s, memory_usage_gb)
            else:
                wall_times.append(math.nan)
                memory_usages.append(math.nan)
        row = wall_times + memory_usages
        rows.append(row)

columns_pandas = []
columns_markdown = []
for metric in ["Wall time (s)", "Memory usage (GB)"]:
    include_metric = True
    last_implementation = ""
    for implementation, execution in [("zarrs", "sync"), ("zarrs", "async"), ("tensorstore", "async")]:
        column_markdown = ""

        # Metric
        if include_metric:
            column_markdown += metric
        column_markdown += "<br>"
        include_metric = False

        # Implemnentation
        if implementation != last_implementation:
            last_implementation = implementation
            column_markdown += implementation
        column_markdown += "<br>"

        # Execution
        column_markdown += execution

        columns_markdown.append(column_markdown)
        columns_pandas.append((metric, implementation, execution))

data = {
    "index": index,
    "columns": columns_pandas,
    "data": rows,
    "index_names": ["Image", "Concurrency"],
    "column_names": ["Metric", "Implementation", "Execution"],
}

print(data)

df = pd.DataFrame.from_dict(data, orient="tight")
print(df)
print()
df.columns = columns_markdown
df.reset_index(inplace=True)
print(df.to_markdown(index=False))
