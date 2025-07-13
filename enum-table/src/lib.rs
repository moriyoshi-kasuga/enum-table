#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]

#[cfg(test)]
pub extern crate self as enum_table;

#[cfg(feature = "derive")]
pub use enum_table_derive::Enumable;

pub mod builder;
mod intrinsics;

pub mod __private {
    pub use crate::intrinsics::sort_variants;
}

mod impls;
pub use impls::*;

mod macros;

use intrinsics::{cast_variant, into_variant, to_usize};

/// A trait for enumerations that can be used with `EnumTable`.
///
/// This trait requires that the enumeration provides a static array of its variants
/// and a constant representing the count of these variants.
pub trait Enumable: Copy + 'static {
    const VARIANTS: &'static [Self];
    const COUNT: usize = Self::VARIANTS.len();
}

/// A table that associates each variant of an enumeration with a value.
///
/// `EnumTable` is a generic struct that uses an enumeration as keys and stores
/// associated values. It provides constant-time access to the values based on
/// the enumeration variant. This is particularly useful when you want to map
/// enum variants to specific values without the overhead of a `HashMap`.
///
/// # Type Parameters
///
/// * `K`: The enumeration type that implements the `Enumable` trait. This trait
///   ensures that the enum provides a static array of its variants and a count
///   of these variants.
/// * `V`: The type of values to be associated with each enum variant.
/// * `N`: The number of variants in the enum, which should match the length of
///   the static array of variants provided by the `Enumable` trait.
///
/// # Note
/// The `new` method allows for the creation of an `EnumTable` in `const` contexts,
/// but it does not perform compile-time checks. For enhanced compile-time safety
/// and convenience, it is advisable to use the [`crate::et`] macro or
/// [`crate::builder::EnumTableBuilder`], which provide these checks.
///
/// # Examples
///
/// ```rust
/// use enum_table::{EnumTable, Enumable};
///
/// #[derive(Enumable)]
/// enum Color {
///     Red,
///     Green,
///     Blue,
/// }
///
/// // Create an EnumTable using the new_with_fn method
/// let table = EnumTable::<Color, &'static str, { Color::COUNT }>::new_with_fn(|color| match color {
///     Color::Red => "Red",
///     Color::Green => "Green",
///     Color::Blue => "Blue",
/// });
///
/// // Access values associated with enum variants
/// assert_eq!(table.get(&Color::Red), &"Red");
/// assert_eq!(table.get(&Color::Green), &"Green");
/// assert_eq!(table.get(&Color::Blue), &"Blue");
/// ```
pub struct EnumTable<K: Enumable, V, const N: usize> {
    table: [(usize, V); N],
    _phantom: core::marker::PhantomData<K>,
}

