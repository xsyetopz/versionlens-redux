//! Language package managers: Typst, Wally, `MoonBit`, Spago, Corral, Qlot, and Lake.
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
