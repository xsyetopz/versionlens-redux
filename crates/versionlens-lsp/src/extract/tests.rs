use super::*;
use crate::catalog::file_type;

fn deps(path: &str, text: &str) -> Vec<(String, String)> {
    let file = file_type(path).unwrap_or_else(|| panic!("{path} not recognised"));
    extract(file, text)
        .into_iter()
        .map(|d| {
            assert_eq!(text[d.span.clone()], d.req, "span of {}", d.name);
            (d.name, d.req)
        })
        .collect()
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
        .collect()
}

#[test]
fn reads_requirements_txt() {
    let text = "-r base.txt\n# c\nrequests==2.31.0\nDjango[argon2] >=4.2, <5 ; python_version>'3'\nflask\n-e git+https://x\nnumpy~=1.26 # pin\n";
    let found = deps("proj/requirements-dev.txt", text);
    assert_eq!(
        found,
        pairs(&[
            ("requests", "==2.31.0"),
            ("Django", ">=4.2, <5"),
            ("flask", ""),
            ("numpy", "~=1.26")
        ])
    );
}

#[test]
fn reads_hxml_libraries() {
    let text = "-cp src\n-lib heaps:1.10.0\n--library format\n-L hxcpp:4.3.2\n-main Main\n";
    assert_eq!(
        deps("libraries.hxml", text),
        pairs(&[("heaps", "1.10.0"), ("format", ""), ("hxcpp", "4.3.2")])
    );
}

#[test]
fn reads_package_json_and_local_paths() {
    let text = r#"{"dependencies":{"a":"^1.0.0","b":"workspace:*","c":"file:../c","d":"github:x/y"},"packageManager":"bun@1.4.3"}"#;
    assert_eq!(
        deps("package.json", text),
        pairs(&[("a", "^1.0.0"), ("c", "file:../c"), ("bun", "1.4.3")])
    );
    assert_eq!(registries("package.json", text)[1].1, DIRECTORY);
}

#[test]
fn reads_cargo_tables_and_targets() {
    let text = "[dependencies]\nserde = { version = \"1\" }\nlocal = { path = \"../x\" }\n[target.'cfg(unix)'.dependencies]\nlibc = \"0.2\"\n";
    assert_eq!(
        deps("Cargo.toml", text),
        pairs(&[("serde", "1"), ("local", "../x"), ("libc", "0.2")])
    );
}

#[test]
fn reads_pom_with_properties() {
    let text = "<project><properties><junit.version>5.10.0</junit.version></properties><dependencies><dependency><groupId>org.junit</groupId><artifactId>junit</artifactId><version>${junit.version}</version></dependency></dependencies></project>";
    assert_eq!(
        deps("pom.xml", text),
        pairs(&[("org.junit:junit", "5.10.0")])
    );
}

#[test]
fn reads_yaml_records_and_specs() {
    let chart = "dependencies:\n  - name: redis\n    version: 17.0.0\n    repository: https://charts.bitnami.com/bitnami\n";
    let file = file_type("Chart.yaml").unwrap();
    let found = extract(file, chart);
    assert_eq!(
        found[0].source.as_deref(),
        Some("https://charts.bitnami.com/bitnami")
    );
    let workflow =
        "jobs:\n  build:\n    steps:\n      - uses: actions/checkout@v4\n      - uses: ./local\n";
    assert_eq!(
        deps(".github/workflows/ci.yml", workflow),
        pairs(&[("actions/checkout", "v4")])
    );
}

#[test]
fn reads_pyproject_and_deno() {
    let pyproject = "[project]\ndependencies = [\"httpx>=0.27\", \"rich\"]\n[tool.poetry.dependencies]\npython = \"^3.11\"\nclick = \"^8.1\"\n";
    assert_eq!(
        deps("pyproject.toml", pyproject),
        pairs(&[("httpx", ">=0.27"), ("rich", ""), ("click", "^8.1")])
    );
    let deno = r#"{"imports":{"@std/path":"jsr:@std/path@^1.0.0","chalk":"npm:chalk@5"}}"#;
    let file = file_type("deno.json").unwrap();
    let found: Vec<_> = extract(file, deno)
        .into_iter()
        .map(|d| (d.name, d.registry))
        .collect();
    assert_eq!(
        found,
        [("@std/path".to_owned(), "jsr"), ("chalk".to_owned(), "npm")]
    );
}

