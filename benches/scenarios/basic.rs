use crate::common::{
    setup::{BenchParams, TEMP_DIR, init_temp_dir, setup_entries},
    utils::{BASE_DIR, run_binary_with_args},
};
use criterion::{BenchmarkId, Criterion};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn process_directory(path: &Path) {
    let binary_path = PathBuf::from(BASE_DIR).join("target/release/project-finder");
    Command::new(binary_path)
        .arg(path)
        .output()
        .expect("failed to run binary");
}

pub fn benchmark_basic(c: &mut Criterion) {
    init_temp_dir();
    let temp_dir = TEMP_DIR.get().unwrap().path();

    let params = vec![
        BenchParams {
            depth: 1,
            max_results: 0,
            verbose: false,
        },
        BenchParams {
            depth: 5,
            max_results: 0,
            verbose: false,
        },
    ];

    let mut group = c.benchmark_group("basic_scenarios");

    group.bench_function("process_directory", |b| {
        b.iter(|| process_directory(temp_dir))
    });

    for param in params {
        let id = BenchmarkId::new(
            format!(
                "depth{}_max{}_verbose{}",
                param.depth, param.max_results, param.verbose
            ),
            param.depth,
        );

        group.bench_with_input(id, &param, |b, param| {
            b.iter(|| {
                run_binary_with_args(temp_dir, param.depth, param.max_results, param.verbose)
                    .expect("Failed to run binary")
            })
        });
    }

    group.finish();
}
