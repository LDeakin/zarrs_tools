#!/usr/bin/env python3

import numpy as np
import timeit
import asyncio
import click
from functools import wraps
import itertools
import multiprocessing

import tensorstore as ts

def coro(f):
    @wraps(f)
    def wrapper(*args, **kwargs):
        return asyncio.run(f(*args, **kwargs))

    return wrapper


# Via https://github.com/ome/ome2024-ngff-challenge/blob/main/src/ome2024_ngff_challenge/utils.py
def chunk_iter(shape: list, chunks: list):
    """
    Returns a series of tuples, each containing chunk slice
    E.g. for 2D shape/chunks: ((slice(0, 512, 1), slice(0, 512, 1)), (slice(0, 512, 1), slice(512, 1024, 1))...)
    Thanks to Davis Bennett.
    """
    assert len(shape) == len(chunks)
    chunk_iters = []
    for chunk_size, dim_size in zip(chunks, shape):
        chunk_tuple = tuple(
            slice(
                c_index * chunk_size,
                min(dim_size, c_index * chunk_size + chunk_size),
                1,
            )
            for c_index in range(-(-dim_size // chunk_size))
        )
        chunk_iters.append(chunk_tuple)
    return tuple(itertools.product(*chunk_iters))

@click.command()
@coro
@click.argument('path', type=str)
@click.argument('output', type=str)
async def main(path, output):
    if path.startswith("http"):
        kvstore = {
            'driver': 'http',
            'base_url': path,
        }
    else:
        kvstore = {
            'driver': 'file',
            'path': path,
        }

    dataset_future = ts.open({
        'driver': 'zarr3',
        'kvstore': kvstore,
        # 'context': {
        #     'cache_pool': {
        #         'total_bytes_limit': 100_000_000
        #     }
        # },
        # 'recheck_cached_data': 'open',
    })
    dataset = dataset_future.result()
    # print(dataset)

    # Create a new dataset at the output path
    new_kvstore = {
        'driver': 'file',
        'path': output,
    }

    new_dataset_future = ts.open({
        'driver': 'zarr3',
        'kvstore': new_kvstore,
        'create': True,
        'delete_existing': True,
        'schema': dataset.schema,
    })
    new_dataset = new_dataset_future.result()

    start_time = timeit.default_timer()

    # new_dataset[:] = dataset[:] # NOPE!

    # Via https://github.com/ome/ome2024-ngff-challenge/blob/main/src/ome2024_ngff_challenge/resave.py
    # TODO: Not sure if this is the fastest API for this
    chunk_shape = dataset.chunk_layout.write_chunk.shape
    threads = multiprocessing.cpu_count()
    for idx, batch in enumerate(itertools.batched(chunk_iter(new_dataset.shape, chunk_shape), threads)):
        with ts.Transaction() as txn:
            for slice_tuple in batch:
                new_dataset.with_transaction(txn)[slice_tuple] = dataset[slice_tuple]


    elapsed = timeit.default_timer() - start_time
    elapsed_ms = elapsed * 1000.0
    print(f"Decoded in {elapsed_ms:.2f}ms")

if __name__ == "__main__":
    asyncio.run(main())
