#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]

#[cfg(feature = "derive")]
pub use enum_table_derive::Enumable;

pub mod builder;

mod impls;
mod intrinsics;
mod macros;

use dev_macros::*;
use intrinsics::from_usize;

/// A trait for enumerations that can be used with `EnumTable`.
///
/// This trait requires that the enumeration provides a static array of its variants
/// and a constant representing the count of these variants.
pub trait Enumable: Sized + 'static {
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
    /// Typically, you would use the [`crate::et`] macro or the [`crate::builder::EnumTableBuilder`] instead.
    ///
    ///
    /// # Arguments
    ///
    /// * `table` - An array of tuples where each tuple contains a discriminant of
    ///   an enumeration variant and its associated value.
    pub const fn new(table: [(usize, V); N]) -> Self {
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
    pub fn try_new_with_fn<E>(
        mut f: impl FnMut(&K) -> Result<V, E>,
    ) -> Result<Self, (&'static K, E)> {
        let mut builder = builder::EnumTableBuilder::<K, V, N>::new();

        for variant in K::VARIANTS {
            match f(variant) {
                Ok(value) => builder.push(variant, value),
                Err(e) => return Err((variant, e)),
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
    pub fn checked_new_with_fn(mut f: impl FnMut(&K) -> Option<V>) -> Result<Self, &'static K> {
        let mut builder = builder::EnumTableBuilder::<K, V, N>::new();

        for variant in K::VARIANTS {
            if let Some(value) = f(variant) {
                builder.push(variant, value);
            } else {
                return Err(variant);
            }
        }

        Ok(builder.build_to())
    }

    /// Returns a reference to the value associated with the given enumeration variant.
    ///
    /// # Arguments
    ///
    /// * `variant` - A reference to an enumeration variant.
    pub const fn get(&self, variant: &K) -> &V {
        use_variant_value!(self, variant, i, {
            return &self.table[i].1;
        });
    }

    /// Returns a mutable reference to the value associated with the given enumeration variant.
    ///
    /// # Arguments
    ///
    /// * `variant` - A reference to an enumeration variant.
    pub const fn get_mut(&mut self, variant: &K) -> &mut V {
        use_variant_value!(self, variant, i, {
            return &mut self.table[i].1;
        });
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
        use_variant_value!(self, variant, i, {
            return core::mem::replace(&mut self.table[i].1, value);
        });
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
            .map(|(discriminant, _)| from_usize(discriminant))
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
            .map(|(discriminant, value)| (from_usize(discriminant), value))
    }

    /// Returns an iterator over mutable references to the values in the table.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.table
            .iter_mut()
            .map(|(discriminant, value)| (from_usize(discriminant), value))
    }
}

mod dev_macros {
    macro_rules! use_variant_value {
        ($self:ident, $variant:ident, $i:ident,{$($tt:tt)+}) => {
            let discriminant = crate::intrinsics::to_usize($variant);

            let mut $i = 0;
            while $i < $self.table.len() {
                if $self.table[$i].0 == discriminant {
                    $($tt)+
                }
                $i += 1;
            }
            unreachable!();
        };
    }

    pub(super) use use_variant_value;
}

#[cfg(test)]
mod tests {
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

        assert_eq!(variant, &Color::Green);
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

        assert_eq!(variant, &Color::Green);
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
        assert_eq!(keys, vec![&Color::Red, &Color::Green, &Color::Blue]);
    }

    #[test]
    fn values() {
        let values: Vec<_> = TABLES.values().collect();
        assert_eq!(values, vec![&"Red", &"Green", &"Blue"]);
    }

    #[test]
    fn iter() {
        let iter: Vec<_> = TABLES.iter().collect();
        assert_eq!(
            iter,
            vec![
                (&Color::Red, &"Red"),
                (&Color::Green, &"Green"),
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
                (&Color::Red, &"Changed Red"),
                (&Color::Green, &"Changed Green"),
                (&Color::Blue, &"Changed Blue")
            ]
        );
    }
}
