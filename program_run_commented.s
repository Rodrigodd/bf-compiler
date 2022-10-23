.section .text.bf_interpreter::Program::run,"ax",@progbits
	.p2align	4, 0x90
	.type	bf_interpreter::Program::run,@function
bf_interpreter::Program::run:

    ; save registers of the caller
	push r15
	push r14
	push r13
	push r12
	push rbx

    ; allocate stack
	sub rsp, 32 

    ; rdi is the first argument
    ; now r13 stores the pointer to self
	mov r13, rdi

	; get stdout in rax
	call qword ptr [rip + std::io::stdio::stdout@GOTPCREL]

    ; lock stdout
	mov qword ptr [rsp + 8], rax ; save stdout on stack
	lea rdi, [rsp + 8]           ; set the address of stdout as argument to the call
	call qword ptr [rip + std::io::stdio::Stdout::lock@GOTPCREL]

	mov qword ptr [rsp], rax ; save lock on stack

	; get stdin in rax
	call qword ptr [rip + std::io::stdio::stdin@GOTPCREL]

    ; lock stdin
	mov qword ptr [rsp + 24], rax ; save stdin on stack
	lea rdi, [rsp + 24]           ; set the address of stdin as argument
	call qword ptr [rip + std::io::stdio::Stdin::lock@GOTPCREL]
	mov qword ptr [rsp + 8], rax ; save lock on stack
	mov byte ptr [rsp + 16], dl  ; ?

	; set program_counter and pointer to 0
	xorps xmm0, xmm0               ; clear the 128-bit XMM register
	movups xmmword ptr [r13], xmm0 ; write it to *self, overwritting program_counter and pointer


    ; 16 is pointer, 24 is capacity, 32 is lenght
	mov rdx, qword ptr [r13 + 32] ;  instructions.len()
	xor esi, esi
	mov rax, qword ptr [r13 + 16] ; instructions.ptr()

    ; rsi is self.pointer
    ; r12 is jump table

    ; make use of a jump table
	movzx ecx, byte ptr [rax + rsi]   ; read current instruction
	lea r12, [rip + .LJTI8_0]         ; address of jump table
	movsxd rdi, dword ptr [r12 + 4*rcx] ; read jump address of current instruction
	add rdi, r12                        ; (it is relative to jump table address)
	xor ecx, ecx 
	jmp rdi ; jump to it

; 'program: loop (not including first iteration)
.PROGRAM:
    ; r13 is &self
    ; rcx is self.program_counter
    ; rdx is self.instructions.len()
	mov rcx, qword ptr [r13]
	mov rdx, qword ptr [r13 + 32]
	add rcx, 1
	mov qword ptr [r13], rcx
	cmp rdx, rcx                  ; break if instructions.len() == program_counter
	je .BREAK_PROGRAM             ; .

    ; rsi is pointer
    ; rax is self.instructios.ptr

	mov rsi, qword ptr [r13 + 8]
	mov rax, qword ptr [r13 + 16]

    ; make use of a jump table
	movzx edi, byte ptr [rax + rcx]     ; instr = instruction[program_counter]
	movsxd rdi, dword ptr [r12 + 4*rdi] ; rel_jump = jump_table[instr]
	add rdi, r12                        ; jump += rel_jump + &jump_table
	jmp rdi                             ; goto `jump`

; Increase => self.memory[self.pointer] = self.memory[self.pointer].wrapping_add(1),
.LBB8_91:
	add byte ptr [r13 + rsi + 40], 1
	jmp .PROGRAM

; Decrease => self.memory[self.pointer] = self.memory[self.pointer].wrapping_sub(1),
.LBB8_90:
	add byte ptr [r13 + rsi + 40], -1
	jmp .PROGRAM

; MoveLeft
.LBB8_3:
	add esi, 29999
	jmp .LBB8_2

; MoveRight
.LBB8_1:
	add esi, 1

; MoveLeft/Right continuation
.LBB8_2:
	movzx eax, si
	mov ecx, eax
	shr ecx, 4
	imul ecx, ecx, 2237
	shr ecx, 22
	imul ecx, ecx, 30000
	sub eax, ecx
	movzx eax, ax
	mov qword ptr [r13 + 8], rax
	jmp .PROGRAM

; Input
.LBB8_4:
	add rsi, r13
	add rsi, 40

	lea rdi, [rsp + 8]

	mov edx, 1
	call qword ptr [rip + <std::io::stdio::StdinLock as std::io::Read>::read_exact@GOTPCREL]

	test rax, rax

	je .PROGRAM

	; error handling 
    ; any branch from here was removed, led to too big code.
	mov	r14, rax
	and	eax, 3
	lea	rcx, [rip + .LJTI8_1]
	movsxd	rax, dword ptr [rcx + 4*rax]
	add	rax, rcx
	jmp	rax

; JumpLeft
.LBB8_58:
	add rcx, -1
	mov edx, 1

	.p2align	4, 0x90
.LBB8_60:
	cmp rcx, -1
	je .BREAK_PROGRAM

	mov qword ptr [r13], rcx

	movzx ebx, byte ptr [rax + rcx]

	xor esi, esi
	cmp bl, 7
	sete sil

	add edx, esi

	xor esi, esi
	cmp bl, 6
	sete sil

	add rcx, -1
	sub edx, esi

	jne .LBB8_60 ; loop if deep is not zero
    jmp .PROGRAM ; (the label .PROGRAM was below here, but I moved it for better organization)

