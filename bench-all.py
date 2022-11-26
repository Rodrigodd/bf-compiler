#!/usr/bin/env python3
import os
import sys
import subprocess
import re

steps = [
    ("bf-optimized", "basic", "689bc2a"),
    ("bf-optimized", "precomputejumps", "0be6f60"),
    ("bf-optimized", "Add", "1ed437e"),
    ("bf-optimized", "Move", "cbd8ba5"),
    ("bf-optimized", "Clear", "bb4ff06"),
    ("bf-optimized", "AddTo", "81d659a"),
    ("bf-optimized", "MoveUntil", "d6b2b23"),

    ("bf-singlepass-jit", "singlepass-jit", "bc3e2aa"),
    ("bf-singlepass-jit", "dynasm-jit", "f5a15d4"),
    ("bf-optimized-jit", "optimized-jit", "872c0c8"),

    ("bf-cranelift-jit", "cranelift-jit", "6ed28d7"),
    ("bf-cranelift-jit", "repeat-cranelift-jit", "9d6664f"),
    ("bf-cranelift-jit", "optimized-cranelift-jit", "eaf72a5"),
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

def compile():
    # compile each commit
    os.system('rm -rf bench_programs')
    os.system('mkdir bench_programs')
    for package, name, commit in steps:
        if os.system(f'git checkout {commit}') != 0:
            raise('git checkout failed')
        os.system(f'cargo build -p {package} --release')
        os.system(f'mv ./target/release/{package} ./bench_programs/{name}')
    os.system("git checkout master")

def run():
    programs = [
        ('mandelbrot','./bench_programs/{name} ./programs/mandelbrot.bf'),
        ('factor', 'echo 179424691 | ./bench_programs/{name} ./programs/factor.bf'),
    ]


    os.system('mkdir times -p')
    for name, command in programs:
        csv_file = open(f'times/{name}.csv', 'w')

        # header
        header = "interation"
        for _, name, _ in steps:
           header = header + "," + name

        csv_file.write(header + '\n')
        csv_file.flush()

        # data
        for interation in range(0,20):
            print(f'interation {interation}')

            text = f"{interation}"
            for _, name, commit in steps:
                print(f'running for {name}')

                t = time(command.format(name = name))
                print(f'{interation:2} {name}: {t}')

                text = text + "," + str(t)

            csv_file.write(text + '\n')
            csv_file.flush()

        csv_file.close()

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print('expected 1 argument')
        exit(1)

    arg = sys.argv[1]
    if arg == '-c' or arg == '--compile':
        compile()
    elif arg == '-r' or arg == '--run':
        run()
    else:
        print('unkown argument: expected --compile (-c) or --run (-r)')
        exit(2)
