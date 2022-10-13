#!/usr/bin/env python3
import numpy as np
import csv
import io
import math

table = [[[] for x in range(0,4)] for i in range(0,8)]
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
        table[i][j] = [mean, std]

    if i > 1:
        prev = table[i-1]
        prev_mean = prev[1][0] + prev[2][0]
        prev_std = math.sqrt(prev[1][1]**2 + prev[2][1]**2)

        now = table[i]
        now_mean = now[1][0] + now[2][0]
        now_std = math.sqrt(now[1][1]**2 + now[2][1]**2)

        change = now_mean/prev_mean
        change_std = change * math.sqrt((now_std/now_mean)**2 + (prev_std/prev_mean)**2)

        perc = (change - 1) * 100.0
        perc_std = change_std * 100.0
        
        table[i][3] = [perc, perc_std]

for i in range(1, len(table)):
    for j in range(1,4):
        if i == 1 and j == 3:
            continue
        mean = table[i][j][0]
        std = table[i][j][1]
        p = math.ceil(-math.log10(std)) + 1
        table[i][j] = f"{mean:.{p}f}±{std:.{p}f}"

with open('times/times_mean.csv', 'w') as f:
    csv.writer(f).writerows(table)

