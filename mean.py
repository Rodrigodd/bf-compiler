#!/usr/bin/env python3
import numpy as np
import csv
import io
import math

table = [[[] for x in range(0,3)] for i in range(0,8)]
for i in range(1,7):
    print(i)
    with open(f"times/times{i}.csv", newline='') as csvfile:
        times = list(csv.reader(csvfile))
        table[0] = times[0]
        for i in range(1,len(times)):
            name, factor, mandel = times[i]
            table[i][0] = name
            table[i][1].append(float(factor))
            table[i][2].append(float(mandel))
for i in range(1, len(table)):
    for j in range(1,3):
        mean = np.mean(table[i][j])
        std = np.std(table[i][j], ddof=1)
        # let p = (-err.log10()).ceil() as usize + 1;
        # format!("{:.p$} +/- {:.p$}", val, err, p = p)
        p = math.ceil(-math.log10(std)) + 1
        table[i][j] = f"{mean:.{p}f}±{std:.{p}f}"

with open('times/times_mean.csv', 'w') as f:
    csv.writer(f).writerows(table)

