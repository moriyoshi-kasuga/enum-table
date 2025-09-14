use core::ops::{Index, IndexMut};

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
        }
    }
}

impl<K: Enumable, V: Copy, const N: usize> Copy for EnumTable<K, V, N> {}

impl<K: Enumable, V: PartialEq, const N: usize> PartialEq for EnumTable<K, V, N> {
    fn eq(&self, other: &Self) -> bool {
        // Manually implement `PartialEq` to avoid adding a `K: PartialEq` bound,
        // which would be a breaking change. We can compare the enum discriminants
        // directly using intrinsics.
        let mut i = 0;
        while i < N {
            if crate::intrinsics::const_enum_eq(&self.table[i].0, &other.table[i].0)
                && self.table[i].1 == other.table[i].1
            {
                i += 1;
            } else {
                return false;
            }
        }
        true
    }
}

impl<K: Enumable, V: Eq, const N: usize> Eq for EnumTable<K, V, N> {}

impl<K: Enumable, V: core::hash::Hash, const N: usize> core::hash::Hash for EnumTable<K, V, N> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        for (discriminant, value) in self.table.iter() {
            crate::intrinsics::hash(discriminant, state);
            value.hash(state);
        }
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

impl<K: Enumable, V, const N: usize> IntoIterator for EnumTable<K, V, N> {
    type Item = (K, V);
    type IntoIter = core::array::IntoIter<(K, V), N>;

    fn into_iter(self) -> Self::IntoIter {
        self.table.into_iter()
    }
}

impl<'a, K: Enumable, V, const N: usize> IntoIterator for &'a EnumTable<K, V, N> {
    type Item = (&'a K, &'a V);
    type IntoIter =
        core::iter::Map<core::slice::Iter<'a, (K, V)>, fn(&'a (K, V)) -> (&'a K, &'a V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.table
            .iter()
            .map(|(discriminant, value)| (discriminant, value))
    }
}

impl<'a, K: Enumable, V, const N: usize> IntoIterator for &'a mut EnumTable<K, V, N> {
    type Item = (&'a K, &'a mut V);
    type IntoIter =
        core::iter::Map<core::slice::IterMut<'a, (K, V)>, fn(&'a mut (K, V)) -> (&'a K, &'a mut V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.table
            .iter_mut()
            .map(|(discriminant, value)| (discriminant, value))
    }
}

#[cfg(test)]
mod tests {
    use core::hash::{Hash, Hasher};

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Enumable)]
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

    const ANOTHER_TABLES: EnumTable<Color, &'static str, { Color::COUNT }> =
        crate::et!(Color, &'static str, |color| match color {
            Color::Red => "Red",
            Color::Green => "Green",
            Color::Blue => "Blue",
        });

    #[test]
    fn debug_impl() {
        assert_eq!(
            format!("{TABLES:?}"),
            r#"{Red: "Red", Green: "Green", Blue: "Blue"}"#
        );
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

        let mut mutable_table = TABLES;
        mutable_table[Color::Red] = "Changed Red";
        assert_eq!(mutable_table[Color::Red], "Changed Red");
    }

    #[test]
    fn into_iter_impl() {
        let mut iter = TABLES.into_iter();
        assert_eq!(iter.next(), Some((Color::Red, "Red")));
        assert_eq!(iter.next(), Some((Color::Green, "Green")));
        assert_eq!(iter.next(), Some((Color::Blue, "Blue")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn into_iter_ref_impl() {
        let mut iter = (&TABLES).into_iter();
        assert_eq!(iter.next(), Some((&Color::Red, &"Red")));
        assert_eq!(iter.next(), Some((&Color::Green, &"Green")));
        assert_eq!(iter.next(), Some((&Color::Blue, &"Blue")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn into_iter_mut_impl() {
        let mut mutable_table = TABLES;
        let mut iter = (&mut mutable_table).into_iter();
        assert_eq!(iter.next(), Some((&Color::Red, &mut "Red")));
        assert_eq!(iter.next(), Some((&Color::Green, &mut "Green")));
        let blue = iter.next().unwrap();
        assert_eq!(blue, (&Color::Blue, &mut "Blue"));
        assert_eq!(iter.next(), None);

        // Modify the value through the mutable reference
        *blue.1 = "Modified Blue";
        assert_eq!(mutable_table[Color::Blue], "Modified Blue");
    }
}
