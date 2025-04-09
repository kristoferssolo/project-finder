use crate::common::{
    default,
    setup::{BenchParams, TEMP_DIR, init_temp_dir},
    utils::run_binary_with_args,
};
use criterion::{BenchmarkId, Criterion};

pub fn benchmark_basic(c: &mut Criterion) {
    init_temp_dir();
    let temp_dir = TEMP_DIR.get().unwrap().path();

    let params = vec![
        BenchParams {
            depth: Some(1),
            ..Default::default()
        },
        BenchParams {
            depth: Some(5),
            ..default()
        },
        BenchParams {
            depth: Some(10),
            ..default()
        },
        BenchParams {
            depth: Some(10),
            max_results: Some(10),
            ..default()
        },
    ];

    let mut group = c.benchmark_group("basic_scenarios");

    for (idx, param) in params.iter().enumerate() {
        let id = BenchmarkId::new(format!("with_param_{idx}"), &param);

        group.bench_with_input(id, &param, |b, param| {
            b.iter(|| run_binary_with_args(temp_dir, param).expect("Failed to run binary"))
        });
    }

    group.finish();
}
