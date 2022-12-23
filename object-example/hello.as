; Code goes in the text section
SECTION .TEXT
	global _main 
	extern my_write
    extern ExitProcess

_main:
    sub rsp, 8+8+8+32
    lea rdi, [rel hello]
    mov rsi, 13
    call my_write

	; Terminate program
    add rsp, 8+8+32+8
    xor eax, eax
    ret
hello:     db 'Hello world!',10,0
