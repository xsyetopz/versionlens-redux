//! Runtime pins that name a release line rather than a version: nvm's `lts/*`, Bun's `canary`,
//! and rustup's dated and undated channels, including a TOML channel written with escapes.

use super::{extract, file_type};

/// Each dependency of the file, as its name, requirement, registry, and the text its span holds.
fn found(path: &str, text: &str) -> Vec<(String, String, &'static str, String)> {
    let file = file_type(path).unwrap_or_else(|| panic!("{path} not recognised"));
    extract(file, text)
        .into_iter()
        .map(|d| (d.name, d.req, d.registry, text[d.span].to_owned()))
        .collect()
}

fn expect(path: &str, text: &str, want: &[(&str, &str, &str, &str)]) {
    let want: Vec<_> = want
        .iter()
        .map(|(n, r, g, s)| ((*n).to_owned(), (*r).to_owned(), *g, (*s).to_owned()))
        .collect();
    assert_eq!(found(path, text), want, "{path}");
}

#[test]
fn reads_nvmrc_lts_alias_among_comments_and_reserved_keys() {
    // Arrange
    let text = "# Project runtime\n  lts/* # follow the active LTS line\nmirror=reserved\n";

    // Act and Assert
    expect(".nvmrc", text, &[("node", "", "nodejs-lts", "lts/*")]);
}

#[test]
fn reads_nvmrc_version_beside_reserved_keys_and_newest_release_alias() {
    // Arrange
    let pinned = "# runtime\nv20.11.1\nmirror=reserved\n";
    let newest = "node\n";

    // Act and Assert
    expect(".nvmrc", pinned, &[("node", "20.11.1", "npm", "20.11.1")]);
    expect(".node-version", newest, &[("node", "", "npm", "node")]);
}

#[test]
fn reads_bun_canary_and_latest_tags() {
    // Act and Assert
    expect(
        ".bun-version",
        "canary\n",
        &[("bun", "", "npm-canary", "canary")],
    );
    expect(".bun-version", "latest\n", &[("bun", "", "npm", "latest")]);
    expect(
        ".bun-version",
        "1.4.5\n",
        &[("bun", "1.4.5", "npm", "1.4.5")],
    );
}

#[test]
fn reads_legacy_rust_toolchain_channels() {
    // Act and Assert
    expect(
        "rust-toolchain",
        "nightly-2026-09-01\n",
        &[("rust", "2026-09-01", "rust-nightly", "2026-09-01")],
    );
    expect(
        "rust-toolchain",
        "beta-2026-11-01-x86_64-apple-darwin\n",
        &[("rust", "2026-11-01", "rust-beta", "2026-11-01")],
    );
    expect(
        "rust-toolchain",
        "stable\n",
        &[("rust", "", "rust", "stable")],
    );
    expect(
        "rust-toolchain",
        "nightly\n",
        &[("rust", "", "rust-nightly", "nightly")],
    );
    expect(
        "rust-toolchain",
        "1.80.0\n",
        &[("rust", "1.80.0", "rust", "1.80.0")],
    );
}

#[test]
fn reads_toml_rust_toolchain_channel_through_escapes() {
    // Arrange
    let escaped =
        "[toolchain]\nchannel = \"nightly\\u002d2026\\u002d09\\u002d01\" # preserve this comment\n";
    let release =
        "[toolchain]\nchannel = \"1.75.0-x86_64-apple-darwin\"\ncomponents = [\"rustfmt\"]\n";

    // Act and Assert
    expect(
        "rust-toolchain.toml",
        escaped,
        &[(
            "rust",
            "2026-09-01",
            "rust-nightly",
            "2026\\u002d09\\u002d01",
        )],
    );
    expect(
        "rust-toolchain.toml",
        release,
        &[("rust", "1.75.0", "rust", "1.75.0")],
    );
}
