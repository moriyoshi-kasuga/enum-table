/// Builds an `EnumTable` for `$variant` and `$value` inside a `const` block.
///
/// The closure body must be valid in a `const` context; passing a non-`const`
/// expression is a compile error. For a runtime equivalent, use
/// [`crate::EnumTable::new_with_fn`], [`crate::EnumTable::try_new_with_fn`], or
/// [`crate::EnumTable::checked_new_with_fn`] instead.
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
/// assert_eq!(TABLE.get(Test::A), &"A");
/// assert_eq!(TABLE.get(Test::B), &"B");
/// assert_eq!(TABLE.get(Test::C), &"C");
/// ```
#[macro_export]
macro_rules! et {
    ($variant:ty, $value:ty, |$variable:ident| $($tt:tt)*) => {
        const {
            let mut builder = $crate::__private::EnumTableBuilder::<
                $variant,
                $value,
                { <$variant as $crate::Enumerable>::COUNT },
            >::new_uninit();

            let mut i = 0;
            while i < <$variant as $crate::Enumerable>::COUNT  {
                let $variable = &<$variant as $crate::Enumerable>::VARIANTS[i];
                let value = $($tt)*;
                unsafe {
                    builder.push_unchecked(i, value);
                }
                i += 1;
            }

            unsafe { builder.build_unchecked() }
        }
    };
}
