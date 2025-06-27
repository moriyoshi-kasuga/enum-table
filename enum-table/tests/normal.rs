use enum_table::{EnumTable, Enumable};

#[derive(Debug, PartialEq, Eq, Enumable)]
#[repr(u8)]
pub enum Test {
    A = 100,
    B = 1,
    C = 20,
}

#[test]
fn test() {
    assert_eq!(Test::VARIANTS, &[Test::B, Test::C, Test::A]);

    let mut table = EnumTable::<Test, &'static str, { Test::COUNT }>::new_with_fn(|t| match t {
        Test::A => "A",
        Test::B => "B",
        Test::C => "C",
    });

    assert_eq!(table.get(&Test::A), &"A");
    assert_eq!(table.get(&Test::B), &"B");
    assert_eq!(table.get(&Test::C), &"C");
    assert_eq!(table.get_mut(&Test::A), &mut "A");

    *table.get_mut(&Test::A) = "AA";

    assert_eq!(table.get(&Test::A), &"AA");

    table.set(&Test::A, "AAA");

    assert_eq!(table.get(&Test::A), &"AAA");
}
