#!/usr/bin/env python3

import subprocess
import re
import pandas as pd
import math
import numpy as np

def clear_cache():
    subprocess.call(['sudo', 'sh', '-c', "sync; echo 3 > /proc/sys/vm/drop_caches"])

implementation_to_args = {
    "zarrs_rust": ["/usr/bin/time", "-v", "zarrs_benchmark_read_sync", "--read-all"],
    "zarrs_rust_async": ["/usr/bin/time", "-v", "zarrs_benchmark_read_async", "--read-all"],
    "tensorstore_python": ["/usr/bin/time", "-v", "./scripts/tensorstore_python_benchmark_read_async.py", "--read_all"],
    "zarr_python": ["/usr/bin/time", "-v", "./scripts/zarr_python_benchmark_read_async.py", "--read_all"],
}

implementations = [
    "zarrs_rust",
    # "zarrs_rust_async", # reenable with async_spawning branch
    "tensorstore_python",
    "zarr_python",
]

images = [
    "data/benchmark.zarr",
    "data/benchmark_compress.zarr",
    "data/benchmark_compress_shard.zarr",
]

best_of = 3

index = []
rows = []
for image in images:
    index.append(image)
    wall_times = []
    memory_usages = []
    for implementation in implementations:
        wall_time_measurements = []
        memory_usage_measurements = []
        for i in range(best_of):
            print(implementation, image, i)
            args = implementation_to_args[implementation] + [image]
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
                memory_usage_kb = int(memory_usage.group(1))
                memory_usage_gb = float(memory_usage_kb) / 1.0e6
                print(wall_time_s, memory_usage_gb)
                wall_time_measurements.append(wall_time_s)
                memory_usage_measurements.append(memory_usage_gb)
            else:
                wall_time_measurements.append(math.nan)
                memory_usage_measurements.append(math.nan)

        wall_time_best = np.nanmin(wall_time_measurements)
        memory_usages_best = np.nanmin(memory_usage_measurements)
        wall_times.append(f"{wall_time_best:.02f}")
        memory_usages.append(f"{memory_usages_best:.02f}")

    row = wall_times + memory_usages
    rows.append(row)

columns_pandas = []
columns_markdown = []
for metric in ["Time (s)", "Memory (GB)"]:
    include_metric = True
    last_implementation = ""
    for implementation in implementations:
        column_markdown = ""

        # Metric
        if include_metric:
            column_markdown += metric
        column_markdown += "<br>"
        include_metric = False

        # Implementation
        if implementation != last_implementation:
            last_implementation = implementation
            column_markdown += implementation.replace("_", "<br>")

        columns_markdown.append(column_markdown)
        columns_pandas.append((metric, implementation))

data = {
    "index": index,
    "columns": columns_pandas,
    "data": rows,
    "index_names": ["Image"],
    "column_names": ["Metric", "Implementation"],
}

# print(data)

df = pd.DataFrame.from_dict(data, orient="tight")
print(df)
print()
df.columns = columns_markdown
print(df.to_markdown())
