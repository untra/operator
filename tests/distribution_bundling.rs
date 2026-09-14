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

/// True if some `COPY` line stages `src` at `dest`, regardless of any flags
/// (`--chown`, `--chmod`, `--from`) between the instruction and its operands.
fn copies(content: &str, src: &str, dest: &str) -> bool {
    content.lines().any(|line| {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("COPY ") else {
            return false;
        };
        let operands: Vec<&str> = rest
            .split_whitespace()
            .filter(|t| !t.starts_with("--"))
            .collect();
        operands == [src, dest]
    })
}

#[test]
fn test_dockerfile_stages_opr8r() {
    let content = read(&repo_root().join("Dockerfile"));

    assert!(
        copies(
            &content,
            "opr8r-linux-${TARGETARCH}",
            "/usr/local/bin/opr8r"
        ),
        "Dockerfile must COPY opr8r-linux-${{TARGETARCH}} alongside operator-linux-${{TARGETARCH}}"
    );
    assert!(
        copies(
            &content,
            "operator-linux-${TARGETARCH}",
            "/usr/local/bin/operator"
        ),
        "Dockerfile must COPY operator-linux-${{TARGETARCH}} to /usr/local/bin/operator"
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
        docker_job.contains("opr8r_artifact: opr8r-linux-x86_64")
            && docker_job.contains("opr8r_artifact: opr8r-linux-arm64"),
        "the docker matrix must download the opr8r artifact for both architectures"
    );
    assert!(
        docker_job.contains("opr8r-linux-${{ matrix.arch }}"),
        "the docker job must stage opr8r under the Dockerfile's TARGETARCH naming convention"
    );
}

/// Slice one top-level job block out of build.yaml: from its `  <name>:`
/// header up to the next two-space-indented key.
fn job_block<'a>(workflow: &'a str, name: &str) -> &'a str {
    let header = format!("\n  {name}:\n");
    let start = workflow
        .find(&header)
        .unwrap_or_else(|| panic!("build.yaml must have a top-level `{name}:` job"))
        + header.len();
    let body = &workflow[start..];
    let end = body
        .match_indices("\n  ")
        .find(|(i, _)| {
            let line = &body[i + 1..];
            !line.starts_with("   ") && line.lines().next().is_some_and(|l| l.ends_with(':'))
        })
        .map_or(body.len(), |(i, _)| i);
    &body[..end]
}

/// The whole point of splitting build/scan from publish is that the bytes that
/// were scanned are the bytes that get pushed. A `build-push-action` in the
/// publish job would rebuild them and defeat the blocking scan.
#[test]
fn test_docker_publish_job_never_rebuilds_the_scanned_image() {
    let content = read(&repo_root().join(".github/workflows/build.yaml"));
    let publish = job_block(&content, "docker-publish");

    assert!(
        !publish.contains("docker/build-push-action"),
        "docker-publish must load the scanned image archives, not rebuild them; \
         a rebuild would publish bytes that Trivy never saw"
    );
    assert!(
        publish.contains("docker load -i"),
        "docker-publish must load the exact archives exported by the scanned build jobs"
    );
    assert!(
        publish.contains("pattern: operator-image-*"),
        "docker-publish must download the per-architecture scanned image artifacts"
    );
}

/// Chart publication must trail successful image publication, so a scan failure
/// cannot leave a chart pointing at an image tag that was never published.
#[test]
fn test_chart_job_depends_on_successful_image_publication() {
    let content = read(&repo_root().join(".github/workflows/build.yaml"));
    let chart = job_block(&content, "chart");

    let needs = chart
        .lines()
        .find(|line| line.trim_start().starts_with("needs:"))
        .expect("the chart job must declare `needs:`");
    assert!(
        needs.contains("docker-publish"),
        "the chart job must depend on docker-publish so a blocked image scan also blocks \
         chart publication; found {needs:?}"
    );
    assert!(
        needs.contains("release"),
        "the chart job must depend on release to package from the version-bump commit; \
         found {needs:?}"
    );
    assert!(
        chart.contains("ref: ${{ needs.release.outputs.commit }}"),
        "the chart job must check out the release commit so chart version, appVersion \
         and image version match the published release"
    );
}

#[test]
fn test_job_block_slices_a_single_job() {
    let content = read(&repo_root().join(".github/workflows/build.yaml"));
    let publish = job_block(&content, "docker-publish");
    assert!(publish.contains("Push scanned platform images"));
    assert!(
        !publish.contains("Package and push Helm chart"),
        "job_block leaked into the following job"
    );
}
