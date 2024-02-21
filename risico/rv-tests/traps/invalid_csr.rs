// option: allow_trap = illegal_instr,ecall,missing_csr
#![no_std]
#![no_main]

include!("../common/panic.rs");
include!("../common/syscalls.rs");

core::arch::global_asm!(r#"
.text
.globl _trap_vector
_trap_vector:
    unimp
    unimp
    j       _trap_illegal_instruction
    unimp
    unimp
    unimp
    unimp
    unimp
    unimp
    unimp
    unimp
    unimp
    unimp
    unimp
    unimp
    unimp
    
.globl _trap_illegal_instruction
_trap_illegal_instruction:
    li a0,0x1337
    mret

.globl _start
_start:
    la    a0,_trap_vector
    ori   a0,a0,1
    csrw  mtvec,a0
    j  {}
"#, sym main);

fn main() -> ! {
    let mut magic: u32 = 0;

    _syscall_assert_eq(magic, 0);

    unsafe {
        core::arch::asm!(
            "csrr   zero,0",
            lateout("a0") magic,
        );
    }

    _syscall_assert_eq(magic, 0x1337);
    _syscall_exit(0);
}