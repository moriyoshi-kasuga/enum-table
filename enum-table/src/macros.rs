/// Builds an `EnumTable` for `$variant` and `$value` in a `const` context, by matching
/// each variant to a value in the given closure body.
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
}
