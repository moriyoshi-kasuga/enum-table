# [enum-table][docsrs]: enum-table

[![enum-table on crates.io][cratesio-image]][cratesio]
[![enum-table on docs.rs][docsrs-image]][docsrs]

[cratesio-image]: https://img.shields.io/crates/v/enum-table.svg
[cratesio]: https://crates.io/crates/enum-table
[docsrs-image]: https://docs.rs/enum-table/badge.svg
[docsrs]: https://docs.rs/enum-table

`enum-table` is a lightweight and efficient Rust library designed for mapping enums to values. It provides a fast and type-safe alternative to using `HashMap` for enum keys, ensuring compile-time safety and performance benefits.

## Features

- **Type Safety**: Ensures that only valid enum variants are used as keys.
- **Compile-Time Checks**: Leverages Rust's type system to provide compile-time guarantees.
- **Efficiency**: Offers constant-time access to values associated with enum variants.
- **Custom Derive**: Includes a procedural macro to automatically implement the `Enumable` trait for enums.

## Usage

### Defining an Enum Table

To use the `enum-table` library, you first define an enum and derive the `Enumable` trait. This trait is necessary for the enum to be used with the `EnumTable`.

```rust
#![cfg(feature = "derive")]

use enum_table::{Enumable};

#[derive(Enumable)]
enum Test {
    A,
    B,
    C,
}
```

### Creating an Enum Table

You can create an `EnumTable` using the `new_with_fn` method or the `et!` macro.

1. **Using `new_with_fn` Method**:
   - This method allows you to create an `EnumTable` by providing a function that maps each enum variant to a value.
   - Example:

     ```rust
     let mut table = enum_table::EnumTable::<Test, &'static str, { Test::COUNT }>::new_with_fn(|t| match t {
         Test::A => "A",
         Test::B => "B",
         Test::C => "C",
     });
     ```

2. **Using `et!` Macro**:
   - The `et!` macro is used to create a constant `EnumTable`.
   - Example:

     ```rust
     const TABLE: enum_table::EnumTable<Test, &'static str, { Test::COUNT }> = enum_table::et!(Test, &'static str, Test::COUNT, |t| match t {
         Test::A => "A",
         Test::B => "B",
         Test::C => "C",
     });
     ```

### Accessing Values

Once the `EnumTable` is created, you can access the values associated with enum variants using the `get` method.

```rust
assert_eq!(TABLE.get(&Test::A), &"A");
assert_eq!(TABLE.get(&Test::B), &"B");
assert_eq!(TABLE.get(&Test::C), &"C");
```

### More Method

more info: [doc.rs](https://docs.rs/enum-table/struct.EnumTable.html)

## License

Licensed under

- [MIT license](https://github.com/moriyoshi-kasuga/enum-table/blob/main/LICENSE)
