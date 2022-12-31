section .text
	global _start 
	extern my_write

_start:
	lea rdi, [rel hello] ; string to write
	mov rsi, helloLen    ; length of string to write
	call my_write

	; Terminate program
	mov rax, 60          ; 'exit' system call = 60
	mov rdi, 0           ; exit with error code 0
	syscall              ; call the kernel
	hello:    db 'Hello world!',10
	helloLen: equ $-hello
