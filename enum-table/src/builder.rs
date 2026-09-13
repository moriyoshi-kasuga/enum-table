use core::marker::PhantomData;
use core::mem::MaybeUninit;

use crate::{EnumTable, Enumerable};

/// Accumulates values for an `EnumTable`, one per variant, before all `N` slots are initialized.
///
/// Has no `Drop` impl, so a builder dropped before `N` values are pushed leaks the values already written.
pub struct EnumTableBuilder<K: Enumerable, V, const N: usize> {
    table: MaybeUninit<[V; N]>,
    _phantom: PhantomData<K>,
}

impl<K: Enumerable, V, const N: usize> EnumTableBuilder<K, V, N> {
    pub const fn new_uninit() -> Self {
        const { EnumTable::<K, V, N>::assert() };

        Self {
            table: MaybeUninit::uninit(),
            _phantom: PhantomData,
        }
    }

    /// # Safety
    ///
    /// Must not be called more than `N` times on the same builder.
    pub const unsafe fn push_unchecked(&mut self, idx: usize, value: V) {
        unsafe {
            self.table.as_mut_ptr().cast::<V>().add(idx).write(value);
        }
    }

    /// # Safety
    ///
    /// [`Self::push_unchecked`] must have already been called exactly `N` times on `self`.
    pub const unsafe fn build_unchecked(self) -> EnumTable<K, V, N> {
        EnumTable::new(unsafe { self.table.assume_init() })
    }
}
