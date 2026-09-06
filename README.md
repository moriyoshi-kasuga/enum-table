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

`EnumTable<K, V, N>` guarantees a value for every variant of `K`. Because of that
guarantee, `EnumTable::get` returns `&V` directly, unlike `HashMap::get`, which must
return `Option<&V>` since a key may or may not be present. If a value can legitimately
be absent, use `EnumTable<K, Option<V>, N>` instead; see `EnumTable::new_fill_with_none`.

- **vs. `HashMap<K, V>`**: no heap allocation for the table structure, better cache
  locality, and constructible in a `const` context. The core has no dependency on
  `alloc` or `std` at all, so it works in `#![no_std]` environments without a global
  allocator.
- **vs. `match` statements**: a table is data rather than code, so it can be passed
  around, mutated at runtime, or loaded from configuration without recompiling.
- **vs. arrays (`[V; N]`)**: works with enums whose discriminants are non-contiguous or
  explicitly assigned (e.g. `enum E { A = 1, B = 100 }`), without manually mapping
  variants to `0..N` indices.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
enum-table = "3.0"
```

*Requires Rust 1.85 or later.*

## The `Enumerable` Trait

`EnumTable` requires its key type to implement `Enumerable`, which lists every variant
of the enum and gives each one an index:

```rust,ignore
pub unsafe trait Enumerable: Copy + 'static {
    const VARIANTS: &'static [Self];
    const COUNT: usize = Self::VARIANTS.len();

    fn variant_index(&self) -> usize { .. }
}
```

`Enumerable` is an `unsafe trait`: implementors must guarantee that `VARIANTS`
lists every variant of `Self` exactly once, sorted in ascending order by the
**unsigned bit-pattern** of its in-memory representation, and that `Self` has
no padding bytes. See the [`Enumerable` trait documentation][enumerable-docs] for
the full safety contract, including how signed discriminants sort.

Use the derive macro `#[derive(Enumerable)]` rather than implementing this trait by
hand; it generates a correct `unsafe impl` (a sorted `VARIANTS` array and an O(1)
`variant_index()` computed at compile time) without requiring any `unsafe` code from you.

[enumerable-docs]: https://docs.rs/enum-table/latest/enum_table/trait.Enumerable.html

### Safety and Memory Layout

`#[derive(Enumerable)]` only supports field-less (C-like) enums, which never
have padding bytes on their own. A primitive representation (e.g. `#[repr(u8)]`)
is still recommended for a stable, minimal-size layout, but it is not required
for soundness.

The one exception is `#[repr(align(N))]`: an alignment larger than the
discriminant's natural size can add trailing padding bytes that this crate's
byte-level comparisons would read as uninitialized memory. Since checking
whether a specific `N` actually does so would require duplicating the
compiler's layout rules, the derive macro rejects `#[repr(align(N))]`
unconditionally at compile time.

```rust
use enum_table::Enumerable;

#[derive(Enumerable, Copy, Clone)]
#[repr(u8)] // <--- Recommended, but not required for soundness.
enum MyEnum {
    A,
    B,
}
```

## Usage Examples

### Basic Usage

```rust
use enum_table::{EnumTable, Enumerable};

#[derive(Enumerable, Copy, Clone)]
#[repr(u8)]
enum Test {
    A = 100, // You can specify custom discriminants
    B = 1,
    C,
}

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
use enum_table::{EnumTable, Enumerable};
#[derive(Enumerable, Copy, Clone)]
#[repr(u8)] 
enum Test {
    A = 100,
    B = 1,
    C
}

static TABLE: EnumTable<Test, &'static str, { Test::VARIANTS.len() }> =
  enum_table::et!(Test, &'static str, |t| match t {
      Test::A => "A",
      Test::B => "B",
      Test::C => "C",
  });

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
use enum_table::{EnumTable, Enumerable};
use serde::{Serialize, Deserialize};

#[derive(Debug, Enumerable, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

let json = serde_json::to_string(&table).unwrap();
assert_eq!(json, r#"{"Active":"running","Inactive":"stopped","Pending":"waiting"}"#);

let deserialized: EnumTable<Status, &str, { Status::VARIANTS.len() }> =
    serde_json::from_str(&json).unwrap();

assert_eq!(table, deserialized);
```

### Error Handling and Alternative Constructors

`try_new_with_fn` builds a table from a closure that may fail per variant, stopping at
the first error:

```rust
use enum_table::{EnumTable, Enumerable};

#[derive(Enumerable, Copy, Clone, Debug, PartialEq)]
enum Color {
    Red,
    Green,
    Blue,
}

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
see the **API Overview** section below and the full [API documentation](https://docs.rs/enum-table/latest/enum_table/struct.EnumTable.html).
For instance, `try_from_vec()` and `try_from_hash_map()` in the **Conversions** API also
handle missing variants.

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
use enum_table::{EnumTable, Enumerable};
#[derive(Enumerable, Debug, PartialEq, Eq, Hash, Copy, Clone)] enum Color { Red, Green, Blue }
let table = EnumTable::<Color, &'static str, 3>::new_with_fn(|c| match c {
    Color::Red => "red", Color::Green => "green", Color::Blue => "blue",
});

let vec = table.into_vec();
assert_eq!(vec.len(), 3);
assert!(vec.contains(&(Color::Red, "red")));
```

