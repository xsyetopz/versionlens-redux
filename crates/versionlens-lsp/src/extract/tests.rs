use super::*;
use crate::catalog::{file_type, file_type_of};

mod build;
mod ecosystems;
mod fixtures;
mod forms;
mod infra;
mod languages;
mod legacy;
mod lockfiles;
mod master;
mod packages;
mod properties;
mod public;
mod runtimes;
mod sections;
mod upstream;

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

/// Each dependency's name, requirement, and registry, for a document opened at `path`, with the
/// content rows a path alone cannot pick.
fn found(path: &str, text: &str) -> Vec<(String, String, &'static str)> {
    let file = file_type_of(path, text).unwrap_or_else(|| panic!("{path} not recognised"));
    extract(file, text)
        .into_iter()
        .map(|d| {
            assert_eq!(text[d.span.clone()], d.req, "span of {}", d.name);
            (d.name, d.req, d.registry)
        })
        .collect()
}

fn expect(path: &str, text: &str, want: &[(&str, &str, &str)]) {
    let want: Vec<_> = want
        .iter()
        .map(|(n, r, g)| ((*n).to_owned(), (*r).to_owned(), *g))
        .collect();
    assert_eq!(found(path, text), want, "{path}");
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
        pairs(&[
            ("a", "^1.0.0"),
            ("b", "workspace:*"),
            ("c", "file:../c"),
            ("d", "github:x/y"),
            ("bun", "1.4.3")
        ])
    );
    assert_eq!(
        registries("package.json", text),
        named(&[
            ("a", "npm"),
            ("b", SOURCE),
            ("c", DIRECTORY),
            ("d", SOURCE),
            ("bun", "npm")
        ])
    );
}

/// The label each source dependency carries, as the fork's master's `non_registry` module named it.
fn sources(path: &str, text: &str) -> Vec<(String, String)> {
    let file = file_type(path).unwrap_or_else(|| panic!("{path} not recognised"));
    extract(file, text)
        .into_iter()
        .filter(|d| d.registry == SOURCE)
        .map(|d| (d.name, d.source.unwrap_or_default()))
        .collect()
}

#[test]
fn labels_git_url_workspace_and_catalog_sources() {
    let npm = r#"{"dependencies":{"a":"git+ssh://git@example.com/a.git#v1","b":"https://github.com/x/b","c":"https://github.com/x/c/archive/v1.tar.gz","d":"https://example.com/d.tgz","e":"workspace:^","f":"catalog:","g":"npm:h@1","i":"${VERSION}"}}"#;
    assert_eq!(
        sources("package.json", npm),
        pairs(&[
            ("a", "git repository"),
            ("b", "git repository"),
            ("c", "package URL"),
            ("d", "package URL"),
            ("e", "workspace package"),
            ("f", "catalog entry")
        ])
    );
    let cargo = "[dependencies]\nregex = { git = \"https://example.com/regex\", tag = \"1.0\" }\nserde = \"1\"\n";
    assert_eq!(
        sources("Cargo.toml", cargo),
        pairs(&[("regex", "git repository")])
    );
    let pubspec = "dependencies:\n  kittens:\n    git:\n      url: https://example.com/kittens\n      ref: main\n  http: ^1.0.0\n";
    assert_eq!(
        sources("pubspec.yaml", pubspec),
        pairs(&[("kittens", "git repository")])
    );
    let uv = "[project]\ndependencies = [\"httpx\", \"foo\", \"bar\"]\n[tool.uv.sources]\nhttpx = { git = \"https://github.com/encode/httpx\", tag = \"0.27.0\" }\nfoo = { url = \"https://example.com/foo-1.0.tar.gz\" }\nbar = { workspace = true }\n";
    assert_eq!(
        sources("pyproject.toml", uv),
        pairs(&[("httpx", "git repository"), ("foo", "package URL")])
    );
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
        pairs(&[("actions/checkout", "v4"), ("./local", "./local")])
    );
}

