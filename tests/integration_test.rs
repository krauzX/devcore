use devcore_academic::{grade_to_points, SemesterStore};
use devcore_challenges::pack::builtin_packs;
use devcore_challenges::ChallengeEngine;
use devcore_core::{DevCoreConfig, Store};
use tempfile::tempdir;

#[test]
fn test_core_store_set_get_delete() {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();

    store.set("test_key", "test_value").unwrap();
    assert_eq!(store.get("test_key").unwrap(), Some("test_value".into()));

    store.delete("test_key").unwrap();
    assert_eq!(store.get("test_key").unwrap(), None);
}

#[test]
fn test_config_load_default() {
    let dir = tempdir().unwrap();
    let config = DevCoreConfig::load(dir.path()).unwrap();
    let default = DevCoreConfig::default();

    assert_eq!(config.institution, default.institution);
    assert_eq!(config.program, default.program);
    assert_eq!(config.batch, default.batch);
    assert_eq!(config.total_semesters, default.total_semesters);
}

#[test]
fn test_grade_to_points_all_grades() {
    assert_eq!(grade_to_points("O"), 10.0);
    assert_eq!(grade_to_points("A+"), 9.0);
    assert_eq!(grade_to_points("A"), 8.0);
    assert_eq!(grade_to_points("B+"), 7.0);
    assert_eq!(grade_to_points("B"), 6.0);
    assert_eq!(grade_to_points("C"), 5.0);
    assert_eq!(grade_to_points("D"), 4.0);
    assert_eq!(grade_to_points("F"), 0.0);
}

#[test]
fn test_builtin_packs_count() {
    let packs = builtin_packs();
    assert_eq!(packs.len(), 5);
}

#[test]
fn test_challenge_engine_new() {
    let dir = tempdir().unwrap();
    let engine = ChallengeEngine::new(dir.path());
    assert!(dir.path().join("challenges/packs").exists());
    assert!(engine.list_available().len() > 0);
}

#[test]
fn test_semester_store_open() {
    let dir = tempdir().unwrap();
    let store = SemesterStore::open(dir.path()).unwrap();
    let sems = store.list_semesters().unwrap();
    assert_eq!(sems.len(), 8);
}
