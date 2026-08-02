use crate::Enumerable;

/// Compares the raw bytes of `left` and `right` for equality.
///
/// # Safety
///
/// `T` must have no padding bytes, otherwise this reads uninitialized memory.
#[inline(always)]
const unsafe fn bytes_eq<T>(left: &T, right: &T) -> bool {
    let left = left as *const T as *const u8;
    let right = right as *const T as *const u8;
    let len = core::mem::size_of::<T>();
    let mut i = 0;
    while i < len {
        // SAFETY: `i < len == size_of::<T>()`, so both reads are in bounds.
        if unsafe { *left.add(i) != *right.add(i) } {
            return false;
        }
        i += 1;
    }
    true
}

/// Orders `left` and `right` by the unsigned bit-pattern of their raw bytes
/// (most-significant byte first, independent of target endianness).
///
/// # Safety
///
/// `T` must have no padding bytes, otherwise this reads uninitialized memory.
#[inline(always)]
const unsafe fn bytes_lt<T>(left: &T, right: &T) -> bool {
    let left = left as *const T as *const u8;
    let right = right as *const T as *const u8;
    let len = core::mem::size_of::<T>();

    let mut i = len;
    while i > 0 {
        i -= 1;
        let byte_index = if cfg!(target_endian = "little") {
            i
        } else {
            len - 1 - i
        };
        // SAFETY: `byte_index < len == size_of::<T>()`, so both reads are in bounds.
        let (l, r) = unsafe { (*left.add(byte_index), *right.add(byte_index)) };
        if l != r {
            return l < r;
        }
    }
    false
}

/// Sorts variants by the unsigned bit-pattern of their in-memory representation.
///
/// This is intended to be called inside `const {}` blocks in the derive macro,
/// so its O(N log N) cost is paid at compile time, not runtime.
///
/// # Safety
///
/// `T` must have no padding bytes (see [`crate::Enumerable`]'s safety contract).
#[doc(hidden)]
pub const unsafe fn sort_variants<const N: usize, T: Copy>(mut arr: [T; N]) -> [T; N] {
    let mut i = 1;
    while i < N {
        let mut j = i;
        // SAFETY: caller upholds the padding-free contract documented above.
        while j > 0 && unsafe { bytes_lt(&arr[j], &arr[j - 1]) } {
            arr.swap(j, j - 1);
            j -= 1;
        }
        i += 1;
    }
    arr
}

/// Finds the index of `variant` in the `variants` slice using byte-level equality.
///
/// This is intended to be called inside `const {}` blocks in the derive macro,
/// so its O(N) cost is paid at compile time, not runtime.
///
/// # Safety
///
/// `T` must have no padding bytes (see [`crate::Enumerable`]'s safety contract).
#[doc(hidden)]
pub const unsafe fn variant_index_of<T>(variant: &T, variants: &[T]) -> usize {
    let mut i = 0;
    while i < variants.len() {
        // SAFETY: caller upholds the padding-free contract documented above.
        if unsafe { bytes_eq(variant, &variants[i]) } {
            return i;
        }
        i += 1;
    }
    panic!(
        "enum-table: variant not found in VARIANTS array. This is a bug in the Enumerable implementation."
    )
}

/// Checks that `arr` is sorted by the unsigned bit-pattern of its elements.
#[cfg(debug_assertions)]
pub(crate) const fn is_sorted<T: Enumerable>(arr: &[T]) -> bool {
    if arr.is_empty() {
        return true;
    }
    let mut i = 0;
    while i < arr.len() - 1 {
        // SAFETY: `T: Enumerable`'s safety contract guarantees no padding bytes.
        if !unsafe { bytes_lt(&arr[i], &arr[i + 1]) } {
            return false;
        }
        i += 1;
    }
    true
}

/// Binary search for a variant's index in the sorted `VARIANTS` array.
///
/// This is a `const fn` used by:
/// - The default `Enumerable::variant_index` implementation (O(log N) fallback).
/// - The `get_const`, `get_mut_const`, `set_const`, and `remove_const` methods.
pub(crate) const fn binary_search_index<T: Enumerable>(variant: &T) -> usize {
    let variants = T::VARIANTS;
    let mut low = 0;
    let mut high = variants.len();

    while low < high {
        let mid = low + (high - low) / 2;
        // SAFETY: `T: Enumerable`'s safety contract guarantees no padding bytes.
        if unsafe { bytes_lt(&variants[mid], variant) } {
            low = mid + 1;
        } else {
            high = mid;
        }
    }

    debug_assert!(
        low < variants.len() && unsafe { bytes_eq(&variants[low], variant) },
        "enum-table: variant not found in VARIANTS via binary search. This is a bug in the Enumerable implementation."
    );

    low
}

