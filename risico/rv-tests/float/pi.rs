// option: features = f

#![no_std]
#![no_main]

include!("../common/panic.rs");
include!("../common/syscalls.rs");

fn findpi() -> f32 {
    let mut pi_over_4 = 0.0f32;

    for i in 0..10 {
        pi_over_4 += ((i * 2 + 1) as f32).recip();
        pi_over_4 -= ((i * 2 + 3) as f32).recip();
    }

    pi_over_4
}

#[no_mangle]
fn _start() -> ! {
    _syscall_assert_eq(findpi(), 3.131);
    _syscall_exit(0);
}