// option: allow_trap = mmode_ecall
#![no_std]
#![no_main]


include!("./common/panic.rs");
include!("./common/syscalls.rs");

#[no_mangle]
fn _start() -> ! {
    _syscall_assert_eq(1, 1);
    _syscall_exit(0);
}