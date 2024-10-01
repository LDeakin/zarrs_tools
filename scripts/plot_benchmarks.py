#!/usr/bin/env python3

import matplotlib.pyplot as plt
import pandas as pd
# from matplotlib._layoutgrid import plot_children
from matplotlib.lines import Line2D

LEGEND_COLS = 3
SPLIT_AXIS = False # Does not use LOG_...
LOG_SCALE_TIME = True
LOG_SCALE_MEMORY = True
LOG2_SCALE_CONCURRENCY = True

implementations = {
    "zarrs_rust": "LDeakin/zarrs (0.17.0-beta.3)",
    "tensorstore_python": "google/tensorstore (0.1.65)",
    "zarr_python": "zarr-developers/zarr-python (3.0.0a6)",
}

images = {
    "data/benchmark.zarr": "Uncompressed",
    "data/benchmark_compress.zarr": "Compressed",
    "data/benchmark_compress_shard.zarr": "Compressed\n + Sharded",
}

plt.rcParams.update({
    "text.usetex": True,
    "font.family": "sans-serif",
    "font.sans-serif": ["lmodern"],
    "axes.autolimit_mode": "round_numbers",
})

def split_axis(ax_h, ax_l):
    ax_h.spines.bottom.set_visible(False)
    ax_l.spines.top.set_visible(False)
    ax_h.xaxis.tick_top()
    ax_h.tick_params(labeltop=False)  # don't put tick labels at the top
    ax_l.xaxis.tick_bottom()

    d = .5  # proportion of vertical to horizontal extent of the slanted line
    kwargs = dict(marker=[(-1, -d), (1, d)], markersize=12,
                linestyle="none", color='k', mec='k', mew=1, clip_on=False)
    ax_h.plot([0, 1], [0, 0], transform=ax_h.transAxes, **kwargs)
    ax_l.plot([0, 1], [1, 1], transform=ax_l.transAxes, **kwargs)

def plot_read_all():
    df = pd.read_csv("docs/benchmark_read_all.csv", header=[0, 1], index_col=0)
    df.index = ["Uncompressed", "Compressed", "Compressed\n+ Sharded"]
    df.rename(level=1, columns=implementations, inplace=True)
    print(df)


    # Prepare split axis figure and axes
    fig = plt.figure(figsize=(9, 4), layout="constrained")
    spec = fig.add_gridspec(2, 2, hspace=0.005)
    fig.get_layout_engine().set(h_pad = 0)
    if SPLIT_AXIS:
        ax_time_h = fig.add_subplot(spec[0, 0])
        ax_time_l = fig.add_subplot(spec[1, 0])
        ax_time = fig.add_subplot(spec[:, 0], frameon=False)
        split_axis(ax_time_h, ax_time_l)
    else:
        ax_time = fig.add_subplot(spec[:, 0])
    ax_mem = fig.add_subplot(spec[:, 1])
    # plot_children(fig)

    # Plot the data
    if SPLIT_AXIS:
        df["Time (s)"].plot(kind='bar', ax=ax_time_h)
        df["Time (s)"].plot(kind='bar', ax=ax_time_l)
        ax_time_l.set_ylim(0, 6)
        ax_time_h.set_ylim(20, 80)
    else:
        df["Time (s)"].plot(kind='bar', ax=ax_time)
        if LOG_SCALE_TIME:
            ax_time.set_yscale('log')
        else:
            ax_time.set_ylim(0, 80)
    fig.legend(loc='outside upper center', ncol=LEGEND_COLS, title="Zarr V3 implementation", borderaxespad=0)
    df["Memory (GB)"].plot(kind='bar', ax=ax_mem)
    ax_mem.set_ylim(0, 30)

    # Styling
    ax_time.set_ylabel("Elapsed time (s)")
    if SPLIT_AXIS:
        ax_time.tick_params(labelcolor='none', which='both', top=False, bottom=False, left=False, right=False)
        ax_time_h.set_ylabel("Phony", color='none')
        ax_time_l.tick_params(axis='x', labelrotation=0)
    else:
        ax_time.tick_params(axis='x', labelrotation=0)
    ax_mem.set_ylabel("Peak memory usage (GB)")
    ax_mem.tick_params(axis='x', labelrotation=0)

    if SPLIT_AXIS:
        ax_time_l.get_legend().remove()
        ax_time_h.get_legend().remove()
    else:
        ax_time.get_legend().remove()
    ax_mem.get_legend().remove()

    fig.savefig("docs/benchmark_read_all.svg")
    fig.savefig("docs/benchmark_read_all.pdf")


