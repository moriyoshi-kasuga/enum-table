use enum_table::Enumerable;

// `align(1)` matches the natural alignment of a `u8` discriminant and cannot
// introduce padding, but `#[repr(align(N))]` is rejected unconditionally.
#[repr(u8, align(1))]
#[derive(Clone, Copy, Enumerable)]
enum AlignedNoPadding {
    A,
    B,
}

fn main() {}
