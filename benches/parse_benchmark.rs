use chrono::Utc;
use criterion::{Criterion, criterion_group, criterion_main};
use cron_parser::parse;
use std::hint::black_box;

fn benchmark_cron_parsing(c: &mut Criterion) {
    let now = Utc::now();

    c.bench_function("parse every 5 minutes", |b| {
        b.iter(|| parse(black_box("*/5 * * * *"), &now))
    });

    c.bench_function("parse daily at midnight", |b| {
        b.iter(|| parse(black_box("0 0 * * *"), &now))
    });

    c.bench_function("parse complex expression", |b| {
        b.iter(|| parse(black_box("15,45 */2 1-5 * 1-5"), &now))
    });
}

criterion_group!(benches, benchmark_cron_parsing);
criterion_main!(benches);
