use super::parse::{helm, terraform};
use super::*;

mod live;

#[test]
fn pages_docker_tags_by_number_and_variant() {
    let docker = get("docker").unwrap();
    assert_eq!(filters_of(docker, "20-alpine3.22"), ["20", "alpine"]);
    assert_eq!(
        filters_of(docker, "v1.25.3-bookworm-slim"),
        ["1", "bookworm-slim"]
    );
    assert_eq!(filters_of(docker, "lts-alpine"), ["alpine"]);
    assert_eq!(filters_of(docker, "20"), ["20"]);
    assert_eq!(
        filters_of(get("npm").unwrap(), "20.1.0-rc.1"),
        Vec::<String>::new()
    );
}

#[test]
fn names_the_application_an_image_packages() {
    let app = |name| expand("{app}", name, None).ok();
    assert_eq!(app("nginx").as_deref(), Some("nginx"));
    assert_eq!(app("library/postgres").as_deref(), Some("postgresql"));
    assert_eq!(app("bitnami/redis").as_deref(), Some("redis"));
    assert_eq!(app("grafana/grafana").as_deref(), Some("grafana"));
    assert_eq!(
        app("quay.io/prometheus/prometheus").as_deref(),
        Some("prometheus")
    );
    assert_eq!(
        app("public.ecr.aws/docker/library/httpd").as_deref(),
        Some("apache")
    );
    assert_eq!(app("myorg/nginx"), None);
}

#[test]
fn expands_placeholders() {
    assert_eq!(expand("{cargo}", "Serde", None).unwrap(), "se/rd/serde");
    assert_eq!(expand("{cargo}", "syn", None).unwrap(), "3/s/syn");
    assert_eq!(
        expand("{enc}", "@types/node", None).unwrap(),
        "@types%2fnode"
    );
    assert_eq!(
        expand("{go}", "github.com/BurntSushi/toml", None).unwrap(),
        "github.com/!burnt!sushi/toml"
    );
    assert_eq!(
        expand("{path}", "org.junit:junit", None).unwrap(),
        "org/junit/junit"
    );
    assert_eq!(expand("{path}", "ring", None).unwrap(), "ring/ring");
    assert_eq!(
        expand("{ns}/{short}", "community.general", None).unwrap(),
        "community/general"
    );
    assert_eq!(expand("{docker}", "node", None).unwrap(), "library/node");
    assert_eq!(
        expand("{docker}", "docker.io/bitnami/redis", None).unwrap(),
        "bitnami/redis"
    );
    assert!(expand("{docker}", "ghcr.io/a/b", None).is_err());
    assert_eq!(
        expand("{host}|{repo}", "ghcr.io/a/b", None).unwrap(),
        "ghcr.io|a/b"
    );
    assert!(expand("{host}", "node", None).is_err());
    let docker = get("docker").unwrap();
    assert_eq!(route(docker, "ghcr.io/a/b").id, "oci");
    assert_eq!(route(docker, "docker.io/a/b").id, "docker");
    assert_eq!(route(docker, "bitnami/redis").id, "docker");
    let npm = get("npm").unwrap();
    assert_eq!(route(npm, "node").id, "nodejs");
    assert_eq!(route(npm, "lodash").id, "npm");
    assert_eq!(
        page("docker", "quay.io/a/b"),
        "https://quay.io/repository/a/b"
    );
    assert_eq!(
        page("docker", "registry.k8s.io/pause"),
        "https://explore.ggcr.dev/?repo=registry.k8s.io/pause"
    );
    assert_eq!(
        expand("{source}/index.yaml", "x", Some("https://c.io/")).unwrap(),
        "https://c.io/index.yaml"
    );
    assert!(expand("{source}", "x", Some("oci://c.io")).is_err());
}

