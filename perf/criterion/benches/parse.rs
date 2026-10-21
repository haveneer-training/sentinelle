//! Débit du parseur de `auth.log`, mesuré par criterion.

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use sentinelle::auth::parse_auth_line;
use sentinelle::journal::parse_auth;
use sentinelle_gen::{Dataset, Options};
use std::hint::black_box;

fn parse(c: &mut Criterion) {
    let data = Dataset::generate(&Options::default());
    let line = "2026-11-11T22:30:00Z bastion-01 sshd[4242]: Failed password for l.roux from 198.51.100.227 port 50022 ssh2";

    let mut group = c.benchmark_group("parse");
    group.throughput(Throughput::Bytes(line.len() as u64));
    group.bench_function("ligne", |b| b.iter(|| parse_auth_line(black_box(line))));
    group.throughput(Throughput::Bytes(data.auth.len() as u64));
    group.sample_size(20);
    group.bench_function("auth.log", |b| {
        b.iter(|| parse_auth(black_box(&data.auth)).entries.len())
    });
    group.finish();
}

criterion_group!(benches, parse);
criterion_main!(benches);
