#!/usr/bin/env python3

import numpy as np
import timeit
import asyncio
import click
from functools import wraps

import zarr
from zarr.store import LocalStore
from zarr.array import BlockIndexer
from zarr.buffer import default_buffer_prototype

def coro(f):
    @wraps(f)
    def wrapper(*args, **kwargs):
        return asyncio.run(f(*args, **kwargs))

    return wrapper

@click.command()
@coro
@click.argument('path')
@click.option('--concurrent_chunks', type=int, default=None, help='Number of concurrent async chunk reads. Ignored if --read-all is set')
@click.option('--read_all', is_flag=True, show_default=True, default=False, help='Read the entire array in one operation.')
async def main(path, concurrent_chunks, read_all):
    store = LocalStore(path)
    dataset = zarr.open(store=store)

    domain_shape = dataset.shape
    chunk_shape = dataset.chunks

    print("Domain shape", domain_shape)
    print("Chunk shape", chunk_shape)
    num_chunks =[(domain + chunk_shape - 1) // chunk_shape for (domain, chunk_shape) in zip(domain_shape, chunk_shape)]
    print("Number of chunks", num_chunks)

    async def chunk_read(chunk_index):
        indexer = BlockIndexer(chunk_index, dataset.shape, dataset.metadata.chunk_grid)
        return await dataset._async_array._get_selection(
            indexer=indexer, prototype=default_buffer_prototype
        )

    start_time = timeit.default_timer()
    if read_all:
        print(dataset[:].shape)
    elif concurrent_chunks is None:
        async with asyncio.TaskGroup() as tg:
            for chunk_index in np.ndindex(*num_chunks):
                tg.create_task(chunk_read(chunk_index))
    elif concurrent_chunks == 1:
        for chunk_index in np.ndindex(*num_chunks):
            dataset.get_block_selection(chunk_index)
    else:
        semaphore = asyncio.Semaphore(concurrent_chunks)
        async def chunk_read_concurrent_limit(chunk_index):
            async with semaphore:
                return await chunk_read(chunk_index)
        async with asyncio.TaskGroup() as tg:
            for chunk_index in np.ndindex(*num_chunks):
                tg.create_task(chunk_read_concurrent_limit(chunk_index))

    elapsed = timeit.default_timer() - start_time
    elapsed_ms = elapsed * 1000.0
    print(f"Decoded in {elapsed_ms:.2f}ms")

if __name__ == "__main__":
    asyncio.run(main())
