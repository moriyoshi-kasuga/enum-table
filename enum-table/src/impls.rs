use std::ops::{Index, IndexMut};

use crate::{EnumTable, Enumable};

impl<K: Enumable, V: Default, const N: usize> Default for EnumTable<K, V, N> {
    fn default() -> Self {
        EnumTable::new_with_fn(|_| Default::default())
    }
}

impl<K: Enumable, V, const N: usize> Index<K> for EnumTable<K, V, N> {
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        self.get(&index)
    }
}

impl<K: Enumable, V, const N: usize> IndexMut<K> for EnumTable<K, V, N> {
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        self.get_mut(&index)
    }
}
