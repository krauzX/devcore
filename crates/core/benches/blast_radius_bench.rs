use criterion::{black_box, criterion_group, criterion_main, Criterion};
use devcore_core::BlastRadiusAnalyzer;
use std::fs;
use tempfile::TempDir;

fn setup_project(dir: &std::path::Path, n_files: usize) {
    for i in 0..n_files {
        let content = format!("import './file_{}';\nexport function f{}() {{}}", (i + 1) % n_files, i);
        fs::write(dir.join(format!("file_{}.ts", i)), content).unwrap();
    }
}

fn bench_blast_radius_small(c: &mut Criterion) {
    let tmp = TempDir::new().unwrap();
    setup_project(tmp.path(), 50);
    let mut analyzer = BlastRadiusAnalyzer::new(tmp.path());
    analyzer.build_graph().unwrap();
    c.bench_function("blast_radius_50_files", |b| {
        b.iter(|| analyzer.analyze(black_box("file_0.ts")))
    });
}

fn bench_blast_radius_large(c: &mut Criterion) {
    let tmp = TempDir::new().unwrap();
    setup_project(tmp.path(), 500);
    let mut analyzer = BlastRadiusAnalyzer::new(tmp.path());
    analyzer.build_graph().unwrap();
    c.bench_function("blast_radius_500_files", |b| {
        b.iter(|| analyzer.analyze(black_box("file_0.ts")))
    });
}

fn bench_build_graph(c: &mut Criterion) {
    let tmp = TempDir::new().unwrap();
    setup_project(tmp.path(), 200);
    c.bench_function("build_graph_200_files", |b| {
        b.iter(|| {
            let mut a = BlastRadiusAnalyzer::new(tmp.path());
            a.build_graph().unwrap();
        })
    });
}

criterion_group!(benches, bench_blast_radius_small, bench_blast_radius_large, bench_build_graph);
criterion_main!(benches);
