//! Every package registry as one row of `catalog/registries/`: where its version list lives, how
//! to read it, where OSV files its advisories, and the page that shows a package. Supporting a
//! new registry means adding a row there.

use std::sync::LazyLock;

use crate::data::{self, Row};
use crate::version::Bare;

mod parse;

/// How a registry response lists versions.
#[derive(Clone, Debug)]
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
    Fn(fn(&str, &str) -> Versions),
}

/// A package registry.
#[derive(Clone, Debug)]
pub(crate) struct Registry {
    pub(crate) id: &'static str,
    /// URL template; see [`expand`] for the placeholders.
    pub(crate) url: &'static str,
    pub(crate) parse: Parse,
    /// What a requirement without an operator means here.
    pub(crate) bare: Bare,
    /// The OSV ecosystem name, where OSV tracks this registry.
    pub(crate) osv: Option<&'static str>,
    /// The package name OSV knows, as an [`expand`] template: GIT names a repository by URL, and
    /// Go names its toolchain `stdlib`.
    pub(crate) osv_name: &'static str,
    /// A regex with `owner` and `repo` groups that finds the package's GitHub repository in the
    /// document `osv_name` names, whose GIT advisories OSV then reports; empty where `osv_name`
    /// is the OSV name itself. `{version}` in `osv_name` names the version asked about. A `url`
    /// with `{upstream}` lists versions under what this pattern finds, by [`upstream_of`].
    pub(crate) upstream: &'static str,
    /// Further OSV ecosystems that hold advisories for the same package, each with its package
    /// name as an [`expand`] template, such as the Typst compiler's `typst-cli` crate on crates.io.
    pub(crate) also: &'static [(&'static str, &'static str)],
    pub(crate) accept: &'static str,
    /// A JSON body, as an [`expand`] template, that `url` is posted instead of fetched; empty for
    /// a GET. `CircleCI` lists an orb's versions only through its GraphQL endpoint.
    pub(crate) body: &'static str,
    /// A regex, read like `regex`, for an answer in HTML where `accept` asked for JSON; empty for
    /// none. PEP 691 lets a Python index answer only PEP 503 HTML, as `download.pytorch.org` does.
    pub(crate) html: &'static str,
    /// The field that marks a JSON version entry yanked, such as JSR's `yanked`; empty for none.
    pub(crate) yanked: &'static str,
    /// The field that marks a JSON version entry deprecated, such as npm's `deprecated`.
    pub(crate) deprecated: &'static str,
    /// A URL template, with `{filter}` for each of [`filters_of`], whose versions join the first
    /// page's; empty for none. Docker Hub lists tags newest first, 100 a page, so the tags of
    /// `node:20-alpine` are only on pages filtered by `20` and by `alpine`.
    pub(crate) filtered: &'static str,
    /// A URL template asked when `url` fails, read the same way; empty for none.
    pub(crate) fallback: &'static str,
    /// Whether the registry splits a version list over pages its `Link` headers name.
    pub(crate) paged: bool,
    /// The URL template of a package's page; empty where the registry has none.
    pub(crate) page: &'static str,
    /// Page templates by image host, such as `ghcr.io`, that outrank `page` for images there.
    pub(crate) pages: &'static [(&'static str, &'static str)],
}

/// The filters a requirement pages by, for a registry with filtered pages: its leading number
/// and its variant, such as `20` and `alpine` of `20-alpine3.22`.
pub(crate) fn filters_of(registry: &Registry, req: &str) -> Vec<String> {
    if registry.filtered.is_empty() {
        return Vec::new();
    }
    let req = req.trim_start_matches('v');
    let digits: String = req.chars().take_while(char::is_ascii_digit).collect();
    let variant = req.split_once('-').map_or("", |(_, rest)| {
        rest.trim_end_matches(|c: char| c.is_ascii_digit() || c == '.')
    });
    [digits, variant.to_owned()]
        .into_iter()
        .filter(|f| !f.is_empty())
        .collect()
}

/// The versions a registry lists for one package, plus its own idea of the latest. Yanked
/// versions are listed apart; deprecated ones stay in `list` and are named again in `deprecated`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Versions {
    pub(crate) list: Vec<String>,
    pub(crate) latest: Option<String>,
    pub(crate) yanked: Vec<String>,
    pub(crate) deprecated: Vec<String>,
}

static FILES: &[(&str, &str)] = &[
    (
        "registries/packages.toml",
        include_str!("../catalog/registries/packages.toml"),
    ),
    (
        "registries/tools.toml",
        include_str!("../catalog/registries/tools.toml"),
    ),
];

