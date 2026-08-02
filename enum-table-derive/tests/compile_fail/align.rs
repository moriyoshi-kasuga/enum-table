use enum_table::Enumerable;

#[repr(u8, align(4))]
#[derive(Clone, Copy, Enumerable)]
enum Aligned {
    A,
    B,
}

fn main() {}
