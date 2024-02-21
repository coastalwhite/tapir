// option: features = f
// option: allow_trap = mmode_ecall
#![no_std]
#![no_main]


include!("../common/panic.rs");
include!("../common/syscalls.rs");
include!("../common/routines.rs");

#[inline(never)]
fn add(a: f32, b: f32) -> f32 {
    a + b
}

#[no_mangle]
fn _start() -> ! {
    _mstatus_initialize_fs();
    _syscall_assert_eq(add(1.0, 1.0), 2.0f32);
    _syscall_exit(0);
}