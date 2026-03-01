use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pramana_sdk::{Gauss, Gint};

fn bench_gauss_arithmetic(c: &mut Criterion) {
    let a = Gauss::new(7, 3, 11, 5);
    let b = Gauss::new(13, 7, 17, 9);

    c.bench_function("gauss_add", |bencher| {
        bencher.iter(|| black_box(a.clone() + b.clone()))
    });

    c.bench_function("gauss_mul", |bencher| {
        bencher.iter(|| black_box(a.clone() * b.clone()))
    });

    c.bench_function("gauss_div", |bencher| {
        bencher.iter(|| black_box(a.clone() / b.clone()))
    });
}

fn bench_gint_gcd(c: &mut Criterion) {
    let a = Gint::new(11, 3);
    let b = Gint::new(1, 8);

    c.bench_function("gint_gcd", |bencher| {
        bencher.iter(|| black_box(Gint::gcd(&a, &b)))
    });
}

fn bench_pramana_id(c: &mut Criterion) {
    let g = Gauss::new(3, 2, 1, 4);
    c.bench_function("pramana_id", |bencher| {
        bencher.iter(|| black_box(g.pramana_id()))
    });
}

criterion_group!(
    benches,
    bench_gauss_arithmetic,
    bench_gint_gcd,
    bench_pramana_id
);
criterion_main!(benches);
