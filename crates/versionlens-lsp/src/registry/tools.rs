//! Registries for language runtimes, build tools, and the smaller package indexes.
//! Runtimes answer for tool-version files such as `.python-version` and `.tool-versions`.

use serde_json::Value;

use super::{Parse, Registry, Versions, json, text};
use crate::version::Bare;

pub(super) static ROWS: &[Registry] = &[
    text(
        "rust",
        "https://raw.githubusercontent.com/rust-lang/rust/master/RELEASES.md",
        r"(?m)^Version (\d+\.\d+\.\d+) \(",
        Bare::Prefix,
        None,
    ),
    json(
        "python",
        "https://raw.githubusercontent.com/actions/python-versions/main/versions-manifest.json",
        "",
        "version",
        Bare::Prefix,
        None,
    ),
    text(
        "ruby",
        "https://raw.githubusercontent.com/ruby/www.ruby-lang.org/master/_data/releases.yml",
        r"(?m)^- version: (\d\S*)",
        Bare::Prefix,
        None,
    ),
    Registry {
        parse: Parse::Fn(go_releases),
        osv_name: "stdlib",
        ..json(
            "golang",
            "https://go.dev/dl/?mode=json&include=all",
            "",
            "",
            Bare::Prefix,
            Some("Go"),
        )
    },
    json(
        "terraform-cli",
        "https://releases.hashicorp.com/terraform/index.json",
        "versions",
        "@key",
        Bare::Prefix,
        None,
    ),
    json(
        "gradle-dist",
        "https://services.gradle.org/versions/all",
        "",
        "version",
        Bare::Exact,
        None,
    ),
    json(
        "conda",
        "https://api.anaconda.org/package/conda-forge/{name}",
        "versions",
        "",
        Bare::Prefix,
        None,
    ),
    json(
        "elm",
        "https://package.elm-lang.org/packages/{name}/releases.json",
        "",
        "@key",
        Bare::Exact,
        None,
    ),
    json(
        "luarocks",
        "https://luarocks.org/manifest.json",
        "repository/{name}",
        "@key",
        Bare::Exact,
        None,
    ),
    text(
        "xmake",
        "https://raw.githubusercontent.com/xmake-io/xmake-repo/master/packages/{first}/{lower}/xmake.lua",
        r#"add_versions\("(?:[a-z]+:)?([^"]+)""#,
        Bare::Exact,
        None,
    ),
];

pub(super) const PAGES: &[(&str, &str)] = &[
    ("rust", "https://www.rust-lang.org/tools/install"),
    ("python", "https://www.python.org/downloads/"),
    ("ruby", "https://www.ruby-lang.org/en/downloads/"),
    ("golang", "https://go.dev/dl/"),
    ("terraform-cli", "https://releases.hashicorp.com/terraform/"),
    ("gradle-dist", "https://gradle.org/releases/"),
    ("conda", "https://anaconda.org/conda-forge/{name}"),
    (
        "elm",
        "https://package.elm-lang.org/packages/{name}/latest/",
    ),
    ("luarocks", "https://luarocks.org/modules/{name}"),
    ("xmake", "https://xmake.io/#/package/remote_package"),
];

/// The Go release feed names versions `go1.22.1`; requirements and `.go-version` drop the prefix.
fn go_releases(body: &str, _name: &str) -> Versions {
    let Ok(Value::Array(items)) = serde_json::from_str::<Value>(body) else {
        return Versions::default();
    };
    items
        .iter()
        .filter_map(|item| item.get("version")?.as_str()?.strip_prefix("go"))
        .map(str::to_owned)
        .collect::<Vec<_>>()
        .into()
}

/// Terraform's provider endpoint lists `versions`; its module endpoint nests them in `modules`.
pub(super) fn terraform(body: &str, _name: &str) -> Versions {
    let Ok(root) = serde_json::from_str::<Value>(body) else {
        return Versions::default();
    };
    let versions = root
        .get("versions")
        .or_else(|| root.pointer("/modules/0/versions"));
    let Some(items) = versions.and_then(Value::as_array) else {
        return Versions::default();
    };
    items
        .iter()
        .filter_map(|item| item.get("version")?.as_str().map(str::to_owned))
        .collect::<Vec<_>>()
        .into()
}
