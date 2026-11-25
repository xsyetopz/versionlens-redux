//! What a dependency's name and requirement say about where it comes from: a local path, a
//! source no registry answers, or a platform rather than a package.

/// The registry id of dependencies on a local directory or file, which need no lookup.
pub(crate) const DIRECTORY: &str = "directory";

/// The local path a requirement names, such as npm's `file:../a` or a `../a` path, if it names one.
pub(crate) fn local_path(req: &str) -> Option<&str> {
    const SCHEMES: &[&str] = &["file://", "file:", "link:", "portal:", "path:"];
    SCHEMES
        .iter()
        .find_map(|s| req.strip_prefix(s))
        .or_else(|| {
            ["./", "../", "/", "~/"]
                .iter()
                .any(|p| req.starts_with(p))
                .then_some(req)
        })
        .filter(|path| !path.is_empty() && !path.starts_with("//"))
}

/// The registry id of a manifest's own version, which gets the next releases as updates, as
/// upstream's project version lenses offer, rather than a lookup.
pub(crate) const PROJECT: &str = "project";

/// The registry id of dependencies on a source no registry answers, such as a git repository or a
/// tarball URL. They need no lookup; their lens names the source, as the fork's master did.
pub(crate) const SOURCE: &str = "source";

/// The kind of source a requirement names when no registry answers it, labeled as the fork's
/// master labeled it: hosted URLs, `.git` paths, and SSH forms are git repositories, other URLs
/// are package URLs, and `workspace:` and `catalog:` point inside the repository.
pub(crate) fn source_label(req: &str) -> Option<&'static str> {
    const GIT: &[&str] = &[
        "git+",
        "git@",
        "git://",
        "ssh://",
        "github:",
        "gitlab:",
        "bitbucket:",
    ];
    const HOSTS: &[&str] = &["github.com", "gitlab.com", "bitbucket.org"];
    let req = req.trim();
    let lower = req.to_ascii_lowercase();
    let host = ["https://", "http://"]
        .iter()
        .find_map(|scheme| lower.strip_prefix(scheme))
        .and_then(|rest| rest.split('/').next())
        // The authority may carry a user and a port: `https://user@github.com:443/o/a`.
        .and_then(|authority| authority.rsplit('@').next())
        .and_then(|host| host.split(':').next());
    let bare = lower.split(['#', '?']).next().unwrap_or(&lower);
    if GIT.iter().any(|p| lower.starts_with(p))
        || std::path::Path::new(bare)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("git"))
        || host.is_some_and(|h| HOSTS.contains(&h) && !is_archive(bare))
    {
        Some("git repository")
    } else if host.is_some() {
        Some("package URL")
    } else if lower.starts_with("workspace:") {
        Some("workspace package")
    } else if lower.starts_with("catalog:") {
        Some("catalog entry")
    } else {
        None
    }
}

/// A release download or archive on a git host, which is a package URL rather than a repository.
fn is_archive(url: &str) -> bool {
    [".tar.gz", ".tgz", ".zip", ".tar.bz2", ".tar.xz", ".whl"]
        .iter()
        .any(|ext| url.ends_with(ext))
}

/// Requirements a registry cannot answer: paths, URLs, git refs, workspace links, and templating.
pub(super) fn is_registry_req(req: &str) -> bool {
    const LOCAL: &[&str] = &[
        "/",
        "./",
        "../",
        "~/",
        "file:",
        "link:",
        "workspace:",
        "catalog:",
        "portal:",
        "path:",
        "git",
        "github:",
        "gitlab:",
        "bitbucket:",
        "http:",
        "https:",
        "npm:",
        "jsr:",
        "sdk:",
        "patch:",
        "exec:",
    ];
    !(req.contains("://")
        || req.contains('$')
        || req.contains("{{")
        || LOCAL.iter().any(|p| req.starts_with(p)))
}

/// Names that are platforms or toolchains rather than packages. The check is per registry because
/// `python` is a runtime in a `PyPI` requirement but the package itself in `.python-version`.
pub(super) fn is_package(name: &str, registry: &str) -> bool {
    !(matches!(
        (registry, name),
        ("composer", "php")
            | ("pypi", "python")
            | ("julia", "julia")
            | ("cran", "R")
            | ("luarocks", "lua")
            | ("esp", "idf")
            // `FROM scratch` starts an empty image: docs.docker.com/build/building/base-images/
            | ("docker", "scratch")
    ) || (name.contains('$') && !is_instance_component(name, registry))
        // An ESP component is `namespace/name`; idf_component.yml's row reads each name once as
        // written and once under the default `espressif/`, and this keeps the one that fits.
        || (registry == "esp" && name.matches('/').count() > 1)
        // A Perl module never starts with `-`; dist.ini's `-phase` and `-relationship` configure
        // the Prereqs plugin itself.
        || (registry == "cpan" && name.starts_with('-'))
        || (registry == "composer"
            && (name.starts_with("ext-")
                || name.starts_with("lib-")
                || name.starts_with("composer-"))))
}

/// A GitLab component or included project on the pipeline's own instance, `$CI_SERVER_FQDN/group/project`, whose
/// variable `project::extract` replaces with the workspace remote's host.
pub(crate) fn is_instance_component(name: &str, registry: &str) -> bool {
    ["gitlab-host", crate::git_ref::GIT_REF].contains(&registry) && name.starts_with(INSTANCE)
}

/// The variable a GitLab pipeline names its own instance by, with the `/` that follows it:
/// <https://docs.gitlab.com/ci/variables/predefined_variables/>.
pub(crate) const INSTANCE: &str = "$CI_SERVER_FQDN/";
