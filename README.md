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

## Why use `enum-table`?

`EnumTable<K, V, N>` holds a value for every variant of `K`, so `EnumTable::get`
returns `&V` directly instead of the `Option<&V>` that `HashMap::get` must return.
If a value can legitimately be absent, use `EnumTable<K, Option<V>, N>` instead;
`EnumTable`'s `Default` implementation then fills every slot with `None`.

- Compared to `HashMap<K, V>`: no heap allocation for the table structure, better cache
  locality, and constructible in a `const` context. The core has no dependency on
  `alloc` or `std` at all, so it works in `#![no_std]` environments without a global
  allocator.
- Compared to `match` statements: a table is data rather than code, so it can be passed
  around, mutated at runtime, or loaded from configuration without recompiling.
- Compared to arrays (`[V; N]`): works with enums whose discriminants are non-contiguous
  or explicitly assigned (e.g. `enum E { A = 1, B = 100 }`), without manually mapping
  variants to `0..N` indices.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
enum-table = "4.0"
```

_Requires Rust 1.85 or later._

## The `Enumerable` Trait

`EnumTable`'s key type must implement `Enumerable`, which lists every variant of the enum and gives each one an index:

```rust,ignore
pub unsafe trait Enumerable: Copy + 'static {
    const VARIANTS: &'static [Self];
    const COUNT: usize = Self::VARIANTS.len();

    fn variant_index(&self) -> usize { .. }
}
```

`Enumerable` is an `unsafe trait`: implementors must guarantee that `Self` has no
padding bytes and that `VARIANTS` lists every variant of `Self` exactly once, sorted
in ascending order by the unsigned bit-pattern of its in-memory representation. See
the [`Enumerable` trait documentation][enumerable-docs] for the full safety contract,
including how signed discriminants sort.

Use `#[derive(Enumerable)]` instead of implementing this trait by hand; it generates
a correct `unsafe impl` with a sorted `VARIANTS` array and a compile-time-computed
`variant_index()`, without requiring any `unsafe` code from you.

[enumerable-docs]: https://docs.rs/enum-table/latest/enum_table/trait.Enumerable.html

### Safety and Memory Layout

`#[derive(Enumerable)]` only supports field-less (C-like) enums, which never
have padding bytes on their own. A primitive representation (e.g. `#[repr(u8)]`)
is recommended for a stable, minimal-size layout, but not required for soundness.

`#[repr(align(N))]` is the exception: an alignment larger than the discriminant's
natural size can add trailing padding bytes that this crate's byte-level
comparisons would read as uninitialized memory, so the derive macro rejects it
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

let mut table = EnumTable::<Test, &'static str, { Test::COUNT }>::from_fn(
  |t| match t {
    Test::A => "A",
    Test::B => "B",
    Test::C => "C",
});

assert_eq!(table.get(Test::A), &"A");

let old_b = table.set(Test::B, "Changed B");
assert_eq!(old_b, "B");
assert_eq!(table.get(Test::B), &"Changed B");
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

static TABLE: EnumTable<Test, &'static str, { Test::COUNT }> =
  enum_table::et!(Test, &'static str, |t| match t {
      Test::A => "A",
      Test::B => "B",
      Test::C => "C",
  });

const A_VAL: &str = TABLE.get_const(Test::A);
assert_eq!(A_VAL, "A");
```

### Serde Support

Enable serde support by adding the `serde` feature:

```toml
[dependencies]
enum-table = { version = "4.0", features = ["serde"] }
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

let table = EnumTable::<Status, &'static str, { Status::COUNT }>::from_fn(|status| match status {
    Status::Active => "running",
    Status::Inactive => "stopped",
    Status::Pending => "waiting",
});

let json = serde_json::to_string(&table).unwrap();
assert_eq!(json, r#"{"Active":"running","Inactive":"stopped","Pending":"waiting"}"#);

let deserialized: EnumTable<Status, &str, { Status::COUNT }> =
    serde_json::from_str(&json).unwrap();

assert_eq!(table, deserialized);
```

### Error Handling with `try_from_fn`

`try_from_fn` builds a table from a closure that may fail per variant, stopping at
the first error:

```rust
use enum_table::{EnumTable, Enumerable};

#[derive(Enumerable, Copy, Clone, Debug, PartialEq)]
enum Color {
    Red,
    Green,
    Blue,
}

let result = EnumTable::<Color, &'static str, { Color::COUNT }>::try_from_fn(
    |color| match color {
        Color::Red => Ok("Red"),
        Color::Green => Err("Failed to get value for Green"),
        Color::Blue => Ok("Blue"),
    }
);

