//! Language package managers: Typst, Wally, `MoonBit`, Spago, Corral, Qlot, Lake, and Raku META6.
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
    let text = "{\n  \"deps\": [\n    {\n      \"locator\": \"github.com/ponylang/ssl.git\",\n      \"version\": \"2.0.1\"\n    },\n    {\n      \"locator\": \"github.com/ponylang/valbytes.git\",\n      \"version\": \"0.6.2\"\n    }\n  ]\n}\n";
    expect(
        "corral.json",
        text,
        &[
            ("ponylang/ssl", "2.0.1", "github"),
            ("ponylang/valbytes", "0.6.2", "github"),
        ],
    );
}

#[test]
fn reads_qlfile_tags() {
    // The Qlot README's examples; a branch, commit, or dist date is not a release tag.
    let text = "ql clack 2014-01-13\ngit clack https://github.com/fukamachi/clack.git :branch develop\ngit datafly https://github.com/fukamachi/datafly.git :tag v0.7.4\ngithub fukamachi/sxql :tag 0.1.0\n";
    expect(
        "qlfile",
        text,
        &[
            ("fukamachi/datafly", "v0.7.4", "github"),
            ("fukamachi/sxql", "0.1.0", "github"),
        ],
    );
}

#[test]
fn reads_lake_git_revisions() {
    let toml = "[[require]]\nname = \"batteries\"\ngit = \"https://github.com/leanprover-community/batteries\"\nrev = \"v4.15.0\"\n\n[[require]]\nname = \"aesop\"\ngit = \"https://github.com/leanprover-community/aesop\"\nrev = \"main\"\n\n[[require]]\nname = \"mathlib\"\nscope = \"leanprover-community\"\n";
    expect(
        "lakefile.toml",
        toml,
        &[("leanprover-community/batteries", "v4.15.0", "github")],
    );
    let lean = "require batteries from git\n  \"https://github.com/leanprover-community/batteries\" @ \"v4.15.0\"\nrequire \"leanprover-community\" / \"mathlib\"\n";
    expect(
        "lakefile.lean",
        lean,
        &[("leanprover-community/batteries", "v4.15.0", "github")],
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
    // leanprover-community/mathlib4's lake-manifest.json; a branch `inputRev` is not a release.
    let text = "{\"version\": \"1.3.0\",\n \"packages\":\n [{\"url\": \"https://github.com/leanprover-community/plausible\",\n   \"type\": \"git\",\n   \"rev\": \"0d8633ef\",\n   \"name\": \"plausible\",\n   \"inputRev\": \"main\"},\n  {\"url\": \"https://github.com/leanprover/lean4-cli\",\n   \"type\": \"git\",\n   \"rev\": \"c830de39\",\n   \"name\": \"Cli\",\n   \"inputRev\": \"v4.37.0-rc1\"}],\n \"name\": \"mathlib\"}\n";
    expect(
        "lake-manifest.json",
        text,
        &[("leanprover/lean4-cli", "v4.37.0-rc1", "github")],
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
    // A pack.toml as stefan-hoeck/idris2-pack's config template writes it; a commit is not read.
    let text = "collection = \"nightly-220503\"\n\n[custom.all.literal]\ntype   = \"git\"\nurl    = \"https://github.com/stefan-hoeck/idris2-literal\"\ncommit = \"latest:main\"\n";
    expect("pack.toml", text, &[("nightly", "220503", "idris-pack")]);
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
