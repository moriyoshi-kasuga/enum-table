use std::{collections::HashMap, hash::Hash};

use crate::{EnumTable, Enumerable};

impl<K: Enumerable + Eq + Hash, V, const N: usize> EnumTable<K, V, N> {
    /// Consumes the table, returning a `HashMap` with the same key-value pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumerable};
    /// use std::collections::HashMap;
    ///
    /// #[derive(Enumerable, Debug, PartialEq, Eq, Hash, Copy, Clone)]
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

    /// Creates an `EnumTable` from `map`, or returns `None` if it doesn't contain exactly
    /// one entry for each variant of `K`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumerable};
    /// use std::collections::HashMap;
    ///
    /// #[derive(Enumerable, Debug, PartialEq, Eq, Hash, Copy, Clone)]
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
    /// # use enum_table::{EnumTable, Enumerable};
    /// # use std::collections::HashMap;
    /// #
    /// # #[derive(Enumerable, Debug, PartialEq, Eq, Hash, Copy, Clone)]
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
            crate::intrinsics::try_collect_array(|i| map.remove(&K::VARIANTS[i]).ok_or(())).ok()?;
        Some(EnumTable::new(table))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Enumerable)]
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
}
