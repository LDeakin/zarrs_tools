#!/usr/bin/env python3

import subprocess
import numpy as np
import click
from functools import reduce
from operator import mul

@click.command()
@click.argument('output_path')
@click.option('--shard', is_flag=True, show_default=True, default=False, help='Shard the array.')
@click.option('--compress', is_flag=True, show_default=True, default=False, help='Compress the array.')
def main(output_path, shard, compress):
    args = [
        'zarrs_binary2zarr',
        '--data-type',
        'uint16',
        '--fill-value',
        '0',
        '--separator',
        '.',
        '--array-shape',
        '1024,1024,1024',
        '--chunk-shape',
        '32,32,32',
        '--shard-shape' if shard else None,
        '128,1024,1024' if shard else None,
        '--bytes-to-bytes-codecs' if compress else None,
        '[ { "name": "blosc", "configuration": { "cname": "blosclz", "clevel": 9, "shuffle": "bitshuffle", "typesize": 2, "blocksize": 0 } } ]' if compress else None,
        output_path
    ]
    args = [arg for arg in args if arg is not None]

    p = subprocess.Popen(args, stdin=subprocess.PIPE)
    shape = [1024, 1024, 1024]
    bytes_per_element = 2

    # Write random bytes
    # def write_data():
    #     import random
    #     random.seed(123)
    #     n_bytes = reduce(mul, shape, 1) * bytes_per_element
    #     while n_bytes > 0:
    #         n = min(n_bytes, 1024)
    #         p.stdin.write(random.randbytes(n))
    #         n_bytes -= n

    # Write a function
    def write_data():
        for z in range(shape[0]):
            for y in range(shape[1]):
                elements = np.fromfunction(lambda x: (x + y**2 / 32 + z**3) % 65536, shape=(shape[2],), dtype=np.uint16).astype(np.uint16)
                p.stdin.write(elements.tobytes())

    write_data()
    p.stdin.close()
    p.wait()

if __name__ == "__main__":
    main()