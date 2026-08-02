use core::marker::PhantomData;
use core::mem::MaybeUninit;

use crate::{EnumTable, Enumerable};

/// Incrementally builds an `EnumTable` by pushing one value per variant.
///
/// Prefer the [`crate::et`] macro over using this type directly.
///
/// Dropping a builder before it is fully pushed leaks its already-pushed
/// elements instead of running their destructors: `build_unchecked` and
/// `build_to_unchecked` are `const fn`, and a `const fn` cannot run a `Drop`
/// implementation, so `EnumTableBuilder` cannot implement `Drop` without
/// losing const-context support.
///
/// # Examples
/// ```rust
/// use enum_table::{EnumTable, Enumerable, builder::EnumTableBuilder,};
///
/// #[derive(Debug, Copy, Clone, Enumerable)]
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
/// assert_eq!(TABLE.get(&Test::A), &"A");
/// assert_eq!(TABLE.get(&Test::B), &"B");
/// assert_eq!(TABLE.get(&Test::C), &"C");
/// ```
pub struct EnumTableBuilder<K: Enumerable, V, const N: usize> {
    idx: usize,
    table: MaybeUninit<[V; N]>,
    #[cfg(debug_assertions)]
    keys: MaybeUninit<[K; N]>,
    _phantom: PhantomData<K>,
}

impl<K: Enumerable, V, const N: usize> EnumTableBuilder<K, V, N> {
    /// Creates a new, empty `EnumTableBuilder`.
    pub const fn new() -> Self {
        Self {
            idx: 0,
            table: MaybeUninit::uninit(),
            #[cfg(debug_assertions)]
            keys: MaybeUninit::uninit(),
            _phantom: PhantomData,
        }
    }

    /// Pushes `value` for `variant` without checking order, duplicates, or capacity.
    ///
    /// # Safety
    ///
    /// The caller must ensure that elements are pushed in ascending discriminant
    /// order, that no variant is pushed more than once, and that the builder
    /// does not exceed capacity `N`.
    pub const unsafe fn push_unchecked(&mut self, _variant: &K, value: V) {
        debug_assert!(self.idx < N, "EnumTableBuilder: too many elements pushed");

        #[cfg(debug_assertions)]
        unsafe {
            self.keys
                .as_mut_ptr()
                .cast::<K>()
                .add(self.idx)
                .write(*_variant);
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

    /// Builds the array of values from the pushed elements.
    ///
    /// # Safety
    ///
    /// The caller must ensure that all `N` variants have been pushed to the
    /// builder; otherwise the resulting array contains uninitialized memory.
    pub const unsafe fn build_unchecked(self) -> [V; N] {
        #[cfg(debug_assertions)]
        assert!(
            self.idx == N,
            "EnumTableBuilder: not all elements have been pushed"
        );

        // SAFETY: caller guarantees all N variants were pushed.
        let table = unsafe { self.table.assume_init() };

        #[cfg(debug_assertions)]
        {
            let keys = unsafe { self.keys.assume_init() };
            assert!(
                crate::intrinsics::is_sorted(&keys),
                "EnumTableBuilder: elements are not sorted by discriminant. Ensure that the elements are pushed in the correct order."
            );
        }

        table
    }

    /// Builds the `EnumTable` from the pushed elements.
    ///
    /// # Safety
    ///
    /// The caller must ensure that all `N` variants have been pushed to the builder.
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

    /// Returns `true` if no elements have been pushed yet.
    pub const fn is_empty(&self) -> bool {
        self.idx == 0
    }
}

impl<K: Enumerable, V, const N: usize> Default for EnumTableBuilder<K, V, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Enumerable)]
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
