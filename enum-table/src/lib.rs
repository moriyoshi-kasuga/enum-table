#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(test)]
pub extern crate self as enum_table;

use core::marker::PhantomData;

#[cfg(feature = "derive")]
pub use enum_table_derive::Enumerable;

mod builder;
mod intrinsics;

#[doc(hidden)]
pub mod __private {
    pub use crate::builder::EnumTableBuilder;
    pub use crate::intrinsics::{sort_variants, variant_index_of};
}

mod impls;

mod macros;

/// Enumerations whose variants `EnumTable` can list and index.
///
/// Prefer `#[derive(Enumerable)]` over a manual `unsafe impl`; it upholds the
/// safety contract below automatically for field-less enums.
///
/// # Safety
///
/// Implementors must guarantee that `Self` has no padding bytes in its
/// in-memory representation. Violating this is undefined behavior: this
/// crate's byte-level comparisons ([`Self::variant_index`]'s default
/// implementation, and every [`EnumTable`] constructor) read `Self` as raw
/// bytes, and padding bytes may be uninitialized memory.
///
/// Implementors must also guarantee that `VARIANTS` contains every variant
/// of `Self` exactly once, sorted in ascending order by the unsigned
/// bit-pattern of that representation. For example, with `#[repr(i8)]`,
/// `-1` sorts *after* `0` and `1`, since its bit pattern (`0xFF`) is
/// numerically larger than theirs. Violating this guarantee is not
/// undefined behavior on its own — [`EnumTable`] only ever indexes into its
/// backing array with bounds checks — but it does silently produce wrong
/// results: [`EnumTable::get`]/[`EnumTable::set`]/etc. return or mutate the
/// value for the wrong variant. Every [`EnumTable`] constructor checks this
/// guarantee at compile time (once per monomorphization of `K`) via a
/// `const` assertion, so hand-written `unsafe impl`s that get this wrong
/// fail to compile as soon as an `EnumTable<K, _, _>` is actually used.
///
/// # Examples
///
/// ```rust
/// use enum_table::Enumerable;
///
/// #[derive(Copy, Clone)]
/// #[repr(u8)]
/// enum Test {
///     A,
///     B,
///     C,
/// }
///
/// // SAFETY: `Test` is a field-less `#[repr(u8)]` enum (no padding bytes),
/// // and `VARIANTS` lists every variant exactly once, sorted by discriminant.
/// unsafe impl Enumerable for Test {
///     const VARIANTS: &'static [Self] = &[Test::A, Test::B, Test::C];
/// }
///
/// assert_eq!(Test::B.variant_index(), 1);
/// ```
pub unsafe trait Enumerable: Copy + 'static {
    const VARIANTS: &'static [Self];
    const COUNT: usize = Self::VARIANTS.len();

    /// Returns the index of this variant within `VARIANTS`.
    ///
    /// The default implementation performs an O(log N) binary search.
    /// `#[derive(Enumerable)]` overrides it with a compile-time-computed
    /// constant per variant, giving O(1) access for dense, sequential
    /// discriminants and a comparison tree for sparse or custom ones.
    fn variant_index(&self) -> usize {
        intrinsics::binary_search_index::<Self>(self)
    }
}

/// A fixed-size table holding one `V` per variant of `K`.
///
/// Because a value is guaranteed to exist for every variant, [`Self::get`]
/// returns `&V` directly, unlike [`std::collections::HashMap::get`], which
/// returns `Option<&V>`. To represent a value that may be absent, use
/// `EnumTable<K, Option<V>, N>`; see [`Self::new_fill_with_default`].
///
/// # Examples
///
/// ```rust
/// use enum_table::{EnumTable, Enumerable};
///
/// #[derive(Enumerable, Copy, Clone)]
/// enum Color {
///     Red,
///     Green,
///     Blue,
/// }
///
/// let table = EnumTable::<Color, &'static str, { Color::COUNT }>::new_with_fn(|color| match color {
///     Color::Red => "Red",
///     Color::Green => "Green",
///     Color::Blue => "Blue",
/// });
///
/// assert_eq!(table.get(Color::Red), &"Red");
/// assert_eq!(table.get(Color::Green), &"Green");
/// assert_eq!(table.get(Color::Blue), &"Blue");
/// ```
pub struct EnumTable<K: Enumerable, V, const N: usize> {
    table: [V; N],
    _phantom: PhantomData<K>,
}

