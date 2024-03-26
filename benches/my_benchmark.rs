use std::io::{BufRead, BufReader};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ws_parse_demo::read_buf;

const TEST_DATA: &str = include_str!("../osc.json");

fn readbuf(data: impl BufRead) -> () {
    for _ in read_buf(data) {}
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("BufReader282 20", |b| {
        b.iter(|| readbuf(black_box(BufReader::new(TEST_DATA.as_bytes()))))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
