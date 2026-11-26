//! Dependencies the fork's master read from public repositories that the rows first missed.

use super::{expect, sources};

#[test]
fn reads_npm_engines_runtimes_only() {
    // Arrange: a VS Code extension's `vscode` engine names no npm package.
    let text = r#"{"engines":{"node":">=20.9.0","vscode":"^1.90.0","pnpm":"^9"},"dependencies":{"a":"1.0.0"}}"#;

    // Act and Assert
    expect(
        "/w/package.json",
        text,
        &[
            ("node", ">=20.9.0", "npm"),
            ("pnpm", "^9", "npm"),
            ("a", "1.0.0", "npm"),
        ],
    );
}

#[test]
fn reads_pnpm_override_selectors_and_aliases() {
    // Arrange: vitejs/vite's pnpm-workspace.yaml; `-` removes the selected dependency.
    let text = "overrides:\n  debug: 'npm:obug@^1.0.2'\n  '@volar/typescript>typescript@>=7': '-'\n  'foo@1>bar': '2.0.0'\n  'a>@s/b@<2': '3.0.0'\n";

    // Act and Assert
    expect(
        "/w/pnpm-workspace.yaml",
        text,
        &[
            ("obug", "^1.0.2", "npm"),
            ("bar", "2.0.0", "npm"),
            ("@s/b", "3.0.0", "npm"),
        ],
    );
}

#[test]
fn reads_galaxy_entries_without_versions() {
    // Arrange: geerlingguy/mac-dev-playbook's requirements.yml names entries only.
    let text = "roles:\n  - name: geerlingguy.dotfiles\n  - include: other.yml\ncollections:\n  - name: geerlingguy.mac\n";

    // Act and Assert
    expect(
        "/w/requirements.yml",
        text,
        &[
            ("geerlingguy.dotfiles", "", "galaxy-role"),
            ("geerlingguy.mac", "", "galaxy"),
        ],
    );
}

#[test]
fn reads_wrapdb_patch_url() {
    // Arrange: GNOME/glib's subprojects/pcre2.wrap; the source's `pcre2-10.46` tag is the wrap's.
    let text = "[wrap-file]\ndirectory = pcre2-10.46\nsource_url = https://github.com/PCRE2Project/pcre2/releases/download/pcre2-10.46/pcre2-10.46.tar.bz2\npatch_url = https://wrapdb.mesonbuild.com/v2/pcre2_10.46-1/get_patch\nwrapdb_version = 10.46-1\n";
    let underscored = "[wrap-file]\npatch_url = https://wrapdb.mesonbuild.com/v2/nlohmann_json_3.11.2-1/get_patch\n";

    // Act and Assert
    expect(
        "/w/subprojects/pcre2.wrap",
        text,
        &[("pcre2", "10.46-1", "wrapdb")],
    );
    expect(
        "/w/subprojects/nlohmann_json.wrap",
        underscored,
        &[("nlohmann_json", "3.11.2-1", "wrapdb")],
    );
}

#[test]
fn reads_dune_constraints_whole() {
    // Arrange: ocaml/ocaml-lsp's dune-project.
    let text = "(package\n (name lsp)\n (depends\n  (jsonrpc (= :version))\n  yojson\n  (ppx_yojson_conv_lib (>= \"v0.14\"))\n  (cinaps :with-test)\n  (top-closure (or :with-test :with-dev-setup))\n  (ocamlformat (and :with-test (= 0.29.0)))\n  (base (and (>= v0.17.0) :with-test))))\n";

    // Act
    let deps = super::deps("dune-project", text);

    // Assert
    assert_eq!(
        deps,
        super::pairs(&[
            ("jsonrpc", ""),
            ("yojson", ""),
            ("ppx_yojson_conv_lib", "v0.14"),
            ("cinaps", ""),
            ("top-closure", ""),
            ("ocamlformat", "= 0.29.0"),
            ("base", ">= v0.17.0"),
        ])
    );
}

#[test]
fn reads_julia_deps_without_compat() {
    // Arrange: JuliaData/DataFrames.jl's Project.toml bounds Compat but no standard library.
    let text = "name = \"DataFrames\"\n\n[deps]\nCompat = \"34da2185-b29b-5c13-b0c7-acf172513d20\"\nRandom = \"9a3f8284-a2c9-5f02-9a11-845980a1fd5c\"\n\n[compat]\nCompat = \"4.2\"\njulia = \"1.10\"\n\n[extras]\nTest = \"8dfed614-e22c-5e08-85e1-65c5234f0b40\"\n";

    // Act and Assert
    expect(
        "/w/Project.toml",
        text,
        &[
            ("Random", "", "julia-stdlib"),
            ("Compat", "4.2", "julia"),
            ("julia", "1.10", "julia-lang"),
        ],
    );
}

