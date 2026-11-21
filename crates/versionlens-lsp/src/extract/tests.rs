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
fn reads_package_json_and_skips_local_specs() {
    let text = r#"{"dependencies":{"a":"^1.0.0","b":"workspace:*","c":"file:../c","d":"github:x/y"},"packageManager":"bun@1.4.3"}"#;
    assert_eq!(
        deps("package.json", text),
        pairs(&[("a", "^1.0.0"), ("bun", "1.4.3")])
    );
}

#[test]
fn reads_cargo_tables_and_targets() {
    let text = "[dependencies]\nserde = { version = \"1\" }\nlocal = { path = \"../x\" }\n[target.'cfg(unix)'.dependencies]\nlibc = \"0.2\"\n";
    assert_eq!(
        deps("Cargo.toml", text),
        pairs(&[("serde", "1"), ("libc", "0.2")])
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
