use std::{collections::HashMap, hash::Hash, hint::black_box};

use criterion::{Criterion, criterion_group, criterion_main};
use enum_table::{EnumTable, Enumable};

#[derive(Clone, Copy, Enumable, Eq, PartialEq, Hash)]
enum Letter {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

fn new() -> EnumTable<Letter, &'static str, { Letter::COUNT }> {
    EnumTable::new_with_fn(|letter| match letter {
        Letter::A => "Alpha",
        Letter::B => "Bravo",
        Letter::C => "Charlie",
        Letter::D => "Delta",
        Letter::E => "Echo",
        Letter::F => "Foxtrot",
        Letter::G => "Golf",
    })
}

fn new_hash_map() -> HashMap<Letter, &'static str> {
    let mut map = HashMap::new();
    map.insert(Letter::A, "Alpha");
    map.insert(Letter::B, "Bravo");
    map.insert(Letter::C, "Charlie");
    map.insert(Letter::D, "Delta");
    map.insert(Letter::E, "Echo");
    map.insert(Letter::F, "Foxtrot");
    map.insert(Letter::G, "Golf");
    map
}

fn enum_table_new_with_fn(criterion: &mut Criterion) {
    criterion.bench_function("EnumTable::new_with_fn", |bencher| {
        bencher.iter(|| black_box(new()))
    });
}

fn enum_table_get(criterion: &mut Criterion) {
    let table = new();
    criterion.bench_function("EnumTable::get", |bencher| {
        bencher.iter(|| {
            for letter in Letter::VARIANTS {
                black_box(black_box(&table).get(black_box(letter)));
            }
        })
    });
}

fn hash_map_get(criterion: &mut Criterion) {
    let map = new_hash_map();

    criterion.bench_function("HashMap::get", |bencher| {
        bencher.iter(|| {
            for letter in Letter::VARIANTS {
                black_box(black_box(&map).get(black_box(letter)));
            }
        })
    });
}

fn enum_table_set(criterion: &mut Criterion) {
    let mut table = new();
    criterion.bench_function("EnumTable::set", |bencher| {
        bencher.iter(|| {
            for letter in Letter::VARIANTS {
                black_box(table.set(black_box(letter), black_box("Updated")));
            }
        })
    });
}

fn hash_map_set(criterion: &mut Criterion) {
    let mut map = new_hash_map();
    criterion.bench_function("HashMap::insert", |bencher| {
        bencher.iter(|| {
            for letter in Letter::VARIANTS {
                black_box(map.insert(black_box(*letter), black_box("Updated")));
            }
        })
    });
}

criterion_group!(
    benches,
    enum_table_new_with_fn,
    enum_table_get,
    hash_map_get,
    enum_table_set,
    hash_map_set,
);
criterion_main!(benches);
