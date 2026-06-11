## [enum-table-v3.0.1] - 2026-06-11

### 🐛 Bug Fixes

- Resolve diagnostics
- Any features diagnostics
- Better validation

### 💼 Other

- Correct benchmark

### 📚 Documentation

- Update changelog

### ⚙️ Miscellaneous Tasks

- Release v3.0.1
## [enum-table-v3.0.0] - 2026-02-09

### 🚀 Features

- [**breaking**] Stop saving keys
- Impl iter
- [**breaking**] Access by O(1)
- Update iter

### 🚜 Refactor

- Reorder target crates for correct build flow
- Adjust code

### 📚 Documentation

- Update changelog
- Explain padding to error
- Update README.md
- Update benchmark

### 🧪 Testing

- Fix compileable by release

### ⚙️ Miscellaneous Tasks

- Add .typos.toml
- Adjust Cargo.toml
- Release v3.0.0
## [enum-table-v2.1.2] - 2025-09-14

### 🚜 Refactor

- *(enum-table)* Store enum variants directly instead of discriminants

### 📚 Documentation

- Update changelog

### ⚙️ Miscellaneous Tasks

- Normalize newline in Cargo.toml
- Release v2.1.2
## [enum-table-v2.1.1] - 2025-09-04

### 🐛 Bug Fixes

- Resolve incorrect variant retrieval on big-endian architectures

### 🚜 Refactor

- Streamline internal `as_usize` macro

### 📚 Documentation

- Update changelog

### ⚙️ Miscellaneous Tasks

- Add update.py
- Release v2.1.1
## [enum-table-v2.1.0] - 2025-08-24

### 🚀 Features

- Add map_with_key and map_mut_with_key fn

### 📚 Documentation

- Update changelog
- *(README)* Update crate version
- *(README)* More powerful API explanation
- *(README)* More clarify document

### ⚙️ Miscellaneous Tasks

- Release v2.1.0
## [enum-table-v2.0.0] - 2025-08-22

### 🚀 Features

- Impl Copy
- Support no-std
- Add map.rs
- Add map and map_mut fn
- Impl IntoIterator

### 🐛 Bug Fixes

- [**breaking**] Change len() to return pushed count, add capacity() method
- Remove extra code
- [**breaking**] Remove map fn
- *(serde)* Add compile-time check for alloc dependency

### 💼 Other

- [**breaking**] Supporting #2

### 🚜 Refactor

- Split impls to files
- [**breaking**] Require Copy trait for Enumable trait
- Add #[inline(always)] to critical intrinsics functions
- Add #[cfg(debug_assetions)] to debug function
- Split vec-related method
- Readable code in derive crate
- Change algrotihm from bubble sort to insertion sort
- Use et! to readable

### 📚 Documentation

- Add doc for map
- Add doc for new_fill_with_copy

### 🎨 Styling

- Remove unneccesary lints
- Run cargo fmt

### 🧪 Testing

- Add test for map

### ⚙️ Miscellaneous Tasks

- Update changelog commit to follow conventional format
- Change edition to 2024
- Release v2.0.0
## [enum-table-v1.1.2] - 2025-07-08

### 🚜 Refactor

- *(const)* Defer variant sort check to EnumTable creation

### 🎨 Styling

- Fix clippy warning by 1.88.0 rule

### ⚙️ Miscellaneous Tasks

- Fix chnagelog to works on tag
- Disable update changelog when push to main branch
- Release v1.1.2
## [enum-table-v1.1.1] - 2025-07-03

### 🐛 Bug Fixes

- Create changelog symlink for sub-package publication

### ⚙️ Miscellaneous Tasks

- Configure git-cliff tag pattern
- *(lint)* Update markdownlint configuration to new format
- Release v1.1.1
## [enum-table-v1.1.0] - 2025-07-03

### 🚀 Features

- Improve developer experience with compile-time variant order validation

### 🐛 Bug Fixes

- *(serde)* Remove Eq and Hash bounds from Deserialize impl

### 🚜 Refactor

- Remove unneccesary test

### 📚 Documentation

- *(error)* Clarify that MissingVariant implies a duplicate entry
- Remove fn main by clippy warning
- *(readme)* Add link to changelog

### ⚙️ Miscellaneous Tasks

- Add workflow to auto-generate and commit CHANGELOG.md
- Update markdown linter and ignore changelog
- Release v1.1.0
## [enum-table-v1.0.0] - 2025-06-28

### 🚀 Features

