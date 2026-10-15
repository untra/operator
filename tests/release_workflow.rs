use std::fs;
use std::path::PathBuf;

use serde_json::Value;

const MAIN_PUSH: &str = "github.event_name == 'push' && github.ref == 'refs/heads/main'";

fn workflow() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/build.yaml");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_saphyr::from_str(&content)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
}

fn needs(job: &Value, dependency: &str) -> bool {
    job["needs"]
        .as_array()
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
    assert!(jobs.get("build-opr8r").is_none());
    assert!(needs(&jobs["sign-windows"], "build"));
}

#[test]
fn test_release_matrix_preserves_both_binary_artifacts() {
    let workflow = workflow();
    let jobs = &workflow["jobs"];
    let matrix = jobs["build"]["strategy"]["matrix"]["include"]
        .as_array()
        .unwrap();
    let platforms = [
        ("x86_64-unknown-linux-gnu", "linux-x86_64"),
        ("aarch64-unknown-linux-gnu", "linux-arm64"),
        ("aarch64-apple-darwin", "macos-arm64"),
        ("x86_64-pc-windows-msvc", "windows-x86_64.exe"),
        ("aarch64-pc-windows-msvc", "windows-arm64.exe"),
    ];
    assert_eq!(matrix.len(), platforms.len());
    for (target, suffix) in platforms {
        let row = matrix
            .iter()
            .find(|row| row["target"].as_str() == Some(target))
            .unwrap();
        let operator = row["artifact_name"].as_str().unwrap();
        assert_eq!(operator, format!("operator-{suffix}"));
        let opr8r = row["opr8r_artifact"].as_str().unwrap();
        assert_eq!(opr8r, operator.replacen("operator-", "opr8r-", 1));
    }
    let steps = jobs["build"]["steps"].as_array().unwrap();
    for field in ["artifact_name", "opr8r_artifact"] {
        assert!(steps.iter().any(|step| {
            step["with"]["name"].as_str() == Some(&format!("${{{{ matrix.{field} }}}}"))
                && step["uses"]
                    .as_str()
                    .is_some_and(|uses| uses.starts_with("actions/upload-artifact@"))
        }));
    }
    assert_eq!(jobs["version"]["if"].as_str(), Some(MAIN_PUSH));
    assert!(jobs["build"].get("if").is_none());
    assert!(jobs["sign-windows"].get("if").is_none());
    assert!(needs(&jobs["release"], "sign-windows"));
    assert!(needs(&jobs["docker"], "release"));
}

#[test]
fn test_checksums_are_committed_before_tagging() {
    let workflow = workflow();
    let steps = workflow["jobs"]["release"]["steps"].as_array().unwrap();
    let position = |name| {
        steps
            .iter()
            .position(|step| step["name"].as_str() == Some(name))
            .unwrap()
    };
    assert!(position("Prepare release assets") < position("Generate checksums.yml"));
    assert!(position("Generate checksums.yml") < position("Commit version bump"));
    assert!(position("Commit version bump") < position("Create tag"));
    assert!(steps[position("Commit version bump")]["run"]
        .as_str()
        .unwrap()
        .contains("docs/_data/checksums.yml"));
    assert!(!steps
        .iter()
        .any(|step| step["name"].as_str() == Some("Commit checksums")));
}

#[test]
fn test_integration_suites_share_setup_without_losing_execution() {
    let workflow = workflow();
    let jobs = &workflow["jobs"];
    assert!(jobs.get("relay-integration").is_none());
    let steps = jobs["launch-integration"]["steps"].as_array().unwrap();
    for suite in [
        "launch_integration",
        "campaign_launch_integration",
        "relay_integration",
    ] {
        assert!(steps
            .iter()
            .any(|step| step["run"]
                .as_str()
                .is_some_and(|run| run.contains(&format!("--test {suite} "))
                    && run.contains("--test-threads=1"))));
    }
    assert!(steps
        .iter()
        .any(|step| step["name"].as_str() == Some("Cleanup sessions")
            && step["if"].as_str() == Some("always()")));
}

#[test]
fn test_recomposition_preserves_validation_and_publication_gates() {
    let workflow = workflow();
    let jobs = &workflow["jobs"];
    let storybook = jobs["storybook"]["steps"].as_array().unwrap();
    assert!(!storybook.iter().any(|step| step["run"]
        .as_str()
        .is_some_and(|run| run.contains("cargo "))));
    let lint = jobs["lint-test"]["steps"].as_array().unwrap();
    assert!(lint.iter().any(|step| step["run"]
        .as_str()
        .is_some_and(|run| run.contains("scripts/check-bindings-fresh.sh"))));
    for check in ["storybook", "launch-integration", "e2e"] {
        assert!(jobs.get(check).is_some());
        assert!(!needs(&jobs["release"], check));
        assert!(!needs(&jobs["version"], check));
    }
    let opr8r: Value = serde_saphyr::from_str(
        &fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/opr8r.yaml"),
        )
        .unwrap(),
    )
    .unwrap();
    let jobs = &opr8r["jobs"];
    assert!(jobs.get("lint-test").is_some());
    assert!(jobs.get("coverage").is_some());
    assert!(jobs.get("build").is_none());
    assert!(jobs.get("sign-windows").is_none());
}
