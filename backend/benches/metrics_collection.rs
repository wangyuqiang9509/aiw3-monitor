// 性能基准测试
// TODO: 实现性能测试

use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_metrics_collection(_c: &mut Criterion) {
    // TODO: 添加性能测试
}

criterion_group!(benches, benchmark_metrics_collection);
criterion_main!(benches);
