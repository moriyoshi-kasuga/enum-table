use core::mem::MaybeUninit;

pub(crate) const fn transmute_uninit<T, const N: usize>(array: [MaybeUninit<T>; N]) -> [T; N] {
    // SAFETY: the array is fully initialized
    let slice = unsafe { core::mem::transmute::<&[MaybeUninit<T>; N], &[T; N]>(&array) };
    // SAFETY: the slice is derived from the passed-in array reference, ensuring no ownership issues.
    unsafe { core::ptr::read(slice) }
}

pub(crate) const fn from_usize<T>(u: &usize) -> &T {
    unsafe {
        // SAFETY: the usize is derived from a valid T
        core::mem::transmute::<&usize, &T>(u)
    }
}

pub(crate) const fn to_usize<T: Copy>(t: T) -> usize {
    #[inline(always)]
    const fn cast<U>(t: &impl Sized) -> &U {
        // SAFETY: This is safe because we ensure that the type T is a valid representation
        unsafe { core::mem::transmute(t) }
    }

    let t = &t;

    match const { core::mem::size_of::<T>() } {
        1 => *cast::<u8>(t) as usize,
        2 => *cast::<u16>(t) as usize,
        4 => *cast::<u32>(t) as usize,
        #[cfg(target_pointer_width = "64")]
        8 => *cast::<u64>(t) as usize,
        #[cfg(target_pointer_width = "32")]
        8 => panic!("Unsupported size: 64-bit value found on a 32-bit architecture"),
        _ => panic!("Values larger than u64 are not supported"),
    }
}
