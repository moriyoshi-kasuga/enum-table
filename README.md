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

See [CHANGELOG](./CHANGELOG.md) for version history and recent updates.

## Why use `enum-table`?

`enum-table` provides a specialized, efficient, and safe way to associate data with enum variants.
Its design is centered around a key guarantee that differentiates it from other data structures.

### Core Guarantee: Completeness

The core design principle of `EnumTable` is that an instance is 
**guaranteed to hold a value for every variant** of its enum key.
This type-level invariant enables a cleaner and more efficient API.

For example, the [`get()`] method returns `&V` directly. This is in contrast to `HashMap::get`,
which must return an `Option<&V>` because a key may or may not be present.
With `EnumTable`, the presence of all keys is guaranteed,
eliminating the need for `unwrap()` or other `Option` handling in your code.

If you need to handle cases where a value might not be present,
you can use `Option<V>` as the value type: `EnumTable<K, Option<V>, N>`.
This pattern is fully supported and provides a clear, explicit way to manage optional values.

### Comparison with Alternatives

- **vs. `HashMap<MyEnum, V>`**: Beyond the completeness guarantee,
  `EnumTable` has no heap allocations for its structure, offers better cache locality,
  and can be created in a `const` context for zero-cost initialization.
  `HashMap` is more flexible for dynamic data but comes with runtime overhead.

- **vs. `match` statements**: `EnumTable` decouples data from logic.
  You can pass tables around, modify them at runtime, or load them from configurations.
  A `match` statement hardcodes the mapping and requires re-compilation to change.

- **vs. arrays (`[V; N]`)**: `EnumTable` works seamlessly with enums that have
  non-continuous or specified discriminants (e.g., `enum E { A = 1, B = 100 }`).
  An array-based approach requires manually mapping variants to `0..N` indices,
  which is error-prone and less flexible.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
enum-table = "2.1"
```

*Requires Rust 1.85 or later.*

## The `Enumable` Trait

The core of the library is the `Enumable` trait. It provides the necessary
information about an enum—its variants and their count—to the `EnumTable`.

```rust
pub trait Enumable: Copy + 'static {
    const VARIANTS: &'static [Self];
    const COUNT: usize = Self::VARIANTS.len();
}
```

A critical requirement for this trait is that the `VARIANTS` array **must be sorted** by the enum's discriminant values.
This ordering is essential for the table's binary search logic to function correctly.

Because manually ensuring this order is tedious and error-prone,
**it is strongly recommended to use the derive macro `#[derive(Enumable)]`**.
The derive macro automatically generates a correct, sorted `VARIANTS` array, guaranteeing proper behavior.

It is also recommended (though optional) to use a `#[repr(u*)]` attribute on your enum.
This ensures the size and alignment of the enum's discriminant are stable and well-defined.

## Usage Examples

### Basic Usage

```rust
use enum_table::{EnumTable, Enumable};

#[derive(Enumable, Copy, Clone)] // Automatically implements the Enumable trait
#[repr(u8)] // Recommended: specifies the discriminant size
enum Test {
    A = 100, // You can specify custom discriminants
    B = 1,
    C,       // Will be 2 (previous value + 1)
}

// Runtime table creation
let mut table = EnumTable::<Test, &'static str, { Test::COUNT }>::new_with_fn(
  |t| match t {
    Test::A => "A",
    Test::B => "B",
    Test::C => "C",
});

assert_eq!(table.get(&Test::A), &"A");

let old_b = table.set(&Test::B, "Changed B");
assert_eq!(old_b, "B");
assert_eq!(table.get(&Test::B), &"Changed B");
```

### `const` Context and `et!` macro

You can create `EnumTable` instances at compile time with zero runtime overhead using the `et!` macro.
This is ideal for static lookup tables.

```rust
# use enum_table::{EnumTable, Enumable};
# #[derive(Enumable, Copy, Clone)] #[repr(u8)] enum Test { A = 100, B = 1, C }
// This table is built at compile time and baked into the binary.
static TABLE: EnumTable<Test, &'static str, { Test::COUNT }> =
  enum_table::et!(Test, &'static str, |t| match t {
      Test::A => "A",
      Test::B => "B",
      Test::C => "C",
  });

// Accessing the value is highly efficient as the table is pre-built.
const A_VAL: &str = TABLE.get(&Test::A);
assert_eq!(A_VAL, "A");
```

### Serde Support

Enable serde support by adding the `serde` feature:

```toml
[dependencies]
enum-table = { version = "2.1", features = ["serde"] }
serde_json = "1.0"
```

```rust
# #[cfg(feature = "serde")]
# {
use enum_table::{EnumTable, Enumable};
use serde::{Serialize, Deserialize};

#[derive(Debug, Enumable, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

// Serialize to JSON
let json = serde_json::to_string(&table).unwrap();
assert_eq!(json, r#"{"Active":"running","Inactive":"stopped","Pending":"waiting"}"#);

// Deserialize from JSON
let deserialized: EnumTable<Status, &str, { Status::COUNT }> =
    serde_json::from_str(&json).unwrap();

assert_eq!(table, deserialized);
# }
```

