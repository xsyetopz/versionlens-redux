//! Every package registry as one data row: where its version list lives and how to read it.
//! Supporting a new registry means adding a row here.

use serde_json::Value;

use crate::extract::regex;
use crate::version::Bare;

mod tools;

/// How a registry response lists versions.
#[derive(Debug)]
pub(crate) enum Parse {
    /// JSON: `at` is a `/`-separated path (a `{name}` segment is the package name) to an array,
    /// object, or string. `field` picks each element's field (`a|b` takes the first present),
    /// `@key` takes an object's keys, and an empty field takes the elements themselves.
    /// `latest` is a path to the registry's own latest tag, if it has one.
    Json {
        at: &'static str,
        field: &'static str,
        latest: &'static str,
    },
    /// The first capture group of each match.
    Regex(&'static str),
    /// Custom reader for formats neither covers.
    Fn(fn(&str, &str) -> Vec<String>),
}

/// A package registry.
#[derive(Debug)]
pub(crate) struct Registry {
    pub(crate) id: &'static str,
    /// URL template; see [`expand`] for the placeholders.
    pub(crate) url: &'static str,
    pub(crate) parse: Parse,
    /// What a requirement without an operator means here.
    pub(crate) bare: Bare,
    /// The OSV ecosystem name, where OSV tracks this registry.
    pub(crate) osv: Option<&'static str>,
    pub(crate) accept: &'static str,
}

/// The versions a registry lists for one package, plus its own idea of the latest.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Versions {
    pub(crate) list: Vec<String>,
    pub(crate) latest: Option<String>,
}

const fn json(
    id: &'static str,
    url: &'static str,
    at: &'static str,
    field: &'static str,
    bare: Bare,
    osv: Option<&'static str>,
) -> Registry {
    Registry {
        id,
        url,
        parse: Parse::Json {
            at,
            field,
            latest: "",
        },
        bare,
        osv,
        accept: "application/json",
    }
}

const fn text(
    id: &'static str,
    url: &'static str,
    pattern: &'static str,
    bare: Bare,
    osv: Option<&'static str>,
) -> Registry {
    Registry {
        id,
        url,
        parse: Parse::Regex(pattern),
        bare,
        osv,
        accept: "*/*",
    }
}

const MAVEN_VERSIONS: &str = r"<version>([^<]+)</version>";

