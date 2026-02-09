use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

use crate::{EnumTable, Enumable};

impl<K: Enumable + Eq + Hash, V, const N: usize> EnumTable<K, V, N> {
    /// Converts the `EnumTable` into a `HashMap`.
    ///
    /// This method consumes the `EnumTable` and creates a new `HashMap` with the same
    /// key-value pairs. Each enum variant is mapped to its associated value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumable};
    /// use std::collections::HashMap;
    ///
    /// #[derive(Enumable, Debug, PartialEq, Eq, Hash, Copy, Clone)]
    /// enum Status {
    ///     Active,
    ///     Inactive,
    ///     Pending,
    /// }
    ///
    /// let table = EnumTable::<Status, &str, { Status::COUNT }>::new_with_fn(|status| match status {
    ///     Status::Active => "running",
    ///     Status::Inactive => "stopped",
    ///     Status::Pending => "waiting",
    /// });
    ///
    /// let hash_map = table.into_hash_map();
    /// assert_eq!(hash_map.get(&Status::Active), Some(&"running"));
    /// assert_eq!(hash_map.get(&Status::Inactive), Some(&"stopped"));
    /// assert_eq!(hash_map.get(&Status::Pending), Some(&"waiting"));
    /// assert_eq!(hash_map.len(), 3);
    /// ```
    pub fn into_hash_map(self) -> HashMap<K, V> {
        self.into_iter().collect()
    }

    /// Creates an `EnumTable` from a `HashMap`.
    ///
    /// Returns `None` if the `HashMap` doesn't contain exactly one entry for each enum variant.
    /// The HashMap must have exactly `N` entries where `N` is the number of enum variants.
    ///
    /// # Arguments
    ///
    /// * `map` - A `HashMap` containing key-value pairs for each enum variant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumable};
    /// use std::collections::HashMap;
    ///
    /// #[derive(Enumable, Debug, PartialEq, Eq, Hash, Copy, Clone)]
    /// enum Priority {
    ///     Low,
    ///     Medium,
    ///     High,
    /// }
    ///
    /// let mut hash_map = HashMap::new();
    /// hash_map.insert(Priority::Low, 1);
    /// hash_map.insert(Priority::Medium, 5);
    /// hash_map.insert(Priority::High, 10);
    ///
    /// let table = EnumTable::<Priority, i32, { Priority::COUNT }>::try_from_hash_map(hash_map)
    ///     .expect("HashMap should contain all variants");
    ///
    /// assert_eq!(table.get(&Priority::Low), &1);
    /// assert_eq!(table.get(&Priority::Medium), &5);
    /// assert_eq!(table.get(&Priority::High), &10);
    /// ```
    ///
    /// ```rust
    /// # use enum_table::{EnumTable, Enumable};
    /// # use std::collections::HashMap;
    /// #
    /// # #[derive(Enumable, Debug, PartialEq, Eq, Hash, Copy, Clone)]
    /// # enum Priority {
    /// #     Low,
    /// #     Medium,
    /// #     High,
    /// # }
    /// // Example with missing variant
    /// let mut incomplete_map = HashMap::new();
    /// incomplete_map.insert(Priority::Low, 1);
    /// incomplete_map.insert(Priority::Medium, 5);
    /// // Missing Priority::High
    ///
    /// let result = EnumTable::<Priority, i32, { Priority::COUNT }>::try_from_hash_map(incomplete_map);
    /// assert!(result.is_none());
    /// ```
    pub fn try_from_hash_map(mut map: HashMap<K, V>) -> Option<EnumTable<K, V, N>> {
        if map.len() != N {
            return None;
        }

        let table =
            crate::intrinsics::try_collect_array(|i| map.remove(&K::VARIANTS[i]).ok_or(()))
                .ok()?;
        Some(EnumTable::new(table))
    }
}

impl<K: Enumable + Ord, V, const N: usize> EnumTable<K, V, N> {
    /// Converts the `EnumTable` into a `BTreeMap`.
    ///
    /// This method consumes the `EnumTable` and creates a new `BTreeMap` with the same
    /// key-value pairs. The resulting map will have keys sorted according to their `Ord` implementation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumable};
    /// use std::collections::BTreeMap;
    ///
    /// #[derive(Enumable, Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone)]
    /// enum Level {
    ///     Beginner,
    ///     Intermediate,
    ///     Advanced,
    /// }
    ///
    /// let table = EnumTable::<Level, u32, { Level::COUNT }>::new_with_fn(|level| match level {
    ///     Level::Beginner => 100,
    ///     Level::Intermediate => 500,
    ///     Level::Advanced => 1000,
    /// });
    ///
    /// let btree_map = table.into_btree_map();
    /// assert_eq!(btree_map.get(&Level::Beginner), Some(&100));
    /// assert_eq!(btree_map.get(&Level::Intermediate), Some(&500));
    /// assert_eq!(btree_map.get(&Level::Advanced), Some(&1000));
    /// assert_eq!(btree_map.len(), 3);
    /// ```
    pub fn into_btree_map(self) -> BTreeMap<K, V> {
        self.into_iter().collect()
    }

    /// Creates an `EnumTable` from a `BTreeMap`.
    ///
    /// Returns `None` if the `BTreeMap` doesn't contain exactly one entry for each enum variant.
    /// The BTreeMap must have exactly `N` entries where `N` is the number of enum variants.
    ///
    /// # Arguments
    ///
    /// * `map` - A `BTreeMap` containing key-value pairs for each enum variant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumable};
    /// use std::collections::BTreeMap;
    ///
    /// #[derive(Enumable, Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone)]
    /// enum Grade {
    ///     A,
    ///     B,
    ///     C,
    /// }
    ///
    /// let mut btree_map = BTreeMap::new();
    /// btree_map.insert(Grade::A, 90.0);
    /// btree_map.insert(Grade::B, 80.0);
    /// btree_map.insert(Grade::C, 70.0);
    ///
    /// let table = EnumTable::<Grade, f64, { Grade::COUNT }>::try_from_btree_map(btree_map)
    ///     .expect("BTreeMap should contain all variants");
    ///
    /// assert_eq!(table.get(&Grade::A), &90.0);
    /// assert_eq!(table.get(&Grade::B), &80.0);
    /// assert_eq!(table.get(&Grade::C), &70.0);
    /// ```
    ///
    /// ```rust
    /// # use enum_table::{EnumTable, Enumable};
    /// # use std::collections::BTreeMap;
    /// #
    /// # #[derive(Enumable, Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone)]
    /// # enum Grade {
    /// #     A,
    /// #     B,
    /// #     C,
    /// # }
    /// // Example with missing variant
    /// let mut incomplete_map = BTreeMap::new();
    /// incomplete_map.insert(Grade::A, 90.0);
    /// incomplete_map.insert(Grade::B, 80.0);
    /// // Missing Grade::C
    ///
    /// let result = EnumTable::<Grade, f64, { Grade::COUNT }>::try_from_btree_map(incomplete_map);
    /// assert!(result.is_none());
    /// ```
    pub fn try_from_btree_map(mut map: BTreeMap<K, V>) -> Option<EnumTable<K, V, N>> {
        if map.len() != N {
            return None;
        }

        let table =
            crate::intrinsics::try_collect_array(|i| map.remove(&K::VARIANTS[i]).ok_or(()))
                .ok()?;
        Some(EnumTable::new(table))
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