### Error Handling and Alternative Constructors

The library provides several ways to create an `EnumTable`,
some of which include built-in error handling for fallible initialization logic.

The example below shows `try_new_with_fn`,
which is useful when each value is generated individually and might fail.

```rust
use enum_table::{EnumTable, Enumable};

#[derive(Enumable, Copy, Clone, Debug, PartialEq)]
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

For other construction methods, such as creating a table from existing data structures,
please see the **API Overview** section below and the full [API documentation](https://docs.rs/enum-table/latest/enum_table/struct.EnumTable.html).
For instance, you can use `try_from_vec()` or `try_from_hash_map()` from the **Conversions** API,
which also handle potential errors like missing variants.

## API Overview

### Key Methods

- `EnumTable::new_with_fn()`: Create a table by mapping each enum variant to a value.
- `EnumTable::try_new_with_fn()`: Create a table with error handling support.
- `EnumTable::checked_new_with_fn()`: Create a table with optional values.
- `EnumTable::get()`: Access the value for a specific enum variant.
- `EnumTable::get_mut()`: Get mutable access to a value.
- `EnumTable::set()`: Update a value and return the old one.

### Transformation

- `map()`: Transforms all values in the table.
- `map_mut()`: Transforms all values in the table in-place.
- `map_with_key()`: Transforms values using both the key and value.
- `map_mut_with_key()`: Transforms values in-place using both the key and value.

### Iterators

- `iter()`, `iter_mut()`: Iterate over key-value pairs.
- `keys()`: Iterate over keys.
- `values()`, `values_mut()`: Iterate over values.

### Conversions

The `EnumTable` can be converted to and from other standard collections.

#### From `EnumTable`

- `into_vec()`: Converts the table into a `Vec<(K, V)>`.
- `into_hash_map()`: Converts the table into a `HashMap<K, V>`.
  Requires the enum key to implement `Eq + Hash`.
- `into_btree_map()`: Converts the table into a `BTreeMap<K, V>`.
  Requires the enum key to implement `Ord`.

```rust
# use enum_table::{EnumTable, Enumable};
# #[derive(Enumable, Debug, PartialEq, Eq, Hash, Copy, Clone)] enum Color { Red, Green, Blue }
# let table = EnumTable::<Color, &'static str, 3>::new_with_fn(|c| match c {
#     Color::Red => "red", Color::Green => "green", Color::Blue => "blue",
# });
// Example: Convert to a Vec
let vec = table.into_vec();
assert_eq!(vec.len(), 3);
assert!(vec.contains(&(Color::Red, "red")));
```

#### To `EnumTable`

- `try_from_vec()`: Creates a table from a `Vec<(K, V)>`.
  Returns an error if any variant is missing or duplicated.
- `try_from_hash_map()`: Creates a table from a `HashMap<K, V>`.
  Returns `None` if the map does not contain exactly one entry for each variant.
- `try_from_btree_map()`: Creates a table from a `BTreeMap<K, V>`.
  Returns `None` if the map does not contain exactly one entry for each variant.

```rust
# use enum_table::{EnumTable, Enumable};
# use std::collections::HashMap;
# #[derive(Enumable, Debug, PartialEq, Eq, Hash, Copy, Clone)] enum Color { Red, Green, Blue }
// Example: Create from a HashMap
let mut map = HashMap::new();
map.insert(Color::Red, 1);
map.insert(Color::Green, 2);
map.insert(Color::Blue, 3);

let table = EnumTable::<Color, i32, 3>::try_from_hash_map(map).unwrap();
assert_eq!(table.get(&Color::Green), &2);
```

For complete API documentation, visit [EnumTable on doc.rs](https://docs.rs/enum-table/latest/enum_table/struct.EnumTable.html).

## Performance

The `enum-table` library is designed for performance:

- **Access Time**: O(log N) lookup time via binary search of enum discriminants.
- **Memory Efficiency**: No heap allocations for the table structure, leading to better cache locality.
- **Compile-Time Optimization**: Static tables can be fully constructed at compile time.

## Feature Flags

- **default**: Enables the `derive` feature by default.
- **derive**: Enables the `Enumable` derive macro for automatic trait implementation.
- **serde**: Enables serialization and deserialization support using Serde.

## License

Licensed under the [MIT license](https://github.com/moriyoshi-kasuga/enum-table/blob/main/LICENSE)

## Benchmarks

Invoke the benchmarks using `cargo bench` to compare the performance of `EnumTable`
with a `HashMap` for enum keys. The benchmarks measure the time taken for
creating a table, getting values, and setting values.

<details>
<summary>Benchmark results</summary>

```text
EnumTable::new_with_fn  time:   [257.10 ps 258.88 ps 260.82 ps]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  4 (4.00%) high severe

EnumTable::get          time:   [275.33 ps 293.95 ps 316.78 ps]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe

HashMap::get            time:   [13.368 ns 13.541 ns 13.765 ns]
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) high mild
  7 (7.00%) high severe

EnumTable::set          time:   [260.57 ps 263.15 ps 267.08 ps]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe

HashMap::insert         time:   [15.664 ns 15.753 ns 15.844 ns]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
```

</details>
