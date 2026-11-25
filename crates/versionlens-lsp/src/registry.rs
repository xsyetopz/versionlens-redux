//! Every package registry as one data row: where its version list lives and how to read it.
//! Supporting a new registry means adding a row here.

use crate::version::Bare;

mod parse;
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
    Fn(fn(&str, &str) -> Versions),
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
    /// The package name OSV knows, as an [`expand`] template: GIT names a repository by URL, and
    /// Go names its toolchain `stdlib`.
    pub(crate) osv_name: &'static str,
    pub(crate) accept: &'static str,
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
        osv_name: "{name}",
        accept: "application/json",
        yanked: "",
        deprecated: "",
        filtered: "",
        fallback: "",
        paged: false,
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
        osv_name: "{name}",
        accept: "*/*",
        yanked: "",
        deprecated: "",
        filtered: "",
        fallback: "",
        paged: false,
    }
}

const MAVEN_VERSIONS: &str = r"<version>([^<]+)</version>";

const GITHUB: Registry = Registry {
    osv_name: "github.com/{name}",
    ..json(
        "github",
        "https://api.github.com/repos/{name}/tags?per_page=100",
        "",
        "name",
        Bare::Prefix,
        Some("GIT"),
    )
};

pub(crate) static REGISTRIES: &[Registry] = &[
    Registry {
        parse: Parse::Fn(parse::cargo),
        ..text(
            "cargo",
            "https://index.crates.io/{cargo}",
            "",
            Bare::Caret,
            Some("crates.io"),
        )
    },
    Registry {
        accept: "application/vnd.npm.install-v1+json",
        deprecated: "deprecated",
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
        yanked: "yanked",
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
        deprecated: "deprecated",
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
        yanked: "retracted",
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
    // Leiningen writes `group/artifact`, and a bare `name` for `name/name`; OSV wants `group:artifact`.
    // Leiningen and the Clojure CLI search Maven Central too, where `org.clojure/clojure` lives.
    Registry {
        osv_name: "{group}:{artifact}",
        fallback: "https://repo1.maven.org/maven2/{path}/maven-metadata.xml",
        ..text(
            "clojars",
            "https://repo.clojars.org/{path}/maven-metadata.xml",
            MAVEN_VERSIONS,
            Bare::Exact,
            Some("Maven"),
        )
    },
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
    Registry {
        filtered: "https://hub.docker.com/v2/repositories/{docker}/tags?page_size=100&name={filter}",
        ..json(
            "docker",
            "https://hub.docker.com/v2/repositories/{docker}/tags?page_size=100",
            "results",
            "name",
            Bare::Prefix,
            None,
        )
    },
    // An image on any registry but Docker Hub, through the OCI distribution API; see [`route`].
    Registry {
        paged: true,
        ..json(
            "oci",
            "https://{host}/v2/{repo}/tags/list?n=1000",
            "tags",
            "",
            Bare::Prefix,
            None,
        )
    },
    // OSV's GitHub Actions ecosystem has no version order, so a versioned query never matches;
    // its GIT ecosystem matches the repository's exact tag, for actions and C/C++ sources alike.
    GITHUB,
    // Swift packages are GitHub repositories; OSV files their advisories under SwiftURL.
    Registry {
        id: "swift",
        osv: Some("SwiftURL"),
        ..GITHUB
    },
    Registry {
        accept: "*/*",
        parse: Parse::Fn(parse::helm),
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
        None,
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
    // Puppetfile names a module `owner/name` or `owner-name`; extraction joins it as `owner:name`.
    json(
        "puppet",
        "https://forgeapi.puppet.com/v3/modules/{group}-{artifact}?exclude_fields=readme,changelog,license,reference",
        "releases",
        "version",
        Bare::Exact,
        None,
    ),
    // Supermarket lists each version as a URL; `latest_version` is skipped by its `:` before the quote.
    text(
        "chef",
        "https://supermarket.chef.io/api/v1/cookbooks/{name}",
        r#"[\[,]"[^"]*/versions/([^"]+)""#,
        Bare::Exact,
        None,
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
    (
        "maven",
        "https://central.sonatype.com/artifact/{group}/{artifact}",
    ),
    // cljdoc covers Clojars and Maven Central alike, as the Clojure tools search both.
    ("clojars", "https://cljdoc.org/d/{group}/{artifact}"),
    ("hex", "https://hex.pm/packages/{name}"),
    ("julia", "https://juliahub.com/ui/Packages/General/{name}"),
    ("haxelib", "https://lib.haxe.org/p/{name}/"),
    ("docker", "https://hub.docker.com/r/{docker}"),
    ("oci", "https://explore.ggcr.dev/?repo={name}"),
    ("oci ghcr.io", "https://ghcr.io/{repo}"),
    ("oci quay.io", "https://quay.io/repository/{repo}"),
    (
        "oci mcr.microsoft.com",
        "https://mcr.microsoft.com/artifact/mar/{repo}",
    ),
    ("oci public.ecr.aws", "https://gallery.ecr.aws/{repo}"),
    (
        "oci registry.gitlab.com",
        "https://gitlab.com/{repo}/container_registry",
    ),
    ("github", "https://github.com/{name}"),
    ("swift", "https://github.com/{name}"),
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
    (
        "puppet",
        "https://forge.puppet.com/modules/{group}/{artifact}",
    ),
    ("chef", "https://supermarket.chef.io/cookbooks/{name}"),
];

/// The human-readable page for a package.
pub(crate) fn page(id: &str, name: &str) -> String {
    let id = get(id).map_or(id, |registry| route(registry, name).id);
    // An image's page depends on its host, so `oci quay.io` outranks `oci`.
    let keys = [
        image_host(name).map(|host| format!("{id} {host}")),
        Some(id.to_owned()),
    ];
    let template = keys
        .iter()
        .flatten()
        .find_map(|key| PAGES.iter().chain(tools::PAGES).find(|(k, _)| k == key))
        .map_or("https://osv.dev/list?q={name}", |(_, t)| t);
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
/// `{group}`/`{artifact}` (the halves of `group:artifact` or `group/artifact`),
/// `{ns}`/`{short}` (the halves of `ns.name`), `{first}`/`{First}` (first letter),
/// `{tf}` (Terraform provider or module path), `{docker}` (Docker Hub repository),
/// `{host}`/`{repo}` (an image's registry host and the path after it), and
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
    if url.contains("{host}") || url.contains("{repo}") {
        let host = image_host(name).ok_or_else(|| format!("{name} names no registry host"))?;
        url = url
            .replace("{host}", host)
            .replace("{repo}", &name[host.len() + 1..]);
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

#[cfg(test)]
mod tests;
