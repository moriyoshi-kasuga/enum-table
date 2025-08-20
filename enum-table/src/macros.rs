/// A macro to create an `EnumTable` for a given enumeration and value type.
///
/// # Arguments
///
/// * `$variant` - The enumeration type that implements the `Enumable` trait.
/// * `$value` - The type of values to be associated with each enumeration variant.
/// * `$count` - The number of variants in the enumeration.
/// * `$variable` - The variable name to use in the closure for each variant.
/// * `$($tt:tt)*` - The closure that maps each variant to a value.
///
/// # Example
///
/// ```rust
/// use enum_table::{EnumTable, Enumable, et};
///
/// #[derive(Copy, Clone)]
/// enum Test {
///     A,
///     B,
///     C,
/// }
///
/// impl enum_table::Enumable for Test {
///     const VARIANTS: &'static [Self] = &[Test::A, Test::B, Test::C];
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
///
#[macro_export]
macro_rules! et {
    ($variant:ty, $value:ty, |$variable:ident| $($tt:tt)*) => {
        {
            let mut builder = $crate::builder::EnumTableBuilder::<$variant, $value, { <$variant as $crate::Enumable>::COUNT }>::new();

            let mut i = 0;
            while i < builder.capacity() {
                let $variable = &<$variant as $crate::Enumable>::VARIANTS[i];
                let value = $($tt)*;
                let t = $variable;
                unsafe {
                    builder.push_unchecked(
                        t,
                        value,
                    );
                }
                i += 1;
            }

            unsafe { builder.build_to_unchecked() }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::{EnumTable, Enumable};

    #[test]
    fn et_macro() {
        #[derive(Clone, Copy, Enumable)]
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
