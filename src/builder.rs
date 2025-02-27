use std::mem::{Discriminant, MaybeUninit};

use crate::{EnumTable, Enumable};

/// A builder for creating an `EnumTable` with a specified number of elements.
///
/// `EnumTableBuilder` allows for the incremental construction of an `EnumTable`
/// by pushing elements one by one and then building the final table.
pub struct EnumTableBuilder<K: Enumable, V, const N: usize> {
    idx: usize,
    table: [MaybeUninit<(Discriminant<K>, V)>; N],
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
            table: [const { MaybeUninit::uninit() }; N],
        }
    }

    /// Pushes a new element into the builder.
    ///
    /// # Arguments
    ///
    /// * `variant` - A reference to an enumeration variant.
    /// * `value` - The value to associate with the variant.
    pub const fn push(&mut self, variant: &K, value: V) {
        self.table[self.idx] = MaybeUninit::new((core::mem::discriminant(variant), value));
        self.idx += 1;
    }

    /// Builds the table from the pushed elements.
    ///
    /// # Returns
    ///
    /// An array of tuples where each tuple contains a discriminant of an enumeration
    /// variant and its associated value.
    pub const fn build(self) -> [(Discriminant<K>, V); N] {
        if self.idx != N {
            panic!("EnumTableBuilder: not enough elements");
        }
        unsafe { core::mem::transmute_copy(&self.table) }
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

    /// Returns `false` as the builder is expected to be filled completely.
    ///
    /// # Returns
    ///
    /// Always returns `false`.
    pub const fn is_empty(&self) -> bool {
        false
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
        enum Test {
            A,
            B,
            C,
        }

        impl Enumable for Test {
            const VARIANTS: &'static [Self] = &[Test::A, Test::B, Test::C];
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
