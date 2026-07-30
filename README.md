# enum-table

[![enum-table on crates.io][cratesio-image]][cratesio]
[![enum-table on docs.rs][docsrs-image]][docsrs]

[cratesio-image]: https://img.shields.io/crates/v/enum-table.svg
[cratesio]: https://crates.io/crates/enum-table
[docsrs-image]: https://docs.rs/enum-table/badge.svg
[docsrs]: https://docs.rs/enum-table

**enum-table** is a lightweight and efficient Rust library for mapping enums to values.
It provides a fast, type-safe, and allocation-free alternative to using `HashMap` for enum keys,
with compile-time safety and constant-time access (O(1)).

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
  Its core has no dependency on `alloc` or `std` at all, so it works in
  `#![no_std]` environments without a global allocator, unlike `HashMap`.
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
enum-table = "3.0"
```

*Requires Rust 1.85 or later.*

## The `Enumable` Trait

The core of the library is the `Enumable` trait. It provides the necessary
information about an enum—its variants—to the `EnumTable`.

```rust,ignore
pub unsafe trait Enumable: Copy + 'static {
    const VARIANTS: &'static [Self];

    // Has a default impl: O(1) when derived, O(log N) fallback otherwise.
    fn variant_index(&self) -> usize { .. }
}
```

`Enumable` is an `unsafe trait`: implementors must guarantee that `VARIANTS`
lists every variant of `Self` exactly once, sorted in ascending order by the
**unsigned bit-pattern** of its in-memory representation, and that `Self` has
no padding bytes. See the [`Enumable` trait documentation][enumable-docs] for
the full safety contract, including how signed discriminants sort.

**It is strongly recommended to use the derive macro `#[derive(Enumable)]`**,
which generates a correct `unsafe impl` for you: a sorted `VARIANTS` array and
an O(1) `variant_index()` using compile-time-computed constants, guaranteeing
both correctness and optimal performance without you writing any `unsafe`
code yourself.

[enumable-docs]: https://docs.rs/enum-table/latest/enum_table/trait.Enumable.html

### Safety and Memory Layout

`#[derive(Enumable)]` only supports field-less (C-like) enums, which never
have padding bytes regardless of `#[repr]`. Using a primitive representation
(e.g., `#[repr(u8)]`) is still recommended for a stable, minimal-size layout,
but it is not required for soundness.

```rust
use enum_table::Enumable;

#[derive(Enumable, Copy, Clone)]
#[repr(u8)] // <--- Recommended, but not required for soundness.
enum MyEnum {
    A,
    B,
}
```

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
let mut table = EnumTable::<Test, &'static str, { Test::VARIANTS.len() }>::new_with_fn(
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
use enum_table::{EnumTable, Enumable};
#[derive(Enumable, Copy, Clone)]
#[repr(u8)] 
enum Test {
    A = 100,
    B = 1,
    C
}

// This table is built at compile time and baked into the binary.
static TABLE: EnumTable<Test, &'static str, { Test::VARIANTS.len() }> =
  enum_table::et!(Test, &'static str, |t| match t {
      Test::A => "A",
      Test::B => "B",
      Test::C => "C",
  });

// Accessing the value is highly efficient as the table is pre-built.
const A_VAL: &str = TABLE.get_const(&Test::A);
assert_eq!(A_VAL, "A");
```

### Serde Support

Enable serde support by adding the `serde` feature:

```toml
[dependencies]
enum-table = { version = "3.0", features = ["serde"] }
serde_json = "1.0"
```

```rust
use enum_table::{EnumTable, Enumable};
use serde::{Serialize, Deserialize};

#[derive(Debug, Enumable, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
enum Status {
    Active,
    Inactive,
    Pending,
}

let table = EnumTable::<Status, &'static str, { Status::VARIANTS.len() }>::new_with_fn(|status| match status {
    Status::Active => "running",
    Status::Inactive => "stopped",
    Status::Pending => "waiting",
});

