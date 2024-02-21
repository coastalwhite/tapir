#option: allow_trap = mmode_ecall
.include "./common/syscalls.S"

_start:
    li  x17, _SYSCALL_ASSERT_EQ
    li  x10, _SYSCALL_ASSERT_TYPE_U32
    li  x11,1
    li  x12,1
    ecall

    li  x17, _SYSCALL_EXIT
    li  x10, 0
    ecall