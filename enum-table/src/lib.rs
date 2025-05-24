#![doc = include_str!(concat!("../", std::env!("CARGO_PKG_README")))]

#[cfg(feature = "derive")]
pub use enum_table_derive::Enumable;

pub mod builder;
mod impls;
mod macros;

use dev_macros::*;

/// A trait for enumerations that can be used with `EnumTable`.
///
/// This trait requires that the enumeration provides a static array of its variants
/// and a constant representing the count of these variants.
pub trait Enumable: Sized + 'static {
    const VARIANTS: &'static [Self];
    const COUNT: usize = Self::VARIANTS.len();
}

const fn to_usize<T: Copy>(t: T) -> usize {
    #[inline(always)]
    const fn cast<U>(t: &impl Sized) -> &U {
        unsafe { std::mem::transmute(t) }
    }

    let t = &t;

    match const { core::mem::size_of::<T>() } {
        1 => *cast::<u8>(t) as usize,
        2 => *cast::<u16>(t) as usize,
        4 => *cast::<u32>(t) as usize,
        #[cfg(target_pointer_width = "64")]
        8 => *cast::<u64>(t) as usize,
        #[cfg(target_pointer_width = "32")]
        8 => panic!("Unsupported size: 64-bit value found on a 32-bit architecture"),
        _ => panic!("Values larger than u64 are not supported"),
    }
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
        let table = core::array::from_fn(|i| {
            let k = &K::VARIANTS[i];
            (to_usize(core::mem::discriminant(k)), f(k))
        });

        Self {
            table,
            _phantom: core::marker::PhantomData,
        }
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

    /// Returns the reference to the value associated with the given discriminant.
    ///
    /// # Arguments
    ///
    /// * `discriminant` - The discriminant of the enumeration variant.
    ///
    /// # Returns
    /// An `Option` containing a reference to the value associated with the discriminant,
    /// or `None` if the discriminant is not found in the table.
    pub const fn get_by_discriminant(&self, discriminant: usize) -> Option<&V> {
        let mut i = 0;
        while i < self.table.len() {
            if self.table[i].0 == discriminant {
                return Some(&self.table[i].1);
            }
            i += 1;
        }
        None
    }

    /// Returns a mutable reference to the value associated with the given discriminant.
    ///
    /// # Arguments
    ///
    /// * `discriminant` - The discriminant of the enumeration variant.
    ///
    /// # Returns
    ///
    /// An `Option` containing a mutable reference to the value associated with the discriminant,
    /// or `None` if the discriminant is not found in the table.
    pub const fn get_mut_by_discriminant(&mut self, discriminant: usize) -> Option<&mut V> {
        let mut i = 0;
        while i < self.table.len() {
            if self.table[i].0 == discriminant {
                return Some(&mut self.table[i].1);
            }
            i += 1;
        }
        None
    }

    /// Sets the value associated with the given discriminant.
    ///
    /// # Arguments
    ///
    /// * `discriminant` - The discriminant of the enumeration variant.
    /// * `value` - The new value to associate with the discriminant.
    ///
    /// # Returns
    ///
    /// Returns `Ok` with the old value associated with the discriminant if it exists,
    /// or `Err` with the provided value if the discriminant is not found in the table.
    /// (Returns the provided value in `Err` because this function is `const` and dropping
    /// values is not allowed in a const context.)
    pub const fn set_by_discriminant(&mut self, discriminant: usize, value: V) -> Result<V, V> {
        let mut i = 0;
        while i < self.table.len() {
            if self.table[i].0 == discriminant {
                return Ok(core::mem::replace(&mut self.table[i].1, value));
            }
            i += 1;
        }
        Err(value)
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.table
            .iter()
            .map(|(discriminant, _)| unsafe { core::mem::transmute::<usize, &K>(*discriminant) })
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.table.iter().map(|(_, value)| value)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.table.iter().map(|(discriminant, value)| {
            (
                unsafe { core::mem::transmute::<usize, &K>(*discriminant) },
                value,
            )
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.table.iter_mut().map(|(discriminant, value)| {
            (
                unsafe { core::mem::transmute::<usize, &K>(*discriminant) },
                value,
            )
        })
    }

    pub fn iter_by_discriminant(&self) -> impl Iterator<Item = (usize, &V)> {
        self.table
            .iter()
            .map(|(discriminant, value)| (*discriminant, value))
    }
}

mod dev_macros {
    macro_rules! use_variant_value {
        ($self:ident, $variant:ident, $i:ident,{$($tt:tt)+}) => {
            let discriminant = to_usize(core::mem::discriminant($variant));

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