assert_eq!(result, Err("Failed to get value for Green"));
```

For other construction methods, such as creating a table from existing data structures,
see the API Overview section below and the full [API documentation](https://docs.rs/enum-table/latest/enum_table/struct.EnumTable.html).

## API Overview

For complete API documentation, visit [EnumTable on doc.rs](https://docs.rs/enum-table/latest/enum_table/struct.EnumTable.html).

### Construction

- `EnumTable::from_fn()`: Create a table by mapping each enum variant to a value.
- `EnumTable::try_from_fn()`: Create a table from a closure that may fail, stopping at the first error.
- `EnumTable::checked_from_fn()`: Create a table from a closure that may return `None`, stopping at the first `None`.
- `EnumTable::checked_from_pairs()`: Create a table from `(K, V)` pairs, or `None` if a variant is missing or duplicated.
- `EnumTable::from_elem()`: Create a table with the same `Copy` value for every variant.
- `EnumTable::default()`: Create a table filled with each variant's `Default` value (requires `V: Default`).

### Access

- `get()`, `get_mut()`, `set()`: O(1) access to the value for a variant.
- `get_const()`, `get_mut_const()`, `set_const()`: `const fn` equivalents, using binary search instead of O(1) lookup.
- `Index`/`IndexMut` (`table[key]`, accepting `K` or `&K`): shorthand for `get`/`get_mut`.

### Transformation

- `map()`: Transforms all values in the table, given each key and value.
- `for_each()`: Calls a function with each key and a reference to its value.
- `for_each_mut()`: Mutates all values in the table in-place, given each key and value.
- `zip_with()`: Combines two tables element-wise using a function, given each key and both values.
- `clear()`: Resets every value to its `Default` (requires `V: Default`).
- `take()`: Replaces a value with its `Default` and returns the old value (requires `V: Default`).

### Iterators

- `iter()`, `iter_mut()`: Iterate over key-value pairs.
- `keys()`: Iterate over keys.
- `values()`, `values_mut()`: Iterate over values.
- `into_iter()`: Consume the table and iterate over owned key-value pairs.
- Implements `Extend<(K, V)>` and `Extend<(&K, &V)>` for updating values from an iterator.

## Performance

- `#[derive(Enumerable)]` overrides `variant_index()` with a match whose arms resolve
  to their index at compile time, which tends to compile down to O(1) for enums with
  dense, sequential discriminants and to a comparison tree for sparse or custom ones —
  either way faster than the O(log N) binary search used by the default
  `variant_index()` implementation.
- The `const fn` variants (`get_const`, etc.) always binary search instead, since
  `variant_index()` cannot be called from a `const fn`.
- No heap allocation for the table structure, for better cache locality than `HashMap`.
- Tables built with the `et!` macro are fully constructed at compile time.

## Benchmarks

- `construction`: building a fully populated table/map from scratch
  (`EnumTable::from_fn` vs. `HashMap::new` + inserting every entry).
- `single_get` / `single_set`: a single lookup/update on one key, measured
  in isolation (also compares `get` against the `const fn` binary-search
  `get_const`).
- `bulk_get_all_variants` / `bulk_set_all_variants`: reading/writing every
  variant once per iteration, representing a whole-table workload rather than
  a single operation.
- `iteration`: iterating over every key-value pair.

<details>
<summary>Benchmark results</summary>

```text
construction/EnumTable::from_fn
                        time:   [3.7364 ns 3.7384 ns 3.7408 ns]
construction/HashMap (new + insert all)
                        time:   [75.696 ns 75.718 ns 75.744 ns]

single_get/EnumTable::get
                        time:   [477.00 ps 477.33 ps 477.66 ps]
single_get/EnumTable::get_const
                        time:   [2.2253 ns 2.2265 ns 2.2281 ns]
single_get/HashMap::get
                        time:   [6.7835 ns 6.7866 ns 6.7905 ns]

single_set/EnumTable::set
                        time:   [3.1015 ns 3.1114 ns 3.1222 ns]
single_set/HashMap::insert
                        time:   [9.7141 ns 9.7258 ns 9.7387 ns]

bulk_get_all_variants/EnumTable::get
                        time:   [2.3113 ns 2.3167 ns 2.3231 ns]
bulk_get_all_variants/HashMap::get
                        time:   [43.538 ns 43.555 ns 43.575 ns]

bulk_set_all_variants/EnumTable::set
                        time:   [21.568 ns 21.627 ns 21.688 ns]
bulk_set_all_variants/HashMap::insert
                        time:   [56.798 ns 56.849 ns 56.916 ns]

iteration/EnumTable::iter
                        time:   [594.43 ps 595.08 ps 595.83 ps]
iteration/HashMap::iter
                        time:   [3.8497 ns 3.8513 ns 3.8531 ns]
```

</details>

## Feature Flags

- `default`: Enables `std` and `derive`.
- `derive`: Enables the `#[derive(Enumerable)]` macro.
- `serde`: Enables `Serialize`/`Deserialize` for `EnumTable`. Implies `alloc`.
- `std`: Builds against `std` instead of `#![no_std]`. Implies `alloc`.
- `alloc`: Links `alloc`, required by `serde`.

Disabling all of the above (`default-features = false`) builds `enum-table` as `#![no_std]`
with no heap-allocation dependency, retaining the core `EnumTable`/`Enumerable` API.

## License

Licensed under the [MIT license](https://github.com/moriyoshi-kasuga/enum-table/blob/main/LICENSE)
