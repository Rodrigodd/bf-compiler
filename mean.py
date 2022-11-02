#!/usr/bin/env python3
import numpy as np
import csv
import io
import math


with open(f"times/factor.csv", newline='') as csv_file:
    header = list(csv.reader(csv_file))[0]

    # tables will be a list of "commit, factor_time, mandelbrot_time, factor_impr, mandelbrot_impr"
    table = [[commit, [0,0], [0,0], [0,0], [0,0]] for commit in header[1:]]

# compute each mean and standart deviation of the mean
for i, name in enumerate(["factor", "mandelbrot"]):
    with open(f"times/{name}.csv", newline='') as csv_file:
        times = list(csv.reader(csv_file))
        times = np.array(times[1:]) # remove header
        times = times.astype(float)

        # each collumn is a commit
        for col in range(1,len(times[0])):
            values = times[:, col]
            mean = np.mean(values)
            stderr = np.std(values) / math.sqrt(len(values))
            
            table[col-1][i+1] = [mean, stderr]

        # because of the way that I collect the data, calculing the mean of the
        # improvements, is better than the improvent of the mean
        for col in range(2,len(times[0])):
            prev = times[:, col - 1]
            curr = times[:, col]

            ratio = curr/prev
            mean = np.mean(ratio)
            stderr = np.std(ratio) / math.sqrt(len(ratio))

            # convert to relative gain
            impr = (mean - 1) * 100.0
            impr_err = stderr * 100.0
            
            table[col-1][i+3] = [impr, impr_err]

# transform [mean, err] to "mean±err"
for i in range(0, len(table)):
    print(table[i])
    for j in [1,2,3,4]:
        mean = table[i][j][0]
        std = table[i][j][1]
        p = 0
        if std > 0:
            p = math.ceil(-math.log10(std)) + 1
        if p > 16:
            p = 16
        table[i][j] = f"{mean:.{p}f}±{std:.{p}f}"

table.insert(0, 'commit,factor.bf,mandelbrot.bf,delta,delta')
with open('times/benchmark.csv', 'w') as f:
    csv.writer(f).writerows(table)