impl<K: Enumerable, V, const N: usize> EnumTable<K, V, N> {
    pub(crate) const fn assert() {
        const {
            assert!(
                N == K::COUNT,
                "EnumTable: N must equal K::COUNT. The const generic N does not match the number of enum variants."
            );
            assert!(
                intrinsics::is_sorted(K::VARIANTS),
                "EnumTable: K::VARIANTS is not sorted in ascending order by unsigned bit-pattern. This is required by the `Enumerable` trait's safety contract; use `#[derive(Enumerable)]` instead of a hand-written `unsafe impl` to avoid this."
            );
        }
    }

    pub(crate) const fn new(table: [V; N]) -> Self {
        const { Self::assert() };

        Self {
            table,
            _phantom: PhantomData,
        }
    }

    /// Creates a new `EnumTable` by applying `f` to each variant of `K`.
    pub fn new_with_fn(mut f: impl FnMut(K) -> V) -> Self {
        Self::new(core::array::from_fn(|i| f(K::VARIANTS[i])))
    }

    /// Creates a new `EnumTable` by applying `f` to each variant of `K`,
    /// stopping at the first `Err`.
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
    /// let result = EnumTable::<Color, &'static str, { Color::COUNT }>::try_new_with_fn(
    ///     |color| match color {
    ///         Color::Red => Ok("Red"),
    ///         Color::Green => Err("Failed to get value for Green"),
    ///         Color::Blue => Ok("Blue"),
    ///     },
    /// );
    ///
    /// let (variant, error) = result.unwrap_err();
    /// assert_eq!(variant, Color::Green);
    /// assert_eq!(error, "Failed to get value for Green");
    /// ```
    pub fn try_new_with_fn<E>(mut f: impl FnMut(K) -> Result<V, E>) -> Result<Self, (K, E)> {
        let table = intrinsics::try_collect_array(|i| {
            let variant = K::VARIANTS[i];
            f(variant).map_err(|e| (variant, e))
        })?;
        Ok(Self::new(table))
    }

    /// Creates a new `EnumTable` by applying `f` to each variant of `K`,
    /// stopping at the first `None`.
    pub fn checked_new_with_fn(mut f: impl FnMut(K) -> Option<V>) -> Result<Self, K> {
        Self::try_new_with_fn(|k| f(k).ok_or(())).map_err(|(k, ())| k)
    }

    /// Returns a reference to the value associated with `variant`, via O(1) lookup.
    pub fn get(&self, variant: K) -> &V {
        &self.table[variant.variant_index()]
    }

    /// Returns a mutable reference to the value associated with `variant`, via O(1) lookup.
    pub fn get_mut(&mut self, variant: K) -> &mut V {
        &mut self.table[variant.variant_index()]
    }

    /// Sets the value associated with `variant`, via O(1) lookup, and returns the old value.
    pub fn set(&mut self, variant: K, value: V) -> V {
        core::mem::replace(&mut self.table[variant.variant_index()], value)
    }

    /// `const fn` equivalent of [`Self::get`], using O(log N) binary search instead of O(1) lookup.
    pub const fn get_const(&self, variant: K) -> &V {
        let idx = intrinsics::binary_search_index::<K>(&variant);
        &self.table[idx]
    }

    /// `const fn` equivalent of [`Self::get_mut`], using O(log N) binary search instead of O(1) lookup.
    pub const fn get_mut_const(&mut self, variant: K) -> &mut V {
        let idx = intrinsics::binary_search_index::<K>(&variant);
        &mut self.table[idx]
    }

    /// `const fn` equivalent of [`Self::set`], using O(log N) binary search instead of O(1) lookup.
    pub const fn set_const(&mut self, variant: K, value: V) -> V {
        let idx = intrinsics::binary_search_index::<K>(&variant);
        core::mem::replace(&mut self.table[idx], value)
    }

