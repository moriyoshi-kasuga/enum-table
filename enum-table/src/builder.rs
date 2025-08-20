use core::mem::MaybeUninit;

use crate::{EnumTable, Enumable, intrinsics::to_usize};

/// Error type indicating that the builder is not filled completely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotFilled {
    pub expected: usize,
    pub current: usize,
}

impl core::fmt::Display for NotFilled {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Builder not filled: expected {} elements, got {}",
            self.expected, self.current
        )
    }
}

impl core::error::Error for NotFilled {}

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
    table: MaybeUninit<[(usize, V); N]>,
    filled: [bool; N],
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
            filled: [false; N],
            _phantom: core::marker::PhantomData,
        }
    }

    /// Pushes a new element into the builder, ensuring that the variant is not already filled.
    ///
    /// # Arguments
    ///
    /// * `variant`: A reference to an enumeration variant.
    /// * `value`: The value to associate with the variant.
    ///
    /// # Returns
    ///
    /// * `Some(V)` if the variant was already filled, containing the previous value.
    /// * `None` if the variant was not filled before.
    pub fn insert(&mut self, variant: &K, value: V) -> Option<V> {
        let idx = to_usize(variant);
        let pointer = unsafe { self.table.as_mut_ptr().cast::<(usize, V)>().add(idx) };

        if self.filled[idx] {
            let element = unsafe { pointer.replace((idx, value)) };
            return Some(element.1);
        }

        unsafe {
            pointer.write((idx, value));
        }

        self.idx += 1;
        self.filled[idx] = true;
        None
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

    /// Builds the table from the pushed elements, ensuring all variants are filled.
    ///
    /// # Returns
    ///
    /// * `Ok([(usize, V); N])` if all variants have been filled.
    /// * `Err(NotFilled)` if not all variants have been filled.
    pub const fn build(self) -> Result<[(usize, V); N], NotFilled> {
        if self.idx != N {
            return Err(NotFilled {
                expected: N,
                current: self.idx,
            });
        }

        // SAFETY: The table is filled as verified by the check above.
        let table = unsafe { self.table.assume_init() };
        Ok(table)
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
    /// An array of tuples where each tuple contains a discriminant of an enumeration
    /// variant and its associated value.
    pub const unsafe fn build_unchecked(self) -> [(usize, V); N] {
        // We can't use debug_assert_eq! in const context, so use a simpler assertion
        // debug_assert_eq!(self.idx, N, "EnumTableBuilder: not enough elements");

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

        // SAFETY: Caller guarantees that the table is filled.
        let table = unsafe { self.table.assume_init() };

        debug_assert!(
            is_sorted(&table),
            "EnumTableBuilder: elements are not sorted by discriminant. Ensure that the elements are pushed in the correct order."
        );

        table
    }

    /// Builds the `EnumTable` from the pushed elements, ensuring all variants are filled.
    ///
    /// # Returns
    ///
    /// * `Ok(EnumTable)` if all variants have been filled.
    /// * `Err(NotFilled)` if not all variants have been filled.
    pub fn build_to(self) -> Result<EnumTable<K, V, N>, NotFilled> {
        Ok(EnumTable::new(self.build()?))
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

    #[test]
    fn safe_builder() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Enumable)]
        enum Test {
            A,
            B,
            C,
        }

        let mut builder = EnumTableBuilder::<Test, &'static str, { Test::COUNT }>::new();

        // Test safe push method
        assert!(builder.insert(&Test::B, "B").is_none());
        assert!(builder.insert(&Test::C, "C").is_none());
        assert!(builder.insert(&Test::A, "A").is_none());

        // Test that the builder is filled correctly
        assert_eq!(builder.len(), 3);
        assert_eq!(builder.insert(&Test::A, "New A"), Some("A"));

        let table = builder.build_to().unwrap();
        assert_eq!(table.get(&Test::A), &"New A");
        assert_eq!(table.get(&Test::B), &"B");
        assert_eq!(table.get(&Test::C), &"C");
    }

    #[test]
    fn builder_not_filled_error() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Enumable)]
        enum Test {
            A,
            B,
            C,
        }

        let mut builder = EnumTableBuilder::<Test, &'static str, { Test::COUNT }>::new();
        assert!(builder.insert(&Test::A, "A").is_none());

        // Only filled 1 out of 3
        assert_eq!(
            builder.build_to(),
            Err(NotFilled {
                expected: 3,
                current: 1
            })
        );
    }
}
