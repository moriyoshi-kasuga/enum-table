use enum_table::Enumerable;

#[derive(Clone, Copy, Enumerable)]
enum WithFields {
    A(u8),
    B,
}

fn main() {}