; Ouput
.LBB8_78:
	mov al, byte ptr [r13 + rsi + 40]

	mov byte ptr [rsp + 24], al

	mov rdi, rsp
	lea rsi, [rsp + 24]
	mov edx, 1
	call qword ptr [rip + <std::io::stdio::StdoutLock as std::io::Write>::write_all@GOTPCREL]

	mov r14, rax
	test rax, rax
	jne .LBB8_80

	mov rdi, rsp
	call qword ptr [rip + <std::io::stdio::StdoutLock as std::io::Write>::flush@GOTPCREL]
	mov r14, rax
	test rax, rax
	je .PROGRAM

.LBB8_80:
	mov rbx, qword ptr [rsp + 8]

	cmp byte ptr [rsp + 16], 0
	jne .LBB8_84

	mov rax, qword ptr [rip + std::panicking::panic_count::GLOBAL_PANIC_COUNT@GOTPCREL]

	mov rax, qword ptr [rax]

	shl rax, 1
	test rax, rax
	jne .LBB8_82

.LBB8_84:
	xor eax, eax

	xchg dword ptr [rbx], eax

	cmp eax, 2
	je .LBB8_85

	mov rdi, qword ptr [rsp]
	add dword ptr [rdi + 52], -1
	jne .LBB8_89

.LBB8_87:
	mov qword ptr [rdi], 0
	xor eax, eax

	xchg dword ptr [rdi + 48], eax

	cmp eax, 2
	jne .LBB8_89

	add rdi, 48
	call qword ptr [rip + std::sys::unix::locks::futex::Mutex::wake@GOTPCREL]
	jmp .LBB8_89

; JumpRight
.LBB8_64:

	cmp byte ptr [r13 + rsi + 40], 0
	jne .PROGRAM

	add rcx, 1
	mov esi, 1

	.p2align	4, 0x90
.LBB8_66:
	cmp rdx, rcx
	je .BREAK_PROGRAM

	mov qword ptr [r13], rcx

	movzx ebx, byte ptr [rax + rcx]

	xor edi, edi
	cmp bl, 6
	sete dil

	add esi, edi

	xor edi, edi
	cmp bl, 7
	sete dil

	add rcx, 1 ; program_counter += 1
	sub esi, edi

	jne .LBB8_66 ; loop if deep is not zero
	jmp .PROGRAM

.BREAK_PROGRAM:
	mov rbx, qword ptr [rsp + 8]

	cmp byte ptr [rsp + 16], 0
	jne .LBB8_72

	mov rax, qword ptr [rip + std::panicking::panic_count::GLOBAL_PANIC_COUNT@GOTPCREL]

	mov rax, qword ptr [rax]

	shl rax, 1
	test rax, rax
	jne .LBB8_70

.LBB8_72:
	xor eax, eax

	xchg dword ptr [rbx], eax

	cmp eax, 2
	je .LBB8_73

	mov rdi, qword ptr [rsp]
	add dword ptr [rdi + 52], -1
	jne .LBB8_77

.LBB8_75:
	mov qword ptr [rdi], 0
	xor eax, eax

	xchg dword ptr [rdi + 48], eax

	cmp eax, 2
	je .LBB8_76

.LBB8_77:
	xor r14d, r14d

.LBB8_89:
	mov rax, r14
	add rsp, 32
	pop rbx
	pop r12
	pop r13

	pop r14
	pop r15
	ret

.LBB8_73:
	mov rdi, rbx
	call qword ptr [rip + std::sys::unix::locks::futex::Mutex::wake@GOTPCREL]

	mov rdi, qword ptr [rsp]
	add dword ptr [rdi + 52], -1
	jne .LBB8_77
	jmp .LBB8_75

.LBB8_70:
	call qword ptr [rip + std::panicking::panic_count::is_zero_slow_path@GOTPCREL]

	test al, al
	jne .LBB8_72

	mov byte ptr [rbx + 4], 1
	jmp .LBB8_72

.LBB8_76:
	add rdi, 48
	call qword ptr [rip + std::sys::unix::locks::futex::Mutex::wake@GOTPCREL]
	jmp .LBB8_77

.LBB8_85:
	mov rdi, rbx
	call qword ptr [rip + std::sys::unix::locks::futex::Mutex::wake@GOTPCREL]

	mov rdi, qword ptr [rsp]
	add dword ptr [rdi + 52], -1
	jne .LBB8_89
	jmp .LBB8_87

.LBB8_82:
	call qword ptr [rip + std::panicking::panic_count::is_zero_slow_path@GOTPCREL]

	test al, al
	jne .LBB8_84

	mov byte ptr [rbx + 4], 1
	jmp .LBB8_84

.Lfunc_end8:
	.size	_ZN14bf_interpreter7Program3run17hdaf0b5f7d1a40593E, .Lfunc_end8-_ZN14bf_interpreter7Program3run17hdaf0b5f7d1a40593E
	.cfi_endproc
	.section	.rodata._ZN14bf_interpreter7Program3run17hdaf0b5f7d1a40593E,"a",@progbits
	.p2align	2
.LJTI8_0:
	.long	.LBB8_91-.LJTI8_0 ; Increase
	.long	.LBB8_90-.LJTI8_0 ; Decrease
	.long	.LBB8_1-.LJTI8_0 ; MoveRight
	.long	.LBB8_3-.LJTI8_0 ; MoveLeft
	.long	.LBB8_4-.LJTI8_0 ; Input
	.long	.LBB8_78-.LJTI8_0 ; Output
	.long	.LBB8_64-.LJTI8_0 ; JumpRight
	.long	.LBB8_58-.LJTI8_0 ; JumpLeft