#### To `EnumTable`

- `try_from_vec()`: Creates a table from a `Vec<(K, V)>`.
  Returns `None` if any variant is missing or duplicated.
- `try_from_hash_map()`: Creates a table from a `HashMap<K, V>`.
  Returns `None` if the map does not contain exactly one entry for each variant.
- `try_from_btree_map()`: Creates a table from a `BTreeMap<K, V>`.
  Returns `None` if the map does not contain exactly one entry for each variant.

```rust
use enum_table::{EnumTable, Enumerable};
use std::collections::HashMap;
#[derive(Enumerable, Debug, PartialEq, Eq, Hash, Copy, Clone)] enum Color { Red, Green, Blue }

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
  which uses compile-time-computed constants for each arm. This tends to compile down to
  O(1) (a single memory read) for enums with dense, sequential discriminants, and to a
  compiler-generated comparison tree for sparse or custom discriminants — the exact
  codegen depends on what LLVM chooses, but both are still faster than the O(log N)
  binary search used by the fallback (manual) implementation and by the `const fn`
  variants (`get_const`, etc.), which use binary search for `const` context compatibility.
- **Memory Efficiency**: No heap allocations for the table structure, leading to better cache locality.
- **Compile-Time Optimization**: Static tables can be fully constructed at compile time.

## Feature Flags

- **default**: Enables `std` and `derive`.
- **derive**: Enables the `Enumerable` derive macro for automatic trait implementation.
- **serde**: Enables serialization and deserialization support using Serde. Implies `alloc`.
- **std**: Enables `std`-dependent APIs, such as conversions to/from `HashMap`. Implies `alloc`.
- **alloc**: Enables `alloc`-dependent APIs, such as conversions to/from `Vec` and `BTreeMap`, without requiring the rest of `std`.

Disabling all of the above (`default-features = false`) builds `enum-table` as `#![no_std]`
with no heap-allocation dependency at all, retaining the core `EnumTable`/`Enumerable` API.

## License

Licensed under the [MIT license](https://github.com/moriyoshi-kasuga/enum-table/blob/main/LICENSE)

## Benchmarks

- **construction**: building a fully populated table/map from scratch
  (`EnumTable::new_with_fn` vs. `HashMap::new` + inserting every entry).
- **single_get** / **single_set**: a single lookup/update on one key, measured
  in isolation (also compares `get` against the `const fn` binary-search
  `get_const`).
- **bulk_get_all_variants** / **bulk_set_all_variants**: reading/writing every
  variant once per iteration, representing a whole-table workload rather than
  a single operation.
- **iteration**: iterating over every key-value pair.
- **conversions**: `into_vec`/`try_from_vec` and `into_hash_map`/
  `try_from_hash_map`, rebuilding the source fresh each iteration so only the
  conversion itself is measured.

<details>
<summary>Benchmark results</summary>

```text
construction/EnumTable::new_with_fn
                        time:   [3.7377 ns 3.7418 ns 3.7462 ns]
construction/HashMap (new + insert all)
                        time:   [72.319 ns 72.352 ns 72.386 ns]

single_get/EnumTable::get
                        time:   [496.33 ps 499.16 ps 502.90 ps]
single_get/EnumTable::get_const
                        time:   [2.2176 ns 2.2185 ns 2.2196 ns]
single_get/HashMap::get 
                        time:   [6.7604 ns 6.7634 ns 6.7673 ns]

single_set/EnumTable::set
                        time:   [3.0835 ns 3.0857 ns 3.0880 ns]
single_set/HashMap::insert
                        time:   [8.1702 ns 8.1880 ns 8.2065 ns]

bulk_get_all_variants/EnumTable::get
                        time:   [2.4252 ns 2.4265 ns 2.4283 ns]
bulk_get_all_variants/HashMap::get
                        time:   [42.776 ns 42.802 ns 42.832 ns]

bulk_set_all_variants/EnumTable::set
                        time:   [21.622 ns 21.649 ns 21.677 ns]
bulk_set_all_variants/HashMap::insert
                        time:   [56.734 ns 56.822 ns 56.923 ns]

iteration/EnumTable::iter
                        time:   [587.40 ps 587.96 ps 588.58 ps]
iteration/HashMap::iter
                        time:   [3.8318 ns 3.8357 ns 3.8405 ns]

conversions/EnumTable::into_vec
                        time:   [43.888 ns 43.943 ns 44.005 ns]
conversions/EnumTable::try_from_vec
                        time:   [20.183 ns 20.328 ns 20.462 ns]
conversions/EnumTable::into_hash_map
                        time:   [84.923 ns 85.154 ns 85.435 ns]
conversions/EnumTable::try_from_hash_map
                        time:   [95.706 ns 95.871 ns 96.046 ns]
```

</details>