#[test]
fn reads_line_formats() {
    assert_eq!(
        deps(
            "go.mod",
            "module x\nrequire (\n\tgolang.org/x/mod v0.17.0\n)\nrequire github.com/a/b v1.2.3 // indirect\n"
        ),
        pairs(&[
            ("golang.org/x/mod", "v0.17.0"),
            ("github.com/a/b", "v1.2.3")
        ])
    );
    assert_eq!(
        deps(
            "Dockerfile",
            "FROM --platform=linux node:20-alpine AS b\nFROM scratch\n"
        ),
        pairs(&[("node", "20-alpine")])
    );
    assert_eq!(
        deps(
            "build.gradle.kts",
            "implementation(\"com.squareup:okio:3.9.0\")\n"
        ),
        pairs(&[("com.squareup:okio", "3.9.0")])
    );
    assert_eq!(
        deps(
            "build.sbt",
            "\"org.typelevel\" %% \"cats\" % \"2.0\"\n\"a.b\" % \"c\" % \"1.0\"\n"
        ),
        pairs(&[("a.b:c", "1.0")])
    );
}

#[test]
fn reads_tool_versions_and_added_formats() {
    assert_eq!(deps(".nvmrc", "v20.11.0\n"), pairs(&[("node", "20.11.0")]));
    assert_eq!(deps(".bun-version", "1.1.0"), pairs(&[("bun", "1.1.0")]));
    assert_eq!(deps(".nvmrc", "lts/iron\n"), []);
    assert_eq!(
        deps("package.json5", "{dependencies: {a: '^1.0.0',},}"),
        pairs(&[("a", "^1.0.0")])
    );
    assert_eq!(
        deps(
            "paket.dependencies",
            "source https://x\nnuget Newtonsoft.Json ~> 13.0\n"
        ),
        pairs(&[("Newtonsoft.Json", "~> 13.0")])
    );
    let renv = r#"{"Packages":{"dplyr":{"Package":"dplyr","Version":"1.1.4"}}}"#;
    assert_eq!(deps("renv.lock", renv), pairs(&[("dplyr", "1.1.4")]));
    let cartfile = "github \"ReactiveX/RxSwift\" ~> 6.0\ngithub \"a/b\" \"1.2.3\"\n";
    assert_eq!(
        deps("Cartfile", cartfile),
        pairs(&[("ReactiveX/RxSwift", "~> 6.0"), ("a/b", "1.2.3")])
    );
    assert_eq!(
        deps("dune-project", "(depends (dune (>= 3.0)) (lwt (= 5.7.0)))"),
        pairs(&[("dune", ">= 3.0"), ("lwt", "= 5.7.0")])
    );
    let cabal =
        "cabal-version: >=1.10\n  build-depends: base >=4.7 && <5,\n                 text ^>=2.0\n";
    assert_eq!(
        deps("demo.cabal", cabal),
        pairs(&[("base", ">=4.7"), ("text", "^>=2.0")])
    );
}

fn one(path: &str, text: &str, name: &str, req: &str) {
    assert_eq!(deps(path, text), pairs(&[(name, req)]), "{path}");
}

fn registries(path: &str, text: &str) -> Vec<(String, &'static str)> {
    let file = file_type(path).unwrap_or_else(|| panic!("{path} not recognised"));
    extract(file, text)
        .into_iter()
        .map(|d| (d.name, d.registry))
        .collect()
}

