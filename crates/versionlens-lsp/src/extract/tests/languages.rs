//! Language package managers: Typst, Wally, `MoonBit`, Spago, Corral, Qlot, Lake, Raku META6, Elm, Kotlin scripts, Maven extensions, uv.toml, and Pixi.
//! Fixtures are excerpts of the real files or docs each row cites.

use super::expect;

#[test]
fn reads_typst_preview_imports() {
    let text = "#import \"@preview/cetz:0.2.2\": canvas, draw\n#import \"@local/mine:0.1.0\"\n#import \"@preview/fletcher:0.5.1\" as fl\n";
    expect(
        "slides/main.typ",
        text,
        &[("cetz", "0.2.2", "typst"), ("fletcher", "0.5.1", "typst")],
    );
}

#[test]
fn reads_a_typst_package_version_and_its_compiler() {
    // cetz 0.3.4's typst.toml from github.com/typst/packages, trimmed to its version lines.
    let text = "[package]\nname = \"cetz\"\nversion = \"0.3.4\"\ncompiler = \"0.13.0\"\nentrypoint = \"src/lib.typ\"\n";
    expect(
        "cetz/typst.toml",
        text,
        &[
            ("typst-package", "0.3.4", "project"),
            ("compiler", "0.13.0", "typst-compiler"),
        ],
    );
}

#[test]
fn reads_wally_dependencies() {
    // The Wally README's manifest; the package's own `name` has no version.
    let text = "[package]\nname = \"roblox/roact\"\nversion = \"1.4.0\"\nregistry = \"https://github.com/UpliftGames/wally-index\"\n\n[dependencies]\nPromise = \"evaera/promise@2.0.1\"\n\n[dev-dependencies]\nTestEZ = \"roblox/testez@0.4.1\"\n";
    expect(
        "wally.toml",
        text,
        &[
            ("evaera/promise", "2.0.1", "wally"),
            ("roblox/testez", "0.4.1", "wally"),
        ],
    );
}

#[test]
fn reads_moonbit_deps() {
    let text = "{\n  \"name\": \"username/hello\",\n  \"deps\": {\n    \"moonbitlang/x\": \"0.4.6\",\n    \"username/other\": { \"path\": \"../other\" }\n  }\n}\n";
    expect(
        "moon.mod.json",
        text,
        &[
            ("moonbitlang/x", "0.4.6", "mooncakes"),
            ("username/other", "../other", "directory"),
        ],
    );
}

#[test]
fn reads_spago_ranges() {
    // A package-set dependency names no range, so it gets no lens.
    let text = "package:\n  name: spago\n  build:\n    censorProjectWarnings:\n      - WildcardInferredType\n  dependencies:\n    - aff\n    - lists: \">=7.0.0 <8.0.0\"\n    - prelude: '>=6.0.0 <7.0.0'\n";
    expect(
        "spago.yaml",
        text,
        &[
            ("lists", ">=7.0.0 <8.0.0", "purescript"),
            ("prelude", ">=6.0.0 <7.0.0", "purescript"),
        ],
    );
}

#[test]
fn reads_corral_deps() {
    let text = "{\n  \"deps\": [\n    {\n      \"locator\": \"github.com/ponylang/ssl.git\",\n      \"version\": \"2.0.1\"\n    },\n    {\n      \"locator\": \"github.com/ponylang/valbytes.git\",\n      \"version\": \"0.6.2\"\n    },\n    {\n      \"locator\": \"gitlab.com/someone/pony-thing.git\",\n      \"version\": \">=1.1.0 <2.0.0\"\n    },\n    {\n      \"locator\": \"../pony-inspect\",\n      \"version\": \"\"\n    }\n  ]\n}\n";
    expect(
        "corral.json",
        text,
        &[
            ("ponylang/ssl", "2.0.1", "github"),
            ("ponylang/valbytes", "0.6.2", "github"),
            (
                "https://gitlab.com/someone/pony-thing",
                ">=1.1.0 <2.0.0",
                "source",
            ),
            ("../pony-inspect", "../pony-inspect", "directory"),
        ],
    );
}

