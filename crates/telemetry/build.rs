mod registry;

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../telemetry");
    println!("cargo:rerun-if-changed={}", root.display());
    for path in registry::model_files(&root).expect("telemetry registry files") {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    let registry = registry::Registry::load(&root).expect("valid telemetry registry");
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR"));
    std::fs::write(output.join("catalog.rs"), registry.rust()).expect("write catalog");
    std::fs::write(output.join("catalog.md"), registry.markdown()).expect("write catalog docs");
}
