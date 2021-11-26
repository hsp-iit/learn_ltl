use criterion::{black_box, criterion_group, criterion_main, Criterion};
use learn_pltl_fast::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn solve_sample(c: &mut Criterion) {
    let file = File::open("sample.ron").expect("open sample file");
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();
    buf_reader
        .read_to_end(&mut contents)
        .expect("read sample file");
    let de_sample: DeSample<3> = ron::de::from_bytes(&contents).expect("deserialize sample");
    let sample = de_sample.into_sample();

    c.bench_function("solve sample", |b| {
        b.iter(|| par_brute_solve(black_box(&sample), false))
    });

    let sample = one_three_0077();

    c.bench_function("solve sample (no load)", |b| {
        b.iter(|| par_brute_solve(black_box(&sample), false))
    });
}

criterion_group!(benches, solve_sample);
criterion_main!(benches);