#[test]
fn reads_qlfile_tags() {
    // The Qlot README's examples: dist dates from Quicklisp and Ultralisp, a branch or ref it
    // pins, and release tags.
    let text = "ql clack 2014-01-13\nql log4cl\nql-dist quicklisp mito 2023-02-15\nultralisp egao1980-cl-idna 20261119062000\nql-dist ultralisp datafly 20250101000000\ngit clack https://github.com/fukamachi/clack.git :branch develop\ngit datafly https://github.com/fukamachi/datafly.git :tag v0.7.4\ngithub fukamachi/sxql :tag 0.1.0\ngithub fukamachi/dexador :branch master\n";
    expect(
        "qlfile",
        text,
        &[
            ("clack", "2014-01-13", "quicklisp"),
            ("mito", "2023-02-15", "quicklisp"),
            ("egao1980-cl-idna", "20261119062000", "ultralisp"),
            ("datafly", "20250101000000", "ultralisp"),
            ("https://github.com/fukamachi/clack", "develop", "source"),
            ("fukamachi/datafly", "v0.7.4", "github"),
            ("fukamachi/sxql", "0.1.0", "github"),
            ("https://github.com/fukamachi/dexador", "master", "source"),
        ],
    );
}

#[test]
fn reads_the_racket_release_an_info_rkt_needs() {
    // The quoted, `list`, and deprecated forms from docs.racket-lang.org/pkg/metadata.html, and
    // the bracketed one threading-lib's info.rkt writes; an unversioned package or another
    // package's bound has no catalog version to compare.
    for text in [
        "#lang info\n(define deps\n  '([\"base\" #:version \"8.2\"]))\n",
        "#lang info\n(define deps '((\"base\" #:version \"8.2\") \"rackunit-lib\" (\"threading-lib\" #:version \"2.0\")))\n",
        "#lang info\n(define deps (list (list \"base\" '#:version \"8.2\")))\n",
        "#lang info\n(define build-deps '((\"base\" \"8.2\")))\n",
    ] {
        expect("info.rkt", text, &[("base", "8.2", "racket")]);
    }
    expect("info.rkt", "#lang info\n(define deps '(\"base\"))\n", &[]);
}

#[test]
fn reads_lake_git_revisions() {
    // The README's TOML and Lean `require` forms; a branch pins its git reference.
    let toml = "[[require]]\nname = \"batteries\"\ngit = \"https://github.com/leanprover-community/batteries\"\nrev = \"v4.15.0\"\n\n[[require]]\nname = \"aesop\"\ngit = \"https://github.com/leanprover-community/aesop\"\nrev = \"main\"\n";
    expect(
        "lakefile.toml",
        toml,
        &[
            ("leanprover-community/batteries", "v4.15.0", "github"),
            (
                "https://github.com/leanprover-community/aesop",
                "main",
                "source",
            ),
        ],
    );
    let lean = "require batteries from git\n  \"https://github.com/leanprover-community/batteries\" @ \"v4.15.0\"\nrequire proofwidgets from git \"https://git.example.com/pw.git\" @ \"v0.0.40\"\n";
    expect(
        "lakefile.lean",
        lean,
        &[
            ("leanprover-community/batteries", "v4.15.0", "github"),
            ("https://git.example.com/pw", "v0.0.40", "source"),
        ],
    );
}

#[test]
fn reads_lake_reservoir_tags() {
    // openai/ten-proofs' `rev` and kkytola/VirasoroProject's `version = "git#…"` lakefile.toml
    // requires; a scope with no tag, a branch, or a bare Reservoir version is not read.
    let toml = "[[require]]\nname = \"mathlib\"\nscope = \"leanprover-community\"\nrev = \"v4.32.0\"\n\n[[require]]\nscope = \"leanprover\"\nname = \"«doc-gen4»\"\nversion = \"git#v4.34.0\"\n\n[[require]]\nname = \"plausible\"\nscope = \"leanprover-community\"\nrev = \"main\"\n\n[[require]]\nname = \"aesop\"\nscope = \"leanprover-community\"\nversion = \"0.0.0\"\n\n[[require]]\nname = \"batteries\"\nscope = \"leanprover-community\"\n";
    expect(
        "lakefile.toml",
        toml,
        &[
            ("leanprover-community:mathlib", "v4.32.0", "reservoir"),
            ("leanprover:doc-gen4", "v4.34.0", "reservoir"),
        ],
    );
    let lean = "require \"leanprover-community\" / \"mathlib\" @ git \"v4.32.0\"\nrequire \"leanprover-community\" / batteries\n";
    expect(
        "lakefile.lean",
        lean,
        &[("leanprover-community:mathlib", "v4.32.0", "reservoir")],
    );
}