- Enable serde support
- Add convertion methods
- Add methods of new and clear for Option and Default
- *(msrv)* Lower minimum supported Rust version to 1.85
- Make new_fill_with_none a const function
- Add `new_fill_with_copy` constructor

### 🐛 Bug Fixes

- Correct implementation
- Great error message
- [**breaking**] Without lifetime on try_new_with_fn
- [**breaking**] Without lifetime on checked_new_with_fn
- Prevent integer overflow in binary search

### 🚜 Refactor

- Rename to try_from_vec from from_vec
- Use macro
- Performance update. O(log n)

### 📚 Documentation

- Update typo
- Update README.md
- Add benchmarks

### 🎨 Styling

- Run cargo fmt

### 🧪 Testing

- Add binary_search test

### ⚙️ Miscellaneous Tasks

- Release v1.0.0
## [enum-table-v0.4.2] - 2025-06-13

### 🚜 Refactor

- Change to wraping

### 📚 Documentation

- Update README. use recommended
- Update optional message

### ⚙️ Miscellaneous Tasks

- Release v0.4.2
## [enum-table-v0.4.1] - 2025-06-02

### 🚜 Refactor

- Move unsafe to intrinsics mod
- Update to_usize logic

### 📚 Documentation

- Update README

### ⚙️ Miscellaneous Tasks

- Release v0.4.1
## [enum-table-v0.4.0] - 2025-05-27

### 🚀 Features

- Add try_new_with_fn and checked_new_with_fn

### ⚙️ Miscellaneous Tasks

- Release 0.4.0
## [enum-table-v0.3.2] - 2025-05-25

### 🚀 Features

- Check the unit

### 🐛 Bug Fixes

- Pretty Debug
- Remove Copy

### 🚜 Refactor

- Craete from_usize fn

### 📚 Documentation

- Update README

### 🧪 Testing

- Fix name
- Add test of impls

### ⚙️ Miscellaneous Tasks

- Release 0.3.2
## [enum-table-v0.3.1] - 2025-05-24

### 🚀 Features

- Add values_mut and add doc

### 🐛 Bug Fixes

- Remove method of discriminant
- Bug

### 📚 Documentation

- Add describe of optional

### 🧪 Testing

- Add

### ⚙️ Miscellaneous Tasks

- Release 0.3.1
## [enum-table-v0.3.0] - 2025-05-24

### 🚀 Features

- Add method of discriminant
- Add method of iter

### 🐛 Bug Fixes

- Remove ManuallyDrop

### ⚙️ Miscellaneous Tasks

- Release 0.3.0
## [enum-table-v0.2.2] - 2025-03-17

### 🚀 Features

- Impl explicit std trait

### 🚜 Refactor

- Improve panic message on 32 bit architecture
- Fix derive macro to simple
- Optimize with to_usize at initialization

### 📚 Documentation

- Add examples and more explit doc

### ⚙️ Miscellaneous Tasks

- Release v0.2.2
## [enum-table-v0.2.1] - 2025-03-10

### 🐛 Bug Fixes

- Fix import location
- Fix impl of use_variant_value macro

### 📚 Documentation

- Update README

### ⚙️ Miscellaneous Tasks

- Release v0.2.1
## [enum-table-v0.2.0] - 2025-03-01

### 🐛 Bug Fixes

- Remove count expr from et macro

### ⚙️ Miscellaneous Tasks

- Release v0.2.0
## [enum-table-v0.1.3] - 2025-03-01

### 🐛 Bug Fixes

- Return old value on set fn

### 📚 Documentation

- Update README

### ⚙️ Miscellaneous Tasks

- Relaese v0.1.3
## [enum-table-v0.1.2] - 2025-02-27

### 📚 Documentation

- Fix link

### ⚙️ Miscellaneous Tasks

- Release v0.1.2
## [enum-table-v0.1.1] - 2025-02-27

### 📚 Documentation

- Update README

### ⚙️ Miscellaneous Tasks

- Update v0.1.1
## [enum-table-v0.1.0] - 2025-02-27

### 🚀 Features

- Initialize rust
- Add base
- Add use_variant_value macro for dev util
- Add builder and et macro
- Add impl some trait
- Use generic_const_exprs
- Remove nightly
- Add Enumable derive macro

### 🐛 Bug Fixes

- Remove maybe_uninit_array_assume_init feature

### 🚜 Refactor

- Rename generic from T to K

### 📚 Documentation

- Add doc
- Add README

### ⚙️ Miscellaneous Tasks

- Add workspace
