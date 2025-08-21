use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

use crate::{EnumTable, Enumable, et, intrinsics::into_variant};

impl<K: Enumable + Eq + Hash + core::fmt::Debug, V, const N: usize> EnumTable<K, V, N> {
    pub fn into_hash_map(self) -> HashMap<K, V> {
        self.table
            .into_iter()
            .map(|(discriminant, value)| (into_variant(discriminant), value))
            .collect()
    }

    pub fn try_from_hash_map(mut map: HashMap<K, V>) -> Option<EnumTable<K, V, N>> {
        if map.len() != N {
            return None;
        }

        Some(et!(K, V, { N }, |key| {
            unsafe { map.remove(key).unwrap_unchecked() }
        }))
    }
}

impl<K: Enumable + Ord, V, const N: usize> EnumTable<K, V, N> {
    pub fn into_btree_map(self) -> BTreeMap<K, V> {
        self.table
            .into_iter()
            .map(|(discriminant, value)| (into_variant(discriminant), value))
            .collect()
    }

    pub fn try_from_btree_map(mut map: BTreeMap<K, V>) -> Option<EnumTable<K, V, N>> {
        if map.len() != N {
            return None;
        }

        Some(et!(K, V, { N }, |key| {
            unsafe { map.remove(key).unwrap_unchecked() }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Enumable, Ord, PartialOrd)]
    enum Color {
        Red,
        Green,
        Blue,
    }

    const TABLES: EnumTable<Color, &'static str, { Color::COUNT }> =
        crate::et!(Color, &'static str, |color| match color {
            Color::Red => "Red",
            Color::Green => "Green",
            Color::Blue => "Blue",
        });

    #[test]
    fn into_hash_map() {
        let table = TABLES;
        let hash_map = table.into_hash_map();

        assert_eq!(hash_map.len(), 3);
        assert_eq!(hash_map.get(&Color::Red), Some(&"Red"));
        assert_eq!(hash_map.get(&Color::Green), Some(&"Green"));
        assert_eq!(hash_map.get(&Color::Blue), Some(&"Blue"));
    }

    #[test]
    fn try_from_hash_map() {
        let hash_map: HashMap<_, _> = [
            (Color::Red, "Red"),
            (Color::Green, "Green"),
            (Color::Blue, "Blue"),
        ]
        .into_iter()
        .collect();

        let table =
            EnumTable::<Color, &str, { Color::COUNT }>::try_from_hash_map(hash_map).unwrap();
        assert_eq!(table.get(&Color::Red), &"Red");
        assert_eq!(table.get(&Color::Green), &"Green");
        assert_eq!(table.get(&Color::Blue), &"Blue");
    }

    #[test]
    fn try_from_hash_map_invalid_size() {
        let hash_map: HashMap<_, _> = [
            (Color::Red, "Red"),
            (Color::Green, "Green"), // Missing Blue
        ]
        .into_iter()
        .collect();

        let result = EnumTable::<Color, &str, { Color::COUNT }>::try_from_hash_map(hash_map);
        assert!(result.is_none());
    }

    #[test]
    fn into_btree_map() {
        let table = TABLES;
        let btree_map = table.into_btree_map();

        assert_eq!(btree_map.len(), 3);
        assert_eq!(btree_map.get(&Color::Red), Some(&"Red"));
        assert_eq!(btree_map.get(&Color::Green), Some(&"Green"));
        assert_eq!(btree_map.get(&Color::Blue), Some(&"Blue"));
    }

    #[test]
    fn try_from_btree_map() {
        let btree_map: BTreeMap<_, _> = [
            (Color::Red, "Red"),
            (Color::Green, "Green"),
            (Color::Blue, "Blue"),
        ]
        .into_iter()
        .collect();

        let table =
            EnumTable::<Color, &str, { Color::COUNT }>::try_from_btree_map(btree_map).unwrap();
        assert_eq!(table.get(&Color::Red), &"Red");
        assert_eq!(table.get(&Color::Green), &"Green");
        assert_eq!(table.get(&Color::Blue), &"Blue");
    }

    #[test]
    fn try_from_btree_map_invalid_size() {
        let btree_map: BTreeMap<_, _> = [
            (Color::Red, "Red"),
            (Color::Green, "Green"), // Missing Blue
        ]
        .into_iter()
        .collect();

        let result = EnumTable::<Color, &str, { Color::COUNT }>::try_from_btree_map(btree_map);
        assert!(result.is_none());
    }
}
