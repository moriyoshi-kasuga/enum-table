use std::hash::Hash;

use crate::{EnumTable, Enumable, et, intrinsics::into_variant};

impl<K: Enumable + Eq + Hash + core::fmt::Debug, V, const N: usize> EnumTable<K, V, N> {
    pub fn into_hash_map(self) -> std::collections::HashMap<K, V> {
        self.table
            .into_iter()
            .map(|(discriminant, value)| (into_variant(discriminant), value))
            .collect()
    }

    pub fn try_from_hash_map(
        mut map: std::collections::HashMap<K, V>,
    ) -> Option<EnumTable<K, V, N>> {
        if map.len() != N {
            return None;
        }

        Some(et!(K, V, { N }, |key| {
            unsafe { map.remove(key).unwrap_unchecked() }
        }))
    }
}

impl<K: Enumable + Ord, V, const N: usize> EnumTable<K, V, N> {
    pub fn into_btree_map(self) -> std::collections::BTreeMap<K, V> {
        self.table
            .into_iter()
            .map(|(discriminant, value)| (into_variant(discriminant), value))
            .collect()
    }

    pub fn try_from_btree_map(
        mut map: std::collections::BTreeMap<K, V>,
    ) -> Option<EnumTable<K, V, N>> {
        if map.len() != N {
            return None;
        }

        Some(et!(K, V, { N }, |key| {
            unsafe { map.remove(key).unwrap_unchecked() }
        }))
    }
}
