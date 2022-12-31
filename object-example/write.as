section .text
	global my_write

my_write:
	push rbp
	mov rbp, rsp
	mov rdx, rsi    ; length of string to write
	mov rsi, rdi    ; string to write
	mov rax, 1      ; 'write' system call = 4
	mov rdi, 1      ; file descriptor 1 = STDOUT
	syscall         ; call the kernel
	pop rbp
	ret
