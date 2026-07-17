use devcore_core::{AiDetector, AiSource};

#[test]
fn detects_cursor_explicit() {
    let d = AiDetector::new();
    let r = d.detect("cursor: fix auth flow", "user").unwrap();
    assert_eq!(r.source, AiSource::Cursor);
    assert!(r.confidence >= 0.7);
    assert!(r.signals.iter().any(|s| s.contains("tool_name")));
}

#[test]
fn detects_copilot_explicit() {
    let d = AiDetector::new();
    let r = d.detect("copilot: add tests", "user").unwrap();
    assert_eq!(r.source, AiSource::Copilot);
}

#[test]
fn detects_claude_explicit() {
    let d = AiDetector::new();
    let r = d.detect("claude: refactor module", "user").unwrap();
    assert_eq!(r.source, AiSource::ClaudeCode);
}

#[test]
fn detects_chatgpt_in_message() {
    let d = AiDetector::new();
    let r = d.detect("gpt-4 generated this function", "user").unwrap();
    assert_eq!(r.source, AiSource::Unknown);
    assert!(r.signals.iter().any(|s| s.contains("chatgpt")));
}

#[test]
fn detects_bot_author() {
    let d = AiDetector::new();
    let r = d.detect("update deps", "dependabot[bot]").unwrap();
    assert!(r.confidence >= 0.5);
    assert!(r.signals.iter().any(|s| s.contains("bot_author")));
}

#[test]
fn no_detection_for_human_commit() {
    let d = AiDetector::new();
    assert!(d.detect("fix: resolve race condition", "john").is_none());
}

#[test]
fn detection_with_many_files_is_higher() {
    let d = AiDetector::new();
    let r1 = d.detect("cursor: refactor", "user").unwrap();
    let r2 = d
        .detect_with_diff("cursor: refactor", "user", 15, 300, 50)
        .unwrap();
    assert!(r2.confidence > r1.confidence);
    assert!(r2.signals.iter().any(|s| s.contains("many_files")));
}

#[test]
fn detect_file_ai_marker() {
    let d = AiDetector::new();
    assert!(d.detect_file_content("// @copilot suggest").is_some());
    assert!(d
        .detect_file_content("/* ai-generated — do not edit */")
        .is_some());
    assert!(d.detect_file_content("fn main() {}").is_none());
}

#[test]
fn extract_intent_takes_first_line() {
    let d = AiDetector::new();
    assert_eq!(
        d.extract_intent("fix auth\n\nDetailed description"),
        "fix auth"
    );
}
