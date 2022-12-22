
rust:
	cargo run -p object-example -- obj
	rustc --crate-type staticlib -Cpanic=abort -Clto -Cdebuginfo=0 -Coverflow-checks=no write_lib.rs
	gcc -o out -nostartfiles out.elf libwrite_lib.a -pthread -ldl

c:
	nasm -o hello.o hello.as -f elf64
	gcc -c -o write_lib.o write_lib.c
	gcc -o out -nostartfiles hello.o write_lib.o

clean:
	rm write_lib.o out out.elf libwrite_lib.a hello.o
