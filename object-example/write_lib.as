
SECTION .TEXT
    global my_write
    extern GetStdHandle
    extern WriteConsoleA

my_write:
    ; hStdOut = GetStdHandle(STD_OUTPUT_HANDLE)
    sub rsp, 8
    mov ecx, -11
    call GetStdHandle

    ; WriteFile(hstdOut, message, length(message), &bytes, 0);
    mov rcx, rax
    mov rdx, rdi
    mov r8, rsi
    lea r9, [rsp-32]
    mov qword [rsp-72], 0
    call WriteConsoleA

    add rsp, 8
    ret