pub(crate) static REGISTRIES: &[Registry] = &[
    text(
        "cargo",
        "https://index.crates.io/{cargo}",
        r#""vers":"([^"]+)"[^\n]*"yanked":false"#,
        Bare::Caret,
        Some("crates.io"),
    ),
    Registry {
        accept: "application/vnd.npm.install-v1+json",
        parse: Parse::Json {
            at: "versions",
            field: "@key",
            latest: "dist-tags/latest",
        },
        ..json(
            "npm",
            "https://registry.npmjs.org/{enc}",
            "",
            "",
            Bare::Exact,
            Some("npm"),
        )
    },
    Registry {
        parse: Parse::Json {
            at: "versions",
            field: "@key",
            latest: "latest",
        },
        ..json(
            "jsr",
            "https://jsr.io/{name}/meta.json",
            "",
            "",
            Bare::Exact,
            None,
        )
    },
    Registry {
        parse: Parse::Json {
            at: "versions",
            field: "@key",
            latest: "dist-tags/latest",
        },
        ..json(
            "unity",
            "https://packages.unity.com/{name}",
            "",
            "",
            Bare::Exact,
            None,
        )
    },
    Registry {
        accept: "application/vnd.pypi.simple.v1+json",
        ..json(
            "pypi",
            "https://pypi.org/simple/{name}/",
            "versions",
            "",
            Bare::Exact,
            Some("PyPI"),
        )
    },
    json(
        "composer",
        "https://repo.packagist.org/p2/{name}.json",
        "packages/{name}",
        "version",
        Bare::Exact,
        Some("Packagist"),
    ),
    text(
        "go",
        "https://proxy.golang.org/{go}/@v/list",
        r"(?m)^(v\S+)$",
        Bare::Exact,
        Some("Go"),
    ),
    json(
        "rubygems",
        "https://rubygems.org/api/v1/versions/{name}.json",
        "",
        "number",
        Bare::Exact,
        Some("RubyGems"),
    ),
    json(
        "cocoapods",
        "https://trunk.cocoapods.org/api/v1/pods/{name}",
        "versions",
        "name",
        Bare::Exact,
        None,
    ),
    Registry {
        parse: Parse::Json {
            at: "versions",
            field: "version",
            latest: "latest/version",
        },
        ..json(
            "pub",
            "https://pub.dev/api/packages/{name}",
            "",
            "",
            Bare::Exact,
            Some("Pub"),
        )
    },
    json(
        "nuget",
        "https://api.nuget.org/v3-flatcontainer/{lower}/index.json",
        "versions",
        "",
        Bare::Min,
        Some("NuGet"),
    ),
    text(
        "maven",
        "https://repo1.maven.org/maven2/{path}/maven-metadata.xml",
        MAVEN_VERSIONS,
        Bare::Exact,
        Some("Maven"),
    ),
    text(
        "clojars",
        "https://repo.clojars.org/{path}/maven-metadata.xml",
        MAVEN_VERSIONS,
        Bare::Exact,
        Some("Maven"),
    ),
    json(
        "hex",
        "https://hex.pm/api/packages/{name}",
        "releases",
        "version",
        Bare::Exact,
        Some("Hex"),
    ),
    text(
        "julia",
        "https://raw.githubusercontent.com/JuliaRegistries/General/master/{First}/{name}/Versions.toml",
        r#"(?m)^\["([^"]+)"\]"#,
        Bare::Caret,
        Some("Julia"),
    ),
    text(
        "haxelib",
        "https://lib.haxe.org/p/{name}/versions/",
        r"haxelib install \S+ ([0-9][\w.+-]*)",
        Bare::Exact,
        None,
    ),
    json(
        "docker",
        "https://hub.docker.com/v2/repositories/{docker}/tags?page_size=100",
        "results",
        "name",
        Bare::Prefix,
        None,
    ),
    json(
        "github",
        "https://api.github.com/repos/{name}/tags?per_page=100",
        "",
        "name",
        Bare::Prefix,
        Some("GitHub Actions"),
    ),
    Registry {
        accept: "*/*",
        parse: Parse::Fn(helm),
        ..json("helm", "{source}/index.yaml", "", "", Bare::Exact, None)
    },
    json(
        "galaxy",
        "https://galaxy.ansible.com/api/v3/plugin/ansible/content/published/collections/index/{ns}/{short}/versions/?limit=100",
        "data",
        "version",
        Bare::Exact,
        None,
    ),
    Registry {
        parse: Parse::Fn(tools::terraform),
        ..json(
            "terraform",
            "https://registry.terraform.io/v1/{tf}/versions",
            "",
            "",
            Bare::Exact,
            None,
        )
    },
    json(
        "bazel",
        "https://bcr.bazel.build/modules/{name}/metadata.json",
        "versions",
        "",
        Bare::Exact,
        None,
    ),
    json(
        "vcpkg",
        "https://raw.githubusercontent.com/microsoft/vcpkg/master/versions/{first}-/{name}.json",
        "versions",
        "version|version-semver|version-string|version-date",
        Bare::Min,
        None,
    ),
    text(
        "conan",
        "https://raw.githubusercontent.com/conan-io/conan-center-index/master/recipes/{name}/config.yml",
        r#"(?m)^  "?(\d[^":\s]*)"?:"#,
        Bare::Exact,
        Some("ConanCenter"),
    ),
    json(
        "cran",
        "https://crandb.r-pkg.org/{name}",
        "Version",
        "",
        Bare::Min,
        Some("CRAN"),
    ),
    json(
        "cpan",
        "https://fastapi.metacpan.org/v1/download_url/{name}",
        "version",
        "",
        Bare::Min,
        None,
    ),
    json(
        "dub",
        "https://code.dlang.org/api/packages/{name}/info",
        "versions",
        "version",
        Bare::Exact,
        None,
    ),
    json(
        "hackage",
        "https://hackage.haskell.org/package/{name}/preferred",
        "normal-version",
        "",
        Bare::Exact,
        Some("Hackage"),
    ),
    text(
        "opam",
        "https://opam.ocaml.org/packages/{name}/",
        r#"href="\.\./([^/]+)/[^/]+?\.(\d[^/"]*)/""#,
        Bare::Exact,
        Some("opam"),
    ),
];