def plot_read_chunks():
    df = pd.read_csv("docs/benchmark_read_chunks.csv", header=[0, 1], index_col=[0, 1])
    # df.assign(Concurrency=df.index.get_level_values('Concurrency'))
    df = df.reset_index(level=1)
    print(df)

    fig = plt.figure(figsize=(9, 4), layout="constrained")
    spec = fig.add_gridspec(2, 2, hspace=0.005)
    fig.get_layout_engine().set(h_pad = 0)
    if SPLIT_AXIS:
        ax_time_h = fig.add_subplot(spec[0, 0])
        ax_time_l = fig.add_subplot(spec[1, 0])
        split_axis(ax_time_h, ax_time_l)
        ax_time = fig.add_subplot(spec[:, 0], frameon=False)
        ax_time.tick_params(labelcolor='none', which='both', top=False, bottom=False, left=False, right=False)
    else:
        ax_time = fig.add_subplot(spec[:, 0])
        
    ax_mem = fig.add_subplot(spec[:, 1])

    cmap = plt.rcParams['axes.prop_cycle'].by_key()['color'][:len(implementations)]
    # print(df.groupby("Image"))
    image_ls = {'data/benchmark.zarr': ":", 'data/benchmark_compress.zarr': '--', 'data/benchmark_compress_shard.zarr': '-'}
    for image, row in df.groupby("Image"):
        if SPLIT_AXIS:
            row.plot(x="Concurrency", y="Time (s)", ax=ax_time_h, color=cmap, ls=image_ls[image])
            row.plot(x="Concurrency", y="Time (s)", ax=ax_time_l, color=cmap, ls=image_ls[image])
        else:
            row.plot(x="Concurrency", y="Time (s)", ax=ax_time, color=cmap, ls=image_ls[image])
        row.plot(x="Concurrency", y="Memory (GB)", ax=ax_mem, color=cmap, ls=image_ls[image])
        # print(row)

    # Custom legend
    cmap = plt.rcParams['axes.prop_cycle'].by_key()['color']
    custom_lines = [Line2D([0], [0], color=cmap[i]) for i in range(len(implementations))]
    fig.legend(custom_lines, [implementation.replace(" ", " ") for implementation in implementations.values()], loc="outside upper left", ncol=2, title="Zarr V3 implementation", borderaxespad=0)
    custom_lines = [Line2D([0], [0], color='k', ls=':'),
                Line2D([0], [0], color='k', ls='--'),
                Line2D([0], [0], color='k', ls='-')]
    fig.legend(custom_lines, images.values(), loc="outside upper right", ncol=2, title="Dataset", borderaxespad=0)

    if SPLIT_AXIS:
        ax_time_h.get_legend().remove()
        ax_time_l.get_legend().remove()
    else:
        ax_time.get_legend().remove()
    ax_mem.get_legend().remove()

    ax_all = fig.add_subplot(spec[1, :], frameon=False)
    ax_all.tick_params(labelcolor='none', which='both', top=False, bottom=False, left=False, right=False)
    ax_all.set_xlabel("Concurrent chunks")

    ax_time.set_ylabel("Elapsed time (s)")

    xticks = [1, 2, 4, 8, 16, 32]
    if SPLIT_AXIS:
        ax_time_h.set_ylim(4, 110)
        ax_time_h.set_yticks([10, 30, 50, 70, 90, 110])
        ax_time_l.set_ylim(0, 4)
        ax_time_l.set_yticks([0, 1, 2, 3, 4])
        ax_time_h.set_xlim(1, 32)
        ax_time_l.set_xlim(1, 32)
        ax_time_h.set_xticks(xticks)
        ax_time_l.set_xticks(xticks)
        ax_time_h.set_xlabel(None)
        ax_time_l.set_xlabel(None)
    else:
        if LOG_SCALE_TIME:
            ax_time.set_yscale('log')
        else:
            ax_time.set_ylim(0, 110)
        if LOG2_SCALE_CONCURRENCY:
            ax_time.set_xscale('log', base=2)
            ax_time.xaxis.set_major_formatter(plt.FuncFormatter("{:.0f}".format))
        ax_time.set_xlim(1, 32) 
        ax_time.set_xticks(xticks)
        ax_time.set_xlabel(None)

    if LOG_SCALE_MEMORY:
        ax_mem.set_yscale('log')
    if LOG2_SCALE_CONCURRENCY:
        ax_mem.set_xscale('log', base=2)
        ax_mem.xaxis.set_major_formatter(plt.FuncFormatter("{:.0f}".format))
    else:
        ax_mem.set_ylim(0, 30)
    ax_mem.set_xlim(1, 32)
    ax_mem.set_xticks(xticks)
    ax_mem.set_xlabel(None)
    ax_mem.set_ylabel("Peak memory usage (GB)")

    fig.savefig("docs/benchmark_read_chunks.svg")
    fig.savefig("docs/benchmark_read_chunks.pdf")


if __name__ == "__main__":
    plot_read_all()
    plot_read_chunks()

plt.show()
