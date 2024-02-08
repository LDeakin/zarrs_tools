#!/usr/bin/env python3

import tensorstore as ts
import numpy as np
import timeit
import asyncio
import click

# TODO: Benchmark against zarrs_benchmark_read_sync
# TODO: Benchmark against zarrs_benchmark_read_async

async def tensorstore_benchmark_read_async(path, concurrent_chunks, read_all):
    dataset_future = ts.open({
        'driver': 'zarr3',
        'kvstore': {
            'driver': 'file',
            'path': path,
        },
        # 'context': {
        #     'cache_pool': {
        #         'total_bytes_limit': 100_000_000
        #     }
        # },
        # 'recheck_cached_data': 'open',
    })
    dataset = dataset_future.result()
    print(dataset)

    domain_shape = dataset.domain.shape
    chunk_shape = dataset.chunk_layout.write_chunk.shape # shard shape

    print("Domain shape", domain_shape)
    print("Chunk shape", chunk_shape)
    num_chunks =[(domain + chunk_shape - 1) // chunk_shape for (domain, chunk_shape) in zip(domain_shape, chunk_shape)]
    print("Number of chunks", num_chunks)

    start_time = timeit.default_timer()
    if read_all:
        print(dataset.read().result().shape)
    else:
        tasks = []
        semaphore = asyncio.Semaphore(concurrent_chunks)
        for chunk_index in np.ndindex(*num_chunks):
            chunk_slice = [ts.Dim(inclusive_min=index*cshape, exclusive_max=min(index * cshape + cshape, dshape)) for (index, cshape, dshape) in zip(chunk_index, chunk_shape, domain_shape)]
            async def chunk_read(chunk_index, chunk_slice):
                # print("Reading", chunk_index)
                await dataset[ts.IndexDomain(chunk_slice)].read()
                # print("Read", chunk_index)
            async def chunk_read_future(chunk_index, chunk_slice):
                async with semaphore:
                    return await chunk_read(chunk_index, chunk_slice)
            tasks.append(asyncio.ensure_future(chunk_read_future(chunk_index, chunk_slice)))

        await asyncio.gather(*tasks)

    elapsed = timeit.default_timer() - start_time
    elapsed_ms = elapsed * 1000.0
    print(f"Decoded in {elapsed_ms:.2f}ms")

@click.command()
@click.argument('path')
@click.option('--concurrent_chunks', default=4, help='Number of concurrent async chunk reads. Ignore if --read-all is set')
@click.option('--read_all', is_flag=True, show_default=True, default=False, help='Read the entire array in one operation.')
def main(path, concurrent_chunks, read_all):
    loop = asyncio.get_event_loop()
    loop.run_until_complete(tensorstore_benchmark_read_async(path, concurrent_chunks, read_all))
    loop.close()

if __name__ == "__main__":
    main()
