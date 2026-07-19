use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Conan;

type ConanDependencies = Vec<Dependency>;

struct ConanRequirementInput<'a> {
    raw_version: &'a str,
    version_start: usize,
    version_end: usize,
    reference: &'a str,
    rest: &'a str,
}

const CONAN_TXT_DEPENDENCY_SECTIONS: &[&str] = &["requires", "tool_requires", "test_requires"];
const CONAN_PY_DEPENDENCY_ATTRIBUTES: &[&str] = &[
    "requires",
    "tool_requires",
    "build_requires",
    "test_requires",
    "python_requires",
];

pub(crate) fn parse_conanfile_txt(text: &str) -> ConanDependencies {
    let mut dependencies = vec![];
    let mut section: Option<&str> = None;
    let mut offset = 0usize;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let name = &trimmed[1..trimmed.len() - 1];
            section = CONAN_TXT_DEPENDENCY_SECTIONS
                .iter()
                .find(|candidate| **candidate == name)
                .copied();
            offset += line.len() + 1;
            continue;
        }

        if let Some(group) = section
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && !trimmed.starts_with(';')
        {
            let leading = line.len() - line.trim_start().len();
            if let Some(dependency) =
                conan_reference_dependency(text, group, trimmed, offset + leading)
            {
                dependencies.push(dependency);
            }
        }

        offset += line.len() + 1;
    }

    dependencies
}

pub(crate) fn parse_conanfile_py(text: &str) -> ConanDependencies {
    let mut dependencies = vec![];
    let mut offset = 0usize;

    for line in text.lines() {
        let trimmed = line.trim_start();
        let leading = line.len() - trimmed.len();
        for group in CONAN_PY_DEPENDENCY_ATTRIBUTES {
            let Some(rest) = trimmed.strip_prefix(group) else {
                continue;
            };
            let rest = rest.trim_start();
            let Some(values) = rest.strip_prefix('=') else {
                continue;
            };
            let values_start = offset + leading + trimmed.len() - values.len();
            dependencies.extend(quoted_conan_references(text, group, values, values_start));
        }
        offset += line.len() + 1;
    }

    dependencies
}

fn quoted_conan_references(
    text: &str,
    group: &str,
    values: &str,
    values_offset: usize,
) -> ConanDependencies {
    let mut dependencies = vec![];
    let mut cursor = 0usize;
    let bytes = values.as_bytes();
    while cursor < bytes.len() {
        if bytes[cursor] != b'\'' && bytes[cursor] != b'"' {
            cursor += 1;
            continue;
        }
        let quote = bytes[cursor];
        let value_start = cursor + 1;
        let mut value_end = value_start;
        while value_end < bytes.len() && bytes[value_end] != quote {
            value_end += 1;
        }
        if value_end >= bytes.len() {
            break;
        }
        let value = &values[value_start..value_end];
        if let Some(dependency) =
            conan_reference_dependency(text, group, value, values_offset + value_start)
        {
            dependencies.push(dependency);
        }
        cursor = value_end + 1;
    }

    dependencies
}

fn conan_reference_dependency(
    text: &str,
    group: &str,
    value: &str,
    value_start: usize,
) -> Option<Dependency> {
    let reference = value.trim();
    let reference_start = value_start + value.find(reference)?;
    let (package, rest) = reference.split_once('/')?;
    if package.is_empty() || rest.is_empty() {
        return None;
    }

    let version_start = reference_start + package.len() + 1;
    let version_end = version_start + conan_version_len(rest);
    if version_end == version_start {
        return None;
    }

    let raw_version = &text[version_start..version_end];
    let (requirement, requirement_start, requirement_end, prefix, suffix) = conan_requirement_parts(
        text,
        ConanRequirementInput {
            raw_version,
            version_start,
            version_end,
            reference,
            rest,
        },
    );

    Some(Dependency {
        name: package.to_owned(),
        requirement,
        ecosystem: Conan,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, reference_start, reference_start + reference.len()),
        requirement_range: offset_range(text, requirement_start, requirement_end),
        requirement_prefix: prefix,
        requirement_suffix: suffix,
    })
}

fn conan_requirement_parts(
    text: &str,
    input: ConanRequirementInput<'_>,
) -> (String, usize, usize, String, String) {
    let mut requirement_start = input.version_start;
    let mut requirement_end = input.version_end;
    let mut prefix = "".to_owned();
    let mut suffix = conan_reference_suffix(
        input.reference,
        input.rest,
        input.version_end - input.version_start,
    );

    if input.raw_version.starts_with('[')
        && input.raw_version.ends_with(']')
        && input.raw_version.len() > 2
    {
        requirement_start += 1;
        requirement_end -= 1;
        prefix.push('[');
        suffix = format!("]{suffix}");
    }

    (
        text[requirement_start..requirement_end].to_owned(),
        requirement_start,
        requirement_end,
        prefix,
        suffix,
    )
}

fn conan_version_len(rest: &str) -> usize {
    rest.find(['@', '#']).unwrap_or(rest.len())
}

fn conan_reference_suffix(reference: &str, rest: &str, version_len: usize) -> String {
    let suffix_in_rest = &rest[version_len..];
    if suffix_in_rest.is_empty() {
        "".to_owned()
    } else {
        reference[reference.len() - suffix_in_rest.len()..].to_owned()
    }
}
