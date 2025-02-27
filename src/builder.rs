use std::mem::{Discriminant, MaybeUninit};

use crate::EnumTable;

pub struct EnumTableBuilder<T, V, const N: usize> {
    idx: usize,
    table: [MaybeUninit<(Discriminant<T>, V)>; N],
}

impl<T, V, const N: usize> EnumTableBuilder<T, V, N> {
    pub const fn new() -> Self {
        Self {
            idx: 0,
            table: [const { MaybeUninit::uninit() }; N],
        }
    }

    pub const fn to_cast(&self, i: usize) -> T {
        crate::to_cast(i)
    }

    pub const fn push(&mut self, variant: &T, value: V) {
        self.table[self.idx] = MaybeUninit::new((core::mem::discriminant(variant), value));
        self.idx += 1;
    }

    pub const fn build(self) -> [(Discriminant<T>, V); N] {
        if self.idx != N {
            panic!("EnumTableBuilder: not enough elements");
        }
        unsafe { MaybeUninit::array_assume_init(self.table) }
    }

    pub const fn build_to(self) -> EnumTable<T, V, N> {
        EnumTable::new(self.build())
    }
}

impl<T, V, const N: usize> Default for EnumTableBuilder<T, V, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder() {
        #[repr(u8)]
        enum Test {
            A,
            B,
            C,
        }

        impl Test {
            const COUNT: usize = unsafe { crate::variant_count::<Test>() };
        }

        const TABLE: EnumTable<Test, &'static str, { Test::COUNT }> = {
            let mut builder = EnumTableBuilder::<Test, &'static str, { Test::COUNT }>::new();

            let mut i = 0;
            while i < Test::COUNT {
                let t = builder.to_cast(i);
                builder.push(
                    &t,
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
