//! Maven's mirror of Central, as <https://maven.apache.org/guides/mini/guide-mirror-settings.html>
//! and <https://maven.apache.org/settings.html> document it: the user's `~/.m2/settings.xml`,
//! dominant, then the install's `conf/settings.xml` (`MAVEN_HOME` or `M2_HOME`). The first
//! `<mirror>` whose `mirrorOf` names `central` outright, else the first whose pattern (`*`,
//! `external:*`, a comma list, `!id` to exclude) covers it, replaces Central; the `<server>` with
//! the mirror's `id` gives its user and password. Values expand `${env.NAME}`. A password
//! encrypted with `settings-security.xml` (`{...}`) is not decrypted, so that mirror goes without.

use std::path::Path;

use super::{Env, Target, Tree, base, basic, read, substitute};
use crate::extract::Dep;
use crate::flatten::Format;

pub(super) fn config(env: &Env) -> impl Fn(&Dep) -> Vec<Target> {
    let files = [
        env.home(".m2/settings.xml"),
        env.var("MAVEN_HOME")
            .or_else(|| env.var("M2_HOME"))
            .map(|h| Path::new(h).join("conf/settings.xml")),
    ];
    let mut mirrors = Vec::new();
    let mut servers = Vec::new();
    for text in files.into_iter().flatten().filter_map(|p| read(&p)) {
        let text = substitute(&text, "${env.", "}", env);
        let tree = Tree::parse(Format::Xml, &text);
        mirrors.extend(tree.records("settings.mirrors.mirror"));
        servers.extend(tree.records("settings.servers.server"));
    }
    let field = |record: &std::collections::HashMap<String, String>, key: &str| {
        record.get(key).map_or_default(|v| v.trim().to_owned())
    };
    let exact = mirrors.iter().find(|m| {
        field(m, "mirrorOf")
            .split(',')
            .any(|p| p.trim() == "central")
    });
    let mirror = exact.or_else(|| {
        mirrors
            .iter()
            .find(|m| covers_central(&field(m, "mirrorOf")))
    });
    let target = mirror.and_then(|m| {
        let url = field(m, "url");
        if url.is_empty() {
            return None;
        }
        let id = field(m, "id");
        let auth = servers.iter().find(|s| field(s, "id") == id).and_then(|s| {
            let password = field(s, "password");
            let encrypted = password.starts_with('{') && password.ends_with('}');
            (!encrypted).then(|| basic(&field(s, "username"), &password))
        });
        Some(Target::new(
            format!("{}/{{path}}/maven-metadata.xml", base(&url)),
            auth,
        ))
    });
    move |_: &Dep| target.clone().into_iter().collect()
}

/// Whether a `mirrorOf` pattern covers the `central` repository, which is external.
fn covers_central(pattern: &str) -> bool {
    let parts: Vec<&str> = pattern.split(',').map(str::trim).collect();
    !parts.contains(&"!central") && parts.iter().any(|p| matches!(*p, "*" | "external:*"))
}
