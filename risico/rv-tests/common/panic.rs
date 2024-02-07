#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    if let Some(loc) = info.location() {
        unsafe {
            ::core::arch::asm!(
                "ecall",
                in("x17") 5,
                in("x10") loc.file().as_ptr(),
                in("x11") loc.file().len() as u32,
                in("x12") loc.line(),
                options(noreturn),
            )
        }
    } else {
        unsafe {
            ::core::arch::asm!(
                "ecall",
                in("x17") 5,
                in("x10") 0,
                options(noreturn),
            )
        }
    }
}