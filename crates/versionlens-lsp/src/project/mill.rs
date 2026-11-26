//! The `scalaVersion` a Mill module takes from another file of its build. A `package.mill` often
//! extends a trait that the root `build.mill` defines, and the trait sets the Scala version:
//! com-lihaoyi/mill, example/large/multifile/10-multi-file-builds, where `foo/package.mill`
//! declares `mvn"com.lihaoyi::mainargs:0.7.8"` in `object package extends build.MyModule` and
//! `build.mill` holds `trait MyModule { def scalaVersion = "3.7.4" }`. The catalog's `within`
//! patterns see one file, so no row can read a setting from a sibling; this reader finds it.

use std::path::Path;

use crate::extract::regex;

/// `def scalaVersion = "3.7.4"` in a `.mill` or `.sc` file, `scalaVersion: 3.7.4` in a
/// `.mill.yaml` one, the forms the catalog's Mill row reads.
const SCALA: &str = r#"scalaVersion(?:[ \t]*=[ \t]*"|:[ \t]*["']?)(\d+\.\d+\.\d+[\w.+-]*)"#;

/// `def scalaVersion = myScalaVersion`, naming a definition that holds the version, as
/// example/large/multifile/11-helper-files's `util.mill` does.
const NAMED: &str = r"scalaVersion[ \t]*=[ \t]*([A-Za-z_]\w*)[ \t]*(?:\r?\n|;|\})";

fn group<'a>(pattern: &'static str, text: &'a str) -> Option<&'a str> {
    let caps = regex(pattern).captures_iter(text).next()?;
    caps.get_group(1).map(|m| &text[m.range()])
}

fn scala(text: &str) -> Option<&str> {
    group(SCALA, text)
}

/// The version a file's `scalaVersion` names, written as a literal or through a definition in
/// the same file.
fn named(text: &str) -> Option<String> {
    let name = group(NAMED, text)?;
    let pattern =
        format!(r#"(?:def|val)[ \t]+{name}[ \t]*(?::[ \t]*String[ \t]*)?=[ \t]*"(\d[\w.+-]*)""#);
    let found = regex_automata::meta::Regex::new(&pattern).ok()?;
    let caps = found.captures_iter(text).next()?;
    caps.get_group(1).map(|m| text[m.range()].to_owned())
}

/// Files that mark the root of a Mill build: mill-build.org, fundamentals/modules.adoc and the
/// Mill 1.0 `build.mill.yaml` example, with `build.sc` before Mill 0.12.
const ROOTS: [&str; 3] = ["build.mill", "build.mill.yaml", "build.sc"];

/// A line setting the Scala version the build at `path` gives a module whose own `text` sets
/// none, from the nearest other build file: those beside it first, then each folder up to the
/// build's root. Helper files such as `util.mill` count, as Mill compiles every `.mill` file of a
/// folder into the build (example/large/multifile/11-helper-files), and a `scalaVersion` that
/// names a definition takes the version that definition holds.
pub(crate) fn inherited(path: &Path, text: &str) -> Option<String> {
    if scala(text).is_some() {
        return None;
    }
    if let Some(version) = named(text) {
        return Some(format!("def scalaVersion = \"{version}\""));
    }
    // A file outside any build has no siblings to inherit from.
    let root = path
        .ancestors()
        .skip(1)
        .find(|dir| ROOTS.iter().any(|r| dir.join(r).is_file()))?;
    for dir in path.ancestors().skip(1) {
        let mut files: Vec<_> = std::fs::read_dir(dir)
            .ok()?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p != path && is_build_file(p))
            .collect();
        files.sort();
        let version = files.iter().find_map(|f| {
            let text = std::fs::read_to_string(f).ok()?;
            scala(&text).map(str::to_owned).or_else(|| named(&text))
        });
        if let Some(version) = version {
            return Some(format!("def scalaVersion = \"{version}\""));
        }
        if dir == root {
            return None;
        }
    }
    None
}

fn is_build_file(path: &Path) -> bool {
    // Mill names its build files exactly, in lower case: `build.mill`, `package.mill`,
    // `build.mill.yaml`, and the older `build.sc`.
    let mill = |p: &Path| p.extension().is_some_and(|e| e == "mill");
    let yaml = path.extension().is_some_and(|e| e == "yaml")
        && path.file_stem().is_some_and(|s| mill(Path::new(s)));
    path.is_file() && (mill(path) || yaml || path.file_name().is_some_and(|n| n == "build.sc"))
}
