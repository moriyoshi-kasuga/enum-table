/// Builds an `EnumTable` for `$variant` and `$value` in a `const` context, by matching
/// each variant to a value in the given closure body.
///
/// Use this macro to build a table in `const` context, such as a `static`. Outside
/// `const` context, use one of `EnumTable`'s constructors instead, such as
/// [`crate::EnumTable::new_with_fn`], [`crate::EnumTable::try_new_with_fn`], or
/// [`crate::EnumTable::checked_new_with_fn`].
///
/// # Panics
///
/// If the closure body panics, values already pushed into the internal builder are
/// leaked rather than dropped; see [`crate::builder::EnumTableBuilder`] for why.
///
/// # Examples
///
/// ```rust
/// use enum_table::{EnumTable, Enumerable, et};
///
/// #[derive(Enumerable, Copy, Clone)]
/// enum Test {
///     A,
///     B,
///     C,
/// }
///
/// const TABLE: EnumTable<Test, &'static str, { Test::COUNT }> =
///     et!(Test, &'static str, |t| match t {
///         Test::A => "A",
///         Test::B => "B",
///         Test::C => "C",
///     });
///
/// assert_eq!(TABLE.get(&Test::A), &"A");
/// assert_eq!(TABLE.get(&Test::B), &"B");
/// assert_eq!(TABLE.get(&Test::C), &"C");
/// ```
#[macro_export]
macro_rules! et {
    ($variant:ty, $value:ty, |$variable:ident| $($tt:tt)*) => {
        {
            let mut builder = $crate::builder::EnumTableBuilder::<
                $variant,
                $value,
                { <$variant as $crate::Enumerable>::COUNT },
            >::new();

            let mut i = 0;
            while i < builder.capacity() {
                let $variable = &<$variant as $crate::Enumerable>::VARIANTS[i];
                let value = $($tt)*;
                unsafe {
                    builder.push_unchecked($variable, value);
                }
                i += 1;
            }

            unsafe { builder.build_to_unchecked() }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::{EnumTable, Enumerable};

    #[test]
    fn et_macro() {
        #[derive(Clone, Copy, Enumerable)]
        enum Test {
            A,
            B,
            C,
        }

        const TABLE: EnumTable<Test, &'static str, { Test::COUNT }> =
            et!(Test, &'static str, |t| match t {
                Test::A => "A",
                Test::B => "B",
                Test::C => "C",
            });

        assert_eq!(TABLE.get(&Test::A), &"A");
        assert_eq!(TABLE.get(&Test::B), &"B");
        assert_eq!(TABLE.get(&Test::C), &"C");
    }

    #[test]
    fn et_macro_panic_leaks_pushed_elements() {
        use std::panic::{AssertUnwindSafe, catch_unwind};
        use std::sync::atomic::{AtomicUsize, Ordering};

        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct Droppable;
        impl Drop for Droppable {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }

        #[derive(Clone, Copy, Enumerable)]
        enum Test {
            A,
            B,
            C,
        }

        DROP_COUNT.store(0, Ordering::SeqCst);

        let result = catch_unwind(AssertUnwindSafe(|| {
            let _: EnumTable<Test, Droppable, { Test::COUNT }> =
                et!(Test, Droppable, |t| match t {
                    Test::A => Droppable,
                    Test::B => panic!("boom"),
                    Test::C => Droppable,
                });
        }));

        assert!(result.is_err());
        // `Test::A`'s `Droppable` was already pushed into the builder when the
        // panic on `Test::B` unwound. Per the macro's documented behavior, it
        // is leaked (the builder stores it in a `MaybeUninit`, which never
        // runs `Drop`) rather than dropped.
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 0);
    }
}
