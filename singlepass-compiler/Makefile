
build:
	rustc --crate-type staticlib -Cpanic=abort -Clto -Cdebuginfo=0 -Coverflow-checks=no bf_lib.rs -o bf_lib.a
	cargo run -p singlepass-compiler -- ../programs/mandelbrot.bf -o
	gcc -o mandelbrot -nostartfiles mandelbrot.o bf_lib.a -pthread -ldl

clean: 
	rm bf_lib.a mandelbrot.o mandelbrot
