LINKER := "D:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.33.31629\bin\Hostx64\x64\link.exe"

build: mandelbrot.o bf_lib.lib
	$(LINKER) /subsystem:console /entry:_start //DEBUG /stack:0x10000,0x7530 "advapi32.lib" "userenv.lib" "kernel32.lib" "ws2_32.lib" "bcrypt.lib" "msvcrt.lib" "vcruntime.lib" mandelbrot.o bf_lib.lib

bf_lib.lib: bf_lib.rs
	rustc --crate-type staticlib -Cpanic=abort -Clto -Cdebuginfo=0 -Copt-level=3 -Coverflow-checks=no bf_lib.rs

mandelbrot.o: src/main.rs
	cargo run -p singlepass-compiler -- ../programs/mandelbrot.bf -o

clean:
	rm bf_lib.lib mandelbrot.o mandelbrot.exe mandelbrot.exe.manifest mandelbrot.pdb
