# option: features = f
# option: allow_trap = ecall
.include "./common/syscalls.S"

_start:
    # Enable the F-ext in the mstatus CSR
    lui      x10,0x2
    csrw     mstatus,x10

    li       x10,0x3F800000

    fmv.w.x  f0,x10
    fmv.w.x  f1,zero

    fdiv.s   f2,f0,f1

    fmv.x.w  x11,f2

    li       x17, _SYSCALL_ASSERT_EQ
    li       x10, _SYSCALL_ASSERT_TYPE_F32
    li       x12,0x7F800000
    ecall

	csrr	 x11,fflags

    li       x17, _SYSCALL_ASSERT_EQ
    li       x10, _SYSCALL_ASSERT_TYPE_U32
    li       x12,0x0000008
    ecall

    li       x17, _SYSCALL_EXIT
    li       x10, 0
    ecall