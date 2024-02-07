# option: features = f
.include "./common/syscalls.S"

_start:
    li       x10,1

    fmv.w.x  f0,x10
    fmv.w.x  f1,x10

    fadd.s   f2,f0,f1

    fmv.x.w  x11,f2

    li  x17, _SYSCALL_ASSERT_EQ
    li  x10, _SYSCALL_ASSERT_TYPE_F32
    li  x12,2
    ecall

    li  x17, _SYSCALL_EXIT
    li  x10,0
    ecall
