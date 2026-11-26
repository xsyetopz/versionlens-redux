//! Objects whose fields name a dependency, such as a POM's `<dependency>`.

use std::ops::Range;

use super::{Dep, path_matches};
use crate::catalog::FileType;
use crate::flatten::{Format, Leaf};

/// The leaf a POM's `${…}` reference names, where `value` is one: a dotted path into the model,
/// such as `project.parent.version`, or a property under `<properties>`. A project that omits its
/// `groupId` or `version` inherits the parent's, which `project.groupId` and `project.version`
/// then name: maven.apache.org/pom.html#Properties and #Inheritance.
fn pom_reference<'a>(value: &str, leaves: &'a [Leaf]) -> Option<&'a Leaf> {
    let var = value.strip_prefix("${")?.strip_suffix('}')?;
    let at = |path: &[&str]| {
        leaves
            .iter()
            .find(|l| l.path.iter().eq(path.iter().copied()))
    };
    let path: Vec<&str> = var.split('.').collect();
    at(&path)
        .or_else(|| match path[..] {
            ["project", field @ ("groupId" | "version")] => at(&["project", "parent", field]),
            _ => None,
        })
        .or_else(|| at(&["project", "properties", var]))
}

pub(super) fn records<'a>(
    path: &str,
    fields: (&str, &str, &str),
    leaves: &'a [Leaf],
    file: &FileType,
    deps: &mut Vec<Dep>,
    moves: &mut Vec<(Range<usize>, Range<usize>)>,
) {
    let (names, version, source) = fields;
    let depth = path.split('.').count();
    let mut groups: Vec<(usize, Vec<&Leaf>)> = Vec::new();
    for leaf in leaves
        .iter()
        .filter(|l| l.path.len() == depth + 1 && path_matches(path, &l.path[..depth]))
    {
        match groups.iter_mut().find(|(id, _)| *id == leaf.parent) {
            Some((_, group)) => group.push(leaf),
            None => groups.push((leaf.parent, vec![leaf])),
        }
    }
    for (_, group) in groups {
        let field = |spec: &str| {
            spec.split('|').find_map(|key| {
                group
                    .iter()
                    .find(|l| l.path.last().is_some_and(|k| k == key))
                    .copied()
            })
        };
        let resolve = |leaf: &'a Leaf| match file.format {
            Format::Xml => pom_reference(&leaf.value, leaves).unwrap_or(leaf),
            _ => leaf,
        };
        let parts: Option<Vec<&str>> = names
            .split(':')
            .map(|n| field(n).map(|l| resolve(l).value.as_str()))
            .collect();
        let Some(parts) = parts else { continue };
        // A trailing `?` marks a version the record may omit, as Galaxy then installs the newest
        // (docs.ansible.com/ansible/latest/galaxy/user_guide.html#installing-roles-and-collections-from-the-same-requirements-yml-file).
        let Some(own) = field(version.trim_end_matches('?')) else {
            if let (true, Some(at)) = (
                version.ends_with('?'),
                field(names.split(':').next().unwrap_or(names)),
            ) {
                deps.push(Dep {
                    name: parts.join(":"),
                    span: at.span.start..at.span.start,
                    registry: file.registry,
                    source: field(source).map(|l| l.value.clone()),
                    ..Dep::default()
                });
            }
            continue;
        };
        let leaf = resolve(own);
        if leaf.span != own.span {
            moves.push((own.span.clone(), leaf.span.clone()));
        }
        deps.push(Dep {
            name: parts.join(":"),
            req: leaf.value.clone(),
            span: own.span.clone(),
            registry: file.registry,
            source: field(source).map(|l| l.value.clone()),
            ..Dep::default()
        });
    }
}