/// Every registry, in catalog order.
pub(crate) static REGISTRIES: LazyLock<Vec<Registry>> =
    LazyLock::new(|| load().expect("the embedded registry catalog loads"));

fn load() -> Result<Vec<Registry>, String> {
    let mut all = Vec::new();
    for (file, text) in FILES {
        for row in data::rows(file, text, "registry")? {
            all.push(registry(&row)?);
        }
    }
    Ok(all)
}

/// One `[[registry]]` row. It reads its versions with exactly one of `json` (a path, with
/// optional `field` and `latest`), `regex`, or `reader`, a name from [`parse::reader`].
fn registry(row: &Row) -> Result<Registry, String> {
    row.only(&[
        "id",
        "url",
        "json",
        "field",
        "latest",
        "regex",
        "reader",
        "bare",
        "osv",
        "osv_name",
        "upstream",
        "also",
        "accept",
        "body",
        "html",
        "yanked",
        "deprecated",
        "filtered",
        "fallback",
        "paged",
        "page",
        "pages",
    ])?;
    let parse = match (row.opt("json")?, row.opt("regex")?, row.opt("reader")?) {
        (Some(at), None, None) => Parse::Json {
            at,
            field: row.text("field")?,
            latest: row.text("latest")?,
        },
        (None, Some(pattern), None) => Parse::Regex(pattern),
        (None, None, Some(name)) => {
            Parse::Fn(parse::reader(name).ok_or_else(|| row.error(&format!("no reader {name}")))?)
        }
        _ => return Err(row.error("needs exactly one of json, regex, and reader")),
    };
    let accept = if matches!(parse, Parse::Json { .. }) {
        "application/json"
    } else {
        "*/*"
    };
    let bare = match row.need("bare")? {
        "exact" => Bare::Exact,
        "caret" => Bare::Caret,
        "min" => Bare::Min,
        "prefix" => Bare::Prefix,
        other => {
            return Err(row.error(&format!("bare {other} is not exact, caret, min, or prefix")));
        }
    };
    Ok(Registry {
        id: row.need("id")?,
        url: row.need("url")?,
        parse,
        bare,
        osv: row.opt("osv")?,
        osv_name: row.opt("osv_name")?.unwrap_or("{name}"),
        upstream: row.text("upstream")?,
        also: row.pairs("also")?.leak(),
        accept: row.opt("accept")?.unwrap_or(accept),
        body: row.text("body")?,
        html: row.text("html")?,
        yanked: row.text("yanked")?,
        deprecated: row.text("deprecated")?,
        filtered: row.text("filtered")?,
        fallback: row.text("fallback")?,
        paged: row.flag("paged")?,
        page: row.text("page")?,
        pages: row.pairs("pages")?.leak(),
    })
}

/// The human-readable page for a package.
pub(crate) fn page(id: &str, name: &str) -> String {
    let template = get(id)
        .map(|registry| route(registry, name))
        .and_then(|registry| {
            // An image's page depends on its host, so a quay.io page outranks the OCI one.
            let host = image_host(name);
            let by_host = registry.pages.iter().find(|(h, _)| Some(*h) == host);
            by_host
                .map(|(_, t)| *t)
                .or_else(|| (!registry.page.is_empty()).then_some(registry.page))
        })
        .unwrap_or("https://osv.dev/list?q={name}");
    expand(template, name, None)
        .unwrap_or_else(|_| format!("https://www.google.com/search?q={name}"))
}

