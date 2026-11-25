//! The `scalaVersion` an sbt build file takes from elsewhere in its build. sbt merges every `.sbt`
//! file of a project's folder, and a subproject's file sees the `ThisBuild / scalaVersion` the
//! root `build.sbt` sets: www.scala-sbt.org/1.x/docs/Basic-Def.html and Multi-Project.html. A
//! build that sets none uses "the version sbt was built against"
//! (www.scala-sbt.org/1.x/docs/Howto-Scala.html), which `project/build.properties`'
//! `sbt.version` picks: Maven Central publishes sbt 1.x as `org.scala-sbt:main_2.12` and sbt 2.x
//! as `main_3`. A build without that file runs the launcher's own sbt version, which
//! `sbt --script-version` prints: a folder holding only a `build.sbt` made the installed 2.1.0
//! launcher fetch `org.scala-sbt:sbt:2.1.0`, the version `--script-version` printed. The catalog's
//! `within` patterns see one file, so no row can read these.

use std::path::Path;

use super::Run;
use crate::extract::regex;

/// `scalaVersion := "3.7.4"`, with `ThisBuild /` before it, as the catalog's sbt row reads it.
const SCALA: &str = r#"scalaVersion[ \t]*:=[ \t]*"(\d[\w.+-]*)""#;

/// `sbt.version=1.10.7` in `project/build.properties`.
const SBT: &str = r"(?m)^[ \t]*sbt\.version[ \t]*=[ \t]*(\d+)\.";

fn group<'a>(pattern: &'static str, text: &'a str) -> Option<&'a str> {
    let caps = regex(pattern).captures_iter(text).next()?;
    caps.get_group(1).map(|m| &text[m.range()])
}

/// A line setting the Scala version the build at `path` gives a file whose own `text` sets none:
/// from another `.sbt` file beside it or in a folder above, up to the build's root, else the
/// series the root's sbt version, or the installed launcher's that `run` asks, was built against.
/// Only the binary series matters to `%%`, so the default line names it with an `x` patch.
pub(crate) fn inherited(path: &Path, text: &str, run: Run) -> Option<String> {
    if group(SCALA, text).is_some() {
        return None;
    }
    for dir in path.ancestors().skip(1) {
        let mut files: Vec<_> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p != path && p.is_file() && p.extension().is_some_and(|e| e == "sbt"))
            .collect();
        files.sort();
        let found = files.iter().find_map(|f| {
            let text = std::fs::read_to_string(f).ok()?;
            group(SCALA, &text).map(str::to_owned)
        });
        if let Some(version) = found {
            return Some(line(&version));
        }
        let properties = dir.join("project").join("build.properties");
        if let Ok(text) = std::fs::read_to_string(&properties) {
            return default(group(SBT, &text)?);
        }
    }
    let launcher = run(path.parent()?, "sbt", &["--script-version"])?;
    default(launcher.split('.').next()?)
}

/// The line naming the Scala series sbt's `major` version was built against.
fn default(major: &str) -> Option<String> {
    match major {
        "1" => Some(line("2.12.x")),
        "2" => Some(line("3.x")),
        _ => None,
    }
}

fn line(version: &str) -> String {
    format!("scalaVersion := \"{version}\"")
}
