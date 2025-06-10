# [enum-table][docsrs]

[![enum-table on crates.io][cratesio-image]][cratesio]
[![enum-table on docs.rs][docsrs-image]][docsrs]

[cratesio-image]: https://img.shields.io/crates/v/enum-table.svg
[cratesio]: https://crates.io/crates/enum-table
[docsrs-image]: https://docs.rs/enum-table/badge.svg
[docsrs]: https://docs.rs/enum-table

**enum-table** is a lightweight and efficient Rust library for mapping enums to values.  
It provides a fast, type-safe, and allocation-free alternative to using `HashMap` for enum keys,
with compile-time safety and constant-time access.

## Features

- **Type Safety**: Only valid enum variants can be used as keys.
- **Compile-Time Checks**: Leverages Rust's type system for compile-time guarantees.
- **Efficiency**: Constant-time access, no heap allocation.
- **Custom Derive**: Procedural macro to automatically implement the `Enumable` trait for enums.
- **Const Support**: Tables can be constructed at compile time.

## Usage

```rust
#![cfg(feature = "derive")] // Enabled by default

use enum_table::{EnumTable, Enumable};

#[derive(Enumable)] // Recommended for automatic implementation Enumable trait
#[repr(u8)] // Optional: but recommended for specification of discriminants
enum Test {
    A = 100, // Optional: You can specify custom discriminants
    B = 1,
    C,
}

// Implementing the Enumable trait manually
// May forget to add all variants. Use derive macro instead. (This is README example)
// impl Enumable for Test {
//     const VARIANTS: &'static [Self] = &[Self::A, Self::B, Self::C];
// }

fn main() {
    // Compile-time table creation using the et! macro
    static TABLE: EnumTable<Test, &'static str, { Test::COUNT }> = 
      enum_table::et!(Test, &'static str, |t| match t {
          Test::A => "A",
          Test::B => "B",
          Test::C => "C",
      });

    // Accessing values from the compile-time table
    const A: &str = TABLE.get(&Test::A);
    assert_eq!(A, "A");

    // Runtime table creation
    let mut table = EnumTable::<Test, &'static str, { Test::COUNT }>::new_with_fn(
      |t| match t {
        Test::A => "A",
        Test::B => "B",
        Test::C => "C",
    });

    assert_eq!(table.get(&Test::A), &"A");

    // This call does not panic and is not wrapped in Result or Option
    // always return old value, because all enum variants are initialized
    let old_b = table.set(&Test::B, "Changed B");
  
    assert_eq!(old_b, "B");
    assert_eq!(table.get(&Test::B), &"Changed B");
}
```

### More Method

more info: [doc.rs](https://docs.rs/enum-table/latest/enum_table/struct.EnumTable.html)

## License

Licensed under

- [MIT license](https://github.com/moriyoshi-kasuga/enum-table/blob/main/LICENSE)
