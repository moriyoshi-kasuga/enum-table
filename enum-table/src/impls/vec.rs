use crate::{EnumTable, Enumable, builder::EnumTableBuilder, into_variant, to_usize};

/// Error type for `EnumTable::try_from_vec`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumTableFromVecError<K> {
    /// The vector has an invalid size.
    InvalidSize { expected: usize, found: usize },
    /// A required enum variant is missing from the vector.
    /// This error happened meaning that the vector duplicated some variant
    MissingVariant(K),
}

impl<K: core::fmt::Debug> core::fmt::Display for EnumTableFromVecError<K> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EnumTableFromVecError::InvalidSize { expected, found } => {
                write!(f, "Invalid vector size: expected {expected}, found {found}")
            }
            EnumTableFromVecError::MissingVariant(variant) => {
                write!(f, "Missing enum variant: {variant:?}")
            }
        }
    }
}

impl<K: core::fmt::Debug> core::error::Error for EnumTableFromVecError<K> {}

impl<K: Enumable, V, const N: usize> EnumTable<K, V, N> {
    /// Converts the `EnumTable` into a `Vec` of key-value pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumable};
    ///
    /// #[derive(Enumable, Debug, PartialEq, Copy, Clone)]
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
        self.table
            .into_iter()
            .map(|(discriminant, value)| (into_variant(discriminant), value))
            .collect()
    }

    /// Creates an `EnumTable` from a `Vec` of key-value pairs.
    ///
    /// Returns an error if the vector doesn't contain exactly one entry for each enum variant.
    ///
    /// # Arguments
    ///
    /// * `vec` - A vector containing key-value pairs for each enum variant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumable};
    ///
    /// #[derive(Enumable, Debug, PartialEq, Copy, Clone)]
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
    pub fn try_from_vec(mut vec: Vec<(K, V)>) -> Result<Self, EnumTableFromVecError<K>> {
        if vec.len() != N {
            return Err(EnumTableFromVecError::InvalidSize {
                expected: N,
                found: vec.len(),
            });
        }

        let mut builder = EnumTableBuilder::<K, V, N>::new();

        // Check that all variants are present and move values out
        for variant in K::VARIANTS {
            if let Some(pos) = vec
                .iter()
                .position(|(k, _)| to_usize(k) == to_usize(variant))
            {
                let (_, value) = vec.swap_remove(pos);
                unsafe {
                    builder.push_unchecked(variant, value);
                }
            } else {
                return Err(EnumTableFromVecError::MissingVariant(*variant));
            }
        }

        Ok(unsafe { builder.build_to_unchecked() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Enumable)]
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
        assert_eq!(
            result,
            Err(crate::EnumTableFromVecError::InvalidSize {
                expected: 3,
                found: 2
            })
        );
    }

    #[test]
    fn try_from_vec_missing_variant() {
        let vec = vec![
            (Color::Red, "Red"),
            (Color::Green, "Green"),
            (Color::Red, "Duplicate Red"), // Duplicate instead of Blue
        ];

        let result = EnumTable::<Color, &str, { Color::COUNT }>::try_from_vec(vec);
        assert_eq!(
            result,
            Err(crate::EnumTableFromVecError::MissingVariant(Color::Blue))
        );
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
