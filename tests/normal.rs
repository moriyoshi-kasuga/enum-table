use enum_table::EnumTable;

#[derive(Debug, PartialEq, Eq)]
pub enum Test {
    A,
    B,
    C,
}

#[test]
fn test() {
    let mut table = EnumTable::<Test, &'static str>::new_with_fn(|t| match t {
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
