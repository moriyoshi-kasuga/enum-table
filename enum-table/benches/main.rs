use std::{collections::HashMap, hash::Hash, hint::black_box};

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use enum_table::{EnumTable, Enumerable};

#[derive(Clone, Copy, Enumerable, Eq, PartialEq, Hash)]
enum Letter {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

const LEN: usize = Letter::COUNT;

fn value_for(letter: &Letter) -> &'static str {
    match letter {
        Letter::A => "Alpha",
        Letter::B => "Bravo",
        Letter::C => "Charlie",
        Letter::D => "Delta",
        Letter::E => "Echo",
        Letter::F => "Foxtrot",
        Letter::G => "Golf",
    }
}

fn new_table() -> EnumTable<Letter, &'static str, LEN> {
    EnumTable::new_with_fn(value_for)
}

fn new_hash_map() -> HashMap<Letter, &'static str> {
    Letter::VARIANTS
        .iter()
        .map(|l| (*l, value_for(l)))
        .collect()
}

fn new_vec() -> Vec<(Letter, &'static str)> {
    Letter::VARIANTS
        .iter()
        .map(|l| (*l, value_for(l)))
        .collect()
}

/// Building a fully populated table/map from scratch.
fn construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("construction");
    group.bench_function("EnumTable::new_with_fn", |b| {
        b.iter(|| black_box(new_table()))
    });
    group.bench_function("HashMap (new + insert all)", |b| {
        b.iter(|| black_box(new_hash_map()))
    });
    group.finish();
}

/// A single get, on one key, in isolation.
fn single_get(c: &mut Criterion) {
    let table = new_table();
    let map = new_hash_map();

    let mut group = c.benchmark_group("single_get");
    group.bench_function("EnumTable::get", |b| {
        b.iter(|| black_box(*black_box(&table).get(black_box(&Letter::D))))
    });
    group.bench_function("EnumTable::get_const", |b| {
        b.iter(|| black_box(*black_box(&table).get_const(black_box(&Letter::D))))
    });
    group.bench_function("HashMap::get", |b| {
        b.iter(|| {
            if let Some(v) = black_box(&map).get(black_box(&Letter::D)) {
                black_box(*v);
            }
        })
    });
    group.finish();
}

/// A single set/insert, on one key, in isolation.
fn single_set(c: &mut Criterion) {
    let mut table = new_table();
    let mut map = new_hash_map();

    let mut group = c.benchmark_group("single_set");
    group.bench_function("EnumTable::set", |b| {
        b.iter(|| black_box(table.set(black_box(&Letter::D), black_box("Updated"))))
    });
    group.bench_function("HashMap::insert", |b| {
        b.iter(|| black_box(map.insert(black_box(Letter::D), black_box("Updated"))))
    });
    group.finish();
}

/// Reading every variant once per iteration: a whole-table workload rather
/// than a single lookup.
fn bulk_get_all_variants(c: &mut Criterion) {
    let table = new_table();
    let map = new_hash_map();

    let mut group = c.benchmark_group("bulk_get_all_variants");
    group.bench_function("EnumTable::get", |b| {
        b.iter(|| {
            for letter in Letter::VARIANTS {
                black_box(*black_box(&table).get(black_box(letter)));
            }
        })
    });
    group.bench_function("HashMap::get", |b| {
        b.iter(|| {
            for letter in Letter::VARIANTS {
                if let Some(v) = black_box(&map).get(black_box(letter)) {
                    black_box(*v);
                }
            }
        })
    });
    group.finish();
}

/// Writing every variant once per iteration: a whole-table workload rather
/// than a single update.
fn bulk_set_all_variants(c: &mut Criterion) {
    let mut table = new_table();
    let mut map = new_hash_map();

    let mut group = c.benchmark_group("bulk_set_all_variants");
    group.bench_function("EnumTable::set", |b| {
        b.iter(|| {
            for letter in Letter::VARIANTS {
                black_box(table.set(black_box(letter), black_box("Updated")));
            }
        })
    });
    group.bench_function("HashMap::insert", |b| {
        b.iter(|| {
            for letter in Letter::VARIANTS {
                black_box(map.insert(black_box(*letter), black_box("Updated")));
            }
        })
    });
    group.finish();
}

/// Iterating over every key-value pair and summing the value lengths.
fn iteration(c: &mut Criterion) {
    let table = new_table();
    let map = new_hash_map();

    let mut group = c.benchmark_group("iteration");
    group.bench_function("EnumTable::iter", |b| {
        b.iter(|| {
            black_box(
                black_box(&table)
                    .iter()
                    .map(|(_, v)| v.len())
                    .sum::<usize>(),
            )
        })
    });
    group.bench_function("HashMap::iter", |b| {
        b.iter(|| black_box(map.values().map(|v| v.len()).sum::<usize>()))
    });
    group.finish();
}

/// Converting to/from a `Vec`/`HashMap`, freshly rebuilding the source each
/// iteration so only the conversion itself is measured.
fn conversions(c: &mut Criterion) {
    let mut group = c.benchmark_group("conversions");
    group.bench_function("EnumTable::into_vec", |b| {
        b.iter_batched(
            new_table,
            |table| black_box(table.into_vec()),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("EnumTable::try_from_vec", |b| {
        b.iter_batched(
            new_vec,
            |vec| black_box(EnumTable::<Letter, &'static str, LEN>::try_from_vec(vec)),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("EnumTable::into_hash_map", |b| {
        b.iter_batched(
            new_table,
            |table| black_box(table.into_hash_map()),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("EnumTable::try_from_hash_map", |b| {
        b.iter_batched(
            new_hash_map,
            |map| {
                black_box(EnumTable::<Letter, &'static str, LEN>::try_from_hash_map(
                    map,
                ))
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

criterion_group!(
    benches,
    construction,
    single_get,
    single_set,
    bulk_get_all_variants,
    bulk_set_all_variants,
    iteration,
    conversions,
);
criterion_main!(benches);
