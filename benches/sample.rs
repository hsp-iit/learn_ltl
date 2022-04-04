use criterion::{black_box, criterion_group, criterion_main, Criterion};
use learn_pltl_fast::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn solve_sample(c: &mut Criterion) {
    // let sample = one_three_0077();

    let file = File::open("_sample_0077.ron").expect("open file");
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents).expect("read");
    let sample = ron::de::from_bytes::<Sample<3>>(&contents).expect("sample");

    c.bench_function("solve sample 0077", |b| {
        b.iter(|| par_brute_solve(black_box(&sample), false))
    });

    let file = File::open("_sample_0197.ron").expect("open file");
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents).expect("read");
    let sample = ron::de::from_bytes::<Sample<3>>(&contents).expect("sample");

    c.bench_function("solve sample 0197", |b| {
        b.iter(|| par_brute_solve(black_box(&sample), false))
    });

    let file = File::open("_sample_tbth0000.ron").expect("open file");
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents).expect("read");
    let sample = ron::de::from_bytes::<Sample<3>>(&contents).expect("sample");

    c.bench_function("solve sample tbth 0000", |b| {
        b.iter(|| par_brute_solve(black_box(&sample), false))
    });
}

criterion_group!(benches, solve_sample);
criterion_main!(benches);