# enum-table

[![enum-table on crates.io][cratesio-image]][cratesio]
[![enum-table on docs.rs][docsrs-image]][docsrs]

[cratesio-image]: https://img.shields.io/crates/v/enum-table.svg
[cratesio]: https://crates.io/crates/enum-table
[docsrs-image]: https://docs.rs/enum-table/badge.svg
[docsrs]: https://docs.rs/enum-table

**enum-table** is a lightweight and efficient Rust library for mapping enums to values.  
It provides a fast, type-safe, and allocation-free alternative to using `HashMap` for enum keys,
with compile-time safety and logarithmic-time access.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
enum-table = "0.4"
```

To enable additional features:

```toml
[dependencies]
enum-table = { version = "0.4", features = ["serde"] }
```

*Requires Rust 1.85 or later.*

## Features at a Glance

- **Type Safety**: Only valid enum variants can be used as keys.
- **Compile-Time Checks**: Leverages Rust's type system for compile-time guarantees.
- **Efficiency**: O(log N) access time via binary search, no heap allocation.
- **Custom Derive**: Procedural macro to automatically implement the `Enumable` trait for enums.
- **Const Support**: Tables can be constructed at compile time.
- **Serde Support**: Optional serialization and deserialization support with the `serde` feature.

## Usage Examples

### Basic Usage

```rust
use enum_table::{EnumTable, Enumable};

#[derive(Enumable)] // Automatically implements Enumable trait
#[repr(u8)] // Optional: but recommended for specification of discriminants
enum Test {
    A = 100, // Optional: You can specify custom discriminants
    B = 1,
    C,       // Will be 2 (previous value + 1)
}

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

    // This call returns the old value as all enum variants are initialized
    let old_b = table.set(&Test::B, "Changed B");
  
    assert_eq!(old_b, "B");
    assert_eq!(table.get(&Test::B), &"Changed B");
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
use enum_table::{EnumTable, Enumable};
use serde::{Serialize, Deserialize};
  
#[derive(Debug, Enumable, Serialize, Deserialize, PartialEq, Eq, Hash)]
enum Status {
    Active,
    Inactive,
    Pending,
}

fn main() {
  let table = EnumTable::<Status, &'static str, { Status::COUNT }>::new_with_fn(|status| match status {
      Status::Active => "running",
      Status::Inactive => "stopped", 
      Status::Pending => "waiting",
  });

  // Serialize to JSON
  let json = serde_json::to_string(&table).unwrap();
  assert_eq!(json, r#"{"Active":"running","Inactive":"stopped","Pending":"waiting"}"#);

  // Deserialize from JSON
  let deserialized: EnumTable<Status, &str, { Status::COUNT }> = 
      serde_json::from_str(&json).unwrap();

  assert_eq!(table, deserialized);
}
```

### Error Handling

The library provides methods for handling potential errors during table creation:

```rust
use enum_table::{EnumTable, Enumable};

#[derive(Enumable, Debug, PartialEq)]
enum Color {
    Red,
    Green,
    Blue,
}

// Using try_new_with_fn for fallible initialization
let result = EnumTable::<Color, &'static str, { Color::COUNT }>::try_new_with_fn(
    |color| match color {
        Color::Red => Ok("Red"),
        Color::Green => Err("Failed to get value for Green"),
        Color::Blue => Ok("Blue"),
    }
);

assert!(result.is_err());
let (variant, error) = result.unwrap_err();
assert_eq!(variant, Color::Green);
assert_eq!(error, "Failed to get value for Green");
```

## API Overview

### Key Methods

- `EnumTable::new_with_fn()`: Create a table by mapping each enum variant to a value
- `EnumTable::try_new_with_fn()`: Create a table with error handling support
- `EnumTable::checked_new_with_fn()`: Create a table with optional values
- `EnumTable::get()`: Access the value for a specific enum variant
- `EnumTable::get_mut()`: Get mutable access to a value
- `EnumTable::set()`: Update a value and return the old one

### Additional Functionality

- `map()`: Transform all values in the table
- `iter()`, `iter_mut()`: Iterate over key-value pairs
- `keys()`, `values()`: Iterate over keys or values separately
- `into_vec()`: Convert the table to a vector of key-value pairs
- `try_from_vec()`: Create a table from a vector of key-value pairs

For complete API documentation, visit [docs.rs/enum-table](https://docs.rs/enum-table/latest/enum_table/struct.EnumTable.html).

## Performance

The `enum-table` library is designed for performance:

- **Access Time**: O(log N) lookup time via binary search of enum discriminants
- **Memory Efficiency**: No heap allocations for the table structure
- **Compile-Time Optimization**: Static tables can be fully constructed at compile time

### Comparison with Alternatives

- Compared to `HashMap<EnumType, V>`: `enum-table` provides compile-time safety, no heap allocations, and potentially better cache locality.
- Compared to `match` statements: `enum-table` offers more flexibility and allows for runtime modification of values.
- Compared to arrays with enum discriminants as indices: `enum-table` works with non-continuous and custom discriminants.

## Feature Flags

- **default**: Enables the `derive` feature by default.
- **derive**: Enables the `Enumable` derive macro for automatic trait implementation.
- **serde**: Enables serialization and deserialization support using Serde.

## License

Licensed under the [MIT license](https://github.com/moriyoshi-kasuga/enum-table/blob/main/LICENSE)
