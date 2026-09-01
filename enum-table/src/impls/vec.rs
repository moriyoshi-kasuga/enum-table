extern crate alloc;

use crate::{EnumTable, Enumerable};
use alloc::vec::Vec;

impl<K: Enumerable, V, const N: usize> EnumTable<K, V, N> {
    /// Consumes the table, returning a `Vec` of key-value pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumerable};
    ///
    /// #[derive(Enumerable, Debug, PartialEq, Copy, Clone)]
    /// enum Color {
    ///     Red,
    ///     Green,
    ///     Blue,
    /// }
    ///
    /// let table = EnumTable::<Color, &str, { Color::COUNT }>::new_with_fn(|color| match color {
    ///     Color::Red => "red",
    ///     Color::Green => "green",
    ///     Color::Blue => "blue",
    /// });
    ///
    /// let vec = table.into_vec();
    /// assert!(vec.contains(&(Color::Red, "red")));
    /// assert!(vec.contains(&(Color::Green, "green")));
    /// assert!(vec.contains(&(Color::Blue, "blue")));
    /// assert_eq!(vec.len(), 3);
    /// ```
    pub fn into_vec(self) -> Vec<(K, V)> {
        self.into_iter().collect()
    }

    /// Creates an `EnumTable` from `vec`, or returns `None` if it doesn't contain exactly
    /// one entry for each variant of `K`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumerable};
    ///
    /// #[derive(Enumerable, Debug, PartialEq, Copy, Clone)]
    /// enum Color {
    ///     Red,
    ///     Green,
    ///     Blue,
    /// }
    ///
    /// let vec = vec![
    ///     (Color::Red, "red"),
    ///     (Color::Green, "green"),
    ///     (Color::Blue, "blue"),
    /// ];
    ///
    /// let table = EnumTable::<Color, &str, { Color::COUNT }>::try_from_vec(vec).unwrap();
    /// assert_eq!(table.get(&Color::Red), &"red");
    /// assert_eq!(table.get(&Color::Green), &"green");
    /// assert_eq!(table.get(&Color::Blue), &"blue");
    /// ```
    pub fn try_from_vec(vec: Vec<(K, V)>) -> Option<Self> {
        if vec.len() != N {
            return None;
        }

        let mut slots: [Option<V>; N] = core::array::from_fn(|_| None);

        for (key, value) in vec {
            slots[key.variant_index()] = Some(value);
        }

        let table = crate::intrinsics::try_collect_array(|i| slots[i].take().ok_or(())).ok()?;

        Some(Self::new(table))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Enumerable)]
    enum Color {
        Red = 33,
        Green = 11,
        Blue = 222,
    }

    const TABLES: EnumTable<Color, &'static str, { Color::COUNT }> =
        crate::et!(Color, &'static str, |color| match color {
            Color::Red => "Red",
            Color::Green => "Green",
            Color::Blue => "Blue",
        });

    #[test]
    fn into_vec() {
        let table = TABLES;
        let vec = table.into_vec();

        assert_eq!(vec.len(), 3);
        assert!(vec.contains(&(Color::Red, "Red")));
        assert!(vec.contains(&(Color::Green, "Green")));
        assert!(vec.contains(&(Color::Blue, "Blue")));
    }

    #[test]
    fn try_from_vec() {
        let vec = vec![
            (Color::Red, "Red"),
            (Color::Green, "Green"),
            (Color::Blue, "Blue"),
        ];

        let table = EnumTable::<Color, &str, { Color::COUNT }>::try_from_vec(vec).unwrap();
        assert_eq!(table.get(&Color::Red), &"Red");
        assert_eq!(table.get(&Color::Green), &"Green");
        assert_eq!(table.get(&Color::Blue), &"Blue");
    }

    #[test]
    fn try_from_vec_invalid_size() {
        let vec = vec![
            (Color::Red, "Red"),
            (Color::Green, "Green"),
            // Missing Blue
        ];

        let result = EnumTable::<Color, &str, { Color::COUNT }>::try_from_vec(vec);
        assert_eq!(result, None);
    }

    #[test]
    fn try_from_vec_missing_variant() {
        let vec = vec![
            (Color::Red, "Red"),
            (Color::Green, "Green"),
            (Color::Red, "Duplicate Red"), // Duplicate instead of Blue
        ];

        let result = EnumTable::<Color, &str, { Color::COUNT }>::try_from_vec(vec);
        assert_eq!(result, None);
    }

    #[test]
    fn conversion_roundtrip() {
        let original = TABLES;
        let vec = original.into_vec();
        let reconstructed = EnumTable::<Color, &str, { Color::COUNT }>::try_from_vec(vec).unwrap();

        assert_eq!(reconstructed.get(&Color::Red), &"Red");
        assert_eq!(reconstructed.get(&Color::Green), &"Green");
        assert_eq!(reconstructed.get(&Color::Blue), &"Blue");
    }
}
