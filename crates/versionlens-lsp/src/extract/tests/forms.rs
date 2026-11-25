//! Forms the fork's master text parsers read inside the file types both baselines share, each
//! pinned to what its catalog row extracts: sbt's `%%`, Gradle classifiers and rich versions,
//! untagged and digest-pinned images, `.nvmrc` comments, versionless and sourced gems, go.work
//! directories, `deps.edn` coordinates, Leiningen's own version, Carthage URLs and references,
//! parenthesized requirements, and target-suffixed Rust channels.

use super::master::expect;

#[test]
fn reads_sbt_double_percent_with_the_scala_binary_version() {
    let text = "ThisBuild / scalaVersion := \"2.13.12\"\nlibraryDependencies ++= Seq(\n  \"org.typelevel\" %% \"cats-core\" % \"2.9.0\",\n  \"com.lihaoyi\" %%% \"upickle\" % \"3.1.0\",\n  \"org.slf4j\" % \"slf4j-api\" % \"2.0.9\"\n)\n";
    expect(
        "build.sbt",
        text,
        &[
            ("org.typelevel:cats-core_2.13", "2.9.0", "maven"),
            ("org.slf4j:slf4j-api", "2.0.9", "maven"),
        ],
    );
    let scala3 = "scalaVersion := \"3.3.1\"\nlibraryDependencies += \"org.typelevel\" %% \"cats-core\" % \"2.10.0\"\n";
    expect(
        "build.sbt",
        scala3,
        &[("org.typelevel:cats-core_3", "2.10.0", "maven")],
    );
    let unknown = "libraryDependencies += \"org.typelevel\" %% \"cats-core\" % \"2.9.0\"\n";
    expect("build.sbt", unknown, &[]);
}

#[test]
fn reads_gradle_classifiers_types_and_map_order() {
    let text = "dependencies {\n  implementation 'org.lwjgl:lwjgl:3.3.3:natives-linux'\n  implementation 'com.android.support:appcompat-v7:28.0.0@aar'\n  implementation name: 'guava', group: 'com.google.guava', version: '33.0.0-jre'\n}\n";
    expect(
        "build.gradle",
        text,
        &[
            ("org.lwjgl:lwjgl", "3.3.3", "maven"),
            ("com.android.support:appcompat-v7", "28.0.0", "maven"),
            ("com.google.guava:guava", "33.0.0-jre", "maven"),
        ],
    );
}

#[test]
fn reads_gradle_catalog_rich_versions() {
    let text = "[versions]\nguava = { strictly = \"33.0.0-jre\" }\n\n[libraries]\nguava = { module = \"com.google.guava:guava\", version.ref = \"guava\" }\nslf4j = { module = \"org.slf4j:slf4j-api\", version = { require = \"2.0.9\" } }\njunit = { group = \"junit\", name = \"junit\", version.prefer = \"4.13.2\" }\n";
    expect(
        "gradle/libs.versions.toml",
        text,
        &[
            ("com.google.guava:guava", "33.0.0-jre", "maven"),
            ("org.slf4j:slf4j-api", "2.0.9", "maven"),
            ("junit:junit", "4.13.2", "maven"),
        ],
    );
}

#[test]
fn reads_untagged_images_outside_build_stages() {
    let text = "FROM golang:1.22 AS build\nFROM build AS test\nFROM scratch\nFROM alpine\nFROM localhost:5000/app\nFROM --platform=$BUILDPLATFORM node AS web\n";
    expect(
        "Dockerfile",
        text,
        &[
            ("golang", "1.22", "docker"),
            ("alpine", "", "docker"),
            ("localhost:5000/app", "", "docker"),
            ("node", "", "docker"),
        ],
    );
}

#[test]
fn reads_compose_untagged_digest_and_extension_images() {
    let text = "x-cache: &cache\n  image: redis:7.2\nservices:\n  web:\n    image: nginx\n  db:\n    image: postgres:16.1@sha256:0123abcd\n  cache:\n    <<: *cache\n";
    expect(
        "compose.yaml",
        text,
        &[
            ("redis", "7.2", "docker"),
            ("nginx", "", "docker"),
            ("postgres", "16.1", "docker"),
        ],
    );
}

#[test]
fn reads_node_versions_after_comments() {
    expect(
        ".nvmrc",
        "# team pin\n20.11.1 # LTS\n",
        &[("node", "20.11.1", "npm")],
    );
    expect(".nvmrc", "lts/iron\n", &[]);
}

