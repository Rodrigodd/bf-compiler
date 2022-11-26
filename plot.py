#!/usr/bin/env python3
import matplotlib.pyplot as plt
import matplotlib as mpl
import numpy as np
import csv
import os

sets = [
    [1, 2],
    [1, 2, 3],
    [1, 2, 3, 4],
    [1, 2, 4, 5],
    [1, 2, 4, 5, 6],
    [1, 2, 4, 5, 6, 7],
    [1, 2, 4, 7],
    [1,2,3,4,5,6,7],

    [1, 7, 8],
    [7, 8, 9],
    [7, 9, 10],
    [1, 2, 4, 7, 9, 10],

    [9, 10, 11],
    [9, 10, 11, 12],
    [9, 10, 11, 12, 13],
    [1, 2, 4, 7, 9, 10, 13],

    [1,2,3,4,5,6,7,8,9,10,11,12,13],
]

plt.rcParams['svg.fonttype'] = 'none'
plt.rcParams["font.family"] = "Verdana"
mpl.use("SVG")

with plt.style.context('ggplot'), open('times/benchmark.csv', newline='') as csvfile:

    table = list(csv.reader(csvfile))
    table = np.array(table)

    # swap factor and mandel
    table[:, [2, 1]] = table[:, [1, 2]]

    labels = [table[0][1], table[0][2]]


    for index, choosed in enumerate(sets):
        bars = [table[i] for i in choosed]

        x = np.arange(len(labels))  # the label locations
        width = 0.9  # the width of the bars

        fig, ax = plt.subplots()

        rects = []
        for i, (label, factor, mandel, _, _) in enumerate(bars):
            factor = factor.split("±")[0]
            mandel = mandel.split("±")[0]
            bar = [float(factor), float(mandel)]
            w = width/len(bars)
            dx = w*i
            rects.append(ax.bar(x - width/2 + dx, bar, w, label=label, color=f'C{choosed[i]-1}'))

        # Add some text for labels, title and custom x-axis tick labels, etc.
        ax.set_ylabel('seconds')
        # ax.set_title('Execution time by change')
        ax.set_xticks(x, labels)
        ax.legend()

        for rect in rects:
            ax.bar_label(rect, padding=3, fmt='%.3g')

        fig.tight_layout()

        if not os.path.exists('plots'):
            os.mkdir('plots')
        plt.savefig(f'plots/plot{index}.svg')
        # plt.show()
