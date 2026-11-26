//! npm's registry configuration, as <https://docs.npmjs.com/cli/v10/configuring-npm/npmrc>
//! documents it: the project `.npmrc` beats the user's (`NPM_CONFIG_USERCONFIG`, else
//! `~/.npmrc`), and `npm_config_*` variables beat both. `@scope:registry` routes a scope, and
//! `//host/path/:_authToken`, `:_auth`, or `:username` with `:_password` authenticate the
//! registry whose URL that prefix starts. Values expand `${NAME}`. A `.env` beside the project
//! `.npmrc` supplies variables the environment lacks, as the fork's master read it.

use std::collections::HashMap;
use std::path::Path;

use super::{Env, Target, base, basic, read, substitute, unbase64};
use crate::extract::Dep;

const DEFAULT: &str = "https://registry.npmjs.org/";

pub(super) fn config(dir: &Path, env: &Env) -> impl Fn(&Dep) -> Vec<Target> {
    let project = dir
        .ancestors()
        .map(|d| d.join(".npmrc"))
        .find(|p| p.is_file());
    let mut vars = env.vars.clone();
    if let Some(text) = project
        .as_ref()
        .and_then(|p| read(&p.with_file_name(".env")))
    {
        for (key, value) in ini(&text) {
            vars.entry(key).or_insert(value);
        }
    }
    let env = Env {
        vars,
        home: env.home.clone(),
        run: env.run,
        nuget: env.nuget.clone(),
    };
    let user = env
        .var("NPM_CONFIG_USERCONFIG")
        .or_else(|| env.var("npm_config_userconfig"))
        .map(Into::into)
        .or_else(|| env.home(".npmrc"));
    let mut values: HashMap<String, String> = HashMap::new();
    for file in [user, project].into_iter().flatten() {
        if let Some(text) = read(&file) {
            values.extend(
                ini(&text)
                    .into_iter()
                    .map(|(k, v)| (k, substitute(&v, "${", "}", &env))),
            );
        }
    }
    for (key, value) in &env.vars {
        let lower = key.to_ascii_lowercase();
        if let Some(name) = lower.strip_prefix("npm_config_")
            && (name == "registry" || name.ends_with(":registry"))
        {
            values.insert(name.to_owned(), value.clone());
        }
    }
    move |dep: &Dep| {
        let scoped = dep
            .name
            .split_once('/')
            .filter(|(scope, _)| scope.starts_with('@'))
            .and_then(|(scope, _)| values.get(&format!("{scope}:registry")));
        let url = scoped
            .or_else(|| values.get("registry"))
            .map_or(DEFAULT, String::as_str);
        let auth = auth(&values, url);
        if url == DEFAULT && auth.is_none() {
            return vec![];
        }
        vec![Target::new(format!("{}/{{enc}}", base(url)), auth)]
    }
}

/// The header for `url` from the longest `//host/path/:` key prefix that starts it.
fn auth(values: &HashMap<String, String>, url: &str) -> Option<String> {
    let bare = url.split_once("//").map_or(url, |(_, rest)| rest);
    let bare = format!("//{}", bare.trim_end_matches('/'));
    let get = |prefix: &str, key: &str| {
        values
            .get(&format!("{prefix}:{key}"))
            .filter(|v| !v.is_empty())
    };
    let mut prefixes: Vec<&str> = values
        .keys()
        .filter_map(|k| k.rsplit_once(':').map(|(prefix, _)| prefix))
        .filter(|p| {
            let p = p.trim_end_matches('/');
            p.starts_with("//")
                && bare
                    .strip_prefix(p)
                    .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
        })
        .collect();
    prefixes.sort_by_key(|p| std::cmp::Reverse(p.len()));
    prefixes.into_iter().find_map(|prefix| {
        if let Some(token) = get(prefix, "_authToken") {
            Some(format!("Bearer {token}"))
        } else if let Some(pair) = get(prefix, "_auth") {
            Some(format!("Basic {pair}"))
        } else {
            let password = unbase64(get(prefix, "_password")?)?;
            Some(basic(get(prefix, "username")?, &password))
        }
    })
}

/// `key=value` lines, skipping `#` and `;` comments and `[section]` headers, with quotes removed.
pub(super) fn ini(text: &str) -> Vec<(String, String)> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with(['#', ';', '[']))
        .filter_map(|l| {
            let (key, value) = l.split_once('=')?;
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .unwrap_or(value);
            Some((key.trim().to_owned(), value.to_owned()))
        })
        .collect()
}
