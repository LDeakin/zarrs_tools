#!/usr/bin/env python3

import subprocess
import re
import pandas as pd
import numpy as np
import math

implementation_to_args = {
    "zarrs_sync": ["/usr/bin/time", "-v", "zarrs_benchmark_read_sync", "--read-all"],
    "zarrs_async": ["/usr/bin/time", "-v", "zarrs_benchmark_read_async", "--read-all"],
    "tensorstore": ["/usr/bin/time", "-v", "./scripts/tensorstore_benchmark_read_async.py", "--read_all"],
}

def clear_cache():
    subprocess.call(['sudo', 'sh', '-c', "sync; echo 3 > /proc/sys/vm/drop_caches"])

best_of = 3

index = []
rows = []
for image in [
    "data/benchmark.zarr",
    "data/benchmark_compress.zarr",
    "data/benchmark_compress_shard.zarr",
]:
    index.append(image)
    wall_times = []
    memory_usages = []
    for implementation in ["zarrs_sync", "zarrs_async", "tensorstore"]:
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
                # print(wall_time_s)
                memory_usage_kb = int(memory_usage.group(1))
                memory_usage_gb = float(memory_usage_kb) / 1.0e6
                # print(memory_usage_gb)
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
    "index_names": ["Image"],
    "column_names": ["Metric", "Implementation", "Execution"],
}

# print(data)

df = pd.DataFrame.from_dict(data, orient="tight")
print(df)
print()
df.columns = columns_markdown
print(df.to_markdown())
