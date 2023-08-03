use criterion::*;

fn d4(c: &mut Criterion) {
    let setting = NoiseBuilder::fbm_4d(8, 8, 8, 8).wrap();
    let mut group = c.benchmark_group("fbm4d");
    group.bench_function("scalar 4d", move |b| {
        b.iter(|| unsafe { scalar::get_4d_noise(&setting) })
    });
    group.bench_function("sse2 4d", move |b| {
        b.iter(|| unsafe { sse2::get_4d_noise(&setting) })
    });
    group.bench_function("sse41 4d", move |b| {
        b.iter(|| unsafe { sse41::get_4d_noise(&setting) })
    });
    group.bench_function("avx2 4d", move |b| {
        b.iter(|| unsafe { avx2::get_4d_noise(&setting) })
    });
    group
        .sample_size(10)
        .warm_up_time(Duration::from_millis(1))
        .measurement_time(Duration::from_secs(5));
}

criterion_group!(benches, d4);
criterion_main!(benches);
