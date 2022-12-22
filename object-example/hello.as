; Code goes in the text section
SECTION .TEXT
	global _start 
	extern my_write

_start:
	; mov rax,1            ; 'write' system call = 4
	; mov rdi,1            ; file descriptor 1 = STDOUT
	; lea rsi, [rel hello] ; string to write
	; mov rdx,13           ; length of string to write
	; syscall              ; call the kernel
	
	lea rdi, [rel hello]
	mov rsi, 12
	call my_write

	; Terminate program
	mov rax,60            ; 'exit' system call
	mov rdi,0            ; exit with error code 0
	syscall              ; call the kernel
hello:     db 'Hello world!',10,0
