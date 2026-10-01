use std::fs;
use std::path::PathBuf;

use serde_yaml::Value;

const MAIN_PUSH: &str = "github.event_name == 'push' && github.ref == 'refs/heads/main'";

fn workflow() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/build.yaml");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_yaml::from_str(&content)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
}

fn needs(job: &Value, dependency: &str) -> bool {
    job["needs"]
        .as_sequence()
        .is_some_and(|needs| needs.iter().any(|need| need.as_str() == Some(dependency)))
}

#[test]
fn test_apple_preflight_starts_on_merge_and_gates_release_builds() {
    let workflow = workflow();
    let jobs = &workflow["jobs"];
    let preflight = &jobs["preflight-apple"];

    assert_eq!(preflight["if"].as_str(), Some(MAIN_PUSH));
    assert!(preflight.get("needs").is_none());
    assert!(needs(&jobs["version"], "preflight-apple"));
    assert!(needs(&jobs["build"], "version"));
    assert!(needs(&jobs["build-opr8r"], "version"));
}
