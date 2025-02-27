#![feature(variant_count)]

// NOTE: I get a compiler error when initializing an array when doing new,
// probably due to an error in reasoning about the length of the generic const expr argument.
// If cured, remove variant_count and use generic const expr
// #![feature(generic_const_exprs)]

use core::mem::Discriminant;

/// # Safety
/// This function is unsafe because it uses [`core::mem::variant_count`]
/// it is nightly only. Use with carefully.
#[inline(always)]
#[must_use]
pub const unsafe fn variant_count<T>() -> usize {
    core::mem::variant_count::<T>()
}

#[inline(always)]
const fn cast<T, U>(t: T) -> U {
    use core::mem::ManuallyDrop;
    unsafe { core::mem::transmute_copy::<ManuallyDrop<T>, U>(&ManuallyDrop::new(t)) }
}

const fn to_cast<T>(i: usize) -> T {
    match core::mem::size_of::<T>() {
        1 => cast(i as u8),
        2 => cast(i as u16),
        4 => cast(i as u32),
        8 => cast(i as u64),
        _ => panic!("Unsupported size"),
    }
}

const fn to_usize<T>(t: T) -> usize {
    match core::mem::size_of::<T>() {
        1 => cast::<T, u8>(t) as usize,
        2 => cast::<T, u16>(t) as usize,
        4 => cast::<T, u32>(t) as usize,
        8 => cast::<T, u64>(t) as usize,
        _ => panic!("Unsupported size"),
    }
}

#[derive(Debug)]
pub struct EnumTable<T, V, const N: usize> {
    table: [(Discriminant<T>, V); N],
}

impl<T, V, const N: usize> EnumTable<T, V, N> {
    pub const fn new(table: [(Discriminant<T>, V); N]) -> Self {
        Self { table }
    }

    pub fn new_with_fn(mut f: impl FnMut(T) -> V) -> Self {
        let table = core::array::from_fn(|i| {
            let t: T = to_cast(i);
            (core::mem::discriminant(&t), f(t))
        });

        Self { table }
    }

    pub const fn get(&self, variant: &T) -> &V {
        macros::use_variant_value!(self, variant, i, {
            return &self.table[i].1;
        });
    }

    pub const fn get_mut(&mut self, variant: &T) -> &mut V {
        macros::use_variant_value!(self, variant, i, {
            return &mut self.table[i].1;
        });
    }

    /// const function is not callable drop.
    /// So, we use forget to avoid calling drop.
    /// Careful, not to call drop on the old value.
    pub const fn const_set(&mut self, variant: &T, value: V) {
        macros::use_variant_value!(self, variant, i, {
            let old = core::mem::replace(&mut self.table[i].1, value);
            std::mem::forget(old);
            return;
        });
    }

    pub fn set(&mut self, variant: &T, value: V) {
        macros::use_variant_value!(self, variant, i, {
            self.table[i].1 = value;
            return;
        });
    }
}

mod macros {
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
            panic!("unreadable Variant not found");
        };
    }

    pub(super) use use_variant_value;
}
