use core::mem::MaybeUninit;

use crate::{EnumTable, Enumable};

/// A builder for creating an `EnumTable` with a specified number of elements.
///
/// `EnumTableBuilder` allows for the incremental construction of an `EnumTable`
/// by pushing elements one by one and then building the final table.
///
/// # Note
/// The builder is expected to be filled completely before building the table.
/// If the builder is not filled completely, the `build` and `build_to` method will panic.
/// For a clearer and more concise approach, consider using the [`crate::et`] macro.
///
/// # Example
/// ```rust
/// use enum_table::{EnumTable, Enumable, builder::EnumTableBuilder,};
///
/// #[derive(Debug, Copy, Clone, Enumable)]
/// enum Test {
///     A,
///     B,
///     C,
/// }
///
/// const TABLE: EnumTable<Test, &'static str, { Test::COUNT }> = {
///    let mut builder = EnumTableBuilder::<Test, &'static str, { Test::COUNT }>::new();
///    unsafe {
///        builder.push_unchecked(&Test::A, "A");
///        builder.push_unchecked(&Test::B, "B");
///        builder.push_unchecked(&Test::C, "C");
///        builder.build_to_unchecked()
///    }
/// };
///
/// // Access values associated with enum variants
/// assert_eq!(TABLE.get(&Test::A), &"A");
/// assert_eq!(TABLE.get(&Test::B), &"B");
/// assert_eq!(TABLE.get(&Test::C), &"C");
/// ```
pub struct EnumTableBuilder<K: Enumable, V, const N: usize> {
    idx: usize,
    table: MaybeUninit<[V; N]>,
    #[cfg(debug_assertions)]
    keys: MaybeUninit<[K; N]>,
}

impl<K: Enumable, V, const N: usize> EnumTableBuilder<K, V, N> {
    /// Creates a new `EnumTableBuilder` with an uninitialized table.
    ///
    /// # Returns
    ///
    /// A new instance of `EnumTableBuilder`.
    pub const fn new() -> Self {
        Self {
            idx: 0,
            table: MaybeUninit::uninit(),
            #[cfg(debug_assertions)]
            keys: MaybeUninit::uninit(),
        }
    }

    /// Pushes a new element into the builder without safety checks.
    ///
    /// # Safety
    ///
    /// * The caller must ensure that elements are pushed in the correct order
    ///   (sorted by discriminant).
    /// * The caller must ensure that no variant is pushed more than once.
    /// * The caller must ensure that the builder doesn't exceed capacity N.
    ///
    /// # Arguments
    ///
    /// * `variant` - A reference to an enumeration variant.
    /// * `value` - The value to associate with the variant.
    pub const unsafe fn push_unchecked(&mut self, variant: &K, value: V) {
        debug_assert!(self.idx < N, "EnumTableBuilder: too many elements pushed");

        #[cfg(debug_assertions)]
        unsafe {
            self.keys
                .as_mut_ptr()
                .cast::<K>()
                .add(self.idx)
                .write(*variant);
        }

        unsafe {
            self.table
                .as_mut_ptr()
                .cast::<V>()
                .add(self.idx)
                .write(value);
        }

        self.idx += 1;
    }

    /// Builds the table from the pushed elements without checking if all variants are filled.
    ///
    /// # Safety
    ///
    /// The caller must ensure that all N variants have been pushed to the builder.
    /// If this is not the case, the resulting table will contain uninitialized memory.
    ///
    /// # Returns
    ///
    /// An array of tuples where each tuple contains an enumeration
    /// variant and its associated value.
    pub const unsafe fn build_unchecked(self) -> [V; N] {
        #[cfg(debug_assertions)]
        assert!(
            self.idx == N,
            "EnumTableBuilder: not all elements have been pushed"
        );

        // SAFETY: Caller guarantees that the table is filled.
        let table = unsafe { self.table.assume_init() };

        #[cfg(debug_assertions)]
        {
            const fn is_sorted<const N: usize, K>(arr: &[K; N]) -> bool {
                let mut i = 0;
                while i < N - 1 {
                    if !crate::intrinsics::const_enum_lt(&arr[i], &arr[i + 1]) {
                        return false;
                    }
                    i += 1;
                }
                true
            }

            let keys = unsafe { self.keys.assume_init() };
            assert!(
                is_sorted(&keys),
                "EnumTableBuilder: elements are not sorted by discriminant. Ensure that the elements are pushed in the correct order."
            );
        }

        table
    }

    /// Builds the `EnumTable` from the pushed elements without checking if all variants are filled.
    ///
    /// # Safety
    ///
    /// The caller must ensure that all N variants have been pushed to the builder.
    ///
    /// # Returns
    ///
    /// An `EnumTable` containing the elements pushed into the builder.
    pub const unsafe fn build_to_unchecked(self) -> EnumTable<K, V, N> {
        EnumTable::new(unsafe { self.build_unchecked() })
    }

    /// Returns the number of elements pushed into the builder.
    pub const fn len(&self) -> usize {
        self.idx
    }

    /// Returns the capacity of the builder.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Returns `true` if the builder has no elements pushed yet.
    ///
    /// # Returns
    ///
    /// `true` if no elements have been pushed, `false` otherwise.
    pub const fn is_empty(&self) -> bool {
        self.idx == 0
    }
}

impl<K: Enumable, V, const N: usize> Default for EnumTableBuilder<K, V, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Enumable)]
        enum Test {
            A,
            B,
            C,
        }

        const TABLE: EnumTable<Test, &'static str, { Test::COUNT }> = {
            let mut builder = EnumTableBuilder::<Test, &'static str, { Test::COUNT }>::new();

            let mut i = 0;
            while i < builder.capacity() {
                let t = &Test::VARIANTS[i];
                unsafe {
                    builder.push_unchecked(
                        t,
                        match t {
                            Test::A => "A",
                            Test::B => "B",
                            Test::C => "C",
                        },
                    );
                }
                i += 1;
            }

            unsafe { builder.build_to_unchecked() }
        };

        assert_eq!(TABLE.get(&Test::A), &"A");
        assert_eq!(TABLE.get(&Test::B), &"B");
        assert_eq!(TABLE.get(&Test::C), &"C");
    }
}
