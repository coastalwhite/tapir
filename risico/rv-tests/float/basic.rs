// option: features = f
#![no_std]
#![no_main]


include!("../common/panic.rs");
include!("../common/syscalls.rs");

#[inline(never)]
fn add(a: f32, b: f32) -> f32 {
    a + b
}

#[no_mangle]
fn _start() -> ! {
    _syscall_assert_eq(add(0.2, 0.3) as u32, 0.5 as u32);
    _syscall_exit(0);
}