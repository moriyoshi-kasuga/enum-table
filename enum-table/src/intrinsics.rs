pub(crate) const fn from_usize<T>(u: &usize) -> &T {
    unsafe {
        // SAFETY: the usize is derived from a valid T
        core::mem::transmute::<&usize, &T>(u)
    }
}

pub(crate) const fn to_usize<T>(t: &T) -> usize {
    macro_rules! as_usize {
        ($t:ident as $type:ident) => {
            unsafe { *(t as *const T as *const $type) as usize }
        };
    }

    match const { core::mem::size_of::<T>() } {
        1 => as_usize!(t as u8),
        2 => as_usize!(t as u16),
        4 => as_usize!(t as u32),

        #[cfg(target_pointer_width = "64")]
        8 => as_usize!(t as u64),
        #[cfg(target_pointer_width = "32")]
        8 => panic!("Unsupported size: 64-bit value found on a 32-bit architecture"),

        _ => panic!("Values larger than 64 bits are not supported"),
    }
}
