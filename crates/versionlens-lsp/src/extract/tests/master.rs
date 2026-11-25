//! The dependency sections the fork's master read beyond upstream's, from the `dependency_paths`
//! of its `manifest/table.rs` and the forms its text parsers accepted.

use super::{extract, file_type};

/// Each dependency of the manifest, as its name, requirement, and registry.
fn found(path: &str, text: &str) -> Vec<(String, String, &'static str)> {
    let file = file_type(path).unwrap_or_else(|| panic!("{path} not recognised"));
    extract(file, text)
        .into_iter()
        .map(|d| (d.name, d.req, d.registry))
        .collect()
}

pub(super) fn expect(path: &str, text: &str, want: &[(&str, &str, &str)]) {
    let want: Vec<(String, String, &str)> = want
        .iter()
        .map(|(n, r, g)| ((*n).to_owned(), (*r).to_owned(), *g))
        .collect();
    assert_eq!(found(path, text), want, "{path}");
}

#[test]
fn reads_cargo_patch_tables() {
    let text = "[dependencies]\nserde = \"1\"\n\n[patch.crates-io]\nserde = { git = \"https://github.com/serde-rs/serde\" }\nlocal = { path = \"../local\" }\n";
    expect(
        "Cargo.toml",
        text,
        &[
            ("serde", "1", "cargo"),
            ("serde", "https://github.com/serde-rs/serde", "source"),
            ("local", "../local", "directory"),
        ],
    );
}

#[test]
fn reads_composer_conflict_replace_and_provide() {
    let text = r#"{"require": {"a/a": "^1.0"}, "conflict": {"b/b": "<2.0"}, "replace": {"c/c": "1.0.0"}, "provide": {"d/d": "^3.0"}}"#;
    expect(
        "composer.json",
        text,
        &[
            ("a/a", "^1.0", "composer"),
            ("b/b", "<2.0", "composer"),
            ("c/c", "1.0.0", "composer"),
            ("d/d", "^3.0", "composer"),
        ],
    );
}

#[test]
fn reads_deno_and_jsr_version() {
    let text = r#"{"version": "1.2.0", "imports": {"std": "jsr:@std/path@^1.0.0"}}"#;
    expect(
        "jsr.json",
        text,
        &[("deno", "1.2.0", "project"), ("@std/path", "^1.0.0", "jsr")],
    );
}

#[test]
fn checks_published_jsr_package_beside_its_increments() {
    let text = "{\n  \"name\": \"@luca/greet\",\n  \"version\": \"1.0.0\",\n  \"exports\": \"./mod.ts\"\n}\n";
    expect(
        "jsr.json",
        text,
        &[
            ("deno", "1.0.0", "project"),
            ("@luca/greet", "1.0.0", "jsr"),
        ],
    );
    // Either order, and only the top-level name: an import's scope is not the package's.
    let text = r#"{"version": "2.0.0", "imports": {"a": "jsr:@std/path@^1"}, "name": "@a/b"}"#;
    expect(
        "deno.json",
        text,
        &[
            ("deno", "2.0.0", "project"),
            ("@a/b", "2.0.0", "jsr"),
            ("@std/path", "^1", "jsr"),
        ],
    );
}

