use std::{env, fs, path::Path};

const TARGETS_JSON: &str = "../../scripts/package/targets.json";

/// Values of every `"rust": "<triple>"` pair, whatever the line layout.
fn rust_triples(text: &str) -> Vec<String> {
    let mut triples = Vec::new();
    let mut rest = text;
    while let Some(index) = rest.find("\"rust\"") {
        rest = &rest[index + "\"rust\"".len()..];
        let Some(after_colon) = rest.trim_start().strip_prefix(':') else {
            continue;
        };
        let Some(value) = after_colon.trim_start().strip_prefix('"') else {
            continue;
        };
        let Some(end) = value.find('"') else { break };
        triples.push(value[..end].to_owned());
        rest = &value[end..];
    }
    triples
}

fn main() {
    println!("cargo:rerun-if-changed={TARGETS_JSON}");
    let text = fs::read_to_string(TARGETS_JSON).expect("read scripts/package/targets.json");
    let triples: Vec<String> = rust_triples(&text)
        .iter()
        .map(|triple| format!("{triple:?}"))
        .collect();
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