impl<K: Enumable, V, const N: usize> EnumTable<K, V, N> {
    /// Creates a new `EnumTable` with the given table of discriminants and values.
    /// Typically, you would use the [`crate::et`] macro or [`crate::builder::EnumTableBuilder`] to create an `EnumTable`.
    pub(crate) const fn new(table: [(usize, V); N]) -> Self {
        #[cfg(debug_assertions)]
        const {
            // Ensure that the variants are sorted by their discriminants.
            // This is a compile-time check to ensure that the variants are in the correct order.
            if !intrinsics::is_sorted(K::VARIANTS) {
                panic!(
                    "Enumable: variants are not sorted by discriminant. Use `enum_table::Enumable` derive macro to ensure correct ordering."
                );
            }
        }

        Self {
            table,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Create a new EnumTable with a function that takes a variant and returns a value.
    /// If you want to define it in const, use [`crate::et`] macro
    /// Creates a new `EnumTable` using a function to generate values for each variant.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that takes a reference to an enumeration variant and returns
    ///   a value to be associated with that variant.
    pub fn new_with_fn(mut f: impl FnMut(&K) -> V) -> Self {
        let mut builder = builder::EnumTableBuilder::<K, V, N>::new();

        for variant in K::VARIANTS {
            builder.push(variant, f(variant));
        }

        builder.build_to()
    }

    /// Creates a new `EnumTable` using a function that returns a `Result` for each variant.
    ///
    /// This method applies the provided closure to each variant of the enum. If the closure
    /// returns `Ok(value)` for all variants, an `EnumTable` is constructed and returned as `Ok(Self)`.
    /// If the closure returns `Err(e)` for any variant, the construction is aborted and
    /// `Err((variant, e))` is returned, where `variant` is the enum variant that caused the error.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that takes a reference to an enum variant and returns a `Result<V, E>`.
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` if all variants succeed.
    /// * `Err((variant, e))` if any variant fails, containing the failing variant and the error.
    pub fn try_new_with_fn<E>(mut f: impl FnMut(&K) -> Result<V, E>) -> Result<Self, (K, E)> {
        let mut builder = builder::EnumTableBuilder::<K, V, N>::new();

        for variant in K::VARIANTS {
            match f(variant) {
                Ok(value) => builder.push(variant, value),
                Err(e) => return Err((*variant, e)),
            }
        }

        Ok(builder.build_to())
    }

    /// Creates a new `EnumTable` using a function that returns an `Option` for each variant.
    ///
    /// This method applies the provided closure to each variant of the enum. If the closure
    /// returns `Some(value)` for all variants, an `EnumTable` is constructed and returned as `Ok(Self)`.
    /// If the closure returns `None` for any variant, the construction is aborted and
    /// `Err(variant)` is returned, where `variant` is the enum variant that caused the failure.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that takes a reference to an enum variant and returns an `Option<V>`.
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` if all variants succeed.
    /// * `Err(variant)` if any variant fails, containing the failing variant.
    pub fn checked_new_with_fn(mut f: impl FnMut(&K) -> Option<V>) -> Result<Self, K> {
        let mut builder = builder::EnumTableBuilder::<K, V, N>::new();

        for variant in K::VARIANTS {
            if let Some(value) = f(variant) {
                builder.push(variant, value);
            } else {
                return Err(*variant);
            }
        }

        Ok(builder.build_to())
    }

    pub(crate) const fn binary_search(&self, variant: &K) -> usize {
        let discriminant = to_usize(variant);
        let mut low = 0;
        let mut high = N;

        while low < high {
            let mid = low + (high - low) / 2;
            if self.table[mid].0 < discriminant {
                low = mid + 1;
            } else {
                high = mid;
            }
        }

        low
    }

    /// Returns a reference to the value associated with the given enumeration variant.
    ///
    /// # Arguments
    ///
    /// * `variant` - A reference to an enumeration variant.
    pub const fn get(&self, variant: &K) -> &V {
        let idx = self.binary_search(variant);
        &self.table[idx].1
    }

    /// Returns a mutable reference to the value associated with the given enumeration variant.
    ///
    /// # Arguments
    ///
    /// * `variant` - A reference to an enumeration variant.
    pub const fn get_mut(&mut self, variant: &K) -> &mut V {
        let idx = self.binary_search(variant);
        &mut self.table[idx].1
    }

    /// Sets the value associated with the given enumeration variant.
    ///
    /// # Arguments
    ///
    /// * `variant` - A reference to an enumeration variant.
    /// * `value` - The new value to associate with the variant.
    /// # Returns
    /// The old value associated with the variant.
    pub const fn set(&mut self, variant: &K, value: V) -> V {
        let idx = self.binary_search(variant);
        core::mem::replace(&mut self.table[idx].1, value)
    }

    /// Returns the number of generic N
    pub const fn len(&self) -> usize {
        N
    }

    /// Returns `false` since the table is never empty.
    pub const fn is_empty(&self) -> bool {
        false
    }

    /// Returns an iterator over references to the keys in the table.
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.table
            .iter()
            .map(|(discriminant, _)| cast_variant(discriminant))
    }

    /// Returns an iterator over references to the values in the table.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.table.iter().map(|(_, value)| value)
    }

    /// Returns an iterator over mutable references to the values in the table.
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.table.iter_mut().map(|(_, value)| value)
    }

    /// Returns an iterator over mutable references to the values in the table.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.table
            .iter()
            .map(|(discriminant, value)| (cast_variant(discriminant), value))
    }

    /// Returns an iterator over mutable references to the values in the table.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.table
            .iter_mut()
            .map(|(discriminant, value)| (cast_variant(discriminant), value))
    }

    /// Maps the values of the table using a closure, creating a new `EnumTable` with the transformed values.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that takes a value and returns a transformed value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumable};
    ///
    /// #[derive(Enumable)]
    /// enum Color {
    ///     Red,
    ///     Green,
    ///     Blue,
    /// }
    ///
    /// let table = EnumTable::<Color, i32, { Color::COUNT }>::new_with_fn(|color| match color {
    ///     Color::Red => 1,
    ///     Color::Green => 2,
    ///     Color::Blue => 3,
    /// });
    ///
    /// let doubled = table.map(|x| x * 2);
    /// assert_eq!(doubled.get(&Color::Red), &2);
    /// assert_eq!(doubled.get(&Color::Green), &4);
    /// assert_eq!(doubled.get(&Color::Blue), &6);
    /// ```
    pub fn map<U, F>(self, mut f: F) -> EnumTable<K, U, N>
    where
        F: FnMut(V) -> U,
    {
        let mut builder = builder::EnumTableBuilder::<K, U, N>::new();

        for (discriminant, value) in self.table {
            let key = cast_variant(&discriminant);
            builder.push(key, f(value));
        }

        builder.build_to()
    }
}

