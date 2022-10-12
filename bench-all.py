#!/usr/bin/env python3
import os
import subprocess
import re

steps = [
    ("basic", "689bc2a"),
    ("precompute jumps", "0be6f60"),
    ("Add", "1ed437e"),
    ("Move", "cbd8ba5"),
    ("Clear", "bb4ff06"),
    ("AddTo", "81d659a"),
    ("MoveUntil", "d6b2b23"),
]

p = re.compile(b"real\t(\d+)m(\d+.\d+)s")

def time(command):
    os.system("git checkout " + commit)
    os.system("cargo build -p bf-optimized --release")
    out = subprocess.run(['bash', '-c',
        f"time ( {command} ) "],
         stderr=subprocess.PIPE)

    m = p.search(out.stderr)
    if m == None:
        print("didn't found real time in output of " + name)
        print(out.stderr)
        exit(1)
    min, secs = m.groups()
    
    t = float(min) * 60.0 + float(secs)

    return t

for interation in range(0,10):
    time_csv = open(f'times/times{interation}.csv', 'w')

    time_csv.write("change,factor.bf,mandelbrot.bf\n")
    for name, commit in steps:
        text = name + ","

        t1 = time('echo 179424691 | ./target/release/bf-optimized ./programs/factor.bf')
        t2 = time('./target/release/bf-optimized ./programs/mandelbrot.bf')

        text = text + str(t1) + "," + str(t2) + "\n"

        print(text)
        time_csv.write(text)

os.system("git checkout master")
