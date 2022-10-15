#!/usr/bin/env python3
import os
import subprocess
import re

steps = [
    ("basic", "689bc2a"),
    ("precomputejumps", "0be6f60"),
    ("Add", "1ed437e"),
    ("Move", "cbd8ba5"),
    ("Clear", "bb4ff06"),
    ("AddTo", "81d659a"),
    ("MoveUntil", "d6b2b23"),
]

p = re.compile(b"real\t(\d+)m(\d+.\d+)s")

def time(command):
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

# compile each commit
os.system('mkdir bench_programs')
for name, commit in steps:
    if os.system(f'git checkout {commit}') != 0:
        raise('git checkout failed')
    os.system('cargo build -p bf-optimized --release')
    os.system(f'mv ./target/release/bf-optimized ./bench_programs/{name}')

for interation in range(0,10):
    print(f'interation {interation}')
    time_csv = open(f'times/times{interation}.csv', 'w')

    time_csv.write("change,factor.bf,mandelbrot.bf\n")
    for name, commit in steps:
        print(f'running for {name}')
        text = name + ","

        t1 = time(f'echo 179424691 | ./bench_programs/{name} ./programs/factor.bf')
        t2 = time(f'./bench_programs/{name} ./programs/mandelbrot.bf')

        text = text + str(t1) + "," + str(t2) + "\n"

        print(text)
        time_csv.write(text)

os.system("git checkout master")