/// The registry that answers for this name: Docker Hub's own API lists only its images, so an image
/// that names another host, such as `ghcr.io/owner/image`, goes to that host's OCI registry. A
/// `node` pin in `.nvmrc`, `.tool-versions`, or volta names the runtime, not npm's `node` package.
pub(crate) fn route(registry: &'static Registry, name: &str) -> &'static Registry {
    let id = match (registry.id, image_host(name)) {
        ("docker", Some(host)) if host != "docker.io" => "oci",
        ("npm", _) if name == "node" => "nodejs",
        _ => return registry,
    };
    get(id).unwrap_or(registry)
}

/// The registry host an image names, such as `ghcr.io` of `ghcr.io/owner/image`.
fn image_host(name: &str) -> Option<&str> {
    let (host, _) = name.split_once('/')?;
    (host.contains('.') || host.contains(':') || host == "localhost").then_some(host)
}

/// The registry with this id.
pub(crate) fn get(id: &str) -> Option<&'static Registry> {
    REGISTRIES.iter().find(|r| r.id == id)
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

/// The `owner/repo` a registry's `upstream` pattern finds in the document its `osv_name` names,
/// or `host/owner/repo` where the pattern captures a `host`,
/// with `{name}` in the pattern standing for the package name, so a search page's one exact hit
/// is the match: VPM's `"url"` names a GitHub repository, a raku.land search a `zef:auth/name`,
/// and a `PlatformIO` search an `owner/library/name` download.
pub(crate) fn upstream_of(pattern: &str, name: &str, text: &str) -> Option<String> {
    let escaped: String = name
        .chars()
        .map(|c| match c {
            // A name in a URL has its spaces encoded, as a PlatformIO download link does.
            ' ' => "(?: |%20)".to_owned(),
            c if r"\.+*?()|[]{}^$#&-~".contains(c) => format!("\\{c}"),
            c => c.to_string(),
        })
        .collect();
    let found = regex_automata::meta::Regex::new(&pattern.replace("{name}", &escaped)).ok()?;
    let caps = found.captures_iter(text).next()?;
    let group = |key| caps.get_group_by_name(key).map(|s| &text[s.range()]);
    let repo = group("repo")?;
    let repo = format!(
        "{}/{}",
        group("owner")?,
        repo.strip_suffix(".git").unwrap_or(repo)
    );
    // A pattern with a `host` group names a repository on any host, as a Racket package's source
    // may be on GitLab or Codeberg; the others name a GitHub one.
    Some(match group("host") {
        Some(host) => format!("{host}/{repo}"),
        None => repo,
    })
}

/// Fills a URL template. Placeholders: `{name}`, `{lower}`, `{enc}` (npm scope escaping),
/// `{go}` (module case escaping), `{cargo}` (sparse index path), `{path}` (Maven group path),
/// `{group}`/`{artifact}` (the halves of `group:artifact` or `group/artifact`),
/// `{ns}`/`{short}` (the halves of `ns.name`), `{first}`/`{First}` (first letter),
/// `{first2}` (first two letters), `{tf}` (Terraform provider or module path), `{docker}` (Docker
/// Hub repository),
/// `{host}`/`{repo}` (an image's registry host and the path after it), `{repo_enc}` (that path
/// with `/` escaped, as GitLab's API takes a project), and
/// `{source}` (the repository the manifest names). A registry URL's `{upstream}` is filled first,
/// by [`upstream_of`].
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
        ("{dist}", name.replace("::", "-")),
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
        ("{group}", group.to_owned()),
        ("{artifact}", artifact.to_owned()),
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
        ("{first2}", lower.chars().take(2).collect()),
        ("{first}", first.clone()),
        ("{First}", first.to_uppercase()),
    ] {
        url = url.replace(key, &value);
    }
    if url.contains("{docker}") {
        let repo = match image_host(name) {
            Some("docker.io") => name.trim_start_matches("docker.io/").to_owned(),
            Some(host) => return Err(format!("{host} is not Docker Hub")),
            None if name.contains('/') => name.to_owned(),
            None => format!("library/{name}"),
        };
        url = url.replace("{docker}", &repo);
    }
    if url.contains("{host}") || url.contains("{repo}") || url.contains("{repo_enc}") {
        let host = image_host(name).ok_or_else(|| format!("{name} names no registry host"))?;
        let repo = &name[host.len() + 1..];
        url = url
            .replace("{host}", host)
            .replace("{repo_enc}", &repo.replace('/', "%2f"))
            .replace("{repo}", repo);
    }
    if url.contains("{app}") {
        let app = image_app(name).ok_or_else(|| format!("{name} names no known application"))?;
        url = url.replace("{app}", app);
    }
    if url.contains("{source}") {
        let source = source
            .filter(|s| s.starts_with("http"))
            .ok_or_else(|| format!("{name} names no http repository"))?;
        url = url.replace("{source}", source.trim_end_matches('/'));
    }
    Ok(url)
}

/// The application an image packages, in OSV's Bitnami names, where the image's name says which:
/// official images, Bitnami's and `HashiCorp`'s, and images named after their publisher, such as
/// `grafana/grafana`. A `myorg/nginx` tag need not version nginx, so it gets none.
fn image_app(name: &str) -> Option<&str> {
    let repo = image_host(name).map_or(name, |host| &name[host.len() + 1..]);
    let repo = repo.strip_prefix("docker/").unwrap_or(repo);
    let app = match repo.rsplit_once('/') {
        None => repo,
        Some(("library" | "bitnami" | "hashicorp", app)) => app,
        Some((owner, app)) if owner.rsplit('/').next() == Some(app) => app,
        Some(_) => return None,
    };
    Some(match app {
        "httpd" => "apache",
        "postgres" => "postgresql",
        "mongo" => "mongodb",
        "openjdk" | "eclipse-temurin" | "amazoncorretto" => "java",
        app => app,
    })
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

#[cfg(test)]
mod tests;