#[test]
fn parses_each_response_shape() {
    let parse = |id: &str, body: &str, name: &str| get(id).unwrap().parse(body, name).unwrap();
    let npm = parse(
        "npm",
        r#"{"versions":{"1.0.0":{"deprecated":"use b"},"2.0.0":{"deprecated":""}},"dist-tags":{"latest":"1.0.0"}}"#,
        "x",
    );
    assert_eq!(
        npm,
        Versions {
            list: vec!["1.0.0".into(), "2.0.0".into()],
            latest: Some("1.0.0".into()),
            yanked: Vec::new(),
            deprecated: vec!["1.0.0".into()],
        }
    );
    assert_eq!(
        parse(
            "composer",
            r#"{"packages":{"a/b":[{"version":"v1.0"}]}}"#,
            "a/b"
        )
        .list,
        ["v1.0"]
    );
    assert_eq!(parse("cran", r#"{"Version":"1.2"}"#, "x").list, ["1.2"]);
    assert_eq!(
        parse(
            "vcpkg",
            r#"{"versions":[{"version-semver":"1.0.0"},{"version":"0.9"}]}"#,
            "x"
        )
        .list,
        ["1.0.0", "0.9"]
    );
    let cargo = "{\"vers\":\"1.0.0\",\"yanked\":false}\n{\"vers\":\"1.1.0\",\"yanked\":true}\n";
    let cargo = parse("cargo", cargo, "x");
    assert_eq!(
        (cargo.list, cargo.yanked),
        (vec!["1.0.0".to_owned()], vec!["1.1.0".to_owned()])
    );
    let jsr = parse(
        "jsr",
        r#"{"versions":{"1.0.0":{},"1.1.0":{"yanked":true}}}"#,
        "x",
    );
    assert_eq!(
        (jsr.list, jsr.yanked),
        (vec!["1.0.0".to_owned()], vec!["1.1.0".to_owned()])
    );
    let pub_dev = r#"{"versions":[{"version":"1.0.0"},{"version":"1.1.0","retracted":true}]}"#;
    assert_eq!(parse("pub", pub_dev, "x").yanked, ["1.1.0"]);
    assert_eq!(
        parse("go", "v1.0.0\nv1.1.0\n", "x").list,
        ["v1.0.0", "v1.1.0"]
    );
    let opam = r#"<a href="../dune/dune.3.17.2/"> <a href="../alba/alba.0.4.4/">"#;
    assert_eq!(parse("opam", opam, "dune").list, ["3.17.2"]);
    assert!(get("npm").unwrap().parse("<html>", "x").is_err());
}

#[test]
fn reads_an_index_that_answers_only_html() {
    // PEP 691 lets a Simple index ignore the JSON `accept` and answer PEP 503 HTML, as PyTorch's
    // download.pytorch.org/whl/cpu/torch/ does; each file name holds its version.
    let html = r#"<h1>Links for torch</h1>
<a href="https://x/torch-0.3.0.post4-cp27-cp27m-linux_x86_64.whl#sha256=0" data-upload-time="t">torch-0.3.0.post4-cp27-cp27m-linux_x86_64.whl</a><br/>
<a href="https://x/torch-2.4.0%2Bcpu-cp312-cp312-linux_x86_64.whl#sha256=0">torch-2.4.0+cpu-cp312-cp312-linux_x86_64.whl</a><br/>
<a href="https://x/torch-2.4.0%2Bcpu-cp311-cp311-win_amd64.whl">torch-2.4.0+cpu-cp311-cp311-win_amd64.whl</a><br/>
<a href="https://x/python-dateutil-2.8.2.tar.gz">python-dateutil-2.8.2.tar.gz</a><br/>
<a href="https://x/foo_bar-1.0rc1.zip">foo_bar-1.0rc1.zip</a><br/>"#;
    let pypi = get("pypi").unwrap();
    assert_eq!(
        pypi.parse(html, "torch").unwrap().list,
        ["0.3.0.post4", "2.4.0+cpu", "2.8.2", "1.0rc1"]
    );
    // The JSON answer still reads as before, and an HTML answer elsewhere is still an error.
    assert_eq!(
        pypi.parse(r#"{"versions":["1.0"]}"#, "x").unwrap().list,
        ["1.0"]
    );
    assert!(get("npm").unwrap().parse(html, "torch").is_err());
}

#[test]
fn reads_helm_index_entries() {
    let index = "apiVersion: v1\nentries:\n  other:\n  - version: 9.9.9\n  redis:\n  - apiVersion: v2\n    version: 17.1.0\n    dependencies:\n    - name: common\n      version: 2.0.0\n  - version: \"17.0.0\"\n  zed:\n  - version: 1.0.0\n";
    assert_eq!(helm(index, "redis").list, ["17.1.0", "17.0.0"]);
}

#[test]
fn reads_runtime_and_tool_registries() {
    let parse = |id: &str, body: &str, name: &str| get(id).unwrap().parse(body, name).unwrap().list;
    assert_eq!(
        parse(
            "golang",
            r#"[{"version":"go1.22.1"},{"version":"go1.21.0"}]"#,
            "golang"
        ),
        ["1.22.1", "1.21.0"]
    );
    assert_eq!(
        parse(
            "python",
            r#"[{"version":"3.12.1"},{"version":"3.11.7"}]"#,
            "python"
        ),
        ["3.12.1", "3.11.7"]
    );
    assert_eq!(
        parse(
            "rust",
            "Version 1.80.0 (2024-07-25)\n===\nVersion 1.79.0 (2024-06-13)\n",
            "rust"
        ),
        ["1.80.0", "1.79.0"]
    );
    // Racket's page links each release to its installers, docs, and PDFs; each counts once.
    assert_eq!(
        parse(
            "racket",
            r#"<a href="releases/9.3/">x</a><a href="releases/9.3/doc">x</a><a href="releases/9.2/pdf-doc">"#,
            "base"
        ),
        ["9.3", "9.2"]
    );
    assert_eq!(
        parse(
            "ruby",
            "- version: 3.3.0\n  date: x\n- version: 3.2.2\n",
            "ruby"
        ),
        ["3.3.0", "3.2.2"]
    );
    assert_eq!(
        parse(
            "terraform-cli",
            r#"{"versions":{"1.9.0":{},"1.8.5":{}}}"#,
            "terraform"
        ),
        ["1.8.5", "1.9.0"]
    );
    assert_eq!(
        parse(
            "luarocks",
            r#"{"repository":{"luasocket":{"3.0-1":[],"2.0-1":[]}}}"#,
            "luasocket"
        ),
        ["2.0-1", "3.0-1"]
    );
    assert_eq!(
        parse("elm", r#"{"1.0.0":1,"1.1.0":2}"#, "elm/core"),
        ["1.0.0", "1.1.0"]
    );
    assert_eq!(
        parse(
            "xmake",
            "add_versions(\"1.2.3\", \"h\")\nadd_versions(\"github:2.0\", \"h\")",
            "zlib"
        ),
        ["1.2.3", "2.0"]
    );
}

#[test]
fn terraform_reads_providers_and_modules() {
    assert_eq!(
        terraform(
            r#"{"versions":[{"version":"5.0.0"},{"version":"4.9.0"}]}"#,
            "hashicorp/aws"
        )
        .list,
        ["5.0.0", "4.9.0"]
    );
    assert_eq!(
        terraform(
            r#"{"modules":[{"versions":[{"version":"3.0.0"}]}]}"#,
            "a/b/c"
        )
        .list,
        ["3.0.0"]
    );
    assert_eq!(terraform("not json", "a/b").list, Vec::<String>::new());
    assert_eq!(
        expand("{tf}", "hashicorp/aws", None).unwrap(),
        "providers/hashicorp/aws"
    );
    assert_eq!(
        expand("{tf}", "terraform-aws-modules/vpc/aws", None).unwrap(),
        "modules/terraform-aws-modules/vpc/aws"
    );
}

#[test]
fn aliases_tool_names_to_registries() {
    assert_eq!(alias("terraform"), Some("terraform-cli"));
    assert_eq!(alias("pipx"), Some("pypi"));
    assert_eq!(alias("python"), Some("python"));
    assert_eq!(alias("npm"), Some("npm"));
    assert_eq!(alias("nonsense"), None);
}
