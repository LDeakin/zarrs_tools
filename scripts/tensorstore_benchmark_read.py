import tensorstore as ts
import numpy as np
import timeit
import asyncio
import click

# TODO: Benchmark against zarrs_benchmark_read_sync
# TODO: Benchmark against zarrs_benchmark_read_async

async def tensortore_benchmark_read_async(path, n_concurrent_chunks):
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
    tasks = []
    semaphore = asyncio.Semaphore(n_concurrent_chunks)
    for chunk_index in np.ndindex(*num_chunks):
        chunk_slice = [ts.Dim(inclusive_min=index*cshape, exclusive_max=min(index * cshape + cshape, dshape)) for (index, cshape, dshape) in zip(chunk_index, chunk_shape, domain_shape)]
        async def chunk_read(chunk_index, chunk_slice):
            print("Reading", chunk_index)
            await dataset[ts.IndexDomain(chunk_slice)].read()
            print("Read", chunk_index)
        async def chunk_read_future(chunk_index, chunk_slice):
            async with semaphore:
                return await chunk_read(chunk_index, chunk_slice)
        tasks.append(asyncio.ensure_future(chunk_read_future(chunk_index, chunk_slice)))

    await asyncio.gather(*tasks)
    elapsed = timeit.default_timer() - start_time
    print("Decoded in", elapsed)

@click.command()
@click.argument('path')
@click.option('--n_concurrent_chunks', default=4, help='Number of concurrent async chunk reads.')
def main(path, n_concurrent_chunks):
    loop = asyncio.get_event_loop()
    loop.run_until_complete(tensortore_benchmark_read_async(path, n_concurrent_chunks))
    loop.close()

if __name__ == "__main__":
    main()
