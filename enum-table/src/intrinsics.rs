#[inline(always)]
pub(crate) const fn cast_variant<T>(u: &usize) -> &T {
    #[cfg(target_endian = "big")]
    unsafe {
        &*(u as *const usize)
            .cast::<u8>()
            .add(core::mem::size_of::<usize>() - core::mem::size_of::<T>())
            .cast::<T>()
    }
    #[cfg(target_endian = "little")]
    unsafe {
        &*(u as *const usize as *const T)
    }
}

#[inline(always)]
pub(crate) const fn into_variant<T: Copy>(u: usize) -> T {
    *cast_variant::<T>(&u)
}

#[inline(always)]
pub(crate) const fn to_usize<T>(t: &T) -> usize {
    macro_rules! as_usize {
        ($type:ident) => {
            unsafe { *(t as *const T as *const $type) as usize }
        };
    }

    match const { core::mem::size_of::<T>() } {
        1 => as_usize!(u8),
        2 => as_usize!(u16),
        4 => as_usize!(u32),

        #[cfg(target_pointer_width = "64")]
        8 => as_usize!(u64),
        #[cfg(target_pointer_width = "32")]
        8 => panic!(
            "enum-table: Cannot handle 64-bit enum discriminants on 32-bit architecture. Consider using smaller discriminant values or compile for 64-bit target."
        ),

        _ => panic!(
            "enum-table: Enum discriminants larger than 64 bits are not supported. This is likely due to an extremely large enum or invalid memory layout."
        ),
    }
}

pub const fn sort_variants<const N: usize, T>(mut arr: [T; N]) -> [T; N] {
    let mut i = 1;
    while i < N {
        let mut j = i;
        while j > 0 && to_usize(&arr[j]) < to_usize(&arr[j - 1]) {
            arr.swap(j, j - 1);
            j -= 1;
        }
        i += 1;
    }
    arr
}

#[cfg(debug_assertions)]
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
