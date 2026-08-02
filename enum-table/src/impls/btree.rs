extern crate alloc;

use alloc::collections::BTreeMap;

use crate::{EnumTable, Enumerable};

impl<K: Enumerable + Ord, V, const N: usize> EnumTable<K, V, N> {
    /// Consumes the table, returning a `BTreeMap` with the same key-value pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumerable};
    /// use std::collections::BTreeMap;
    ///
    /// #[derive(Enumerable, Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone)]
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

    /// Creates an `EnumTable` from `map`, or returns `None` if it doesn't contain exactly
    /// one entry for each variant of `K`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumerable};
    /// use std::collections::BTreeMap;
    ///
    /// #[derive(Enumerable, Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone)]
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
    /// # use enum_table::{EnumTable, Enumerable};
    /// # use std::collections::BTreeMap;
    /// #
    /// # #[derive(Enumerable, Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone)]
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
            crate::intrinsics::try_collect_array(|i| map.remove(&K::VARIANTS[i]).ok_or(())).ok()?;
        Some(EnumTable::new(table))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Enumerable, Ord, PartialOrd)]
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