impl<K: Enumable, V, const N: usize> EnumTable<K, Option<V>, N> {
    /// Creates a new `EnumTable` with `None` values for each variant.
    pub const fn new_fill_with_none() -> Self {
        let mut builder = builder::EnumTableBuilder::<K, Option<V>, N>::new();

        let mut i = 0;
        while i < N {
            let variant = &K::VARIANTS[i];
            builder.push(variant, None);
            i += 1;
        }

        builder.build_to()
    }

    /// Clears the table, setting each value to `None`.
    pub fn clear_to_none(&mut self) {
        for (_, value) in &mut self.table {
            *value = None;
        }
    }
}

impl<K: Enumable, V: Copy, const N: usize> EnumTable<K, V, N> {
    pub const fn new_fill_with_copy(value: V) -> Self {
        let mut builder = builder::EnumTableBuilder::<K, V, N>::new();

        let mut i = 0;
        while i < N {
            let variant = &K::VARIANTS[i];
            builder.push(variant, value);
            i += 1;
        }

        builder.build_to()
    }
}

impl<K: Enumable, V: Default, const N: usize> EnumTable<K, V, N> {
    /// Creates a new `EnumTable` with default values for each variant.
    ///
    /// This method initializes the table with the default value of type `V` for each
    /// variant of the enumeration.
    pub fn new_fill_with_default() -> Self {
        let mut builder = builder::EnumTableBuilder::<K, V, N>::new();

        for variant in K::VARIANTS {
            builder.push(variant, V::default());
        }

        builder.build_to()
    }

    /// Clears the table, setting each value to its default.
    pub fn clear_to_default(&mut self) {
        for (_, value) in &mut self.table {
            *value = V::default();
        }
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
    fn new_with_fn() {
        let table =
            EnumTable::<Color, &'static str, { Color::COUNT }>::new_with_fn(|color| match color {
                Color::Red => "Red",
                Color::Green => "Green",
                Color::Blue => "Blue",
            });

