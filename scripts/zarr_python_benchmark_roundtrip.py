#!/usr/bin/env python3

import numpy as np
import timeit
import asyncio
import click
from functools import wraps
import sys

import zarr
from zarr.store import LocalStore, RemoteStore
from zarr.core.indexing import BlockIndexer
from zarr.core.buffer import default_buffer_prototype

def coro(f):
    @wraps(f)
    def wrapper(*args, **kwargs):
        return asyncio.run(f(*args, **kwargs))

    return wrapper

@click.command()
@coro
@click.argument('path', type=str)
@click.argument('output', type=str)
async def main(path, output):
    if "benchmark_compress_shard.zarr" in path:
        sys.exit(1)

    if path.startswith("http"):
        store = RemoteStore(url=path) # broken with zarr-python 3.0.0a0
    else:
        store = LocalStore(path)

    dataset = zarr.open(store=store)
    dataset_out = zarr.create(store=LocalStore(output), shape=dataset.shape, chunks=dataset.chunks, dtype=dataset.dtype, codecs=dataset.metadata.codecs)

    start_time = timeit.default_timer()

    # Simple
    dataset_out[:] = dataset[:] # TODO: Faster approach? Chunk-by-chunk with concurrency?

    # # Chunk by chunk
    # domain_shape = dataset.shape
    # chunk_shape = dataset.chunks
    # print("Domain shape", domain_shape)
    # print("Chunk shape", chunk_shape)
    # num_chunks =[(domain + chunk_shape - 1) // chunk_shape for (domain, chunk_shape) in zip(domain_shape, chunk_shape)]
    # print("Number of chunks", num_chunks)

    # for chunk_index in np.ndindex(*num_chunks):
    #     # TODO: Run in parallel over chunks
    #     print(chunk_index)
    #     dataset_out.set_block_selection(chunk_index, dataset.get_block_selection(chunk_index))

    elapsed = timeit.default_timer() - start_time
    elapsed_ms = elapsed * 1000.0

    print(f"Round trip in {elapsed_ms:.2f}ms")

if __name__ == "__main__":
    asyncio.run(main())
