use std::{env, fs, path::Path};

const TARGETS_JSON: &str = "../../scripts/package/targets.json";

fn main() {
    println!("cargo:rerun-if-changed={TARGETS_JSON}");
    let text = fs::read_to_string(TARGETS_JSON).expect("read scripts/package/targets.json");
    let mut triples = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.split("\"rust\"").nth(1) {
            let triple = rest.split('"').nth(1).expect("rust triple value");
            triples.push(format!("{triple:?}"));
        }
    }
    assert!(!triples.is_empty(), "targets.json lists no rust targets");
    let out = Path::new(&env::var("OUT_DIR").expect("OUT_DIR")).join("targets.rs");
    fs::write(
        out,
        format!(
            "const RELEASE_TARGETS: &[&str] = &[{}];\n",
            triples.join(", ")
        ),
    )
    .expect("write targets.rs");
}
