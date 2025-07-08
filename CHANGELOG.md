## [enum-table-v1.1.2] - 2025-07-08

### Chore

- Fix chnagelog to works on tag
- Disable update changelog when push to main branch
- Release v1.1.2

### Refactor

- *(const)* Defer variant sort check to EnumTable creation

### Style

- Fix clippy warning by 1.88.0 rule

## [enum-table-v1.1.1] - 2025-07-03

### Chore

- Configure git-cliff tag pattern
- *(lint)* Update markdownlint configuration to new format
- Release v1.1.1

### Fix

- Create changelog symlink for sub-package publication

## [enum-table-v1.1.0] - 2025-07-03

### Chore

- Add workflow to auto-generate and commit CHANGELOG.md
- Update markdown linter and ignore changelog
- Release v1.1.0

### Docs

- *(error)* Clarify that MissingVariant implies a duplicate entry
- Remove fn main by clippy warning
- *(readme)* Add link to changelog

### Feat

- Improve developer experience with compile-time variant order validation

### Fix

- *(serde)* Remove Eq and Hash bounds from Deserialize impl

### Refactor

- Remove unneccesary test

## [enum-table-v1.0.0] - 2025-06-28

### Chore

- Release v1.0.0

### Docs

- Update typo
- Update README.md
- Add benchmarks

### Feat

- Enable serde support
- Add convertion methods
- Add methods of new and clear for Option and Default
- *(msrv)* Lower minimum supported Rust version to 1.85
- Make new_fill_with_none a const function
- Add `new_fill_with_copy` constructor

### Fix

- Correct implementation
- Great error message
- [**breaking**] Without lifetime on try_new_with_fn
- [**breaking**] Without lifetime on checked_new_with_fn
- Prevent integer overflow in binary search

### Refactor

- Rename to try_from_vec from from_vec
- Use macro
- Performance update. O(log n)

### Style

- Run cargo fmt

### Test

- Add binary_search test

## [enum-table-v0.4.2] - 2025-06-13

### Chore

- Release v0.4.2

### Docs

- Update README. use recommended
- Update optional message

### Refactor

- Change to wraping

## [enum-table-v0.4.1] - 2025-06-02

### Chore

- Release v0.4.1

### Docs

- Update README

### Refactor

- Move unsafe to intrinsics mod
- Update to_usize logic

## [enum-table-v0.4.0] - 2025-05-27

### Chore

- Release 0.4.0

### Feat

- Add try_new_with_fn and checked_new_with_fn

## [enum-table-v0.3.2] - 2025-05-25

### Chore

- Release 0.3.2

### Docs

- Update README

### Feat

- Check the unit

### Fix

- Pretty Debug
- Remove Copy

### Refactor

- Craete from_usize fn

### Test

- Fix name
- Add test of impls

## [enum-table-v0.3.1] - 2025-05-24

### Chore

- Release 0.3.1

### Docs

- Add describe of optional

### Feat

- Add values_mut and add doc

### Fix

- Remove method of discriminant
- Bug

### Test

- Add

## [enum-table-v0.3.0] - 2025-05-24

### Chore

- Release 0.3.0

### Feat

- Add method of discriminant
- Add method of iter

### Fix

- Remove ManuallyDrop

## [enum-table-v0.2.2] - 2025-03-17

### Chore

- Release v0.2.2

### Docs

- Add examples and more explit doc

### Feat

- Impl explicit std trait

### Refactor

- Improve panic message on 32 bit architecture
- Fix derive macro to simple
- Optimize with to_usize at initialization

## [enum-table-v0.2.1] - 2025-03-10

### Chore

- Release v0.2.1

### Docs

- Update README

### Fix

- Fix import location
- Fix impl of use_variant_value macro

## [enum-table-v0.2.0] - 2025-03-01

### Chore

- Release v0.2.0

### Fix

- Remove count expr from et macro

## [enum-table-v0.1.3] - 2025-03-01

### Chore

- Relaese v0.1.3

### Docs

- Update README

### Fix

- Return old value on set fn

## [enum-table-v0.1.2] - 2025-02-27

### Chore

- Release v0.1.2

### Docs

- Fix link

## [enum-table-v0.1.1] - 2025-02-27

### Chore

- Update v0.1.1

### Docs

- Update README

## [enum-table-v0.1.0] - 2025-02-27

### Chore

- Add workspace

### Docs

- Add doc
- Add README

### Feat

- Initialize rust
- Add base
- Add use_variant_value macro for dev util
- Add builder and et macro
- Add impl some trait
- Use generic_const_exprs
- Remove nightly
- Add Enumable derive macro

### Fix

- Remove maybe_uninit_array_assume_init feature

### Refactor

- Rename generic from T to K

