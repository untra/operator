//! Chart packaging and rendering guarantees that `helm lint` does not cover.
//!
//! Skips cleanly when `helm` is absent so a contributor without it can still
//! run the suite; CI always has it (`azure/setup-helm` in the lint-test job).

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn chart_dir() -> PathBuf {
    repo_root().join("charts/operator")
}

fn helm_available() -> bool {
    Command::new("helm")
        .arg("version")
        .output()
        .is_ok_and(|out| out.status.success())
}

/// Run helm and return stdout, asserting it succeeded.
fn helm(args: &[&str]) -> String {
    let out = Command::new("helm")
        .args(args)
        .output()
        .expect("failed to run helm");
    assert!(
        out.status.success(),
        "helm {args:?} failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// `helm install --dry-run` is the only way to render NOTES.txt; `helm template`
/// skips it.
fn rendered_notes(args: &[&str]) -> String {
    let mut full = vec!["install", "op"];
    let dir = chart_dir();
    let dir = dir.to_str().unwrap();
    full.push(dir);
    full.push("--dry-run");
    full.extend_from_slice(args);
    let output = helm(&full);
    output
        .split_once("NOTES:")
        .map(|(_, notes)| notes.trim().to_string())
        .expect("helm install --dry-run must render NOTES.txt")
}

/// The README is the packaged install guide - `helm show readme` and the GHCR
/// listing both read it out of the archive. Packaging it is easy to lose.
#[test]
fn test_readme_is_packaged_into_the_chart_archive() {
    if !helm_available() {
        eprintln!("skipping: helm not on PATH");
        return;
    }
    let dest = tempfile::TempDir::new().unwrap();
    helm(&[
        "package",
        chart_dir().to_str().unwrap(),
        "--destination",
        dest.path().to_str().unwrap(),
    ]);

    let archive = std::fs::read_dir(dest.path())
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.extension().is_some_and(|ext| ext == "tgz"))
        .expect("helm package must produce a .tgz");

    let listing = Command::new("tar")
        .arg("-tzf")
        .arg(&archive)
        .output()
        .expect("failed to list the chart archive");
    let listing = String::from_utf8_lossy(&listing.stdout);

    assert!(
        listing.lines().any(|entry| entry == "operator/README.md"),
        "operator/README.md is missing from the packaged chart; archive contains:\n{listing}"
    );
}

/// The ingress branch of NOTES dereferences `.Values.ingress.tls.secretName`,
/// which the default render never reaches.
#[test]
fn test_notes_render_a_usable_setup_url_on_both_branches() {
    if !helm_available() {
        eprintln!("skipping: helm not on PATH");
        return;
    }

    let ingress = rendered_notes(&[
        "--set",
        "ingress.enabled=true",
        "--set",
        "ingress.host=op.example.com",
        "--set",
        "ingress.tls.secretName=op-tls",
    ]);
    assert!(
        ingress.contains("https://op.example.com/setup"),
        "TLS ingress NOTES must link the https /setup URL:\n{ingress}"
    );

    let forwarded = rendered_notes(&["--namespace", "ops"]);
    assert!(
        forwarded.contains("kubectl -n ops port-forward service/op-operator 7008:7008"),
        "port-forward command must carry the release namespace:\n{forwarded}"
    );
    assert!(
        forwarded.contains("http://127.0.0.1:7008/setup"),
        "NOTES must give the local /setup URL when no ingress is configured:\n{forwarded}"
    );
}

/// `publicUrl` only changes generated links; it provisions no Ingress. Saying
/// otherwise sends operators looking for a route that does not exist.
#[test]
fn test_notes_separate_public_url_from_ingress() {
    if !helm_available() {
        eprintln!("skipping: helm not on PATH");
        return;
    }
    let notes = rendered_notes(&["--set", "publicUrl=https://op.example.com"]);
    assert!(
        notes.contains("publicUrl does not create an Ingress"),
        "NOTES must state that publicUrl creates no Ingress:\n{notes}"
    );
    assert!(
        notes.contains("http://127.0.0.1:7008/setup"),
        "the local /setup URL must still be shown alongside configured public access:\n{notes}"
    );
}

/// The packaged README is not Jekyll-rendered, so its value table is hand-kept
/// and drifts silently from values.yaml.
#[test]
fn test_packaged_readme_documents_the_current_default_values() {
    let values = std::fs::read_to_string(chart_dir().join("values.yaml")).unwrap();
    let readme = std::fs::read_to_string(chart_dir().join("README.md")).unwrap();

    for key in [
        "extraVolumes",
        "extraVolumeMounts",
        "terminationGracePeriodSeconds",
        "shutdownDrainSeconds",
        "shutdownCleanupSeconds",
    ] {
        assert!(
            values.contains(&format!("\n{key}:")),
            "{key} must exist in values.yaml"
        );
        assert!(
            readme.contains(key),
            "the packaged README must document the {key} value"
        );
    }

    for (key, default) in [
        ("terminationGracePeriodSeconds", "90"),
        ("shutdownDrainSeconds", "60"),
        ("shutdownCleanupSeconds", "15"),
    ] {
        let declared = values
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{key}:")))
            .map(str::trim)
            .unwrap_or_else(|| panic!("{key} must be a top-level value"));
        assert_eq!(
            declared, default,
            "values.yaml default for {key} changed; update the packaged README table too"
        );
        assert!(
            readme.contains(&format!("`{default}`")),
            "the README value table must show {key}'s default of {default}"
        );
    }
}

/// Chart, appVersion and the rendered image tag must move together; the release
/// job rewrites all three from one version and CI verifies the published chart
/// against them.
#[test]
fn test_rendered_image_tag_matches_chart_app_version() {
    if !helm_available() {
        eprintln!("skipping: helm not on PATH");
        return;
    }
    let chart = std::fs::read_to_string(chart_dir().join("Chart.yaml")).unwrap();
    let app_version = chart
        .lines()
        .find_map(|line| line.trim().strip_prefix("appVersion:"))
        .map(|value| value.trim().trim_matches('"').to_string())
        .expect("Chart.yaml must declare appVersion");

    let rendered = helm(&["template", "op", chart_dir().to_str().unwrap()]);
    let image = rendered
        .lines()
        .find_map(|line| line.trim().strip_prefix("image:"))
        .map(|value| value.trim().trim_matches('"').to_string())
        .expect("the StatefulSet must render a container image");

    assert_eq!(
        image,
        format!("untra/operator:{app_version}"),
        "the chart must deploy its own appVersion, never a floating tag"
    );
}

/// Kept honest against the helper above: a chart path that does not exist must
/// not silently pass as "helm unavailable".
#[test]
fn test_chart_directory_exists() {
    assert!(
        Path::new(&chart_dir()).join("Chart.yaml").exists(),
        "charts/operator/Chart.yaml is missing"
    );
}
