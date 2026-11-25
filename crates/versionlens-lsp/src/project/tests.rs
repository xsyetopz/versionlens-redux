use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::*;
use crate::catalog::file_type_of;
use crate::extract::extract;

/// A fresh folder holding `files`, with `home/` as the user's home.
fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("versionlens-project-{name}-{}", std::process::id()));
    drop(std::fs::remove_dir_all(&root));
    for (path, text) in files {
        let path = root.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    root
}

fn env(root: &Path, vars: &[(&str, &str)]) -> Env {
    Env {
        vars: vars
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect(),
        home: Some(root.join("home")),
        run: |_, _, _| None,
        nuget: Vec::new(),
    }
}

/// A dependency's name with its targets' URLs and headers.
type Resolved = (String, Vec<(String, Option<String>)>);

/// Each dependency of the manifest at `path` under `root`, with its targets' URLs and headers.
fn targets(root: &Path, path: &str, env: &Env) -> Vec<Resolved> {
    let path = root.join(path);
    let text = std::fs::read_to_string(&path).unwrap();
    let file = file_type_of(path.to_str().unwrap(), &text).unwrap();
    let mut deps = extract(file, &text);
    resolve(&path, &text, &mut deps, env);
    deps.into_iter()
        .filter(|d| d.registry != crate::extract::PROJECT)
        .map(|d| {
            (
                d.name,
                d.targets.into_iter().map(|t| (t.url, t.auth)).collect(),
            )
        })
        .collect()
}

fn target(url: &str, auth: Option<&str>) -> (String, Option<String>) {
    (url.to_owned(), auth.map(str::to_owned))
}

mod builds;
mod gitlab;
mod live;
mod registries;
