use enum_table::Enumerable;

#[derive(Enumerable)]
enum Generic<T> {
    A,
    B(core::marker::PhantomData<T>),
}

fn main() {}
