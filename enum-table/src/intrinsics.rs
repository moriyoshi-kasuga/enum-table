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

#[cfg(any(debug_assertions, test))]
pub(crate) const fn is_sorted<T>(arr: &[T]) -> bool {
    if arr.is_empty() {
        return true;
    }
    let mut i = 0;
    while i < arr.len() - 1 {
        if !const_enum_lt(&arr[i], &arr[i + 1]) {
            return false;
        }
        i += 1;
    }
    true
}

/// Finds the index of `variant` in the `variants` slice using const-compatible equality.
///
/// This function is intended to be called inside `const { }` blocks in the derive macro,
/// so its O(N) cost is paid at compile time, not runtime.
pub const fn variant_index_of<T>(variant: &T, variants: &[T]) -> usize {
    let mut i = 0;
    while i < variants.len() {
        if const_enum_eq(variant, &variants[i]) {
            return i;
        }
        i += 1;
    }
    panic!(
        "enum-table: variant not found in VARIANTS array. This is a bug in the Enumable implementation."
    )
}

/// Binary search for a variant's index in the sorted `VARIANTS` array.
///
/// This is a `const fn` used by:
/// - The default `Enumable::variant_index` implementation (O(log N) fallback).
/// - The `get_const`, `get_mut_const`, `set_const`, and `remove_const` methods.
pub const fn binary_search_index<T: crate::Enumable>(variant: &T) -> usize {
    let variants = T::VARIANTS;
    let mut low = 0;
    let mut high = variants.len();

    while low < high {
        let mid = low + (high - low) / 2;
        if const_enum_lt(&variants[mid], variant) {
            low = mid + 1;
        } else {
            high = mid;
        }
    }

    debug_assert!(
        low < variants.len() && const_enum_eq(&variants[low], variant),
        "enum-table: variant not found in VARIANTS via binary search. This is a bug in the Enumable implementation."
    );

    low
}

