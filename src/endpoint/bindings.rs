//! Generated endpoint-protocol artifacts under `bindings/endpoint/`.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use super::csharp::endpoint_protocol_csharp;
use super::protocol::{fixtures, DownlinkFrame, UplinkFrame};

pub const BINDINGS_DIR: &str = "bindings/endpoint";
pub const SCHEMA_FILE: &str = "protocol.schema.json";
pub const CSHARP_FILE: &str = "EndpointProtocol.g.cs";
pub const FIXTURES_DIR: &str = "fixtures";

const UPLINK_PREFIX: &str = "uplink";
const DOWNLINK_PREFIX: &str = "downlink";
const FIXTURE_EXT: &str = "json";

fn fixture_path(direction: &str, name: &str) -> String {
    format!("{FIXTURES_DIR}/{direction}-{name}.{FIXTURE_EXT}")
}

fn schema_json() -> String {
    let schema = serde_json::json!({
        "uplink": schemars::schema_for!(UplinkFrame),
        "downlink": schemars::schema_for!(DownlinkFrame),
    });
    serde_json::to_string_pretty(&schema).expect("schema serializes") + "\n"
}

/// Every generated file, keyed by its path relative to [`BINDINGS_DIR`].
/// Fixtures are compact single-line JSON so the C# client can compare bytes.
pub fn generated_files() -> BTreeMap<String, String> {
    let mut files = BTreeMap::from([
        (SCHEMA_FILE.to_string(), schema_json()),
        (CSHARP_FILE.to_string(), endpoint_protocol_csharp()),
    ]);
    let f = fixtures();
    for (name, frame) in &f.uplink {
        let json = serde_json::to_string(frame).expect("fixture serializes");
        files.insert(fixture_path(UPLINK_PREFIX, name), json);
    }
    for (name, frame) in &f.downlink {
        let json = serde_json::to_string(frame).expect("fixture serializes");
        files.insert(fixture_path(DOWNLINK_PREFIX, name), json);
    }
    files
}

fn stale_fixtures(dir: &Path, expected: &BTreeMap<String, String>) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir.join(FIXTURES_DIR)) else {
        return Vec::new();
    };
    let mut stale: Vec<String> = entries
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .map(|name| format!("{FIXTURES_DIR}/{name}"))
        .filter(|rel| !expected.contains_key(rel))
        .collect();
    stale.sort();
    stale
}

/// Human-readable problems with `dir` versus the generated output; empty when
/// up to date. Unexpected files in the fixtures directory count as stale.
pub fn outdated(dir: &Path) -> Vec<String> {
    let expected = generated_files();
    let mut problems: Vec<String> = expected
        .iter()
        .filter_map(|(rel, content)| match fs::read_to_string(dir.join(rel)) {
            Ok(existing) if &existing == content => None,
            Ok(_) => Some(format!("{rel} (outdated)")),
            Err(_) => Some(format!("{rel} (missing)")),
        })
        .collect();
    problems.extend(
        stale_fixtures(dir, &expected)
            .into_iter()
            .map(|rel| format!("{rel} (stale)")),
    );
    problems
}

/// Write every generated file under `dir` and delete stale fixtures.
pub fn write(dir: &Path) -> io::Result<()> {
    let expected = generated_files();
    fs::create_dir_all(dir.join(FIXTURES_DIR))?;
    for rel in stale_fixtures(dir, &expected) {
        fs::remove_file(dir.join(rel))?;
    }
    for (rel, content) in &expected {
        fs::write(dir.join(rel), content)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn includes_schema_csharp_and_every_fixture() {
        let files = generated_files();
        assert!(files.contains_key(SCHEMA_FILE));
        assert!(files.contains_key(CSHARP_FILE));
        let f = fixtures();
        for (name, _) in &f.uplink {
            assert!(files.contains_key(&format!("{FIXTURES_DIR}/uplink-{name}.json")));
        }
        for (name, _) in &f.downlink {
            assert!(files.contains_key(&format!("{FIXTURES_DIR}/downlink-{name}.json")));
        }
        assert_eq!(files.len(), 2 + f.uplink.len() + f.downlink.len());
    }

    #[test]
    fn fixture_files_parse_back_to_their_frames() {
        let files = generated_files();
        for (name, frame) in fixtures().uplink {
            let text = &files[&format!("{FIXTURES_DIR}/uplink-{name}.json")];
            assert_eq!(serde_json::from_str::<UplinkFrame>(text).unwrap(), frame);
        }
        for (name, frame) in fixtures().downlink {
            let text = &files[&format!("{FIXTURES_DIR}/downlink-{name}.json")];
            assert_eq!(serde_json::from_str::<DownlinkFrame>(text).unwrap(), frame);
        }
    }

    #[test]
    fn schema_holds_both_directions() {
        let schema: serde_json::Value =
            serde_json::from_str(&generated_files()[SCHEMA_FILE]).unwrap();
        assert_eq!(schema["uplink"]["title"], "UplinkFrame");
        assert_eq!(schema["downlink"]["title"], "DownlinkFrame");
    }

    #[test]
    fn written_dir_is_up_to_date() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!outdated(dir.path()).is_empty());
        write(dir.path()).unwrap();
        assert_eq!(outdated(dir.path()), Vec::<String>::new());
    }

    #[test]
    fn reports_changed_and_stale_files_and_write_removes_stale() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path()).unwrap();
        std::fs::write(dir.path().join(CSHARP_FILE), "// edited").unwrap();
        let stale = dir.path().join(FIXTURES_DIR).join("uplink-gone.json");
        std::fs::write(&stale, "{}").unwrap();

        let report = outdated(dir.path());
        assert!(report.iter().any(|r| r.contains(CSHARP_FILE)), "{report:?}");
        assert!(report.iter().any(|r| r.contains("uplink-gone.json")), "{report:?}");

        write(dir.path()).unwrap();
        assert!(!stale.exists());
        assert!(outdated(dir.path()).is_empty());
    }
}