#[test]
fn reads_pyproject_and_deno() {
    let pyproject = "[project]\ndependencies = [\"httpx>=0.27\", \"rich\"]\n[tool.poetry.dependencies]\npython = \"^3.11\"\nclick = \"^8.1\"\n";
    assert_eq!(
        deps("pyproject.toml", pyproject),
        pairs(&[
            ("httpx", ">=0.27"),
            ("rich", ""),
            ("python", "^3.11"),
            ("click", "^8.1")
        ])
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
fn reads_layouts_master_read() {
    one(
        "cabal.project",
        "packages: .\nconstraints: aeson == 2.2.1.0\n",
        "aeson",
        "== 2.2.1.0",
    );
    one(
        "main.tf",
        "terraform { required_providers { aws = { source = \"hashicorp/aws\" version = \"~> 5.0\" } } }\n",
        "hashicorp/aws",
        "~> 5.0",
    );
    one(
        "main.tf",
        "required_providers {\n  aws = {\n    version = \"~> 5.0\"\n    source  = \"hashicorp/aws\"\n  }\n}\n",
        "hashicorp/aws",
        "~> 5.0",
    );
    one(
        "flake.nix",
        "{ inputs.nixpkgs.url = \"github:NixOS/nixpkgs/nixos-24.05\"; }\n",
        "NixOS/nixpkgs",
        "24.05",
    );
}

/// Each manifest's own version, at the paths upstream's `dependencyProperties` list.
#[test]
fn reads_project_version_where_upstream_did() {
    let own = |path: &str, text: &str| -> Vec<String> {
        let file = file_type(path).unwrap_or_else(|| panic!("{path} not recognised"));
        extract(file, text)
            .into_iter()
            .filter(|d| d.registry == PROJECT)
            .map(|d| {
                assert_eq!(text[d.span.clone()], d.req);
                d.req
            })
            .collect()
    };
    let cases = [
        (
            "package.json",
            r#"{"name":"a","version":"1.2.3","dependencies":{"b":"1"}}"#,
        ),
        (
            "composer.json",
            r#"{"version":"1.2.3","require":{"a/b":"1"}}"#,
        ),
        (
            "pubspec.yaml",
            "name: a\nversion: 1.2.3\ndependencies:\n  http: ^1.0.0\n",
        ),
        (
            "Cargo.toml",
            "[package]\nname = \"a\"\nversion = \"1.2.3\"\n[dependencies]\nserde = \"1\"\n",
        ),
        (
            "pyproject.toml",
            "[project]\nname = \"a\"\nversion = \"1.2.3\"\ndependencies = []\n",
        ),
        (
            "pom.xml",
            "<project><version>1.2.3</version><parent><version>9</version></parent></project>",
        ),
        (
            "a.csproj",
            "<Project><PropertyGroup><Version>1.2.3</Version></PropertyGroup></Project>",
        ),
    ];
    for (path, text) in cases {
        assert_eq!(own(path, text), ["1.2.3"], "{path}");
    }
    // A workspace-inherited version names no version of its own.
    let inherited = "[package]\nversion.workspace = true\n";
    assert_eq!(own("Cargo.toml", inherited), Vec::<String>::new());
}

/// The requirement forms master's `non_registry` module answered without a registry
/// (origin/master crates/versionlens-engine/src/non_registry/{git.rs,local_path/requirement.rs}).
#[test]
fn labels_every_source_form_master_answered_without_registry() {
    for req in [
        "git+https://example.com/a.git",
        "git@example.com:o/a.git",
        "git://example.com/o/a",
        "ssh://git@example.com/o/a",
        "https://example.com/o/a.git",
        "https://github.com/o/a",
        "http://gitlab.com/o/a",
        "https://bitbucket.org/o/a",
        "https://user@github.com/o/a",
        "https://github.com:443/o/a",
    ] {
        assert_eq!(source_label(req), Some("git repository"), "{req}");
    }
    assert_eq!(
        source_label("https://example.com/a.tgz"),
        Some("package URL")
    );
    assert_eq!(source_label("workspace:*"), Some("workspace package"));
    assert_eq!(source_label("catalog:react18"), Some("catalog entry"));
    for (req, path) in [
        ("file:../a", "../a"),
        ("link:../a", "../a"),
        ("portal:../a", "../a"),
        ("path:../a", "../a"),
        ("/opt/a", "/opt/a"),
        ("./a", "./a"),
        ("../a", "../a"),
        ("~/a", "~/a"),
    ] {
        assert_eq!(local_path(req), Some(path), "{req}");
    }
}
