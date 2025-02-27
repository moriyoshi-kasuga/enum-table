use std::mem::{Discriminant, MaybeUninit};

use crate::{EnumTable, Enumable};

pub struct EnumTableBuilder<K: Enumable, V, const N: usize> {
    idx: usize,
    table: [MaybeUninit<(Discriminant<K>, V)>; N],
}

impl<K: Enumable, V, const N: usize> EnumTableBuilder<K, V, N> {
    pub const fn new() -> Self {
        Self {
            idx: 0,
            table: [const { MaybeUninit::uninit() }; N],
        }
    }

    pub const fn push(&mut self, variant: &K, value: V) {
        self.table[self.idx] = MaybeUninit::new((core::mem::discriminant(variant), value));
        self.idx += 1;
    }

    pub const fn build(self) -> [(Discriminant<K>, V); N] {
        if self.idx != N {
            panic!("EnumTableBuilder: not enough elements");
        }
        unsafe { core::mem::transmute_copy(&self.table) }
    }

    pub const fn build_to(self) -> EnumTable<K, V, N> {
        EnumTable::new(self.build())
    }

    pub const fn len(&self) -> usize {
        N
    }

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
