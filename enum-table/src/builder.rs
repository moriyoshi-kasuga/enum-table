use core::mem::MaybeUninit;

use crate::{EnumTable, Enumable, intrinsics::to_usize};

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
/// #[derive(Debug, Enumable)]
/// enum Test {
///     A,
///     B,
///     C,
/// }
///
/// const TABLE: EnumTable<Test, &'static str, { Test::COUNT }> = {
///    let mut builder = EnumTableBuilder::<Test, &'static str, { Test::COUNT }>::new();
///    builder.push(&Test::A, "A");
///    builder.push(&Test::B, "B");
///    builder.push(&Test::C, "C");
///    builder.build_to()
/// };
///
/// // Access values associated with enum variants
/// assert_eq!(TABLE.get(&Test::A), &"A");
/// assert_eq!(TABLE.get(&Test::B), &"B");
/// assert_eq!(TABLE.get(&Test::C), &"C");
/// ```
pub struct EnumTableBuilder<K: Enumable, V, const N: usize> {
    idx: usize,
    table: MaybeUninit<[(usize, V); N]>,
    _phantom: core::marker::PhantomData<K>,
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
            _phantom: core::marker::PhantomData,
        }
    }

    /// Pushes a new element into the builder.
    ///
    /// # Arguments
    ///
    /// * `variant` - A reference to an enumeration variant.
    /// * `value` - The value to associate with the variant.
    pub const fn push(&mut self, variant: &K, value: V) {
        if self.idx >= N {
            panic!("EnumTableBuilder: too many elements pushed");
        }
        let element = (to_usize(variant), value);

        unsafe {
            self.table
                .as_mut_ptr()
                .cast::<(usize, V)>()
                .add(self.idx)
                .write(element);
        }

        self.idx += 1;
    }

    /// Builds the table from the pushed elements.
    ///
    /// # Returns
    ///
    /// An array of tuples where each tuple contains a discriminant of an enumeration
    /// variant and its associated value.
    pub const fn build(self) -> [(usize, V); N] {
        if self.idx != N {
            panic!("EnumTableBuilder: not enough elements");
        }

        const fn is_sorted<const N: usize, V>(arr: &[(usize, V); N]) -> bool {
            let mut i = 0;
            while i < N - 1 {
                if arr[i].0 >= arr[i + 1].0 {
                    return false;
                }
                i += 1;
            }
            true
        }

        // SAFETY: The table is filled.
        let table = unsafe { self.table.assume_init() };

        debug_assert!(
            is_sorted(&table),
            "EnumTableBuilder: elements are not sorted by discriminant. Ensure that the elements are pushed in the correct order."
        );

        table
    }

    /// Builds the `EnumTable` from the pushed elements.
    ///
    /// # Returns
    ///
    /// An `EnumTable` containing the elements pushed into the builder.
    pub const fn build_to(self) -> EnumTable<K, V, N> {
        EnumTable::new(self.build())
    }

    /// Returns the number of elements the builder is expected to hold.
    ///
    /// # Returns
    ///
    /// The number of elements `N`.
    pub const fn len(&self) -> usize {
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
        #[derive(Enumable)]
        enum Test {
            A,
            B,
            C,
        }

        const TABLE: EnumTable<Test, &'static str, { Test::COUNT }> = {
            let mut builder = EnumTableBuilder::<Test, &'static str, { Test::COUNT }>::new();

            let mut i = 0;
            while i < builder.len() {
                let t = &Test::VARIANTS[i];
                builder.push(
                    t,
                    match t {
                        Test::A => "A",
                        Test::B => "B",
                        Test::C => "C",
                    },
                );
                i += 1;
            }

            builder.build_to()
        };

        assert_eq!(TABLE.get(&Test::A), &"A");
        assert_eq!(TABLE.get(&Test::B), &"B");
        assert_eq!(TABLE.get(&Test::C), &"C");
    }
}
