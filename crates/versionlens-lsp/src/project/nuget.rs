//! `NuGet`'s package sources, as
//! <https://learn.microsoft.com/en-us/nuget/reference/nuget-config-file> and
//! <https://learn.microsoft.com/en-us/nuget/consume-packages/configuring-nuget-behavior>
//! document them: the user's `NuGet.Config`, then each `nuget.config` from the root down to the
//! manifest's folder, the nearer winning and `<clear />` dropping what came before.
//! `disabledPackageSources` turns a source off, `packageSourceCredentials` holds a
//! `Username` and `ClearTextPassword` per source name (spaces written `_x0020_`), and
//! `packageSourceMapping` routes package-id patterns to sources, the most specific pattern
//! winning. Values expand `%NAME%`. When no file names a source, the original
//! `versionlens.dotnet.nuget.sources` setting gives them, and without it the sources
//! `dotnet nuget list source --format Short` reports enabled (`E url`) are used instead, which
//! covers machine-wide files this reader does not look for.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex, PoisonError};

use super::{Env, Target, Tree, basic, glob, read, substitute};
use crate::extract::Dep;
use crate::flatten::Format;

const NUGET_ORG: &str = "https://api.nuget.org/v3/index.json";
const FLAT: &str = "https://api.nuget.org/v3-flatcontainer/{lower}/index.json";
const NAMES: &[&str] = &["nuget.config", "NuGet.config", "NuGet.Config"];

#[derive(Default)]
struct Sources {
    /// Name and URL, in the order `NuGet` lists them.
    list: Vec<(String, String)>,
    disabled: Vec<String>,
    auth: HashMap<String, String>,
    /// Source name and the package-id patterns routed to it.
    mapping: Vec<(String, String)>,
}

pub(super) fn config(dir: &Path, env: &Env) -> impl Fn(&Dep) -> Vec<Target> {
    let mut files: Vec<PathBuf> = dir
        .ancestors()
        .filter_map(|d| NAMES.iter().map(|n| d.join(n)).find(|p| p.is_file()))
        .collect();
    files.extend(
        env.var("APPDATA")
            .map(|d| Path::new(d).join("NuGet/NuGet.Config"))
            .or_else(|| env.home(".nuget/NuGet/NuGet.Config")),
    );
    let mut sources = Sources::default();
    for file in files.iter().rev() {
        if let Some(text) = read(file) {
            apply(&mut sources, &text, env);
        }
    }
    if sources.list.is_empty() {
        sources.list = env
            .nuget
            .iter()
            .map(|u| (String::new(), u.clone()))
            .collect();
    }
    if sources.list.is_empty()
        && let Some(listed) = listed(dir, env)
    {
        sources.list = listed
            .lines()
            .filter_map(|l| l.trim().strip_prefix("E "))
            .map(|url| (String::new(), url.trim().to_owned()))
            .collect();
    }
    move |dep: &Dep| {
        let matching: Vec<&(String, String)> = sources
            .mapping
            .iter()
            .filter(|(_, pattern)| glob(pattern, &dep.name))
            .collect();
        let longest = matching.iter().map(|(_, p)| p.len()).max();
        sources
            .list
            .iter()
            .filter(|(name, _)| !sources.disabled.contains(name))
            .filter(|(name, _)| {
                sources.mapping.is_empty()
                    || matching
                        .iter()
                        .any(|(source, p)| source == name && Some(p.len()) == longest)
            })
            .filter_map(|(name, url)| target(url, sources.auth.get(name).cloned()))
            .collect()
    }
}

/// What `dotnet nuget list source` prints in `dir`, run once per folder: it takes a third of a
/// second, and the server reads a manifest again on every edit.
fn listed(dir: &Path, env: &Env) -> Option<String> {
    static LISTED: LazyLock<Mutex<HashMap<PathBuf, Option<String>>>> =
        LazyLock::new(Mutex::default);
    let mut cache = LISTED.lock().unwrap_or_else(PoisonError::into_inner);
    cache
        .entry(dir.to_owned())
        .or_insert_with(|| {
            (env.run)(
                dir,
                "dotnet",
                &["nuget", "list", "source", "--format", "Short"],
            )
        })
        .clone()
}

/// A v3 source as a target; v2 feeds, which list versions through `OData`, are left out.
fn target(url: &str, auth: Option<String>) -> Option<Target> {
    if url.eq_ignore_ascii_case(NUGET_ORG) && auth.is_none() {
        return Some(Target::new(FLAT.to_owned(), None));
    }
    let lower = url.to_ascii_lowercase();
    (lower.starts_with("http")
        && Path::new(&lower)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("json")))
    .then(|| Target {
        url: url.to_owned(),
        auth,
        index: true,
    })
}

fn apply(sources: &mut Sources, text: &str, env: &Env) {
    let tree = Tree::parse(Format::Xml, text);
    let section = |name: &str| {
        let start = text.find(&format!("<{name}"))?;
        let end = text[start..].find(&format!("</{name}>"))?;
        Some(&text[start..start + end])
    };
    if section("packageSources").is_some_and(|s| s.contains("<clear")) {
        sources.list.clear();
    }
    for record in tree.records("configuration.packageSources.add") {
        if let (Some(key), Some(value)) = (record.get("@key"), record.get("@value")) {
            let url = substitute(value, "%", "%", env);
            sources.list.retain(|(name, _)| name != key);
            sources.list.push((key.clone(), url));
        }
    }
    for record in tree.records("configuration.disabledPackageSources.add") {
        if let Some(key) = record.get("@key")
            && record
                .get("@value")
                .is_some_and(|v| v.eq_ignore_ascii_case("true"))
        {
            sources.disabled.push(key.clone());
        }
    }
    // Each `add` element is one field: its attributes share the element as their parent.
    let mut fields: HashMap<usize, (String, Option<String>, Option<String>)> = HashMap::new();
    for (rest, value, parent) in tree.under("configuration.packageSourceCredentials") {
        if let Some((source, attr)) = rest.split_once(".add.") {
            let field = fields
                .entry(parent)
                .or_insert_with(|| (source.replace("_x0020_", " "), None, None));
            match attr {
                "@key" => field.1 = Some(value.to_owned()),
                "@value" => field.2 = Some(substitute(value, "%", "%", env)),
                _ => {}
            }
        }
    }
    let mut credentials: HashMap<String, HashMap<String, String>> = HashMap::new();
    for (source, key, value) in fields.into_values() {
        if let (Some(key), Some(value)) = (key, value) {
            credentials.entry(source).or_default().insert(key, value);
        }
    }
    for (source, fields) in credentials {
        if let (Some(user), Some(password)) =
            (fields.get("Username"), fields.get("ClearTextPassword"))
        {
            sources.auth.insert(source, basic(user, password));
        }
    }
    // Leaves come in document order, so each `package` follows the `packageSource` holding it.
    let mut current = None;
    for (rest, value, _) in tree.under("configuration.packageSourceMapping.packageSource") {
        match rest.as_str() {
            "@key" => current = Some(value.to_owned()),
            "package.@pattern" => {
                if let Some(source) = &current {
                    sources.mapping.push((source.clone(), value.to_owned()));
                }
            }
            _ => {}
        }
    }
}
