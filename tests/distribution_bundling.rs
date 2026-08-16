//! Asserts the Docker and Coder distributions stage both halves of the
//! operator/opr8r server-client pair, not just the `operator` server.
//!
//! `opr8r` is the client binary agent sessions use to report step completion
//! back to `operator` for multi-step ticket workflows. It ships as a
//! separate release artifact (`opr8r/Cargo.toml`, kept in lockstep with
//! `VERSION` by `version_parity.rs`), but the Dockerfile, the Coder module's
//! install script, and the CI job that stages the Docker build context all
//! only reference `operator` unless they're kept in sync by hand. These
//! tests fail if `opr8r` staging drifts out of any of the three.

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

#[test]
fn test_dockerfile_stages_opr8r() {
    let content = read(&repo_root().join("Dockerfile"));

    assert!(
        content.contains("COPY opr8r-linux-${TARGETARCH} /usr/local/bin/opr8r"),
        "Dockerfile must COPY opr8r-linux-${{TARGETARCH}} alongside operator-linux-${{TARGETARCH}}"
    );
    assert!(
        content.contains(r#"RUN ["/usr/local/bin/opr8r", "--version"]"#),
        "Dockerfile must smoke-test the staged opr8r binary at build time, like it does for operator"
    );
}

#[test]
fn test_coder_run_script_installs_opr8r() {
    let content = read(&repo_root().join("coder-module/run.sh"));

    assert!(
        content.contains("opr8r-$PLATFORM"),
        "coder-module/run.sh must download an opr8r-$PLATFORM release asset, like it does for operator"
    );
    assert!(
        content.contains("$CODER_SCRIPT_BIN_DIR/opr8r"),
        "coder-module/run.sh must symlink opr8r into $CODER_SCRIPT_BIN_DIR, like it does for operator"
    );

    // Terraform's templatefile only treats `${` specially, so `$$` is emitted
    // verbatim and `$$(cmd)` is a bash syntax error in the rendered script.
    // Only `$${` (escaping a literal `${`) is legitimate here.
    let over_escaped: Vec<_> = content
        .lines()
        .enumerate()
        .filter(|(_, l)| l.contains("$$") && l.replace("$${", "").contains("$$"))
        .map(|(i, l)| format!("  line {}: {}", i + 1, l.trim()))
        .collect();
    assert!(
        over_escaped.is_empty(),
        "coder-module/run.sh over-escapes shell variables; `$$` renders literally and breaks the script.\nUse `$VAR` / `$(cmd)`, reserving `$${{` for a literal `${{`:\n{}",
        over_escaped.join("\n")
    );
}

#[test]
fn test_docker_ci_job_stages_opr8r_artifacts() {
    let content = read(&repo_root().join(".github/workflows/build.yaml"));

    let docker_job_start = content
        .find("\n  docker:\n")
        .expect("build.yaml must have a top-level `docker:` job");
    let docker_job = &content[docker_job_start..];

    assert!(
        docker_job.contains("opr8r-linux-*"),
        "the docker job must download opr8r-linux-* release artifacts, like it does for operator-linux-*"
    );
    assert!(
        docker_job.contains("opr8r-linux-amd64") && docker_job.contains("opr8r-linux-arm64"),
        "the docker job must stage opr8r-linux-amd64/arm64 into the build context, like it does for operator"
    );
}
