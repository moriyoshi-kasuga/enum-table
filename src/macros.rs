#[macro_export]
macro_rules! et {
    ($variant:ty, $value:ty, $count:expr, |$variable:ident| $($tt:tt)*) => {
        {
            let mut builder = $crate::builder::EnumTableBuilder::<$variant, $value, { $count }>::new();

            let mut i = 0;
            while i < $count {
                let $variable = builder.to_cast(i);
                builder.push(
                    &$variable,
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
    use crate::EnumTable;

    #[test]
    fn et_macro() {
        #[repr(u8)]
        enum Test {
            A,
            B,
            C,
        }

        impl Test {
            const COUNT: usize = unsafe { crate::variant_count::<Test>() };
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
