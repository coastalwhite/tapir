/// Safely convert a [`u32`] to a [`usize`] with zero cost
///
/// This will attempt to throw a compiler warning when compiled for a target with a usize that is
/// is not supported. If that cannot be done, it will throw a runtime warning.
#[inline]
pub fn u32_to_usize(n: u32) -> usize {
    #[cfg(target_pointer_width = "16")]
    compile_error!("RISICO does not a usize of size 16");

    #[cfg(target_pointer_width = "8")]
    compile_error!("RISICO does not a usize of size 8");

    if usize::BITS < 32 {
        panic!("This platform is not supported. Usize is smaller than 32 bits.");
    }

    n as usize
}

#[inline]
pub const fn sign_extend(n: u32, num_bits: u32) -> i32 {
    (n << (31 - num_bits)) as i32 >> (31 - num_bits)
}
