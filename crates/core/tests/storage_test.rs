use devcore_core::{Store, ChangeReceipt, AiSource};
use chrono::Utc;
use tempfile::TempDir;

fn test_receipt() -> ChangeReceipt {
    ChangeReceipt {
        id: "test-1".to_string(),
        commit_oid: "abc123".to_string(),
        timestamp: Utc::now(),
        is_ai_generated: true,
        ai_source: Some(AiSource::Cursor),
        intent: "test commit".to_string(),
        files_changed: vec![],
        decisions: vec![],
        risks: vec![],
        blast_radius: Default::default(),
        risk_score: 5,
    }
}

#[test]
fn save_and_retrieve_receipt() {
    let tmp = TempDir::new().unwrap();
    let store = Store::open(tmp.path()).unwrap();
    let receipt = test_receipt();
    store.save_receipt(&receipt).unwrap();
    let retrieved = store.get_receipt("abc123").unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().intent, "test commit");
}

#[test]
fn recent_receipts_returns_descending_order() {
    let tmp = TempDir::new().unwrap();
    let store = Store::open(tmp.path()).unwrap();
    for i in 0..5 {
        let mut r = test_receipt();
        r.id = format!("test-{}", i);
        r.commit_oid = format!("commit-{}", i);
        store.save_receipt(&r).unwrap();
    }
    let receipts = store.recent_receipts(3).unwrap();
    assert_eq!(receipts.len(), 3);
}
