use core::marker::PhantomData;
use core::mem::MaybeUninit;

use crate::{EnumTable, Enumerable};

/// The mechanism [`crate::et`] uses to construct an `EnumTable` at compile time by
/// pushing one value per variant.
///
/// `pub` only because `et!`'s expansion must reference this type from the
/// caller's crate; `et!` covers every construction need this type exists for,
/// so there is no supported way to use it directly.
///
/// Dropping a builder before it is fully pushed leaks its already-pushed
/// elements instead of running their destructors: `build_unchecked` and
/// `build_to_unchecked` are `const fn`, and a `const fn` cannot run a `Drop`
/// implementation, so `EnumTableBuilder` cannot implement `Drop` without
/// losing const-context support.
pub struct EnumTableBuilder<K: Enumerable, V, const N: usize> {
    idx: usize,
    table: MaybeUninit<[V; N]>,
    #[cfg(debug_assertions)]
    last_key: Option<K>,
    _phantom: PhantomData<K>,
}

impl<K: Enumerable, V, const N: usize> EnumTableBuilder<K, V, N> {
    /// Creates a new, empty `EnumTableBuilder`.
    pub const fn new() -> Self {
        Self {
            idx: 0,
            table: MaybeUninit::uninit(),
            #[cfg(debug_assertions)]
            last_key: None,
            _phantom: PhantomData,
        }
    }

    /// Pushes `value` for `variant` without checking order, duplicates, or capacity.
    ///
    /// # Safety
    ///
    /// The caller must ensure that elements are pushed in ascending order by
    /// the unsigned bit-pattern of each variant's in-memory representation
    /// (see [`Enumerable`]'s safety contract for how signed discriminants
    /// sort), that no variant is pushed more than once, and that the builder
    /// does not exceed capacity `N`.
    pub const unsafe fn push_unchecked(&mut self, _variant: &K, value: V) {
        debug_assert!(self.idx < N, "EnumTableBuilder: too many elements pushed");

        #[cfg(debug_assertions)]
        {
            if let Some(last) = self.last_key {
                assert!(
                    crate::intrinsics::is_sorted(&[last, *_variant]),
                    "EnumTableBuilder: elements are not pushed in ascending order. Ensure that the elements are pushed in the correct order."
                );
            }
            self.last_key = Some(*_variant);
        }

        unsafe {
            self.table
                .as_mut_ptr()
                .cast::<V>()
                .add(self.idx)
                .write(value);
        }

        self.idx += 1;
    }

    /// Builds the array of values from the pushed elements.
    ///
    /// # Safety
    ///
    /// The caller must ensure that all `N` variants have been pushed to the
    /// builder; otherwise the resulting array contains uninitialized memory.
    pub const unsafe fn build_unchecked(self) -> [V; N] {
        #[cfg(debug_assertions)]
        assert!(
            self.idx == N,
            "EnumTableBuilder: not all elements have been pushed"
        );

        // SAFETY: caller guarantees all N variants were pushed.
        unsafe { self.table.assume_init() }
    }

    /// Builds the `EnumTable` from the pushed elements.
    ///
    /// # Safety
    ///
    /// The caller must ensure that all `N` variants have been pushed to the builder.
    pub const unsafe fn build_to_unchecked(self) -> EnumTable<K, V, N> {
        EnumTable::new(unsafe { self.build_unchecked() })
    }
}

impl<K: Enumerable, V, const N: usize> Default for EnumTableBuilder<K, V, N> {
    fn default() -> Self {
        Self::new()
    }
}
