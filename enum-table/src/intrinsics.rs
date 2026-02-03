macro_rules! const_operator {
    ($T:ident,$left:ident ($operator:tt) $right:ident) => {
        match const { core::mem::size_of::<$T>() } {
            1 => unsafe { *($left as *const $T as *const u8) $operator *($right as *const $T as *const u8) },
            2 => unsafe { *($left as *const $T as *const u16) $operator *($right as *const $T as *const u16) },
            4 => unsafe { *($left as *const $T as *const u32) $operator *($right as *const $T as *const u32) },
            8 => unsafe { *($left as *const $T as *const u64) $operator *($right as *const $T as *const u64) },
            16 => unsafe { *($left as *const $T as *const u128) $operator *($right as *const $T as *const u128) },

            _ => panic!(
                "enum-table: Enum discriminants larger than 128 bits are not supported. This is likely due to an extremely large enum or invalid memory layout."
            ),
        }
    };
}

#[inline(always)]
pub(crate) const fn const_enum_eq<T>(left: &T, right: &T) -> bool {
    const_operator!(T, left (==) right)
}

#[inline(always)]
#[cfg(debug_assertions)]
pub(crate) const fn const_enum_gt<T>(left: &T, right: &T) -> bool {
    const_operator!(T, left (>) right)
}

#[inline(always)]
pub(crate) const fn const_enum_lt<T>(left: &T, right: &T) -> bool {
    const_operator!(T, left (<) right)
}

pub const fn sort_variants<const N: usize, T>(mut arr: [T; N]) -> [T; N] {
    let mut i = 1;
    while i < N {
        let mut j = i;
        while j > 0 && const_enum_lt(&arr[j], &arr[j - 1]) {
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
        if const_enum_gt(&arr[i], &arr[i + 1]) {
            return false;
        }
        i += 1;
    }
    true
}
