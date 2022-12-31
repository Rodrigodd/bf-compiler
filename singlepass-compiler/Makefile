
build:
	rustc --crate-type staticlib bf_lib.rs -o bf_lib.a
	cargo run -p singlepass-compiler -- ../programs/mandelbrot.bf -o
	gcc -o mandelbrot -nostartfiles mandelbrot.o bf_lib.a -pthread -ldl

windows:
	rustc --crate-type staticlib bf_lib.rs --target=x86_64-pc-windows-msvc -Copt-level=2 -Clto -Cpanic=abort
	cargo run -p singlepass-compiler -- ../programs/factor.bf -o
	link /subsystem:console /entry:WinMain advapi32.lib advapi32.lib userenv.lib kernel32.lib kernel32.lib ws2_32.lib bcrypt.lib msvcrt.lib vcruntime.lib factor.o bf_lib.lib

clean: 
	rm -f bf_lib.a bf_lib.lib factor.o factor factor.exe mandelbrot.o mandelbrot mandelbrot.exe
