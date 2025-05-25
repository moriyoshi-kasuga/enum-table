use std::ops::{Index, IndexMut};

use crate::{EnumTable, Enumable};

impl<K: Enumable + core::fmt::Debug, V: core::fmt::Debug, const N: usize> core::fmt::Debug
    for EnumTable<K, V, N>
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K: Enumable, V: Clone, const N: usize> Clone for EnumTable<K, V, N> {
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<K: Enumable, V: PartialEq, const N: usize> PartialEq for EnumTable<K, V, N> {
    fn eq(&self, other: &Self) -> bool {
        self.table.eq(&other.table)
    }
}

impl<K: Enumable, V: Eq, const N: usize> Eq for EnumTable<K, V, N> {}

impl<K: Enumable, V: std::hash::Hash, const N: usize> std::hash::Hash for EnumTable<K, V, N> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.table.hash(state);
    }
}

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