const PAGES: &[(&str, &str)] = &[
    ("cargo", "https://crates.io/crates/{name}"),
    ("npm", "https://www.npmjs.com/package/{name}"),
    ("jsr", "https://jsr.io/{name}"),
    ("unity", "https://packages.unity.com/{name}"),
    ("pypi", "https://pypi.org/project/{name}/"),
    ("composer", "https://packagist.org/packages/{name}"),
    ("go", "https://pkg.go.dev/{name}"),
    ("rubygems", "https://rubygems.org/gems/{name}"),
    ("cocoapods", "https://cocoapods.org/pods/{name}"),
    ("pub", "https://pub.dev/packages/{name}"),
    ("nuget", "https://www.nuget.org/packages/{name}"),
    ("maven", "https://central.sonatype.com/artifact/{path}"),
    ("clojars", "https://clojars.org/{name}"),
    ("hex", "https://hex.pm/packages/{name}"),
    ("julia", "https://juliahub.com/ui/Packages/General/{name}"),
    ("haxelib", "https://lib.haxe.org/p/{name}/"),
    ("docker", "https://hub.docker.com/r/{docker}"),
    ("github", "https://github.com/{name}"),
    (
        "helm",
        "https://artifacthub.io/packages/search?ts_query_web={name}",
    ),
    (
        "galaxy",
        "https://galaxy.ansible.com/ui/repo/published/{ns}/{short}/",
    ),
    ("terraform", "https://registry.terraform.io/{tf}"),
    ("bazel", "https://registry.bazel.build/modules/{name}"),
    ("vcpkg", "https://vcpkg.io/en/package/{name}"),
    ("conan", "https://conan.io/center/recipes/{name}"),
    ("cran", "https://cran.r-project.org/package={name}"),
    ("cpan", "https://metacpan.org/pod/{name}"),
    ("dub", "https://code.dlang.org/packages/{name}"),
    ("hackage", "https://hackage.haskell.org/package/{name}"),
    ("opam", "https://opam.ocaml.org/packages/{name}/"),
];

/// The human-readable page for a package.
pub(crate) fn page(id: &str, name: &str) -> String {
    let template = PAGES
        .iter()
        .chain(tools::PAGES)
        .find(|(key, _)| *key == id)
        .map_or("https://osv.dev/list?q={name}", |(_, t)| t);
    expand(template, name, None)
        .unwrap_or_else(|_| format!("https://www.google.com/search?q={name}"))
}

/// The registry with this id.
pub(crate) fn get(id: &str) -> Option<&'static Registry> {
    REGISTRIES.iter().chain(tools::ROWS).find(|r| r.id == id)
}

/// Maps a registry named inside a requirement, such as deno's `npm:` prefix, to its row.
pub(crate) fn alias(name: &str) -> Option<&'static str> {
    // Tool-version files name tools differently from the registries that answer for them.
    const TOOLS: &[(&str, &str)] = &[("terraform", "terraform-cli"), ("pipx", "pypi")];
    TOOLS
        .iter()
        .find(|(tool, _)| *tool == name)
        .and_then(|(_, id)| get(id))
        .or_else(|| get(name))
        .map(|r| r.id)
}

/// Fills a URL template. Placeholders: `{name}`, `{lower}`, `{enc}` (npm scope escaping),
/// `{go}` (module case escaping), `{cargo}` (sparse index path), `{path}` (Maven group path),
/// `{ns}`/`{short}` (the halves of `ns.name`), `{first}`/`{First}` (first letter),
/// `{tf}` (Terraform provider or module path), `{docker}` (Docker Hub repository), and
/// `{source}` (the repository the manifest names).
///
/// # Errors
/// Names a placeholder that cannot be filled for this package.
pub(crate) fn expand(template: &str, name: &str, source: Option<&str>) -> Result<String, String> {
    let lower = name.to_lowercase();
    let first: String = lower.chars().take(1).collect();
    let (group, artifact) = name.split_once([':', '/']).unwrap_or((name, name));
    let (ns, short) = name.split_once('.').unwrap_or(("", name));
    let mut url = template.to_owned();
    for (key, value) in [
        ("{name}", name.to_owned()),
        ("{lower}", lower.clone()),
        ("{enc}", name.replace('/', "%2f")),
        (
            "{go}",
            name.chars()
                .map(|c| {
                    if c.is_ascii_uppercase() {
                        format!("!{}", c.to_ascii_lowercase())
                    } else {
                        c.to_string()
                    }
                })
                .collect(),
        ),
        ("{cargo}", cargo_path(&lower)),
        ("{path}", format!("{}/{artifact}", group.replace('.', "/"))),
        ("{ns}", ns.to_owned()),
        ("{short}", short.to_owned()),
        (
            "{tf}",
            if name.matches('/').count() >= 2 {
                format!("modules/{name}")
            } else {
                format!("providers/{name}")
            },
        ),
        ("{first}", first.clone()),
        ("{First}", first.to_uppercase()),
    ] {
        url = url.replace(key, &value);
    }
    if url.contains("{docker}") {
        let host = name
            .split('/')
            .next()
            .filter(|h| name.contains('/') && (h.contains('.') || h.contains(':')));
        let repo = match host {
            Some("docker.io") => name.trim_start_matches("docker.io/").to_owned(),
            Some(host) => {
                return Err(format!(
                    "images on {host} are not supported; only Docker Hub is"
                ));
            }
            None if name.contains('/') => name.to_owned(),
            None => format!("library/{name}"),
        };
        url = url.replace("{docker}", &repo);
    }
    if url.contains("{source}") {
        let source = source
            .filter(|s| s.starts_with("http"))
            .ok_or_else(|| format!("{name} names no http repository"))?;
        url = url.replace("{source}", source.trim_end_matches('/'));
    }
    Ok(url)
}

