LINKER := "D:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.33.31629\bin\Hostx64\x64\link.exe"

all: rust

write_lib.lib:
	rustc --crate-type staticlib -Cpanic=abort -Clto -Cdebuginfo=0 -Copt-level=3 -Coverflow-checks=no write_lib.rs

out.elf:
	cargo run -p object-example -- obj

rust: write_lib.lib out.elf
	$(LINKER) /subsystem:console /entry:_start \
"advapi32.lib" "userenv.lib" "kernel32.lib" "ws2_32.lib" "bcrypt.lib" "msvcrt.lib" "vcruntime.lib" \
out.elf write_lib.lib

c:
	nasm -o hello.o hello.as -f elf64
	gcc -c -o write_lib.o write_lib.c
	gcc -o out -nostartfiles hello.o write_lib.o

windows:
	nasm -o hello.obj hello.as -f win64
	nasm -o write_lib.obj write_lib.as -f win64
	link /subsystem:console /entry:_main /defaultlib:kernel32.lib hello.obj write_lib.obj

clean:
	rm hello.obj out.exe write_lib.lib write_lib.obj