#[test]
fn reads_v_module_ranges() {
    // The range forms of V's vpm README; a bare module follows its default branch.
    let text = "Module {\n\tname: 'app'\n\tversion: '0.1.0'\n\tdependencies: [\n\t\t'vsl@^0.1.47',\n\t\t'nedpals.args@>=0.4.0 <0.6.0',\n\t\t'markdown',\n\t]\n}\n";
    expect(
        "v.mod",
        text,
        &[
            ("vsl", "^0.1.47", "vpm"),
            ("nedpals.args", ">=0.4.0 <0.6.0", "vpm"),
        ],
    );
}

#[test]
fn reads_lake_manifest_input_revisions() {
    // leanprover-community/mathlib4's lake-manifest.json; a branch `inputRev` pins its reference.
    let text = "{\"version\": \"1.3.0\",\n \"packages\":\n [{\"url\": \"https://github.com/leanprover-community/plausible\",\n   \"type\": \"git\",\n   \"rev\": \"0d8633ef\",\n   \"name\": \"plausible\",\n   \"inputRev\": \"main\"},\n  {\"url\": \"https://github.com/leanprover/lean4-cli\",\n   \"type\": \"git\",\n   \"rev\": \"c830de39\",\n   \"name\": \"Cli\",\n   \"inputRev\": \"v4.37.0-rc1\"}],\n \"name\": \"mathlib\"}\n";
    expect(
        "lake-manifest.json",
        text,
        &[
            (
                "https://github.com/leanprover-community/plausible",
                "main",
                "source",
            ),
            ("leanprover/lean4-cli", "v4.37.0-rc1", "github"),
        ],
    );
}

#[test]
fn reads_flake_lock_github_refs() {
    // neohaskell/NeoHaskell's neo-starter-flake.lock; a branch ref and a tarball input are not read.
    let text = "{\n  \"nodes\": {\n    \"cabal-32\": {\n      \"locked\": {\n        \"owner\": \"haskell\",\n        \"repo\": \"cabal\",\n        \"rev\": \"48bf1078\",\n        \"type\": \"github\"\n      },\n      \"original\": {\n        \"owner\": \"haskell\",\n        \"ref\": \"3.2\",\n        \"repo\": \"cabal\",\n        \"type\": \"github\"\n      }\n    },\n    \"flake-compat\": {\n      \"original\": {\n        \"owner\": \"input-output-hk\",\n        \"ref\": \"hkm/gitlab-fix\",\n        \"repo\": \"flake-compat\",\n        \"type\": \"github\"\n      }\n    },\n    \"nixpkgs\": {\n      \"original\": {\n        \"owner\": \"NixOS\",\n        \"ref\": \"nixos-24.05\",\n        \"repo\": \"nixpkgs\",\n        \"type\": \"github\"\n      }\n    },\n    \"tarball\": {\n      \"original\": {\n        \"type\": \"tarball\",\n        \"url\": \"https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.zst\"\n      }\n    }\n  },\n  \"root\": \"root\",\n  \"version\": 7\n}\n";
    expect(
        "flake.lock",
        text,
        &[
            ("haskell:cabal", "3.2", "github"),
            ("NixOS:nixpkgs", "24.05", "github"),
        ],
    );
}

#[test]
fn reads_idris_pack_collection() {
    // A pack.toml as stefan-hoeck/idris2-pack's config template writes it; a custom package's
    // commit pins its git reference.
    let text = "collection = \"nightly-220503\"\n\n[custom.all.literal]\ntype   = \"git\"\nurl    = \"https://github.com/stefan-hoeck/idris2-literal\"\ncommit = \"latest:main\"\n";
    expect(
        "pack.toml",
        text,
        &[
            ("nightly", "220503", "idris-pack"),
            (
                "https://github.com/stefan-hoeck/idris2-literal",
                "main",
                "source",
            ),
        ],
    );
}

#[test]
fn reads_raku_meta6_depends() {
    // croservices/cro-http's META6.json, and the S22 object form; a native library is not read.
    let text = "{\n  \"name\": \"Cro::HTTP\",\n  \"version\": \"0.8.13\",\n  \"auth\": \"zef:cro\",\n  \"depends\": [\n    \"IO::Socket::Async::SSL\",\n    \"HTTP::HPACK:ver<1.0.1+>\",\n    \"Cro::Core:ver<0.8.10+>:api<0>:auth<zef:cro>\",\n    \"curl:from<native>\"\n  ],\n  \"test-depends\": [\"Test::META\"],\n  \"provides\": {\"Cro::HTTP::Auth\": \"lib/Cro/HTTP/Auth.rakumod\"}\n}\n";
    expect(
        "META6.json",
        text,
        &[
            ("IO::Socket::Async::SSL", "", "raku"),
            ("HTTP::HPACK", "1.0.1+", "raku"),
            ("Cro::Core", "0.8.10+", "raku"),
            ("Test::META", "", "raku"),
        ],
    );
    let phased = "{\"depends\": {\"runtime\": {\"requires\": [\"JSON::Fast:ver<0.19+>\"]}, \"build\": {\"requires\": [\"System::Info\"]}}}";
    expect(
        "META6.json",
        phased,
        &[
            ("JSON::Fast", "0.19+", "raku"),
            ("System::Info", "", "raku"),
        ],
    );
}

