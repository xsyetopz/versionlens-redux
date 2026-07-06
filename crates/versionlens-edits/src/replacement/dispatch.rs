use versionlens_parsers::Dependency;
use versionlens_versions::requirement_has_empty_comparator_intersection;

use super::python::python_replacement;
use super::ruby::{ruby_prefixed_replacement, ruby_replacement};
use super::semver::{preserve_semver_range_prefix, semver_selector_latest};
use versionlens_parsers::Ecosystem::{
    AnsibleGalaxy, Composer, Cran, Deno, Helm, Npm, Opam, Python, Ruby,
};

pub(crate) fn replacement_text(dependency: &Dependency, latest: &str) -> String {
    if let Some(replacement) = registry_alias_replacement(&dependency.requirement, latest) {
        return replacement;
    }

    if requirement_has_empty_comparator_intersection(&dependency.requirement) {
        return latest.to_owned();
    }

    if !dependency.requirement_prefix.is_empty() || !dependency.requirement_suffix.is_empty() {
        if matches!(dependency.ecosystem, Helm | AnsibleGalaxy) {
            return latest.to_owned();
        }
        if dependency.ecosystem == Ruby {
            return ruby_prefixed_replacement(
                &dependency.requirement_prefix,
                &dependency.requirement_suffix,
                latest,
            );
        }

        let latest = prefixed_latest(dependency, latest);
        return format!(
            "{}{}{}",
            dependency.requirement_prefix, latest, dependency.requirement_suffix
        );
    }

    match dependency.ecosystem {
        Python => python_replacement(&dependency.requirement, latest),
        Cran | Opam | Ruby => ruby_replacement(&dependency.requirement, latest),
        _ => preserve_semver_range_prefix(&dependency.requirement, latest),
    }
}

fn prefixed_latest(dependency: &Dependency, latest: &str) -> String {
    if (dependency.ecosystem == Npm && dependency.requirement_prefix.starts_with("npm:"))
        || (dependency.ecosystem == Deno
            && (dependency.requirement_prefix.starts_with("jsr:")
                || dependency.requirement_prefix.starts_with("npm:")))
        || dependency.ecosystem == Composer
    {
        return preserve_semver_range_prefix(&dependency.requirement, latest);
    }

    semver_selector_latest(&dependency.requirement_prefix, latest).to_owned()
}

fn registry_alias_replacement(requirement: &str, latest: &str) -> Option<String> {
    let (registry, spec) = requirement
        .strip_prefix("jsr:")
        .map(|spec| ("jsr", spec))
        .or_else(|| requirement.strip_prefix("npm:").map(|spec| ("npm", spec)))?;
    let Some(split) = spec.rfind('@').filter(|index| *index > 0) else {
        return Some(format!("{registry}:{spec}@{latest}"));
    };
    let name = &spec[..split];
    let current = &spec[split + 1..];
    let replacement = preserve_semver_range_prefix(current, latest);
    Some(format!("{registry}:{name}@{replacement}"))
}
