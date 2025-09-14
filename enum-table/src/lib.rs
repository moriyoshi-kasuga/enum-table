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
/// associated values. It provides efficient logarithmic-time access (O(log N))
/// to the values based on the enumeration variant. This is particularly useful
/// when you want to map enum variants to specific values without the overhead
/// of a `HashMap`.
///
/// # Guarantees and Design
///
/// The core design principle of `EnumTable` is that an instance is guaranteed to hold a
/// value for every variant of the enum `K`. This guarantee allows for a cleaner API
/// than general-purpose map structures.
///
/// For example, the [`Self::get`] method returns `&V` directly. This is in contrast to
/// [`std::collections::HashMap::get`], which returns an `Option<&V>` because a key may or may not be
/// present. With `EnumTable`, the presence of all keys (variants) is a type-level
/// invariant, eliminating the need for `unwrap()` or other `Option` handling.
///
/// If you need to handle cases where a value might not be present or will be set
/// later, you can use `Option<V>` as the value type: `EnumTable<K, Option<V>, N>`.
/// The struct provides convenient methods like [`Self::new_fill_with_none`] for this pattern.
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
/// # Examples
///
/// ```rust
/// use enum_table::{EnumTable, Enumable};
///
/// #[derive(Enumable, Copy, Clone)]
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
    table: [(K, V); N],
}

impl<K: Enumable, V, const N: usize> EnumTable<K, V, N> {
    /// Creates a new `EnumTable` with the given table of variants and values.
    /// Typically, you would use the [`crate::et`] macro or [`crate::builder::EnumTableBuilder`] to create an `EnumTable`.
    pub(crate) const fn new(table: [(K, V); N]) -> Self {
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

        Self { table }
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
        et!(K, V, { N }, |variant| f(variant))
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
        Ok(et!(K, V, { N }, |variant| {
            f(variant).map_err(|e| (*variant, e))?
        }))
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
        Ok(et!(K, V, { N }, |variant| f(variant).ok_or(*variant)?))
    }

    pub(crate) const fn binary_search(&self, variant: &K) -> usize {
        let mut low = 0;
        let mut high = N;

        while low < high {
            let mid = low + (high - low) / 2;
            if intrinsics::const_enum_lt(&self.table[mid].0, variant) {
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
        self.table.iter().map(|(discriminant, _)| discriminant)
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
            .map(|(discriminant, value)| (discriminant, value))
    }

    /// Returns an iterator over mutable references to the values in the table.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.table
            .iter_mut()
            .map(|(discriminant, value)| (&*discriminant, value))
    }

    /// Transforms all values in the table using the provided function.
    ///
    /// This method consumes the table and creates a new one with values
    /// transformed by the given closure.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that takes an owned value and returns a new value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumable};
    ///
    /// #[derive(Enumable, Copy, Clone)]
    /// enum Size {
    ///     Small,
    ///     Medium,
    ///     Large,
    /// }
    ///
    /// let table = EnumTable::<Size, i32, { Size::COUNT }>::new_with_fn(|size| match size {
    ///     Size::Small => 1,
    ///     Size::Medium => 2,
    ///     Size::Large => 3,
    /// });
    ///
    /// let doubled = table.map(|value| value * 2);
    ///
    /// assert_eq!(doubled.get(&Size::Small), &2);
    /// assert_eq!(doubled.get(&Size::Medium), &4);
    /// assert_eq!(doubled.get(&Size::Large), &6);
    /// ```
    pub fn map<U>(self, mut f: impl FnMut(V) -> U) -> EnumTable<K, U, N> {
        EnumTable::new(
            self.table
                .map(|(discriminant, value)| (discriminant, f(value))),
        )
    }

    /// Transforms all values in the table using the provided function, with access to the key.
    ///
    /// This method consumes the table and creates a new one with values
    /// transformed by the given closure, which receives both the key and the value.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that takes a key reference and an owned value, and returns a new value.
    pub fn map_with_key<U>(self, mut f: impl FnMut(&K, V) -> U) -> EnumTable<K, U, N> {
        EnumTable::new(
            self.table
                .map(|(discriminant, value)| (discriminant, f(&discriminant, value))),
        )
    }

    /// Transforms all values in the table in-place using the provided function.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that takes a mutable reference to a value and modifies it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumable};
    ///
    /// #[derive(Enumable, Copy, Clone)]
    /// enum Level {
    ///     Low,
    ///     Medium,
    ///     High,
    /// }
    ///
    /// let mut table = EnumTable::<Level, i32, { Level::COUNT }>::new_with_fn(|level| match level {
    ///     Level::Low => 10,
    ///     Level::Medium => 20,
    ///     Level::High => 30,
    /// });
    ///
    /// table.map_mut(|value| *value += 5);
    ///
    /// assert_eq!(table.get(&Level::Low), &15);
    /// assert_eq!(table.get(&Level::Medium), &25);
    /// assert_eq!(table.get(&Level::High), &35);
    /// ```
    pub fn map_mut(&mut self, mut f: impl FnMut(&mut V)) {
        self.table.iter_mut().for_each(|(_, value)| {
            f(value);
        });
    }

    /// Transforms all values in the table in-place using the provided function, with access to the key.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that takes a key reference and a mutable reference to a value, and modifies it.
    pub fn map_mut_with_key(&mut self, mut f: impl FnMut(&K, &mut V)) {
        self.table.iter_mut().for_each(|(discriminant, value)| {
            f(discriminant, value);
        });
    }
}

impl<K: Enumable, V, const N: usize> EnumTable<K, Option<V>, N> {
    /// Creates a new `EnumTable` with `None` values for each variant.
    pub const fn new_fill_with_none() -> Self {
        et!(K, Option<V>, { N }, |variant| None)
    }

