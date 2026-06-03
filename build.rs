use std::path::Path;

fn main() {
    if std::env::var("CARGO_FEATURE_EMBED_UI").is_err() {
        return;
    }

    let ui_dist = Path::new("ui/dist");
    let index = ui_dist.join("index.html");

    // Create a minimal placeholder so rust-embed's derive macro doesn't fail
    // during CI `--all-features` when the UI hasn't been built yet.
    if !index.exists() {
        std::fs::create_dir_all(ui_dist).expect("create ui/dist");
        std::fs::write(
            &index,
            "<!doctype html><html><body><!-- operator:placeholder run `cd ui && bun run build` --></body></html>",
        )
        .expect("write placeholder index.html");
        println!(
            "cargo:warning=ui/dist/index.html is a placeholder — run `cd ui && bun run build` for real UI"
        );
    }

    // Size gate: walk ui/dist, sum file sizes, fail if over 15MB uncompressed
    let total = walk_dir_size(ui_dist);
    assert!(
        total <= 15_728_640,
        "UI dist is {}B ({:.1}MB) — exceeds 15MB uncompressed budget",
        total,
        total as f64 / 1_048_576.0
    );

    println!("cargo:rerun-if-changed=ui/dist");
}

fn walk_dir_size(dir: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += walk_dir_size(&path);
            } else if let Ok(meta) = path.metadata() {
                total += meta.len();
            }
        }
    }
    total
}
