use devcore_core::BlastRadiusAnalyzer;
use std::fs;
use tempfile::TempDir;

fn create_test_project(dir: &std::path::Path) {
    fs::write(
        dir.join("mod_a.ts"),
        "import './mod_b'\nexport function a() {}",
    )
    .unwrap();
    fs::write(
        dir.join("mod_b.ts"),
        "import './mod_c'\nexport function b() {}",
    )
    .unwrap();
    fs::write(dir.join("mod_c.ts"), "export function c() {}").unwrap();
    fs::write(dir.join("main.ts"), "import './mod_a'\nimport './mod_b'").unwrap();
}

#[test]
fn finds_direct_dependents() {
    let tmp = TempDir::new().unwrap();
    create_test_project(tmp.path());
    let mut analyzer = BlastRadiusAnalyzer::new(tmp.path());
    analyzer.build_graph().unwrap();
    let br = analyzer.analyze("mod_b.ts");
    assert!(br.direct_dependents.contains(&"mod_a.ts".to_string()));
    assert!(br.direct_dependents.contains(&"main.ts".to_string()));
}

#[test]
fn finds_indirect_dependents() {
    let tmp = TempDir::new().unwrap();
    create_test_project(tmp.path());
    let mut analyzer = BlastRadiusAnalyzer::new(tmp.path());
    analyzer.build_graph().unwrap();
    let br = analyzer.analyze("mod_c.ts");
    assert!(br.indirect_dependents.contains(&"mod_a.ts".to_string()));
    assert!(br.indirect_dependents.contains(&"main.ts".to_string()));
}

#[test]
fn isolated_file_has_no_dependents() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("isolated.rs"), "fn main() {}").unwrap();
    let mut analyzer = BlastRadiusAnalyzer::new(tmp.path());
    analyzer.build_graph().unwrap();
    let br = analyzer.analyze("isolated.rs");
    assert!(br.direct_dependents.is_empty());
    assert!(br.indirect_dependents.is_empty());
}

#[test]
fn list_all_files_returns_sorted() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("z.rs"), "").unwrap();
    fs::write(tmp.path().join("a.rs"), "").unwrap();
    fs::write(tmp.path().join("m.rs"), "").unwrap();
    let mut analyzer = BlastRadiusAnalyzer::new(tmp.path());
    analyzer.build_graph().unwrap();
    let files = analyzer.list_all_files();
    assert_eq!(files, vec!["a.rs", "m.rs", "z.rs"]);
}