fn named(list: &[(&str, &'static str)]) -> Vec<(String, &'static str)> {
    list.iter().map(|(n, r)| ((*n).to_owned(), *r)).collect()
}

#[test]
fn reads_restored_manifest_names() {
    one(
        "rust-toolchain.toml",
        "[toolchain]\nchannel = \"1.80.0\"\n",
        "rust",
        "1.80.0",
    );
    one("rust-toolchain", "1.80.0\n", "rust", "1.80.0");
    one(
        "compose.dev.yaml",
        "services:\n  a:\n    image: redis:7.2\n",
        "redis",
        "7.2",
    );
    one("a/dockerfile", "FROM node:20\n", "node", "20");
    one("a/api.dockerfile", "FROM node:20\n", "node", "20");
}

#[test]
fn reads_runtime_version_files() {
    one(".ruby-version", "ruby-3.3.0\n", "ruby", "3.3.0");
    one(".python-version", "3.12.1\n", "python", "3.12.1");
    one(".terraform-version", "1.9.0\n", "terraform-cli", "1.9.0");
    assert_eq!(
        registries(
            ".tool-versions",
            "nodejs 20.11.0\npython 3.12.1\nterraform 1.9.0\n"
        ),
        named(&[
            ("node", "npm"),
            ("python", "python"),
            ("terraform", "terraform-cli")
        ])
    );
    assert_eq!(
        registries(
            "mise.toml",
            "[tools]\nnode = \"22\"\npython = { version = \"3.12\" }\n\"npm:prettier\" = \"3.0.0\"\n"
        ),
        named(&[("node", "npm"), ("python", "python"), ("prettier", "npm")])
    );
}

#[test]
fn reads_tooling_configuration() {
    one(
        "a/.pre-commit-config.yaml",
        "repos:\n  - repo: https://github.com/psf/black\n    rev: 24.1.0\n",
        "psf/black",
        "24.1.0",
    );
    one(
        "a/.devcontainer/devcontainer.json",
        "{\"image\": \"mcr.microsoft.com/devcontainers/base:ubuntu\"}",
        "mcr.microsoft.com/devcontainers/base",
        "ubuntu",
    );
    one(
        "a/flake.nix",
        "{ inputs.x.url = \"github:NixOS/nixpkgs/v24.05\"; }",
        "NixOS/nixpkgs",
        "v24.05",
    );
    one(
        "a/foo-1.0-1.rockspec",
        "dependencies = {\n  \"lua >= 5.1\",\n  \"luasocket >= 3.0\"\n}\n",
        "luasocket",
        ">= 3.0",
    );
}

#[test]
fn reads_native_and_functional_manifests() {
    one(
        "a/Cartfile.private",
        "github \"Alamofire/Alamofire\" ~> 5.0\n",
        "Alamofire/Alamofire",
        "~> 5.0",
    );
    one(
        "build.zig.zon",
        ".{ .dependencies = .{ .x = .{ .url = \"https://github.com/a/b/archive/refs/tags/v1.2.3.tar.gz\" } } }",
        "a/b",
        "v1.2.3",
    );
    one(
        "subprojects/zlib.wrap",
        "[wrap-file]\nsource_url = https://github.com/madler/zlib/releases/download/v1.3.1/zlib.tar.gz\n",
        "madler/zlib",
        "v1.3.1",
    );
    one(
        "WORKSPACE.bazel",
        "http_archive(urls = [\"https://github.com/a/b/archive/v2.0.0.tar.gz\"])",
        "a/b",
        "v2.0.0",
    );
    one(
        "a/Manifest-v1.11.toml",
        "[[deps.JSON]]\nuuid = \"x\"\ngit-tree-sha1 = \"abc\"\nversion = \"0.21.4\"\n",
        "JSON",
        "0.21.4",
    );
    one(
        "stack.yaml",
        "extra-deps:\n  - aeson-2.2.1.0\n",
        "aeson",
        "2.2.1.0",
    );
    one(
        "a/pkg.nimble",
        "requires \"https://github.com/dom96/jester >= 0.5.0\"\n",
        "dom96/jester",
        ">= 0.5.0",
    );
}

#[test]
fn reads_pipeline_and_package_manager_files() {
    one(
        "jsr.jsonc",
        "{\"imports\": {\"x\": \"jsr:@std/path@^1.0.0\"}}",
        "@std/path",
        "^1.0.0",
    );
    one(
        "a/.yarnrc.yml",
        "catalog:\n  left-pad: 1.3.0\n",
        "left-pad",
        "1.3.0",
    );
    one(
        "a/package.yml",
        "dependencies:\n  left-pad: 1.3.0\n",
        "left-pad",
        "1.3.0",
    );
    one(
        ".github/workflows/a/b.yml",
        "jobs:\n  t:\n    steps:\n      - uses: actions/checkout@v4\n",
        "actions/checkout",
        "v4",
    );
    one(
        ".github/actions/x/action.yml",
        "runs:\n  steps:\n    - uses: actions/cache@v4\n",
        "actions/cache",
        "v4",
    );
    one(
        "Kustomization",
        "images:\n  - name: nginx\n    newTag: 1.25.0\n",
        "nginx",
        "1.25.0",
    );
    one(
        "a/gradle/wrapper/gradle-wrapper.properties",
        "distributionUrl=https\\://services.gradle.org/distributions/gradle-8.5-bin.zip\n",
        "gradle",
        "8.5",
    );
    one(
        "global.json",
        "{\"msbuild-sdks\": {\"Microsoft.Build.Traversal\": \"4.1.0\"}}",
        "Microsoft.Build.Traversal",
        "4.1.0",
    );
}

#[test]
fn reads_cargo_config_patches() {
    let text =
        "[patch.crates-io]\nserde = { version = \"1.0.200\" }\nlocal = { path = \"../local\" }\n";
    let both = pairs(&[("serde", "1.0.200"), ("local", "../local")]);
    assert_eq!(deps(".cargo/config.toml", text), both);
    assert_eq!(deps("a/.cargo/config", text), both);
}

#[test]
fn reads_local_paths_as_directories() {
    for (req, path) in [
        ("file:../c", "../c"),
        ("file:///opt/c", "/opt/c"),
        ("link:./c", "./c"),
        ("portal:~/c", "~/c"),
    ] {
        assert_eq!(local_path(req), Some(path), "{req}");
    }
    for req in ["^1.0.0", "file:", "//host/c", "workspace:*"] {
        assert_eq!(local_path(req), None, "{req}");
    }
    let cargo = "[dependencies]\nlocal = { path = \"../local\" }\nserde = \"1\"\n";
    assert_eq!(
        registries("Cargo.toml", cargo),
        named(&[("local", DIRECTORY), ("serde", "cargo")])
    );
}
