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

#[cfg(test)]
mod tests {
    use std::hash::{Hash, Hasher};

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Color {
        Red,
        Green,
        Blue,
    }

    impl Enumable for Color {
        const VARIANTS: &'static [Self] = &[Color::Red, Color::Green, Color::Blue];
    }

    const TABLES: EnumTable<Color, &'static str, { Color::COUNT }> =
        crate::et!(Color, &'static str, |color| match color {
            Color::Red => "Red",
            Color::Green => "Green",
            Color::Blue => "Blue",
        });

    const ANOTHER_TABLES: EnumTable<Color, &'static str, { Color::COUNT }> =
        crate::et!(Color, &'static str, |color| match color {
            Color::Red => "Red",
            Color::Green => "Green",
            Color::Blue => "Blue",
        });

    #[test]
    fn debug_impl() {
        assert_eq!(
            format!("{:?}", TABLES),
            r#"{Red: "Red", Green: "Green", Blue: "Blue"}"#
        );
    }

    #[test]
    fn clone_impl() {
        let cloned = TABLES.clone();
        assert_eq!(cloned, TABLES);
    }

    #[test]
    fn eq_impl() {
        assert!(TABLES == ANOTHER_TABLES);
        assert!(TABLES != EnumTable::new_with_fn(|_| "Unknown"));
    }

    #[test]
    fn hash_impl() {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        TABLES.hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher2 = std::collections::hash_map::DefaultHasher::new();
        ANOTHER_TABLES.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn default_impl() {
        let default_table: EnumTable<Color, &'static str, { Color::COUNT }> = EnumTable::default();
        assert_eq!(default_table.get(&Color::Red), &"");
        assert_eq!(default_table.get(&Color::Green), &"");
        assert_eq!(default_table.get(&Color::Blue), &"");
    }

    #[test]
    fn index_impl() {
        assert_eq!(TABLES[Color::Red], "Red");
        assert_eq!(TABLES[Color::Green], "Green");
        assert_eq!(TABLES[Color::Blue], "Blue");

        let mut mutable_table = TABLES.clone();
        mutable_table[Color::Red] = "Changed Red";
        assert_eq!(mutable_table[Color::Red], "Changed Red");
    }
}
