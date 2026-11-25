//! Registries for language runtimes, build tools, and the smaller package indexes.
//! Runtimes answer for tool-version files such as `.python-version` and `.tool-versions`.

use serde_json::Value;

use super::{Parse, Registry, Versions, json, text};
use crate::version::Bare;

pub(super) static ROWS: &[Registry] = &[
    Registry {
        osv_name: "https://github.com/rust-lang/rust",
        ..text(
            "rust",
            "https://raw.githubusercontent.com/rust-lang/rust/master/RELEASES.md",
            r"(?m)^Version (\d+\.\d+\.\d+) \(",
            Bare::Prefix,
            Some("GIT"),
        )
    },
    // OSV's Bitnami ecosystem carries the runtimes' own CVEs, under each product's name.
    Registry {
        osv_name: "python",
        ..json(
            "python",
            "https://raw.githubusercontent.com/actions/python-versions/main/versions-manifest.json",
            "",
            "version",
            Bare::Prefix,
            Some("Bitnami"),
        )
    },
    Registry {
        osv_name: "ruby",
        ..text(
            "ruby",
            "https://raw.githubusercontent.com/ruby/www.ruby-lang.org/master/_data/releases.yml",
            r"(?m)^- version: (\d\S*)",
            Bare::Prefix,
            Some("Bitnami"),
        )
    },
    Registry {
        osv_name: "node",
        ..json(
            "nodejs",
            "https://nodejs.org/dist/index.json",
            "",
            "version",
            Bare::Prefix,
            Some("Bitnami"),
        )
    },
    Registry {
        osv_name: "ghc",
        ..text(
            "ghc",
            "https://downloads.haskell.org/~ghc/",
            r#"href="(\d+\.\d+\.\d+)/""#,
            Bare::Prefix,
            Some("GHC"),
        )
    },
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
    Registry {
        osv_name: "https://github.com/hashicorp/terraform",
        ..json(
            "terraform-cli",
            "https://releases.hashicorp.com/terraform/index.json",
            "versions",
            "@key",
            Bare::Prefix,
            Some("GIT"),
        )
    },
    Registry {
        osv_name: "gradle",
        ..json(
            "gradle-dist",
            "https://services.gradle.org/versions/all",
            "",
            "version",
            Bare::Exact,
            Some("Bitnami"),
        )
    },
    Registry {
        osv_name: "https://api.anaconda.org/package/conda-forge/{name}",
        upstream: github!(r#""dev_url":\s*""#),
        ..json(
            "conda",
            "https://api.anaconda.org/package/conda-forge/{name}",
            "versions",
            "",
            Bare::Prefix,
            Some("GIT"),
        )
    },
    Registry {
        osv_name: "https://github.com/{name}",
        ..json(
            "elm",
            "https://package.elm-lang.org/packages/{name}/releases.json",
            "",
            "@key",
            Bare::Exact,
            Some("GIT"),
        )
    },
    Registry {
        osv_name: "https://luarocks.org/{name}-{version}.rockspec",
        upstream: r#"url\s*=\s*['"](?:git\+)?(?:https?|git)://github\.com/(?P<owner>[\w.-]+)/(?P<repo>[\w.-]+?)(?:\.git)?/?['"]"#,
        ..json(
            "luarocks",
            "https://luarocks.org/manifest.json",
            "repository/{name}",
            "@key",
            Bare::Exact,
            Some("GIT"),
        )
    },
    Registry {
        osv_name: "https://raw.githubusercontent.com/xmake-io/xmake-repo/master/packages/{first}/{lower}/xmake.lua",
        upstream: r#""https?://github\.com/(?P<owner>[\w.-]+)/(?P<repo>[\w.-]+?)(?:\.git)?["/]"#,
        ..text(
            "xmake",
            "https://raw.githubusercontent.com/xmake-io/xmake-repo/master/packages/{first}/{lower}/xmake.lua",
            r#"add_versions\("(?:[a-z]+:)?([^"]+)""#,
            Bare::Exact,
            Some("GIT"),
        )
    },
];

pub(super) const PAGES: &[(&str, &str)] = &[
    ("rust", "https://www.rust-lang.org/tools/install"),
    ("python", "https://www.python.org/downloads/"),
    ("ruby", "https://www.ruby-lang.org/en/downloads/"),
    ("nodejs", "https://nodejs.org/en/about/previous-releases"),
    ("ghc", "https://www.haskell.org/ghc/"),
    ("golang", "https://go.dev/dl/"),
    ("terraform-cli", "https://releases.hashicorp.com/terraform/"),
    ("gradle-dist", "https://gradle.org/releases/"),
    ("conda", "https://anaconda.org/conda-forge/{name}"),
    (
        "elm",
        "https://package.elm-lang.org/packages/{name}/latest/",
    ),
    ("luarocks", "https://luarocks.org/search?q={name}"),
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
