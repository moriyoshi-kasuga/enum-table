pub(crate) const fn from_usize<T>(u: &usize) -> &T {
    unsafe {
        // SAFETY: This function is only called with usize values that were originally
        // derived from valid enum discriminants via to_usize(). The transmute is safe
        // because we maintain the invariant that the usize was created from a T of the
        // same type and the memory layout is preserved.
        core::mem::transmute::<&usize, &T>(u)
    }
}

pub(crate) const fn copy_variant<T>(t: &T) -> T {
    unsafe { core::ptr::read(t) }
}

pub(crate) const fn copy_from_usize<T>(u: &usize) -> T {
    let reference = from_usize::<T>(u);
    copy_variant(reference)
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
        8 => panic!("enum-table: Cannot handle 64-bit enum discriminants on 32-bit architecture. Consider using smaller discriminant values or compile for 64-bit target."),

        _ => panic!("enum-table: Enum discriminants larger than 64 bits are not supported. This is likely due to an extremely large enum or invalid memory layout."),
    }
}

pub const fn sort_variants<const N: usize, T>(mut arr: [T; N]) -> [T; N] {
    let mut i = 0;
    while i < N {
        let mut j = i + 1;
        while j < N {
            if to_usize(&arr[j]) < to_usize(&arr[i]) {
                arr.swap(i, j);
            }
            j += 1;
        }
        i += 1;
    }
    arr
}

pub(crate) const fn is_sorted<T>(arr: &[T]) -> bool {
    let mut i = 0;
    while i < arr.len() - 1 {
        if to_usize(&arr[i]) > to_usize(&arr[i + 1]) {
            return false;
        }
        i += 1;
    }
    true
}