// Serialize to JSON
let json = serde_json::to_string(&table).unwrap();
assert_eq!(json, r#"{"Active":"running","Inactive":"stopped","Pending":"waiting"}"#);

// Deserialize from JSON
let deserialized: EnumTable<Status, &str, { Status::VARIANTS.len() }> =
    serde_json::from_str(&json).unwrap();

assert_eq!(table, deserialized);
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
let result = EnumTable::<Color, &'static str, { Color::VARIANTS.len() }>::try_new_with_fn(
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
- `EnumTable::default()`: Create a table filled with each variant's `Default` value (requires `V: Default`).
- `EnumTable::get()`: Access the value for a specific enum variant (O(1)).
- `EnumTable::get_mut()`: Get mutable access to a value (O(1)).
- `EnumTable::set()`: Update a value and return the old one (O(1)).
- `EnumTable::as_slice()`: Access the underlying values as a slice.
- `EnumTable::into_array()`: Consume the table and get the underlying array.

### Transformation

- `map()`: Transforms all values in the table.
- `map_mut()`: Transforms all values in the table in-place.
- `map_with_key()`: Transforms values using both the key and value.
- `map_mut_with_key()`: Transforms values in-place using both the key and value.
- `zip()`: Combines two tables element-wise using a function.

### Iterators

- `iter()`, `iter_mut()`: Iterate over key-value pairs.
- `keys()`: Iterate over keys.
- `values()`, `values_mut()`: Iterate over values.
- `into_iter()`: Consume the table and iterate over owned key-value pairs.
- Implements `Extend<(K, V)>` for updating values from an iterator.

### Conversions

The `EnumTable` can be converted to and from other standard collections.

#### From `EnumTable`

- `into_vec()`: Converts the table into a `Vec<(K, V)>`.
- `into_hash_map()`: Converts the table into a `HashMap<K, V>`.
  Requires the enum key to implement `Eq + Hash`.
- `into_btree_map()`: Converts the table into a `BTreeMap<K, V>`.
  Requires the enum key to implement `Ord`.

```rust
use enum_table::{EnumTable, Enumable};
#[derive(Enumable, Debug, PartialEq, Eq, Hash, Copy, Clone)] enum Color { Red, Green, Blue }
let table = EnumTable::<Color, &'static str, 3>::new_with_fn(|c| match c {
    Color::Red => "red", Color::Green => "green", Color::Blue => "blue",
});

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
use enum_table::{EnumTable, Enumable};
use std::collections::HashMap;
#[derive(Enumable, Debug, PartialEq, Eq, Hash, Copy, Clone)] enum Color { Red, Green, Blue }

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

- **Access Time**: Fast lookup time at runtime via the derived `variant_index()` method,
  which uses compile-time-computed constants for each arm. This compiles down to O(1)
  (a single memory read) for enums with dense, sequential discriminants, and to a
  compiler-generated comparison tree for sparse or custom discriminants — still faster
  than the O(log N) binary search used by the fallback (manual) implementation and by
  the `const fn` variants (`get_const`, etc.), which use binary search for `const`
  context compatibility.
- **Memory Efficiency**: No heap allocations for the table structure, leading to better cache locality.
- **Compile-Time Optimization**: Static tables can be fully constructed at compile time.

## Feature Flags

- **default**: Enables `std` and `derive`.
- **derive**: Enables the `Enumable` derive macro for automatic trait implementation.
- **serde**: Enables serialization and deserialization support using Serde. Implies `alloc`.
- **std**: Enables `std`-dependent APIs, such as conversions to/from `HashMap`. Implies `alloc`.
- **alloc**: Enables `alloc`-dependent APIs, such as conversions to/from `Vec`, without requiring the rest of `std`.

Disabling all of the above (`default-features = false`) builds `enum-table` as `#![no_std]`
with no heap-allocation dependency at all, retaining the core `EnumTable`/`Enumable` API.

## License

Licensed under the [MIT license](https://github.com/moriyoshi-kasuga/enum-table/blob/main/LICENSE)

## Benchmarks

Invoke the benchmarks using `cargo bench` to compare the performance of `EnumTable`
with a `HashMap` for enum keys. The benchmarks measure the time taken for
creating a table, getting values, and setting values.

Inputs and outputs are wrapped in `std::hint::black_box` to prevent the compiler
from constant-folding the lookups. The `get`/`set` benchmarks iterate over all 7
variants per iteration, so the reported time covers 7 operations (divide by 7 for
a rough per-operation cost).

<details>
<summary>Benchmark results</summary>

```text
EnumTable::new_with_fn  time:   [3.7279 ns 3.7305 ns 3.7334 ns]
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe

EnumTable::get          time:   [1.9207 ns 1.9229 ns 1.9254 ns]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

HashMap::get            time:   [44.039 ns 44.070 ns 44.110 ns]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe

EnumTable::set          time:   [21.530 ns 21.541 ns 21.554 ns]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe

HashMap::insert         time:   [53.874 ns 53.926 ns 53.983 ns]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
```

</details>
