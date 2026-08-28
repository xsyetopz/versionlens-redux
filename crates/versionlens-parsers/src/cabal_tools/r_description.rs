use serde_json::Value;
use serde_json::from_str;

use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Cran;
type RDependencies = Vec<Dependency>;

use crate::positions::offset_range;

const DESCRIPTION_DEPENDENCY_FIELDS: &[&str] =
    &["Depends", "Imports", "Suggests", "Enhances", "LinkingTo"];

#[derive(Debug, Clone)]
struct Field {
    name: String,
    value: String,
    start: usize,
    end: usize,
    value_start: usize,
}

pub(crate) fn parse_r_description(text: &str) -> RDependencies {
    let fields = description_fields(text);
    let mut dependencies = vec![];
    let package = field_value(&fields, "Package");
    let version = fields.iter().find(|field| field.name == "Version");
    if let (Some(package), Some(version)) = (package, version) {
        dependencies.push(Dependency {
            name: package.to_owned(),
            requirement: version.value.trim().to_owned(),
            ecosystem: Cran,
            group: "Version".to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: offset_range(text, version.start, version.end),
            requirement_range: offset_range(
                text,
                version.value_start + leading_ws_len(&version.value),
                version.end - trailing_ws_len(&version.value),
            ),
            requirement_prefix: "".to_owned(),
            requirement_suffix: "".to_owned(),
        });
    }

    for field in fields
        .iter()
        .filter(|field| DESCRIPTION_DEPENDENCY_FIELDS.contains(&field.name.as_str()))
    {
        dependencies.extend(description_dependency_entries(text, field));
    }

    dependencies
}

pub(crate) fn parse_renv_lock(text: &str) -> RDependencies {
    let Ok(value) = from_str::<Value>(text) else {
        return vec![];
    };
    let Some(packages) = value.get("Packages").and_then(|value| value.as_object()) else {
        return vec![];
    };

    packages
        .iter()
        .filter_map(|(key, package)| renv_package_dependency(text, key, package))
        .collect()
}

fn description_fields(text: &str) -> Vec<Field> {
    let mut fields = vec![];
    let mut current: Option<Field> = None;
    for (line_start, line) in line_offsets(text) {
        if line.trim().is_empty() {
            continue;
        }
        if line.starts_with(' ') || line.starts_with('\t') {
            if let Some(field) = current.as_mut() {
                field.value.push(' ');
                field.value.push_str(line.trim());
                field.end = line_start + line.len();
            }
            continue;
        }
        if let Some(field) = current.take() {
            fields.push(field);
        }
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let value_offset = line_start + name.len() + 1;
        current = Some(Field {
            name: name.trim().to_owned(),
            value: value.trim().to_owned(),
            start: line_start,
            end: line_start + line.len(),
            value_start: value_offset + leading_ws_len(value),
        });
    }
    if let Some(field) = current {
        fields.push(field);
    }
    fields
}

fn field_value<'a>(fields: &'a [Field], name: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| field.value.trim())
}

fn description_dependency_entries(text: &str, field: &Field) -> RDependencies {
    field
        .value
        .split(',')
        .map(|value| value.trim())
        .filter(|entry| !entry.is_empty())
        .filter_map(|entry| description_dependency_entry(text, field, entry))
        .collect()
}

fn description_dependency_entry(text: &str, field: &Field, entry: &str) -> Option<Dependency> {
    let (name, requirement) = if let Some((name, rest)) = entry.split_once('(') {
        let requirement = rest.trim().strip_suffix(')').unwrap_or(rest).trim();
        (name.trim(), requirement)
    } else {
        (entry.trim(), "latest")
    };
    if name.is_empty() {
        return None;
    }
    let entry_start = text[field.start..field.end]
        .find(entry)
        .map(|index| field.start + index)
        .unwrap_or(field.value_start);
    let requirement_start = if requirement == "latest" {
        entry_start + entry.len()
    } else {
        text[entry_start..field.end]
            .find(requirement)
            .map(|index| entry_start + index)
            .unwrap_or(entry_start)
    };

    Some(Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Cran,
        group: field.name.as_str().to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, entry_start, entry_start + entry.len()),
        requirement_range: offset_range(
            text,
            requirement_start,
            requirement_start + requirement.len(),
        ),
        requirement_prefix: if requirement == "latest" {
            " (>= ".to_owned()
        } else {
            "".to_owned()
        },
        requirement_suffix: if requirement == "latest" {
            ")".to_owned()
        } else {
            "".to_owned()
        },
    })
}

fn renv_package_dependency(text: &str, key: &str, package: &Value) -> Option<Dependency> {
    let name = package
        .get("Package")
        .and_then(|value| value.as_str())
        .unwrap_or(key)
        .to_owned();
    let version = package
        .get("Version")
        .and_then(|value| value.as_str())?
        .to_owned();
    let source = package
        .get("Source")
        .and_then(|value| value.as_str())
        .unwrap_or("Repository");
    let hosted_url = match source {
        "Repository" | "CRAN" | "Bioconductor" | "RSPM" => None,
        "GitHub" | "Git" => Some("git".to_owned()),
        "Local" => Some("local".to_owned()),
        "URL" => Some("url".to_owned()),
        other => Some(other.to_ascii_lowercase()),
    };
    let (range_start, range_end, requirement_start, requirement_end) =
        renv_version_offsets(text, &name, &version);

    Some(Dependency {
        name,
        requirement: version,
        ecosystem: Cran,
        group: "Packages".to_owned(),
        hosted_url,
        hosted_name: None,
        range: offset_range(text, range_start, range_end),
        requirement_range: offset_range(text, requirement_start, requirement_end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn renv_version_offsets(text: &str, name: &str, version: &str) -> (usize, usize, usize, usize) {
    let package_start = text.find(&format!("\"{name}\"")).unwrap_or(0);
    let search = &text[package_start..];
    let version_key = search
        .find("\"Version\"")
        .map(|index| package_start + index);
    let version_start = version_key
        .and_then(|start| {
            text[start..]
                .find(&format!("\"{version}\""))
                .map(|index| start + index + 1)
        })
        .unwrap_or(package_start);
    let line_start = text[..version_start]
        .rfind('\n')
        .map_or(0, |index| index + 1);
    let line_end = text[version_start..]
        .find('\n')
        .map_or(text.len(), |index| version_start + index);
    (
        line_start,
        line_end,
        version_start,
        version_start + version.len(),
    )
}

fn line_offsets(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.split_inclusive('\n').scan(0, |offset, line| {
        let start = *offset;
        *offset += line.len();
        Some((start, line.trim_end_matches('\n').trim_end_matches('\r')))
    })
}

fn leading_ws_len(value: &str) -> usize {
    value.len() - value.trim_start().len()
}

fn trailing_ws_len(value: &str) -> usize {
    value.len() - value.trim_end().len()
}
