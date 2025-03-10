# [enum-table][docsrs]: enum-table

[![enum-table on crates.io][cratesio-image]][cratesio]
[![enum-table on docs.rs][docsrs-image]][docsrs]

[cratesio-image]: https://img.shields.io/crates/v/enum-table.svg
[cratesio]: https://crates.io/crates/enum-table
[docsrs-image]: https://docs.rs/enum-table/badge.svg
[docsrs]: https://docs.rs/enum-table

`enum-table` is a lightweight and efficient Rust library designed for mapping enums to values.
It provides a fast and type-safe alternative to using `HashMap` for enum keys,
ensuring compile-time safety and performance benefits.

You can create an EnumTable struct at const time,
ensuring that values are returned reliably with get, and there is no runtime check.

## Features

- **Type Safety**: Ensures that only valid enum variants are used as keys.
- **Compile-Time Checks**: Leverages Rust's type system to provide compile-time guarantees.
- **Efficiency**: Offers constant-time access to values associated with enum variants.
- **Custom Derive**: Includes a procedural macro to automatically implement the `Enumable` trait for enums.

## Usage

Below is a complete example demonstrating how to define an enum, create an `EnumTable`,
and access values. This example is designed to be run as a single doctest.

```rust
#![cfg(feature = "derive")]

use enum_table::{EnumTable, Enumable};

// Define an enum and derive the Enumable trait
// #[derive(Enumable)]
#[repr(u8)]
enum Test {
    // ok with discriminant
    A = 100,
    B = 1,
    C = 34,
}

// Equivalent code not using `#[derive(enum_table::Enumable)]`
impl Enumable for Test {
  const VARIANTS: &[Self] = &[Self::A, Self::B, Self::C];

  // The COUNT constant is automatically implemented by default when VARIANTS is implemented.
  // const COUNT: usize = Self::VARIANTS.len();
}

fn main() {
    // Create an EnumTable using the et! macro at compile time.
    // This allows for constant-time access to values associated with enum variants.
    const TABLE: enum_table::EnumTable<Test, &'static str, { Test::COUNT }> = enum_table::et!(Test, &'static str, |t| match t {
        Test::A => "A",
        Test::B => "B",
        Test::C => "C",
    });

    // Access values using the constant table.
    // The get method retrieves the value associated with the given enum variant.
    const A: &'static str = TABLE.get(&Test::A);
    const B: &'static str = TABLE.get(&Test::B);
    const C: &'static str = TABLE.get(&Test::C);

    assert_eq!(A, "A");
    assert_eq!(B, "B");
    assert_eq!(C, "C");

    // Alternatively, create an EnumTable using the new_with_fn method at runtime.
    // This method allows for dynamic initialization of the table.
    let mut table = enum_table::EnumTable::<Test, &'static str, { Test::COUNT }>::new_with_fn(|t| match t {
        Test::A => "A",
        Test::B => "B",
        Test::C => "C",
    });

    // Access values associated with enum variants using the runtime-initialized table.
    // The get method guarantees that a value is always returned without runtime checks,
    // ensuring safety once the EnumTable struct is created.
    assert_eq!(table.get(&Test::A), &"A");
    assert_eq!(table.get(&Test::B), &"B");
    assert_eq!(table.get(&Test::C), &"C");
}
```

### More Method

more info: [doc.rs](https://docs.rs/enum-table/latest/enum_table/struct.EnumTable.html)

## License

Licensed under

- [MIT license](https://github.com/moriyoshi-kasuga/enum-table/blob/main/LICENSE)