    /// Combines `self` and `other` into a new table by applying `f` to each pair of values
    /// sharing a variant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumerable};
    ///
    /// #[derive(Enumerable, Copy, Clone)]
    /// enum Stat {
    ///     Hp,
    ///     Attack,
    ///     Defense,
    /// }
    ///
    /// let base = EnumTable::<Stat, i32, { Stat::COUNT }>::new_with_fn(|s| match s {
    ///     Stat::Hp => 100,
    ///     Stat::Attack => 50,
    ///     Stat::Defense => 30,
    /// });
    /// let bonus = EnumTable::<Stat, i32, { Stat::COUNT }>::new_with_fn(|s| match s {
    ///     Stat::Hp => 20,
    ///     Stat::Attack => 10,
    ///     Stat::Defense => 5,
    /// });
    ///
    /// let total = base.zip(bonus, |_stat, a, b| a + b);
    /// assert_eq!(total.get(Stat::Hp), &120);
    /// assert_eq!(total.get(Stat::Attack), &60);
    /// assert_eq!(total.get(Stat::Defense), &35);
    /// ```
    pub fn zip<U, W>(
        self,
        other: EnumTable<K, U, N>,
        mut f: impl FnMut(K, V, U) -> W,
    ) -> EnumTable<K, W, N> {
        let mut other_iter = other.table.into_iter();
        let mut i = 0;
        EnumTable::new(self.table.map(|v| {
            // SAFETY: both arrays have exactly N elements, and map calls this exactly N times
            let u = unsafe { other_iter.next().unwrap_unchecked() };
            let key = K::VARIANTS[i];
            i += 1;
            f(key, v, u)
        }))
    }

    /// Consumes the table, returning a new one with each value transformed by `f`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumerable};
    ///
    /// #[derive(Enumerable, Copy, Clone)]
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
    /// let doubled = table.map(|_size, value| value * 2);
    ///
    /// assert_eq!(doubled.get(Size::Small), &2);
    /// assert_eq!(doubled.get(Size::Medium), &4);
    /// assert_eq!(doubled.get(Size::Large), &6);
    /// ```
    pub fn map<U>(self, mut f: impl FnMut(K, V) -> U) -> EnumTable<K, U, N> {
        let mut i = 0;
        EnumTable::new(self.table.map(|value| {
            let key = K::VARIANTS[i];
            i += 1;
            f(key, value)
        }))
    }

    /// Transforms each value in the table in place via `f`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumerable};
    ///
    /// #[derive(Enumerable, Copy, Clone)]
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
    /// table.map_mut(|_level, value| *value += 5);
    ///
    /// assert_eq!(table.get(Level::Low), &15);
    /// assert_eq!(table.get(Level::Medium), &25);
    /// assert_eq!(table.get(Level::High), &35);
    /// ```
    pub fn map_mut(&mut self, mut f: impl FnMut(K, &mut V)) {
        self.table.iter_mut().enumerate().for_each(|(i, value)| {
            f(K::VARIANTS[i], value);
        });
    }
}

impl<K: Enumerable, V: Copy, const N: usize> EnumTable<K, V, N> {
    /// Creates a new `EnumTable` with `value` copied for every variant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use enum_table::{EnumTable, Enumerable};
    ///
    /// #[derive(Enumerable, Copy, Clone)]
    /// enum Status {
    ///     Active,
    ///     Inactive,
    ///     Pending,
    /// }
    ///
    /// let table = EnumTable::<Status, i32, { Status::COUNT }>::new_fill_with_copy(42);
    ///
    /// assert_eq!(table.get(Status::Active), &42);
    /// assert_eq!(table.get(Status::Inactive), &42);
    /// assert_eq!(table.get(Status::Pending), &42);
    /// ```
    pub const fn new_fill_with_copy(value: V) -> Self {
        Self::new([value; N])
    }
}

impl<K: Enumerable, V: Default, const N: usize> EnumTable<K, V, N> {
    /// Creates a new `EnumTable` with `V::default()` for every variant.
    pub fn new_fill_with_default() -> Self {
        Self::new(core::array::from_fn(|_| V::default()))
    }

    /// Clears the table, setting each value to its default.
    pub fn clear(&mut self) {
        self.table.fill_with(V::default);
    }

    /// Replaces the value associated with `variant` with its default,
    /// and returns the old value.
    pub fn take(&mut self, variant: K) -> V {
        core::mem::take(&mut self.table[variant.variant_index()])
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
    fn new_with_fn() {
        let table =
            EnumTable::<Color, &'static str, { Color::COUNT }>::new_with_fn(|color| match color {
                Color::Red => "Red",
                Color::Green => "Green",
                Color::Blue => "Blue",
            });