/// Stable polyfill for `core::array::try_from_fn` (unstable `array_try_from_fn`).
pub(crate) fn try_collect_array<V, E, const N: usize>(
    mut f: impl FnMut(usize) -> Result<V, E>,
) -> Result<[V; N], E> {
    struct InitGuard<V> {
        ptr: *mut V,
        len: usize,
    }

    impl<V> Drop for InitGuard<V> {
        fn drop(&mut self) {
            for i in 0..self.len {
                // SAFETY: elements `0..self.len` were initialized by the caller
                // and have not been read out yet.
                unsafe { self.ptr.add(i).drop_in_place() };
            }
        }
    }

    let mut array = core::mem::MaybeUninit::<[V; N]>::uninit();
    let mut guard = InitGuard {
        ptr: array.as_mut_ptr().cast::<V>(),
        len: 0,
    };

    for i in 0..N {
        let v = f(i)?;
        // SAFETY: index `i` is within bounds (`i < N`) and not yet initialized.
        unsafe { guard.ptr.add(i).write(v) };
        guard.len = i + 1;
    }

    core::mem::forget(guard);
    // SAFETY: all N elements have been initialized in the loop above.
    Ok(unsafe { array.assume_init() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, crate::Enumerable)]
    enum Color {
        Red = 33,
        Green = 11,
        Blue = 222,
    }

    #[repr(i8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, crate::Enumerable)]
    enum Signed {
        Neg = -1,
        Zero = 0,
        Pos = 1,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, crate::Enumerable)]
    enum Zst {
        Only,
    }

    // --- bytes_eq / bytes_lt (via Color) ---

    #[test]
    fn bytes_eq_same_variant() {
        assert!(unsafe { bytes_eq(&Color::Red, &Color::Red) });
        assert!(unsafe { bytes_eq(&Color::Green, &Color::Green) });
    }

    #[test]
    fn bytes_eq_different_variant() {
        assert!(!unsafe { bytes_eq(&Color::Red, &Color::Green) });
    }

    #[test]
    fn bytes_lt_ordering() {
        // Green(11) < Red(33) < Blue(222)
        assert!(unsafe { bytes_lt(&Color::Green, &Color::Red) });
        assert!(unsafe { bytes_lt(&Color::Red, &Color::Blue) });
        assert!(!unsafe { bytes_lt(&Color::Red, &Color::Green) });
        assert!(!unsafe { bytes_lt(&Color::Red, &Color::Red) });
    }

    #[test]
    fn bytes_lt_unsigned_bit_pattern_order() {
        // Neg(-1) has bit pattern 0xFF, which is greater than Zero(0) and Pos(1)
        // under unsigned ordering, even though -1 < 0 numerically.
        assert!(unsafe { bytes_lt(&Signed::Zero, &Signed::Neg) });
        assert!(unsafe { bytes_lt(&Signed::Pos, &Signed::Neg) });
        assert!(!unsafe { bytes_lt(&Signed::Neg, &Signed::Zero) });
    }

    #[test]
    fn bytes_eq_lt_zero_sized() {
        assert!(unsafe { bytes_eq(&Zst::Only, &Zst::Only) });
        assert!(!unsafe { bytes_lt(&Zst::Only, &Zst::Only) });
    }

    // --- sort_variants ---

    #[test]
    fn sort_variants_already_sorted() {
        let arr = [Color::Green, Color::Red, Color::Blue];
        let sorted = unsafe { sort_variants(arr) };
        assert_eq!(sorted, [Color::Green, Color::Red, Color::Blue]);
    }

    #[test]
    fn sort_variants_reverse_order() {
        let arr = [Color::Blue, Color::Red, Color::Green];
        let sorted = unsafe { sort_variants(arr) };
        assert_eq!(sorted, [Color::Green, Color::Red, Color::Blue]);
    }

    #[test]
    fn sort_variants_single_element() {
        let arr = [Color::Red];
        let sorted = unsafe { sort_variants(arr) };
        assert_eq!(sorted, [Color::Red]);
    }

    #[test]
    fn sort_variants_empty() {
        let arr: [Color; 0] = [];
        let sorted = unsafe { sort_variants(arr) };
        assert_eq!(sorted, []);
    }

    // --- is_sorted ---

    #[cfg(debug_assertions)]
    #[test]
    fn is_sorted_sorted_slice() {
        let arr = [Color::Green, Color::Red, Color::Blue];
        assert!(is_sorted(&arr));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn is_sorted_unsorted_slice() {
        let arr = [Color::Red, Color::Green, Color::Blue];
        assert!(!is_sorted(&arr));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn is_sorted_single_element() {
        let arr = [Color::Red];
        assert!(is_sorted(&arr));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn is_sorted_empty() {
        let arr: [Color; 0] = [];
        assert!(is_sorted(&arr));
    }

    // --- variant_index_of ---

    #[test]
    fn variant_index_of_finds_each() {
        let sorted = [Color::Green, Color::Red, Color::Blue];
        assert_eq!(unsafe { variant_index_of(&Color::Green, &sorted) }, 0);
        assert_eq!(unsafe { variant_index_of(&Color::Red, &sorted) }, 1);
        assert_eq!(unsafe { variant_index_of(&Color::Blue, &sorted) }, 2);
    }

    // --- binary_search_index ---

    #[test]
    fn binary_search_index_finds_each() {
        // VARIANTS sorted by discriminant: Green(11), Red(33), Blue(222)
        assert_eq!(binary_search_index(&Color::Green), 0);
        assert_eq!(binary_search_index(&Color::Red), 1);
        assert_eq!(binary_search_index(&Color::Blue), 2);
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
    fn try_collect_array_drops_on_panic() {
        use std::panic::{AssertUnwindSafe, catch_unwind};
        use std::sync::atomic::{AtomicUsize, Ordering};

        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct Droppable;
        impl Drop for Droppable {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }

        DROP_COUNT.store(0, Ordering::SeqCst);

        let result = catch_unwind(AssertUnwindSafe(|| {
            let _: Result<[Droppable; 5], ()> = try_collect_array(|i| {
                if i == 3 {
                    panic!("boom");
                }
                Ok(Droppable)
            });
        }));

        assert!(result.is_err());
        // Elements 0, 1, 2 were initialized then must be dropped by the guard's Drop impl.
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
