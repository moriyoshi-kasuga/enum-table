use crate::{EnumTable, Enumerable, intrinsics};

impl<K: Enumerable, V, const N: usize> EnumTable<K, V, N> {
    /// Creates a new `EnumTable` from `pairs`, or returns `None` if `pairs` doesn't
    /// contain exactly one entry for each variant of `K`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumerable};
    ///
    /// #[derive(Enumerable, Copy, Clone, Debug, PartialEq)]
    /// enum Color {
    ///     Red,
    ///     Green,
    ///     Blue,
    /// }
    ///
    /// let pairs = [
    ///     (Color::Red, "Red"),
    ///     (Color::Green, "Green"),
    ///     (Color::Blue, "Blue"),
    /// ];
    /// let table = EnumTable::<Color, &str, { Color::COUNT }>::checked_from_pairs(pairs).unwrap();
    /// assert_eq!(table.get(Color::Red), &"Red");
    /// assert_eq!(table.get(Color::Green), &"Green");
    /// assert_eq!(table.get(Color::Blue), &"Blue");
    /// ```
    ///
    /// A duplicate entry is rejected, instead of silently producing a table with an
    /// unrelated variant missing:
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumerable};
    ///
    /// #[derive(Enumerable, Copy, Clone, Debug, PartialEq)]
    /// enum Color {
    ///     Red,
    ///     Green,
    ///     Blue,
    /// }
    ///
    /// let pairs = [
    ///     (Color::Red, "Red"),
    ///     (Color::Green, "Green"),
    ///     (Color::Red, "Duplicate Red"),
    /// ];
    /// assert_eq!(
    ///     EnumTable::<Color, &str, { Color::COUNT }>::checked_from_pairs(pairs),
    ///     None
    /// );
    /// ```
    pub fn checked_from_pairs(
        pairs: impl IntoIterator<Item = (K, V), IntoIter: ExactSizeIterator>,
    ) -> Option<Self> {
        let pairs = pairs.into_iter();
        if pairs.len() != N {
            return None;
        }

        let mut slots: [Option<V>; N] = core::array::from_fn(|_| None);
        for (key, value) in pairs {
            let idx = key.variant_index();
            if slots[idx].is_some() {
                return None;
            }
            slots[idx] = Some(value);
        }

        let table = intrinsics::try_collect_array(|i| slots[i].take().ok_or(())).ok()?;
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

    #[test]
    fn checked_from_pairs() {
        let pairs = [
            (Color::Red, "Red"),
            (Color::Green, "Green"),
            (Color::Blue, "Blue"),
        ];

        let table = EnumTable::<Color, &str, { Color::COUNT }>::checked_from_pairs(pairs).unwrap();
        assert_eq!(table.get(Color::Red), &"Red");
        assert_eq!(table.get(Color::Green), &"Green");
        assert_eq!(table.get(Color::Blue), &"Blue");
    }

    #[test]
    fn checked_from_pairs_wrong_len() {
        let pairs = [(Color::Red, "Red"), (Color::Green, "Green")];
        assert_eq!(
            EnumTable::<Color, &str, { Color::COUNT }>::checked_from_pairs(pairs),
            None
        );
    }

    #[test]
    fn checked_from_pairs_duplicate_key() {
        let pairs = [
            (Color::Red, "Red"),
            (Color::Green, "Green"),
            (Color::Red, "Duplicate Red"),
        ];
        assert_eq!(
            EnumTable::<Color, &str, { Color::COUNT }>::checked_from_pairs(pairs),
            None
        );
    }

    #[test]
    fn checked_from_pairs_missing_variant() {
        let pairs = [(Color::Red, "Red"), (Color::Green, "Green")];
        // 2 pairs against N=3 is caught by the length check before the missing-variant
        // path is ever reached, so exercise it via an iterator whose (inaccurate)
        // `ExactSizeIterator::len` matches `N` but whose actual items still leave a
        // variant unfilled.
        struct LyingLen<I>(I, usize);
        impl<I: Iterator> Iterator for LyingLen<I> {
            type Item = I::Item;
            fn next(&mut self) -> Option<Self::Item> {
                self.0.next()
            }
        }
        impl<I: Iterator> ExactSizeIterator for LyingLen<I> {
            fn len(&self) -> usize {
                self.1
            }
        }

        let iter = LyingLen(pairs.into_iter(), 3);
        assert_eq!(
            EnumTable::<Color, &str, { Color::COUNT }>::checked_from_pairs(iter),
            None
        );
    }
}