        assert_eq!(table.get(Color::Red), &"Red");
        assert_eq!(table.get(Color::Green), &"Green");
        assert_eq!(table.get(Color::Blue), &"Blue");
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

        assert_eq!(table.get(Color::Red), &"Red");
        assert_eq!(table.get(Color::Green), &"Green");
        assert_eq!(table.get(Color::Blue), &"Blue");

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

        assert_eq!(table.get(Color::Red), &"Red");
        assert_eq!(table.get(Color::Green), &"Green");
        assert_eq!(table.get(Color::Blue), &"Blue");

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
        assert_eq!(TABLES.get(Color::Red), &"Red");
        assert_eq!(TABLES.get(Color::Green), &"Green");
        assert_eq!(TABLES.get(Color::Blue), &"Blue");
    }

    #[test]
    fn get_mut() {
        let mut table = TABLES;
        assert_eq!(table.get_mut(Color::Red), &mut "Red");
        assert_eq!(table.get_mut(Color::Green), &mut "Green");
        assert_eq!(table.get_mut(Color::Blue), &mut "Blue");

        *table.get_mut(Color::Red) = "Changed Red";
        *table.get_mut(Color::Green) = "Changed Green";
        *table.get_mut(Color::Blue) = "Changed Blue";

        assert_eq!(table.get(Color::Red), &"Changed Red");
        assert_eq!(table.get(Color::Green), &"Changed Green");
        assert_eq!(table.get(Color::Blue), &"Changed Blue");
    }

    #[test]
    fn set() {
        let mut table = TABLES;
        assert_eq!(table.set(Color::Red, "New Red"), "Red");
        assert_eq!(table.set(Color::Green, "New Green"), "Green");
        assert_eq!(table.set(Color::Blue, "New Blue"), "Blue");

        assert_eq!(table.get(Color::Red), &"New Red");
        assert_eq!(table.get(Color::Green), &"New Green");
        assert_eq!(table.get(Color::Blue), &"New Blue");
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

        let mapped = table.map(|key, value| match key {
            Color::Red => value + 10,   // 1 + 10 = 11
            Color::Green => value + 20, // 2 + 20 = 22
            Color::Blue => value + 30,  // 3 + 30 = 33
        });

        assert_eq!(mapped.get(Color::Red), &11);
        assert_eq!(mapped.get(Color::Green), &22);
        assert_eq!(mapped.get(Color::Blue), &33);
    }

    #[test]
    fn map_mut() {
        let mut table =
            EnumTable::<Color, i32, { Color::COUNT }>::new_with_fn(|color| match color {
                Color::Red => 10,
                Color::Green => 20,
                Color::Blue => 30,
            });

        table.map_mut(|key, value| {
            *value += match key {
                Color::Red => 1,
                Color::Green => 2,
                Color::Blue => 3,
            }
        });

        assert_eq!(table.get(Color::Red), &11);
        assert_eq!(table.get(Color::Green), &22);
        assert_eq!(table.get(Color::Blue), &33);
    }

    macro_rules! run_variants_test {
        ($($variant:ident),+) => {{
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Enumerable)]
            #[repr(u8)]
            enum Test {
                $($variant,)*
            }

            let map = EnumTable::<Test, &'static str, { Test::COUNT }>::new_with_fn(|t| match t {
                $(Test::$variant => stringify!($variant),)*
            });
            $(
                assert_eq!(map.get(Test::$variant), &stringify!($variant));
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

    #[test]
    fn variant_index() {
        // Color discriminants: Green=11, Red=33, Blue=222
        // Sorted order: Green(0), Red(1), Blue(2)
        assert_eq!(Color::Green.variant_index(), 0);
        assert_eq!(Color::Red.variant_index(), 1);
        assert_eq!(Color::Blue.variant_index(), 2);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Enumerable)]
    #[repr(i8)]
    enum Signed {
        Neg = -1,
        Zero = 0,
        Pos = 1,
    }

    #[test]
    fn signed_repr_end_to_end() {
        // Unsigned bit-pattern order: Zero(0x00), Pos(0x01), Neg(0xFF)
        assert_eq!(Signed::VARIANTS, &[Signed::Zero, Signed::Pos, Signed::Neg]);
        assert_eq!(Signed::Zero.variant_index(), 0);
        assert_eq!(Signed::Pos.variant_index(), 1);
        assert_eq!(Signed::Neg.variant_index(), 2);

        let table =
            EnumTable::<Signed, &'static str, { Signed::COUNT }>::new_with_fn(|s| match s {
                Signed::Neg => "neg",
                Signed::Zero => "zero",
                Signed::Pos => "pos",
            });

        assert_eq!(table.get(Signed::Neg), &"neg");
        assert_eq!(table.get(Signed::Zero), &"zero");
        assert_eq!(table.get(Signed::Pos), &"pos");
    }

    #[test]
    fn get_const() {
        const RED: &str = TABLES.get_const(Color::Red);
        const GREEN: &str = TABLES.get_const(Color::Green);
        const BLUE: &str = TABLES.get_const(Color::Blue);

        assert_eq!(RED, "Red");
        assert_eq!(GREEN, "Green");
        assert_eq!(BLUE, "Blue");
    }

    #[test]
    fn set_const() {
        const fn make_table() -> EnumTable<Color, &'static str, { Color::COUNT }> {
            let mut table = TABLES;
            table.set_const(Color::Red, "New Red");
            table
        }
        const TABLE: EnumTable<Color, &'static str, { Color::COUNT }> = make_table();
        assert_eq!(TABLE.get_const(Color::Red), &"New Red");
        assert_eq!(TABLE.get_const(Color::Green), &"Green");
    }

    #[test]
    fn get_mut_const() {
        const fn make_table() -> EnumTable<Color, &'static str, { Color::COUNT }> {
            let mut table = TABLES;
            *table.get_mut_const(Color::Green) = "Changed Green";
            table
        }
        const TABLE: EnumTable<Color, &'static str, { Color::COUNT }> = make_table();
        assert_eq!(TABLE.get_const(Color::Green), &"Changed Green");
    }

    #[test]
    fn take_option() {
        let mut table =
            EnumTable::<Color, Option<i32>, { Color::COUNT }>::new_with_fn(|color| match color {
                Color::Red => Some(1),
                Color::Green => Some(2),
                Color::Blue => None,
            });

        assert_eq!(table.take(Color::Red), Some(1));
        assert_eq!(table.get(Color::Red), &None);

        assert_eq!(table.take(Color::Blue), None);
        assert_eq!(table.get(Color::Blue), &None);
    }

    #[test]
    fn take_default() {
        let mut table =
            EnumTable::<Color, i32, { Color::COUNT }>::new_with_fn(|color| match color {
                Color::Red => 1,
                Color::Green => 2,
                Color::Blue => 3,
            });

        assert_eq!(table.take(Color::Red), 1);
        assert_eq!(table.get(Color::Red), &0);
        assert_eq!(table.get(Color::Green), &2);
    }

    #[test]
    fn clear_option() {
        let mut table =
            EnumTable::<Color, Option<i32>, { Color::COUNT }>::new_with_fn(|color| match color {
                Color::Red => Some(1),
                Color::Green => Some(2),
                Color::Blue => Some(3),
            });

        table.clear();

        assert_eq!(table.get(Color::Red), &None);
        assert_eq!(table.get(Color::Green), &None);
        assert_eq!(table.get(Color::Blue), &None);
    }

    #[test]
    fn zip() {
        let a = EnumTable::<Color, i32, { Color::COUNT }>::new_with_fn(|c| match c {
            Color::Red => -10,
            Color::Green => -20,
            Color::Blue => -30,
        });
        let b = EnumTable::<Color, u32, { Color::COUNT }>::new_with_fn(|c| match c {
            Color::Red => 1,
            Color::Green => 2,
            Color::Blue => 3,
        });

        let sum = a.zip(b, |key, x, y| match key {
            Color::Blue => x + y as i32 - 100, // distinguish Blue via the key
            _ => x + y as i32,
        });
        assert_eq!(sum.get(Color::Red), &-9);
        assert_eq!(sum.get(Color::Green), &-18);
        assert_eq!(sum.get(Color::Blue), &-127);
    }

    #[test]
    fn clear_default() {
        let mut table =
            EnumTable::<Color, i32, { Color::COUNT }>::new_with_fn(|color| match color {
                Color::Red => 1,
                Color::Green => 2,
                Color::Blue => 3,
            });

        table.clear();

        assert_eq!(table.get(Color::Red), &0);
        assert_eq!(table.get(Color::Green), &0);
        assert_eq!(table.get(Color::Blue), &0);
    }
}
