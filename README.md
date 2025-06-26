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
- **Serde Support**: Optional serialization and deserialization support with the `serde` feature.

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

### Conversion Methods

EnumTable provides convenient methods for transforming and converting data:

```rust
use enum_table::{EnumTable, Enumable};

#[derive(Enumable, Debug, PartialEq, Copy, Clone)]
enum Color {
    Red,
    Green,
    Blue,
}

fn main() {
    let table = EnumTable::<Color, i32, { Color::COUNT }>::new_with_fn(|color| match color {
        Color::Red => 1,
        Color::Green => 2,
        Color::Blue => 3,
    });

    // Transform values with map
    let doubled = table.map(|x| x * 2);
    assert_eq!(doubled.get(&Color::Red), &2);

    // Convert to vector
    let vec = doubled.into_vec();
    assert!(vec.contains(&(Color::Red, 2)));

    // Convert back from vector
    let restored = EnumTable::<Color, i32, { Color::COUNT }>::from_vec(vec).unwrap();
    assert_eq!(restored.get(&Color::Red), &2);
}
```

### Serde Support

Enable serde support by adding the `serde` feature:

```toml
[dependencies]
enum-table = { version = "0.4", features = ["serde"] }
serde_json = "1.0"
```

```rust
# #[cfg(all(feature = "serde", feature = "derive"))]
# fn main() {
use enum_table::{EnumTable, Enumable};
use serde::{Serialize, Deserialize};

#[derive(Debug, Enumable, Serialize, Deserialize, PartialEq, Eq, Hash)]
enum Status {
    Active,
    Inactive,
    Pending,
}

let table = EnumTable::<Status, &'static str, { Status::COUNT }>::new_with_fn(|status| match status {
    Status::Active => "running",
    Status::Inactive => "stopped", 
    Status::Pending => "waiting",
});

const JSON_FIXED: &str = r#"{"Active":"running","Inactive":"stopped","Pending":"waiting"}"#;

let json = serde_json::to_string(&table).unwrap();
assert_eq!(json, JSON_FIXED);

let deserialized: EnumTable<Status, &str, { Status::COUNT }> = 
    serde_json::from_str(&json).unwrap();

assert_eq!(table, deserialized);
# }
# #[cfg(not(all(feature = "serde", feature = "derive")))]
# fn main() {}
```

### More Methods

more info: [doc.rs](https://docs.rs/enum-table/latest/enum_table/struct.EnumTable.html)

## License

Licensed under

- [MIT license](https://github.com/moriyoshi-kasuga/enum-table/blob/main/LICENSE)
