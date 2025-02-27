#![doc = include_str!(concat!("../", std::env!("CARGO_PKG_README")))]

#[cfg(feature = "derive")]
pub use enum_table_derive::Enumable;

pub mod builder;
mod impls;
mod macros;

use core::mem::Discriminant;
use dev_macros::*;

/// A trait for enumerations that can be used with `EnumTable`.
///
/// This trait requires that the enumeration provides a static array of its variants
/// and a constant representing the count of these variants.
pub trait Enumable: Sized + 'static {
    const VARIANTS: &'static [Self];
    const COUNT: usize = Self::VARIANTS.len();
}

const fn to_usize<T>(t: T) -> usize {
    #[inline(always)]
    const fn cast<T, U>(t: T) -> U {
        use core::mem::ManuallyDrop;
        unsafe { core::mem::transmute_copy::<ManuallyDrop<T>, U>(&ManuallyDrop::new(t)) }
    }

    match const { core::mem::size_of::<T>() } {
        1 => cast::<T, u8>(t) as usize,
        2 => cast::<T, u16>(t) as usize,
        4 => cast::<T, u32>(t) as usize,
        8 => cast::<T, u64>(t) as usize,
        _ => panic!("Unsupported size"),
    }
}

/// A table that associates each variant of an enumeration with a value.
///
/// `EnumTable` is a generic struct that uses an enumeration as keys and stores
/// associated values. It provides constant-time access to the values based on
/// the enumeration variant.
#[derive(Debug, Clone, Copy)]
pub struct EnumTable<K: Enumable, V, const N: usize> {
    table: [(Discriminant<K>, V); N],
}

impl<K: Enumable, V, const N: usize> EnumTable<K, V, N> {
    /// Creates a new `EnumTable` with the given table of discriminants and values.
    ///
    /// # Arguments
    ///
    /// * `table` - An array of tuples where each tuple contains a discriminant of
    ///   an enumeration variant and its associated value.
    pub const fn new(table: [(Discriminant<K>, V); N]) -> Self {
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
        let table = core::array::from_fn(|i| {
            let k = &K::VARIANTS[i];
            (core::mem::discriminant(k), f(k))
        });

        Self { table }
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

    /// const function is not callable drop.
    /// So, we use forget to avoid calling drop.
    /// Careful, not to call drop on the old value.
    /// Sets the value associated with the given enumeration variant in a constant context.
    ///
    /// # Arguments
    ///
    /// * `variant` - A reference to an enumeration variant.
    /// * `value` - The new value to associate with the variant.
    ///
    /// # Safety
    ///
    /// This method uses `std::mem::forget` to avoid calling `drop` on the old value.
    /// Be careful not to call `drop` on the old value manually.
    pub const fn const_set(&mut self, variant: &K, value: V) {
        use_variant_value!(self, variant, i, {
            let old = core::mem::replace(&mut self.table[i].1, value);
            std::mem::forget(old);
            return;
        });
    }

    /// Sets the value associated with the given enumeration variant.
    ///
    /// # Arguments
    ///
    /// * `variant` - A reference to an enumeration variant.
    /// * `value` - The new value to associate with the variant.
    pub fn set(&mut self, variant: &K, value: V) {
        use_variant_value!(self, variant, i, {
            self.table[i].1 = value;
            return;
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
}

mod dev_macros {
    macro_rules! use_variant_value {
        ($self:ident, $variant:ident, $i:ident,{$($tt:tt)+}) => {
            let discriminant = core::mem::discriminant($variant);

            let mut $i = 0;
            while $i < $self.table.len() {
                if to_usize($self.table[$i].0) == to_usize(discriminant) {
                    $($tt)+
                }
                $i += 1;
            }
            unreachable!();
        };
    }

    pub(super) use use_variant_value;
}
