use crate::{EnumTable, Enumable};

impl<K: Enumable, V, const N: usize> EnumTable<K, V, N> {
    /// Returns an iterator over references to the keys in the table.
    pub fn keys(&self) -> core::slice::Iter<'_, K> {
        K::VARIANTS.iter()
    }

    /// Returns an iterator over references to the values in the table.
    pub fn values(&self) -> core::slice::Iter<'_, V> {
        self.table.iter()
    }

    /// Returns an iterator over mutable references to the values in the table.
    pub fn values_mut(&mut self) -> core::slice::IterMut<'_, V> {
        self.table.iter_mut()
    }

    /// Returns an iterator over references to the key-value pairs in the table.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.into_iter()
    }

    /// Returns an iterator over mutable references to the key-value pairs in the table.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.into_iter()
    }
}

impl<K: Enumable, V, const N: usize> IntoIterator for EnumTable<K, V, N> {
    type Item = (K, V);
    type IntoIter = core::iter::Map<
        core::iter::Enumerate<core::array::IntoIter<V, N>>,
        fn((usize, V)) -> (K, V),
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.table
            .into_iter()
            .enumerate()
            .map(|(i, v)| (K::VARIANTS[i], v))
    }
}

impl<'a, K: Enumable, V, const N: usize> IntoIterator for &'a EnumTable<K, V, N> {
    type Item = (&'a K, &'a V);
    type IntoIter = core::iter::Map<
        core::iter::Enumerate<core::slice::Iter<'a, V>>,
        fn((usize, &'a V)) -> (&'a K, &'a V),
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.table
            .iter()
            .enumerate()
            .map(|(i, v)| (&K::VARIANTS[i], v))
    }
}

impl<'a, K: Enumable, V, const N: usize> IntoIterator for &'a mut EnumTable<K, V, N> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = core::iter::Map<
        core::iter::Enumerate<core::slice::IterMut<'a, V>>,
        fn((usize, &'a mut V)) -> (&'a K, &'a mut V),
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.table
            .iter_mut()
            .enumerate()
            .map(|(i, v)| (&K::VARIANTS[i], v))
    }
}

#[cfg(test)]
mod tests {
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