#[test]
fn skips_leiningen_string_vectors() {
    // Arrange: ring-clojure/ring's project.clj.
    let text = "(defproject ring \"1.15.6\"\n  :dependencies [[ring/ring-core \"1.15.6\"]\n                 ;; dev\n                 [ring/ring-devel \"1.15.6\"]]\n  :sub [\"ring-core\"\n        \"ring-devel\"]\n  :aliases {\"test\" [\"sub\" \"test\"]})\n";

    // Act
    let deps = super::deps("project.clj", text);

    // Assert
    assert_eq!(
        deps,
        super::pairs(&[
            ("leiningen", "1.15.6"),
            ("ring/ring-core", "1.15.6"),
            ("ring/ring-devel", "1.15.6"),
        ])
    );
}

#[test]
fn reads_cabal_entries_with_and_without_versions() {
    // Arrange: hackage.haskell.org cabal files list one entry per line in leading- or
    // trailing-comma style; `tested-with` names compilers, not packages.
    let text = "cabal-version: 2.4\nname: x\ntested-with: GHC ==8.6.5\nlibrary\n  build-depends:\n      base >=4.12 && <5\n    , text\n    , containers ^>=0.6\n  if flag(dev)\n    build-depends: mtl,\n                   transformers >=0.5,\n";

    // Act and Assert
    expect(
        "/w/x.cabal",
        text,
        &[
            ("base", ">=4.12", "hackage"),
            ("text", "", "hackage"),
            ("containers", "^>=0.6", "hackage"),
            ("mtl", "", "hackage"),
            ("transformers", ">=0.5", "hackage"),
        ],
    );
}

#[test]
fn reads_bazel_override_arguments_in_either_order() {
    // Arrange: github.com/bazelbuild/bazel MODULE.bazel puts `patches` before `version`.
    let text = "single_version_override(\n    module_name = \"grpc\",\n    patches = [\"//:a.patch\"],\n    version = \"1.66.0\",\n)\nsingle_version_override(version = \"2.0\", module_name = \"zlib\")\n";

    // Act and Assert
    expect(
        "/w/MODULE.bazel",
        text,
        &[("grpc", "1.66.0", "bazel"), ("zlib", "2.0", "bazel")],
    );
}

#[test]
fn resolves_msbuild_property_from_same_file() {
    // Arrange: learn.microsoft.com/visualstudio/msbuild/msbuild-properties
    let text = "<Project><PropertyGroup><AVersion>1.2.3</AVersion></PropertyGroup><ItemGroup><PackageVersion Include=\"A\" Version=\"$(AVersion)\" /></ItemGroup></Project>";

    // Act and Assert
    expect(
        "/w/Directory.Packages.props",
        text,
        &[("A", "1.2.3", "nuget")],
    );
}

#[test]
fn labels_property_defined_elsewhere() {
    // Arrange: a parent POM or Directory.Build.props defines the property.
    let csproj = "<Project><ItemGroup><PackageReference Include=\"A\" Version=\"$(AVersion)\" /></ItemGroup></Project>";
    let pom = "<project><dependencies><dependency><groupId>g</groupId><artifactId>a</artifactId><version>${a.version}</version></dependency></dependencies></project>";

    // Act
    let nuget = sources("/w/x.csproj", csproj);
    let maven = sources("/w/pom.xml", pom);

    // Assert
    assert_eq!(nuget, [("A".to_owned(), "version property".to_owned())]);
    assert_eq!(maven, [("g:a".to_owned(), "version property".to_owned())]);
}

#[test]
fn labels_gradle_version_variable() {
    // Arrange: square/okhttp's build.gradle.kts takes `alpnBootVersion` from buildSrc.
    let text = "dependencies.create(\"org.mortbay.jetty.alpn:alpn-boot:$alpnBootVersion\")\nimplementation(\"g:b:${bVersion}\")\n";

    // Act
    let deps = sources("/w/build.gradle.kts", text);

    // Assert
    assert_eq!(
        deps,
        [
            (
                "org.mortbay.jetty.alpn:alpn-boot".to_owned(),
                "version property".to_owned()
            ),
            ("g:b".to_owned(), "version property".to_owned()),
        ]
    );
}
