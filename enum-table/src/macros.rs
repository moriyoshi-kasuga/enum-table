/// Builds an `EnumTable` for `$variant` and `$value` by matching each variant to a
/// value in the given closure body.
///
/// The closure body is always evaluated inside a `const` block, so it may only use
/// expressions that are valid in a `const` context, such as `const fn` calls. If you
/// need to build a table at runtime, using non-`const` operations, use one of
/// `EnumTable`'s constructors instead, such as [`crate::EnumTable::new_with_fn`],
/// [`crate::EnumTable::try_new_with_fn`], or [`crate::EnumTable::checked_new_with_fn`].
///
/// # Panics
///
/// If the closure body panics, evaluation fails at compile time, so the crate using
/// this macro fails to compile rather than panicking at runtime.
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
        const {
            let mut builder = $crate::__private::EnumTableBuilder::<
                $variant,
                $value,
                { <$variant as $crate::Enumerable>::COUNT },
            >::new();

            let mut i = 0;
            while i < <$variant as $crate::Enumerable>::COUNT  {
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
    fn et_macro_in_fn_body() {
        #[derive(Clone, Copy, Enumerable)]
        enum Test {
            A,
            B,
            C,
        }

        let table: EnumTable<Test, &'static str, { Test::COUNT }> =
            et!(Test, &'static str, |t| match t {
                Test::A => "A",
                Test::B => "B",
                Test::C => "C",
            });

        assert_eq!(table.get(&Test::A), &"A");
        assert_eq!(table.get(&Test::B), &"B");
        assert_eq!(table.get(&Test::C), &"C");
    }
}