#[test]
fn reads_elm_direct_indirect_and_test_dependencies() {
    // An application's elm.json, as `elm init` writes it: github.com/elm/compiler/blob/master/docs/elm.json/application.md
    let text = r#"{"type":"application","source-directories":["src"],"elm-version":"0.19.1","dependencies":{"direct":{"elm/core":"1.0.5"},"indirect":{"elm/json":"1.1.3"}},"test-dependencies":{"direct":{"elm-explorations/test":"2.2.0"},"indirect":{"elm/random":"1.0.0"}}}"#;
    expect(
        "elm.json",
        text,
        &[
            ("elm/core", "1.0.5", "elm"),
            ("elm/json", "1.1.3", "elm"),
            ("elm-explorations/test", "2.2.0", "elm"),
            ("elm/random", "1.0.0", "elm"),
        ],
    );
}

#[test]
fn reads_kotlin_script_depends_on() {
    // kotlinlang.org/docs/custom-script-deps-tutorial.html: `@file:DependsOn("group:artifact:version")`.
    let text = "@file:Repository(\"https://repo.maven.apache.org/maven2/\")\n@file:DependsOn(\"com.squareup.okhttp3:okhttp:4.12.0\")\nprintln(1)\n";
    expect(
        "build.main.kts",
        text,
        &[("com.squareup.okhttp3:okhttp", "4.12.0", "maven")],
    );
}

#[test]
fn reads_maven_core_extensions() {
    // maven.apache.org/configure.html#mvn-extensions-xml-file
    let text = "<extensions>\n  <extension>\n    <groupId>kr.motd.maven</groupId>\n    <artifactId>os-maven-plugin</artifactId>\n    <version>1.7.1</version>\n  </extension>\n</extensions>\n";
    expect(
        ".mvn/extensions.xml",
        text,
        &[("kr.motd.maven:os-maven-plugin", "1.7.1", "maven")],
    );
}

#[test]
fn reads_uv_toml_dependency_lists() {
    // uv.toml holds the `[tool.uv]` settings at its top level: docs.astral.sh/uv/reference/settings/
    let text = "dev-dependencies = [\"pytest>=8.0\"]\nconstraint-dependencies = [\"grpcio<1.65\"]\noverride-dependencies = [\"werkzeug==2.3.0\"]\n";
    expect(
        "uv.toml",
        text,
        &[
            ("pytest", ">=8.0", "pypi"),
            ("grpcio", "<1.65", "pypi"),
            ("werkzeug", "==2.3.0", "pypi"),
        ],
    );
}

#[test]
fn reads_pixi_dependency_tables() {
    // pixi.sh/latest/reference/pixi_manifest/#dependencies
    let text = "[workspace]\nname = \"app\"\nchannels = [\"conda-forge\"]\n\n[dependencies]\npython = \">=3.12\"\n\n[build-dependencies]\ncmake = \"3.29.*\"\n\n[host-dependencies]\nnumpy = \">=1.26\"\n\n[feature.test.dependencies]\npytest = \"*\"\n\n[target.linux-64.dependencies]\npatchelf = \"0.17.*\"\n";
    expect(
        "pixi.toml",
        text,
        &[
            ("python", ">=3.12", "conda"),
            ("cmake", "3.29.*", "conda"),
            ("numpy", ">=1.26", "conda"),
            ("pytest", "*", "conda"),
            ("patchelf", "0.17.*", "conda"),
        ],
    );
}

#[test]
fn reads_rescript_package_names() {
    // rescript-lang.org/docs/manual/build-configuration: the versions live in package.json.
    let text = r#"{"name":"app","sources":{"dir":"src"},"dependencies":["@rescript/react"],"dev-dependencies":["rescript-vitest"],"bs-dependencies":["@glennsl/rescript-json-combinators"]}"#;
    expect(
        "rescript.json",
        text,
        &[
            ("@rescript/react", "", "npm"),
            ("rescript-vitest", "", "npm"),
            ("@glennsl/rescript-json-combinators", "", "npm"),
        ],
    );
}