#[test]
fn reads_gems_without_a_version_and_from_sources() {
    let text = "gem \"rails\"\ngem 'rspec', group: :test, require: false\ngem \"local\", path: \"../local\"\ngem \"edge\", git: \"https://github.com/o/edge.git\"\ngem \"tagged\", github: \"o/tagged\", tag: \"v1.2.0\"\ngem \"branchy\", :github => \"o/branchy\", :branch => \"main\"\ngem \"private\", source: \"https://gems.example.com\"\n";
    expect(
        "Gemfile",
        text,
        &[
            ("rails", "", "rubygems"),
            ("rspec", "", "rubygems"),
            ("local", "../local", "directory"),
            ("edge", "https://github.com/o/edge.git", "source"),
            ("o/tagged", "v1.2.0", "github"),
            ("o/branchy", "main", "source"),
        ],
    );
    expect(
        "a.gemspec",
        "  s.add_dependency 'rack'\n",
        &[("rack", "", "rubygems")],
    );
}

#[test]
fn reads_go_work_directories() {
    let text = "go 1.22\n\nuse ./api\nuse (\n\t./web\n\t../shared\n)\n";
    expect(
        "go.work",
        text,
        &[
            ("go", "1.22", "golang"),
            ("./api", "./api", "directory"),
            ("./web", "./web", "directory"),
            ("../shared", "../shared", "directory"),
        ],
    );
}

#[test]
fn reads_deps_edn_coordinates_in_any_form() {
    let text = "{:deps {cheshire {:mvn/version \"5.12.0\"}\n        a/b$linux {:exclusions [c/d] :mvn/version \"1.0\"}\n        my/lib {:local/root \"../lib\"}\n        io.github.o/x {:git/tag \"v0.1.0\" :git/sha \"abc1234\"}}}\n";
    expect(
        "deps.edn",
        text,
        &[
            ("cheshire", "5.12.0", "clojars"),
            ("a/b", "1.0", "clojars"),
            ("my/lib", "../lib", "directory"),
            ("io.github.o/x", "v0.1.0", "source"),
        ],
    );
}

#[test]
fn reads_leiningen_own_version_and_quoted_names() {
    let text = "(defproject app \"0.1.0-SNAPSHOT\"\n  :dependencies [[\"org.clojure/clojure\" \"1.11.1\"]])\n";
    expect(
        "project.clj",
        text,
        &[
            ("leiningen", "0.1.0-SNAPSHOT", "project"),
            ("org.clojure/clojure", "1.11.1", "clojars"),
        ],
    );
}

#[test]
fn reads_carthage_urls_and_references() {
    let text = "git \"https://github.com/ReactiveX/RxSwift.git\" ~> 6.0\ngithub \"git@github.com:Alamofire/Alamofire.git\" >= 5.0\ngithub \"o/edge\" \"main\"\n";
    expect(
        "Cartfile",
        text,
        &[
            ("ReactiveX/RxSwift", "~> 6.0", "github"),
            ("Alamofire/Alamofire", ">= 5.0", "github"),
            ("o/edge", "main", "source"),
        ],
    );
}

#[test]
fn reads_parenthesized_requirements() {
    expect(
        "requirements.txt",
        "requests (>=2.0,<3)\n",
        &[("requests", ">=2.0,<3", "pypi")],
    );
}

#[test]
fn reads_each_version_bazel_lets_coexist() {
    let text = "bazel_dep(name = \"protobuf\", version = \"27.0\")\nmultiple_version_override(\n    module_name = \"protobuf\",\n    versions = [\"21.7\", \"27.0\"],\n)\n";
    expect(
        "MODULE.bazel",
        text,
        &[
            ("protobuf", "27.0", "bazel"),
            ("protobuf", "21.7", "bazel"),
            ("protobuf", "27.0", "bazel"),
        ],
    );
}

#[test]
fn reads_stack_snapshots_of_every_kind() {
    expect(
        "stack.yaml",
        "resolver: nightly-2026-05-29\n",
        &[("nightly", "2026-05-29", "stackage-nightly")],
    );
    expect(
        "stack.yaml",
        "snapshot: ghc-9.10.3\n",
        &[("ghc", "9.10.3", "ghc")],
    );
    let url = "https://raw.githubusercontent.com/commercialhaskell/stackage-snapshots/master/lts/24/43.yaml";
    expect(
        "stack.yaml",
        &format!("snapshot:\n  url: {url}\n  size: 729011\n"),
        &[("snapshot", url, "source")],
    );
    expect(
        "stack.yaml",
        "resolver: github:o/snaps:lts/1/2.yaml\n",
        &[("resolver", "github:o/snaps:lts/1/2.yaml", "source")],
    );
}

#[test]
fn reads_rust_channels_with_a_host() {
    expect(
        "rust-toolchain.toml",
        "[toolchain]\nchannel = \"1.75.0-x86_64-apple-darwin\"\n",
        &[("rust", "1.75.0", "rust")],
    );
}
