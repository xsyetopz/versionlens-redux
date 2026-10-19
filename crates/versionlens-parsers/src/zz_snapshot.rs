use crate::{
    DocumentInput, MANIFESTS, parse_document_as_manifest_with_dependency_paths,
    parse_project_versions_as_manifest,
};
use std::fmt::Write;
use std::path::Path;

fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out);
        } else {
            out.push(path);
        }
    }
}

const EXTRA: &[&str] = &[
    "name = \"App\"\nuuid = \"abc\"\nversion = \"0.1.0\"\n\n[deps]\nJSON = \"682c06a0\"\nPlots = \"91a5bcdd\"\n\n[compat]\nJSON = \"0.21\"\njulia = \"1.6\"\n\n[sources]\nPlots = {url = \"https://x/Plots.jl\", rev = \"main\"}\nLocal = {path = \"../Local\"}\n",
    "name = \"App\"\nversion = \"0.1.0\"\n[deps]\nA = \"1\"  # c\n[compat]\nB = \"2\"\n",
    "[[deps.JSON]]\nuuid = \"x\"\nversion = \"0.21.4\"\n\n[[deps.Local]]\npath = \"/home/Local\"\nuuid = \"y\"\n\n[[deps.Git]]\nrepo-url = \"https://x/y\"\nversion = \"1\"\n",
    "resolver: lts-21.25\npackages:\n- .\nextra-deps:\n  - acme-1.2.3\n  - foo-bar-0.1@sha256:abc,123\n  - github: a/b\n  - ./local\n  - git: https://x/y.git\n    commit: abc\nflags: {}\n",
    "resolver: ghc-9.4.7\nextra-deps:\n- text-2.0\n",
    "resolver: nightly-2024-01-01\nextra-deps:\n- a-1.0   # c\n- \"q-2.0\"\n- 'r-3.0'\n",
    "resolver: lts-22.0\nextra-deps: []\n",
    "name: demo\nversion: 1.0.0\ndependencies:\n  a: ^1.0.0\nworkspace:\n  - pkg\n",
];

#[test]
#[ignore = "writes the snapshot named by SNAPSHOT_OUT"]
fn write_snapshot() {
    let Ok(out_path) = std::env::var("SNAPSHOT_OUT") else {
        return;
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures");
    let mut files = Vec::new();
    walk(&root, &mut files);
    files.sort();
    let mut inputs: Vec<(String, String)> = Vec::new();
    for file in files {
        if let Ok(text) = std::fs::read_to_string(&file) {
            inputs.push((file.display().to_string(), text));
        }
    }
    for (index, text) in EXTRA.iter().enumerate() {
        inputs.push((format!("extra-{index}"), (*text).to_owned()));
    }
    let mut out = String::new();
    for (name, text) in inputs {
        let name = name.rsplit("tests/fixtures/").next().unwrap().to_owned();
        let input = DocumentInput::new("file:///x/doc", "plaintext", &text, None);
        for row in MANIFESTS {
            let deps =
                parse_document_as_manifest_with_dependency_paths::<&str>(&input, row.kind, &[]);
            let own = parse_project_versions_as_manifest(&input, row.kind);
            let user = parse_document_as_manifest_with_dependency_paths(
                &input,
                row.kind,
                &[
                    "version",
                    "dependencies",
                    "package",
                    "deps",
                    "compat",
                    "sources",
                    "extra-deps",
                    "resolver",
                ],
            );
            if !deps.is_empty() || !own.is_empty() || !user.is_empty() {
                writeln!(out, "{name} {:?}\n  d={deps:?}\n  o={own:?}", row.kind).unwrap();
                writeln!(out, "  u={user:?}").unwrap();
            }
        }
    }
    std::fs::write(out_path, out).unwrap();
}