    /// Clears the table, setting each value to `None`.
    pub fn clear_to_none(&mut self) {
        for (_, value) in &mut self.table {
            *value = None;
        }
    }
}

impl<K: Enumable, V: Copy, const N: usize> EnumTable<K, V, N> {
    /// Creates a new `EnumTable` with the same copied value for each variant.
    ///
    /// This method initializes the table with the same value for each
    /// variant of the enumeration. The value must implement `Copy`.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to copy for each enum variant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumable};
    ///
    /// #[derive(Enumable, Copy, Clone)]
    /// enum Status {
    ///     Active,
    ///     Inactive,
    ///     Pending,
    /// }
    ///
    /// let table = EnumTable::<Status, i32, { Status::COUNT }>::new_fill_with_copy(42);
    ///
    /// assert_eq!(table.get(&Status::Active), &42);
    /// assert_eq!(table.get(&Status::Inactive), &42);
    /// assert_eq!(table.get(&Status::Pending), &42);
    /// ```
    pub const fn new_fill_with_copy(value: V) -> Self {
        et!(K, V, { N }, |variant| value)
    }
}

impl<K: Enumable, V: Default, const N: usize> EnumTable<K, V, N> {
    /// Creates a new `EnumTable` with default values for each variant.
    ///
    /// This method initializes the table with the default value of type `V` for each
    /// variant of the enumeration.
    pub fn new_fill_with_default() -> Self {
        et!(K, V, { N }, |variant| V::default())
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

        let doubled = table.map(|value| value * 2);

        assert_eq!(doubled.get(&Color::Red), &2);
        assert_eq!(doubled.get(&Color::Green), &4);
        assert_eq!(doubled.get(&Color::Blue), &6);
    }

    #[test]
    fn map_with_key() {
        let table = EnumTable::<Color, i32, { Color::COUNT }>::new_with_fn(|color| match color {
            Color::Red => 1,
            Color::Green => 2,
            Color::Blue => 3,
        });

        let mapped = table.map_with_key(|key, value| match key {
            Color::Red => value + 10,   // 1 + 10 = 11
            Color::Green => value + 20, // 2 + 20 = 22
            Color::Blue => value + 30,  // 3 + 30 = 33
        });

        // Note: The order in the underlying table is based on discriminant value (Green, Red, Blue)
        assert_eq!(mapped.get(&Color::Red), &11);
        assert_eq!(mapped.get(&Color::Green), &22);
        assert_eq!(mapped.get(&Color::Blue), &33);
    }

    #[test]
    fn map_mut() {
        let mut table =
            EnumTable::<Color, i32, { Color::COUNT }>::new_with_fn(|color| match color {
                Color::Red => 10,
                Color::Green => 20,
                Color::Blue => 30,
            });

        table.map_mut(|value| *value += 5);

        assert_eq!(table.get(&Color::Red), &15);
        assert_eq!(table.get(&Color::Green), &25);
        assert_eq!(table.get(&Color::Blue), &35);
    }

    #[test]
    fn map_mut_with_key() {
        let mut table =
            EnumTable::<Color, i32, { Color::COUNT }>::new_with_fn(|color| match color {
                Color::Red => 10,
                Color::Green => 20,
                Color::Blue => 30,
            });

        table.map_mut_with_key(|key, value| {
            *value += match key {
                Color::Red => 1,   // 10 + 1 = 11
                Color::Green => 2, // 20 + 2 = 22
                Color::Blue => 3,  // 30 + 3 = 33
            }
        });

        assert_eq!(table.get(&Color::Red), &11);
        assert_eq!(table.get(&Color::Green), &22);
        assert_eq!(table.get(&Color::Blue), &33);
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