        assert_eq!(table.get(&Color::Red), &"Red");
        assert_eq!(table.get(&Color::Green), &"Green");
        assert_eq!(table.get(&Color::Blue), &"Blue");
    }

    #[test]
    fn try_new_with_fn() {
        let table =
            EnumTable::<Color, &'static str, { Color::COUNT }>::try_new_with_fn(
                |color| match color {
                    Color::Red => Ok::<&'static str, core::convert::Infallible>("Red"),
                    Color::Green => Ok("Green"),
                    Color::Blue => Ok("Blue"),
                },
            );

        assert!(table.is_ok());
        let table = table.unwrap();

        assert_eq!(table.get(&Color::Red), &"Red");
        assert_eq!(table.get(&Color::Green), &"Green");
        assert_eq!(table.get(&Color::Blue), &"Blue");

        let error_table = EnumTable::<Color, &'static str, { Color::COUNT }>::try_new_with_fn(
            |color| match color {
                Color::Red => Ok("Red"),
                Color::Green => Err("Error on Green"),
                Color::Blue => Ok("Blue"),
            },
        );

        assert!(error_table.is_err());
        let (variant, error) = error_table.unwrap_err();

        assert_eq!(variant, Color::Green);
        assert_eq!(error, "Error on Green");
    }

    #[test]
    fn checked_new_with_fn() {
        let table =
            EnumTable::<Color, &'static str, { Color::COUNT }>::checked_new_with_fn(|color| {
                match color {
                    Color::Red => Some("Red"),
                    Color::Green => Some("Green"),
                    Color::Blue => Some("Blue"),
                }
            });

        assert!(table.is_ok());
        let table = table.unwrap();

        assert_eq!(table.get(&Color::Red), &"Red");
        assert_eq!(table.get(&Color::Green), &"Green");
        assert_eq!(table.get(&Color::Blue), &"Blue");

        let error_table =
            EnumTable::<Color, &'static str, { Color::COUNT }>::checked_new_with_fn(|color| {
                match color {
                    Color::Red => Some("Red"),
                    Color::Green => None,
                    Color::Blue => Some("Blue"),
                }
            });

        assert!(error_table.is_err());
        let variant = error_table.unwrap_err();

        assert_eq!(variant, Color::Green);
    }

    #[test]
    fn get() {
        assert_eq!(TABLES.get(&Color::Red), &"Red");
        assert_eq!(TABLES.get(&Color::Green), &"Green");
        assert_eq!(TABLES.get(&Color::Blue), &"Blue");
    }

    #[test]
    fn get_mut() {
        let mut table = TABLES;
        assert_eq!(table.get_mut(&Color::Red), &mut "Red");
        assert_eq!(table.get_mut(&Color::Green), &mut "Green");
        assert_eq!(table.get_mut(&Color::Blue), &mut "Blue");

        *table.get_mut(&Color::Red) = "Changed Red";
        *table.get_mut(&Color::Green) = "Changed Green";
        *table.get_mut(&Color::Blue) = "Changed Blue";

        assert_eq!(table.get(&Color::Red), &"Changed Red");
        assert_eq!(table.get(&Color::Green), &"Changed Green");
        assert_eq!(table.get(&Color::Blue), &"Changed Blue");
    }

    #[test]
    fn set() {
        let mut table = TABLES;
        assert_eq!(table.set(&Color::Red, "New Red"), "Red");
        assert_eq!(table.set(&Color::Green, "New Green"), "Green");
        assert_eq!(table.set(&Color::Blue, "New Blue"), "Blue");

        assert_eq!(table.get(&Color::Red), &"New Red");
        assert_eq!(table.get(&Color::Green), &"New Green");
        assert_eq!(table.get(&Color::Blue), &"New Blue");
    }

    #[test]
    fn keys() {
        let keys: Vec<_> = TABLES.keys().collect();
        assert_eq!(keys, vec![&Color::Green, &Color::Red, &Color::Blue]);
    }

    #[test]
    fn values() {
        let values: Vec<_> = TABLES.values().collect();
        assert_eq!(values, vec![&"Green", &"Red", &"Blue"]);
    }

    #[test]
    fn iter() {
        let iter: Vec<_> = TABLES.iter().collect();
        assert_eq!(
            iter,
            vec![
                (&Color::Green, &"Green"),
                (&Color::Red, &"Red"),
                (&Color::Blue, &"Blue")
            ]
        );
    }

    #[test]
    fn iter_mut() {
        let mut table = TABLES;
        for (key, value) in table.iter_mut() {
            *value = match key {
                Color::Red => "Changed Red",
                Color::Green => "Changed Green",
                Color::Blue => "Changed Blue",
            };
        }
        let iter: Vec<_> = table.iter().collect();
        assert_eq!(
            iter,
            vec![
                (&Color::Green, &"Changed Green"),
                (&Color::Red, &"Changed Red"),
                (&Color::Blue, &"Changed Blue")
            ]
        );
    }

    #[test]
    fn map() {
        let table = EnumTable::<Color, i32, { Color::COUNT }>::new_with_fn(|color| match color {
            Color::Red => 1,
            Color::Green => 2,
            Color::Blue => 3,
        });

        let doubled = table.map(|x| x * 2);
        assert_eq!(doubled.get(&Color::Red), &2);
        assert_eq!(doubled.get(&Color::Green), &4);
        assert_eq!(doubled.get(&Color::Blue), &6);
    }

    macro_rules! run_variants_test {
        ($($variant:ident),+) => {{
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Enumable)]
            #[repr(u8)]
            enum Test {
                $($variant,)*
            }

            let map = EnumTable::<Test, &'static str, { Test::COUNT }>::new_with_fn(|t| match t {
                $(Test::$variant => stringify!($variant),)*
            });
            $(
                assert_eq!(map.get(&Test::$variant), &stringify!($variant));
            )*
        }};
    }

    #[test]
    fn binary_search_correct_variants() {
        run_variants_test!(A);
        run_variants_test!(A, B);
        run_variants_test!(A, B, C);
        run_variants_test!(A, B, C, D);
        run_variants_test!(A, B, C, D, E);
    }
}
