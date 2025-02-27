#[macro_export]
macro_rules! et {
    ($variant:ty, $value:ty, |$variable:ident| $($tt:tt)*) => {
        {
            let mut builder = $crate::builder::EnumTableBuilder::<$variant, $value>::new();

            let mut i = 0;
            while i < builder.len() {
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
        enum Test {
            A,
            B,
            C,
        }

        const TABLE: EnumTable<Test, &'static str> = et!(Test, &'static str, |t| match t {
            Test::A => "A",
            Test::B => "B",
            Test::C => "C",
        });

        assert_eq!(TABLE.get(&Test::A), &"A");
        assert_eq!(TABLE.get(&Test::B), &"B");
        assert_eq!(TABLE.get(&Test::C), &"C");
    }
}