fn cargo_path(name: &str) -> String {
    match name.len() {
        1 => format!("1/{name}"),
        2 => format!("2/{name}"),
        3 => format!("3/{}/{name}", &name[..1]),
        _ => format!(
            "{}/{}/{name}",
            &name[..2],
            name.get(2..4).unwrap_or_default()
        ),
    }
}

impl Registry {
    /// Reads a response body into versions.
    ///
    /// # Errors
    /// Explains a body that is not the expected shape.
    pub(crate) fn parse(&self, body: &str, name: &str) -> Result<Versions, String> {
        let list = match &self.parse {
            // With two groups, the first names the package, for pages that also link other packages.
            Parse::Regex(pattern) => regex(pattern)
                .captures_iter(body)
                .filter_map(|c| match (c.get_group(1), c.get_group(2)) {
                    (Some(owner), Some(version)) => {
                        (&body[owner.range()] == name).then(|| body[version.range()].to_owned())
                    }
                    (Some(version), None) => Some(body[version.range()].to_owned()),
                    _ => None,
                })
                .collect(),
            Parse::Fn(read) => read(body, name),
            Parse::Json { at, field, latest } => {
                let root: Value =
                    serde_json::from_str(body).map_err(|e| format!("invalid JSON: {e}"))?;
                let node =
                    pointer(&root, at, name).ok_or_else(|| format!("no `{at}` in the response"))?;
                let latest = pointer(&root, latest, name)
                    .filter(|_| !latest.is_empty())
                    .and_then(Value::as_str)
                    .map(str::to_owned);
                return Ok(Versions {
                    list: elements(node, field),
                    latest,
                });
            }
        };
        Ok(Versions { list, latest: None })
    }
}

fn pointer<'a>(root: &'a Value, path: &str, name: &str) -> Option<&'a Value> {
    path.split('/')
        .filter(|s| !s.is_empty())
        .try_fold(root, |node, seg| {
            let seg = if seg == "{name}" { name } else { seg };
            match node {
                Value::Array(items) => items.get(seg.parse::<usize>().ok()?),
                _ => node.get(seg),
            }
        })
}

fn elements(node: &Value, field: &str) -> Vec<String> {
    let text = |v: &Value| {
        v.as_str()
            .map(str::to_owned)
            .or_else(|| v.as_number().map(ToString::to_string))
    };
    let pick = |v: &Value| {
        if field.is_empty() {
            text(v)
        } else {
            field.split('|').find_map(|f| v.get(f).and_then(text))
        }
    };
    match node {
        Value::Object(map) if field == "@key" => map.keys().cloned().collect(),
        Value::Object(map) => map.values().filter_map(pick).collect(),
        Value::Array(items) => items.iter().filter_map(pick).collect(),
        other => text(other).into_iter().collect(),
    }
}

/// A Helm repository index lists every chart; this reads the `version` of each entry of one.
fn helm(body: &str, name: &str) -> Vec<String> {
    let header = format!("{name}:");
    let mut lines = body
        .lines()
        .skip_while(|l| l.trim_start() != header || l.len() - l.trim_start().len() > 2)
        .skip(1);
    let mut item = None;
    let mut out = Vec::new();
    for line in lines.by_ref() {
        let indent = line.len() - line.trim_start().len();
        let content = line.trim_start();
        if content.is_empty() {
            continue;
        }
        if indent <= 2 && !content.starts_with('-') {
            break;
        }
        let (at, content) = match content.strip_prefix("- ") {
            Some(rest) => (indent + 2, rest),
            None => (indent, content),
        };
        let item_indent = *item.get_or_insert(at);
        if at == item_indent
            && let Some(version) = content.strip_prefix("version:")
        {
            out.push(version.trim().trim_matches(['"', '\'']).to_owned());
        }
    }
    out
}

#[cfg(test)]
mod tests;