/// Stable polyfill for `core::array::try_from_fn` (unstable `array_try_from_fn`).
///
/// Builds an array of `N` elements by calling `f(0)`, `f(1)`, …, `f(N-1)`.
/// If any call returns `Err(e)`, already-initialized elements are properly
/// dropped and the error is propagated.
pub(crate) fn try_collect_array<V, E, const N: usize>(
    mut f: impl FnMut(usize) -> Result<V, E>,
) -> Result<[V; N], E> {
    let mut array = core::mem::MaybeUninit::<[V; N]>::uninit();
    let mut initialized: usize = 0;

    for i in 0..N {
        match f(i) {
            Ok(v) => unsafe {
                array.as_mut_ptr().cast::<V>().add(i).write(v);
            },
            Err(e) => {
                for i in 0..initialized {
                    unsafe {
                        array.as_mut_ptr().cast::<V>().add(i).drop_in_place();
                    }
                }
                return Err(e);
            }
        }

        initialized += 1;
    }

    // SAFETY: all N elements have been initialized in the loop above.
    Ok(unsafe { array.assume_init() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Color {
        Red = 33,
        Green = 11,
        Blue = 222,
    }

    // --- const_enum_eq ---

    #[test]
    fn const_enum_eq_same_variant() {
        assert!(const_enum_eq(&Color::Red, &Color::Red));
        assert!(const_enum_eq(&Color::Green, &Color::Green));
        assert!(const_enum_eq(&Color::Blue, &Color::Blue));
    }

    #[test]
    fn const_enum_eq_different_variant() {
        assert!(!const_enum_eq(&Color::Red, &Color::Green));
        assert!(!const_enum_eq(&Color::Green, &Color::Blue));
        assert!(!const_enum_eq(&Color::Red, &Color::Blue));
    }

    // --- const_enum_lt ---

    #[test]
    fn const_enum_lt_ordering() {
        // Green(11) < Red(33) < Blue(222)
        assert!(const_enum_lt(&Color::Green, &Color::Red));
        assert!(const_enum_lt(&Color::Red, &Color::Blue));
        assert!(const_enum_lt(&Color::Green, &Color::Blue));
    }

    #[test]
    fn const_enum_lt_not_less() {
        assert!(!const_enum_lt(&Color::Red, &Color::Green));
        assert!(!const_enum_lt(&Color::Blue, &Color::Red));
        assert!(!const_enum_lt(&Color::Red, &Color::Red));
    }

    // --- sort_variants ---

    #[test]
    fn sort_variants_already_sorted() {
        let arr = [Color::Green, Color::Red, Color::Blue];
        let sorted = sort_variants(arr);
        assert_eq!(sorted, [Color::Green, Color::Red, Color::Blue]);
    }

    #[test]
    fn sort_variants_reverse_order() {
        let arr = [Color::Blue, Color::Red, Color::Green];
        let sorted = sort_variants(arr);
        assert_eq!(sorted, [Color::Green, Color::Red, Color::Blue]);
    }

    #[test]
    fn sort_variants_single_element() {
        let arr = [Color::Red];
        let sorted = sort_variants(arr);
        assert_eq!(sorted, [Color::Red]);
    }

    #[test]
    fn sort_variants_empty() {
        let arr: [Color; 0] = [];
        let sorted = sort_variants(arr);
        assert_eq!(sorted, []);
    }

    // --- is_sorted ---

    #[test]
    fn is_sorted_sorted_slice() {
        let arr = [Color::Green, Color::Red, Color::Blue];
        assert!(is_sorted(&arr));
    }

    #[test]
    fn is_sorted_unsorted_slice() {
        let arr = [Color::Red, Color::Green, Color::Blue];
        assert!(!is_sorted(&arr));
    }

    #[test]
    fn is_sorted_single_element() {
        let arr = [Color::Red];
        assert!(is_sorted(&arr));
    }

    #[test]
    fn is_sorted_empty() {
        let arr: [Color; 0] = [];
        assert!(is_sorted(&arr));
    }

    // --- variant_index_of ---

    #[test]
    fn variant_index_of_finds_each() {
        let sorted = [Color::Green, Color::Red, Color::Blue];
        assert_eq!(variant_index_of(&Color::Green, &sorted), 0);
        assert_eq!(variant_index_of(&Color::Red, &sorted), 1);
        assert_eq!(variant_index_of(&Color::Blue, &sorted), 2);
    }

    // --- binary_search_index ---

    #[test]
    fn binary_search_index_finds_each() {
        // Uses Enumable impl, so we need the derive
        #[derive(Debug, Clone, Copy, PartialEq, Eq, crate::Enumable)]
        #[repr(u8)]
        enum Fruit {
            Apple = 50,
            Banana = 10,
            Cherry = 200,
        }
        // VARIANTS sorted by discriminant: Banana(10), Apple(50), Cherry(200)
        assert_eq!(binary_search_index(&Fruit::Banana), 0);
        assert_eq!(binary_search_index(&Fruit::Apple), 1);
        assert_eq!(binary_search_index(&Fruit::Cherry), 2);
    }

    // --- try_collect_array ---

    #[test]
    fn try_collect_array_all_ok() {
        let result: Result<[i32; 4], &str> = try_collect_array(|i| Ok(i as i32 * 10));
        assert_eq!(result, Ok([0, 10, 20, 30]));
    }

    #[test]
    fn try_collect_array_error_at_first() {
        let result: Result<[i32; 3], &str> = try_collect_array(|_| Err("fail"));
        assert_eq!(result, Err("fail"));
    }

    #[test]
    fn try_collect_array_error_in_middle() {
        let result: Result<[i32; 5], usize> =
            try_collect_array(|i| if i == 2 { Err(i) } else { Ok(i as i32) });
        assert_eq!(result, Err(2));
    }

    #[test]
    fn try_collect_array_zero_length() {
        let result: Result<[i32; 0], &str> = try_collect_array(|_| unreachable!());
        assert_eq!(result, Ok([]));
    }

    #[test]
    fn try_collect_array_drops_on_error() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct Droppable;
        impl Drop for Droppable {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }

        DROP_COUNT.store(0, Ordering::SeqCst);

        let result: Result<[Droppable; 5], &str> =
            try_collect_array(|i| if i == 3 { Err("boom") } else { Ok(Droppable) });

        assert!(result.is_err());
        // Elements 0, 1, 2 were initialized then must be dropped by the guard
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn try_collect_array_no_leak_on_success() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct Droppable;
        impl Drop for Droppable {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }

        DROP_COUNT.store(0, Ordering::SeqCst);

        {
            let result: Result<[Droppable; 3], &str> = try_collect_array(|_| Ok(Droppable));
            assert!(result.is_ok());
            // Array is still alive, nothing dropped yet
            assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 0);
        }
        // Array goes out of scope, all 3 elements dropped
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 3);
    }
}