#[test]
fn reads_julia_standard_libraries_apart_from_registered_and_developed_packages() {
    // Shapes from pkgdocs.julialang.org/v1/toml-files/ and JuliaLang/julia stdlib/Manifest.toml.
    let text = "\
[[deps.Dates]]
deps = [\"Printf\"]
uuid = \"ade2ca70-3891-5945-98fb-dc099432e06a\"
version = \"1.11.0\"

[[deps.LibCURL_jll]]
deps = [\"LibSSH2_jll\", \"Libdl\"]
uuid = \"deac9b47-8bc7-5906-a0fe-35ac56dc84c0\"
version = \"8.6.0+0\"

[[deps.Example]]
deps = [\"DependencyA\"]
git-tree-sha1 = \"8eb7b4d4ca487caade9ba3e85932e28ce6d6e1f8\"
uuid = \"7876af07-990d-54b4-ab0e-23690620f79a\"
version = \"1.2.3\"

[[deps.Local]]
path = \"dev/Local\"
uuid = \"00000000-0000-0000-0000-000000000002\"
version = \"0.1.0\"
";
    expect(
        "Manifest.toml",
        text,
        &[
            ("Dates", "1.11.0", "julia-stdlib"),
            ("LibCURL_jll", "8.6.0+0", "julia"),
            ("Example", "1.2.3", "julia"),
        ],
    );
}

#[test]
fn reads_project_json_runtimes() {
    let text = r#"{"dependencies": {"A": "1.0.0"}, "frameworks": {"net472": {"dependencies": {"B": "2.0.0"}}}, "runtimes": {"win": {"dependencies": {"C": "3.0.0"}}}}"#;
    expect(
        "project.json",
        text,
        &[
            ("A", "1.0.0", "nuget"),
            ("B", "2.0.0", "nuget"),
            ("C", "3.0.0", "nuget"),
        ],
    );
}

#[test]
fn reads_pom_profiles() {
    let text = "<project><profiles><profile><id>dev</id>\
        <dependencies><dependency><groupId>g</groupId><artifactId>dep</artifactId><version>1.0</version></dependency></dependencies>\
        <dependencyManagement><dependencies><dependency><groupId>g</groupId><artifactId>managed</artifactId><version>2.0</version></dependency></dependencies></dependencyManagement>\
        <build><plugins><plugin><groupId>g</groupId><artifactId>plugin</artifactId><version>3.0</version></plugin></plugins>\
        <pluginManagement><plugins><plugin><groupId>g</groupId><artifactId>managed-plugin</artifactId><version>4.0</version></plugin></plugins></pluginManagement></build>\
        </profile></profiles></project>";
    expect(
        "pom.xml",
        text,
        &[
            ("g:dep", "1.0", "maven"),
            ("g:managed", "2.0", "maven"),
            ("g:plugin", "3.0", "maven"),
            ("g:managed-plugin", "4.0", "maven"),
        ],
    );
}

#[test]
fn reads_vcpkg_bare_names_and_features() {
    let text = r#"{"dependencies": ["fmt", {"name": "zlib", "version>=": "1.3"}], "features": {"tls": {"description": "TLS", "dependencies": ["openssl", {"name": "curl", "version>=": "8.0"}]}}}"#;
    let names: Vec<(String, String)> = found("vcpkg.json", text)
        .into_iter()
        .map(|(n, r, _)| (n, r))
        .collect();
    assert_eq!(
        names,
        [
            ("fmt", ""),
            ("zlib", "1.3"),
            ("openssl", ""),
            ("curl", "8.0")
        ]
        .map(|(n, r)| (n.to_owned(), r.to_owned()))
    );
}

#[test]
fn reads_dub_configuration_dependencies() {
    let text = r#"{"dependencies": {"vibe-d": "~>0.9"}, "configurations": [{"name": "app", "dependencies": {"arsd-official": "~>11.0"}}]}"#;
    expect(
        "dub.json",
        text,
        &[
            ("vibe-d", "~>0.9", "dub"),
            ("arsd-official", "~>11.0", "dub"),
        ],
    );
}

#[test]
fn reads_pubspec_workspace_folders() {
    let text = "name: root\nworkspace:\n  - packages/shared\n  - packages/client\ndependencies:\n  http: ^1.2.0\n";
    expect(
        "pubspec.yaml",
        text,
        &[
            ("packages/shared", "packages/shared", "directory"),
            ("packages/client", "packages/client", "directory"),
            ("http", "^1.2.0", "pub"),
        ],
    );
}

#[test]
fn reads_go_replace_exclude_and_quoted_lines() {
    let text = "module example.test/app\n\nrequire example.test/one v1.2.3\nreplace example.test/local => ../local\nreplace example.test/old v1.0.0 => ./vendor/old\n\nrequire (\n\tgithub.com/docker/cli v26.1.3+incompatible\n)\n\nexclude example.test/bad v0.5.0\nrequire \"example.test/quoted\" \"v1.2.3\"\nreplace \"example.test/was\" \"v1.0.0\" => \"example.test/new\" \"v1.1.0\"\n";
    expect(
        "go.mod",
        text,
        &[
            ("example.test/one", "v1.2.3", "go"),
            ("example.test/local", "../local", "directory"),
            ("example.test/old", "./vendor/old", "directory"),
            ("github.com/docker/cli", "v26.1.3+incompatible", "go"),
            ("example.test/bad", "v0.5.0", "go"),
            ("example.test/quoted", "v1.2.3", "go"),
            ("example.test/new", "v1.1.0", "go"),
        ],
    );
}

#[test]
fn reads_gradle_map_notation_and_kotlin_plugins() {
    let groovy = "dependencies {\n    implementation group: 'com.google.guava', name: 'guava', version: '33.0.0-jre'\n}\n";
    expect(
        "build.gradle",
        groovy,
        &[("com.google.guava:guava", "33.0.0-jre", "maven")],
    );
    let kotlin = "plugins {\n    kotlin(\"jvm\") version \"2.0.0\"\n}\ndependencies {\n    implementation(group = \"g\", name = \"a\", version = \"1.0\")\n}\n";
    expect(
        "build.gradle.kts",
        kotlin,
        &[
            ("org.jetbrains.kotlin.jvm", "2.0.0", "gradle-plugins"),
            ("g:a", "1.0", "maven"),
        ],
    );
}

#[test]
fn reads_pep508_direct_references_as_sources() {
    let requirements = "requests==2.31.0\nmylib @ https://example.com/mylib-1.0.tar.gz ; python_version > '3.8'\ntool[cli] @ git+https://github.com/o/tool@v1\n";
    expect(
        "requirements.txt",
        requirements,
        &[
            ("requests", "==2.31.0", "pypi"),
            ("mylib", "https://example.com/mylib-1.0.tar.gz", "source"),
            ("tool", "git+https://github.com/o/tool@v1", "source"),
        ],
    );
    let pyproject =
        "[project]\ndependencies = [\"httpx>=0.27\", \"pkg @ https://example.com/pkg.whl\"]\n";
    expect(
        "pyproject.toml",
        pyproject,
        &[
            ("httpx", ">=0.27", "pypi"),
            ("pkg", "https://example.com/pkg.whl", "source"),
        ],
    );
}

#[test]
fn reads_swift_hosts_branches_and_paths() {
    let text = r#"dependencies: [
    .package(url: "https://github.com/apple/swift-nio.git", .exact("2.60.0")),
    .package(url: "https://gitlab.com/o/lib.git", from: "1.0.0"),
    .package(url: "https://github.com/o/edge.git", branch: "main"),
    .package(path: "../Local"),
]"#;
    expect(
        "Package.swift",
        text,
        &[
            ("apple/swift-nio", "2.60.0", "swift"),
            ("https://gitlab.com/o/lib.git", "1.0.0", "source"),
            ("https://github.com/o/edge.git", "main", "source"),
            ("../Local", "../Local", "directory"),
        ],
    );
}

#[test]
fn reads_zig_path_dependencies() {
    let text = ".{\n    .dependencies = .{\n        .local = .{ .path = \"../local\" },\n        .@\"with-dash\" = .{\n            .path = \"libs/dash\",\n        },\n    },\n}\n";
    expect(
        "build.zig.zon",
        text,
        &[
            ("local", "../local", "directory"),
            ("with-dash", "libs/dash", "directory"),
        ],
    );
}

#[test]
fn reads_nimble_moving_branches_as_sources() {
    let text =
        "requires \"https://github.com/o/r >= 1.0\"\nfeature \"x\":\n  requires \"jester#head\"\n";
    expect(
        "demo.nimble",
        text,
        &[("o/r", ">= 1.0", "github"), ("jester", "head", "source")],
    );
}

#[test]
fn reads_dub_sdl_attributes_in_any_order() {
    let text = "dependency \"vibe-d\" optional=true version=\"~>0.9\"\ndependency \"local\" path=\"../local\"\ndependency \"forked\" repository=\"git+https://example.com/f.git\" version=\"a1b2c3\"\n";
    expect(
        "dub.sdl",
        text,
        &[
            ("vibe-d", "~>0.9", "dub"),
            ("local", "../local", "directory"),
            ("forked", "a1b2c3", "source"),
        ],
    );
}

#[test]
fn reads_helm_local_alias_and_oci_repositories() {
    let text = "dependencies:\n  - name: common\n    version: 2.0.0\n    repository: https://charts.bitnami.com/bitnami\n  - name: sub\n    version: 0.1.0\n    repository: file://../sub\n  - name: aliased\n    version: 1.0.0\n    repository: \"@stable\"\n  - name: nginx\n    version: 18.0.0\n    repository: oci://registry-1.docker.io/bitnamicharts\n";
    expect(
        "Chart.yaml",
        text,
        &[
            ("common", "2.0.0", "helm"),
            ("sub", "0.1.0", "directory"),
            ("aliased", "1.0.0", "source"),
            ("registry-1.docker.io/bitnamicharts/nginx", "18.0.0", "oci"),
        ],
    );
}

#[test]
fn labels_pinned_sources_by_their_repository() {
    let swift = r#".package(url: "https://github.com/o/edge.git", branch: "main")"#;
    let nimble = "requires \"jester#head\"\n";
    let label = |path, text| {
        let file = file_type(path).unwrap_or_else(|| panic!("{path} not recognised"));
        extract(file, text)
            .into_iter()
            .map(|d| d.source)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        label("Package.swift", swift),
        [Some("git repository".to_owned())]
    );
    assert_eq!(
        label("demo.nimble", nimble),
        [Some("git reference".to_owned())]
    );
}

#[test]
fn reads_gradle_catalog_version_refs_and_plugins() {
    let text = r#"[versions]
kotlin = "2.0.0"
okio = "3.9.0"
agp = "8.5.0"

[libraries]
kotlin-stdlib = { module = "org.jetbrains.kotlin:kotlin-stdlib", version.ref = "kotlin" }
kotlin-test = { module = "org.jetbrains.kotlin:kotlin-test", version.ref = "kotlin" }
okio = { group = "com.squareup.okio", name = "okio", version = { ref = "okio" } }
guava = "com.google.guava:guava:33.0.0-jre"

[plugins]
android = { id = "com.android.application", version.ref = "agp" }
kotlin-jvm = { id = "org.jetbrains.kotlin.jvm", version.ref = "kotlin" }
detekt = { id = "io.gitlab.arturbosch.detekt", version = "1.23.6" }
spotless = "com.diffplug.spotless:6.25.0"
"#;
    expect(
        "gradle/libs.versions.toml",
        text,
        &[
            ("org.jetbrains.kotlin:kotlin-stdlib", "2.0.0", "maven"),
            ("com.squareup.okio:okio", "3.9.0", "maven"),
            ("com.android.application", "8.5.0", "gradle-plugins"),
            ("com.google.guava:guava", "33.0.0-jre", "maven"),
            ("io.gitlab.arturbosch.detekt", "1.23.6", "gradle-plugins"),
            ("com.diffplug.spotless", "6.25.0", "gradle-plugins"),
        ],
    );
}

#[test]
fn reads_setup_action_runtimes() {
    let text = "jobs:\n  build:\n    steps:\n      - uses: actions/setup-node@v4\n        with:\n          node-version: '20.11'\n      - uses: actions/setup-python@v5\n        with:\n          python-version: \"3.12\"\n      - uses: actions/setup-go@v5\n        with:\n          go-version: 1.22.1\n      - uses: actions/setup-node@v4\n        with:\n          node-version: ${{ matrix.node }}\n";
    expect(
        ".github/workflows/ci.yml",
        text,
        &[
            ("actions/setup-node", "v4", "github"),
            ("node", "20.11", "npm"),
            ("actions/setup-python", "v5", "github"),
            ("python", "3.12", "python"),
            ("actions/setup-go", "v5", "github"),
            ("go", "1.22.1", "golang"),
            ("actions/setup-node", "v4", "github"),
        ],
    );
}

#[test]
fn reads_cargo_rust_version() {
    let text = "[package]\nname = \"demo\"\nversion = \"1.0.0\"\nrust-version = \"1.78\"\n";
    expect(
        "Cargo.toml",
        text,
        &[("cargo", "1.0.0", "project"), ("rust", "1.78", "rust")],
    );
}

#[test]
fn reads_bundled_and_trusted_dependency_names() {
    let text = r#"{"dependencies": {"a": "^1.0.0"}, "bundleDependencies": ["a"], "bundledDependencies": ["@s/b"], "trustedDependencies": ["esbuild"]}"#;
    expect(
        "package.json",
        text,
        &[
            ("a", "^1.0.0", "npm"),
            ("a", "", "npm"),
            ("@s/b", "", "npm"),
            ("esbuild", "", "npm"),
        ],
    );
}

#[test]
fn reads_jspm_maps_by_registry_each_value_names() {
    let text = r#"{"jspm": {"dependencies": {"lodash": "npm:lodash@^4.17.0", "repo": "github:owner/repo@1.0.0", "plain": "^2.0.0"}}}"#;
    expect(
        "package.json",
        text,
        &[
            ("lodash", "^4.17.0", "npm"),
            ("owner/repo", "1.0.0", "github"),
            ("plain", "^2.0.0", "npm"),
        ],
    );
}

#[test]
fn reads_opam_packages_only_in_their_lists() {
    let text =
        "opam-version: \"2.0\"\ntags: [\"web\" \"async\"]\ndepends: [\n  \"lwt\" {>= \"5.0\"}\n]\n";
    expect("a.opam", text, &[("lwt", "5.0", "opam")]);
}
