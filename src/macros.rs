#[macro_export]
macro_rules! et {
    ($variant:ty, $value:ty, $count:expr, |$variable:ident| $($tt:tt)*) => {
        {
            let mut builder = $crate::builder::EnumTableBuilder::<$variant, $value, { $count }>::new();

            let mut i = 0;
            while i < builder.len() {
                let $variable = &<$variant as $crate::Enumable>::VARIANTS[i];
                builder.push(
                    $variable,
                    $($tt)*
                );
                i += 1;
            }

            builder.build_to()
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::{EnumTable, Enumable};

    #[test]
    fn et_macro() {
        enum Test {
            A,
            B,
            C,
        }

        impl Enumable for Test {
            const VARIANTS: &'static [Self] = &[Test::A, Test::B, Test::C];
        }

        const TABLE: EnumTable<Test, &'static str, { Test::COUNT }> =
            et!(Test, &'static str, Test::COUNT, |t| match t {
                Test::A => "A",
                Test::B => "B",
                Test::C => "C",
            });

        assert_eq!(TABLE.get(&Test::A), &"A");
        assert_eq!(TABLE.get(&Test::B), &"B");
        assert_eq!(TABLE.get(&Test::C), &"C");
    }
}
