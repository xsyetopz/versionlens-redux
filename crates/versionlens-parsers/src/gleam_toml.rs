use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Hex;

pub(crate) fn parse_gleam_toml(text: &str) -> Vec<Dependency> {
    let mut dependencies = vec![];
    let mut group: Option<&str> = None;
    let mut block: Option<GleamDependencyBlock> = None;
    let mut project_name: Option<String> = None;
    let mut project_version: Option<GleamProjectVersion> = None;
    let mut in_root_table = true;
    let mut offset = 0;

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(mut pending) = block.take() {
            pending.text.push('\n');
            pending.text.push_str(line);
            if trimmed.starts_with('}') {
                if let Some(dependency) =
                    parse_gleam_dependency_block(text, &pending, offset + line.len())
                {
                    dependencies.push(dependency);
                }
            } else {
                block = Some(pending);
            }
            offset += line.len() + 1;
            continue;
        }

        if trimmed.starts_with('[') {
            in_root_table = false;
        }

        group = match trimmed {
            "[dependencies]" => Some("dependencies"),
            "[dev_dependencies]" => Some("dev_dependencies"),
            "[dev-dependencies]" => Some("dev-dependencies"),
            _ => group,
        };
        if trimmed.starts_with('[')
            && !matches!(
                trimmed,
                "[dependencies]" | "[dev_dependencies]" | "[dev-dependencies]"
            )
        {
            group = None;
        }

        if in_root_table {
            if let Some(name) = parse_gleam_root_string(line, "name") {
                project_name = Some(name.to_owned());
            } else if let Some((version, requirement_offset)) =
                parse_gleam_root_string_with_offset(line, offset, "version")
            {
                project_version = Some(GleamProjectVersion {
                    requirement: version.to_owned(),
                    line_start: offset,
                    line_end: offset + line.len(),
                    requirement_start: requirement_offset,
                    requirement_end: requirement_offset + version.len(),
                });
            }
        } else if let Some(group) = group {
            if let Some((key_start, name)) = gleam_dependency_block_start(line) {
                block = Some(GleamDependencyBlock {
                    name: name.to_owned(),
                    group: group.to_owned(),
                    start_offset: offset,
                    key_start,
                    text: line.to_owned(),
                });
            } else if let Some(dependency) = parse_gleam_dependency_line(text, line, offset, group)
            {
                dependencies.push(dependency);
            }
        }

        offset += line.len() + 1;
    }

    if let (Some(name), Some(version)) = (project_name, project_version) {
        dependencies.insert(
            0,
            Dependency {
                name,
                requirement: version.requirement,
                ecosystem: Hex,
                group: "version".to_owned(),
                hosted_url: None,
                hosted_name: None,
                range: offset_range(text, version.line_start, version.line_end),
                requirement_range: offset_range(
                    text,
                    version.requirement_start,
                    version.requirement_end,
                ),
                requirement_prefix: "".to_owned(),
                requirement_suffix: "".to_owned(),
            },
        );
    }

    dependencies
}

struct GleamProjectVersion {
    requirement: String,
    line_start: usize,
    line_end: usize,
    requirement_start: usize,
    requirement_end: usize,
}

struct GleamDependencyBlock {
    name: String,
    group: String,
    start_offset: usize,
    key_start: usize,
    text: String,
}

fn parse_gleam_dependency_block(
    text: &str,
    block: &GleamDependencyBlock,
    end_offset: usize,
) -> Option<Dependency> {
    let (requirement, hosted_url) = inline_field(&block.text, "path")
        .map(|path| (path, Some("path")))
        .or_else(|| inline_field(&block.text, "git").map(|git| (git, Some("git"))))?;
    let requirement_offset = block
        .text
        .find(requirement)
        .map(|index| block.start_offset + index)?;

    Some(Dependency {
        name: block.name.as_str().to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Hex,
        group: block.group.as_str().to_owned(),
        hosted_url: hosted_url.map(|value| value.to_owned()),
        hosted_name: None,
        range: offset_range(text, block.start_offset + block.key_start, end_offset),
        requirement_range: offset_range(
            text,
            requirement_offset,
            requirement_offset + requirement.len(),
        ),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn parse_gleam_dependency_line(
    text: &str,
    line: &str,
    line_offset: usize,
    group: &str,
) -> Option<Dependency> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('[') {
        return None;
    }
    let key_start = line.len() - trimmed.len();
    let equals = line.find('=')?;
    let name = line.get(key_start..equals)?.trim();
    if name.is_empty() {
        return None;
    }

    let value = line.get(equals + 1..)?.trim();
    let (requirement, hosted_url) = if value.starts_with('{') {
        inline_field(value, "path")
            .map(|path| (path, Some("path")))
            .or_else(|| inline_field(value, "git").map(|git| (git, Some("git"))))?
    } else {
        (quoted_value(value)?, None)
    };
    let requirement_offset = line.find(requirement).map(|index| line_offset + index)?;

    Some(Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Hex,
        group: group.to_owned(),
        hosted_url: hosted_url.map(|value| value.to_owned()),
        hosted_name: None,
        range: offset_range(text, line_offset + key_start, line_offset + line.len()),
        requirement_range: offset_range(
            text,
            requirement_offset,
            requirement_offset + requirement.len(),
        ),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn gleam_dependency_block_start(line: &str) -> Option<(usize, &str)> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('[') {
        return None;
    }
    let key_start = line.len() - trimmed.len();
    let equals = line.find('=')?;
    let name = line.get(key_start..equals)?.trim();
    if name.is_empty() {
        return None;
    }
    let value = line.get(equals + 1..)?.trim();
    if value.starts_with('{') && !value.contains('}') {
        Some((key_start, name))
    } else {
        None
    }
}

fn quoted_value(value: &str) -> Option<&str> {
    let value = value.strip_prefix('"')?;
    let end = value.find('"')?;
    value.get(..end)
}

fn parse_gleam_root_string<'a>(line: &'a str, field: &str) -> Option<&'a str> {
    parse_gleam_root_string_with_offset(line, 0, field).map(|(value, _)| value)
}

fn parse_gleam_root_string_with_offset<'a>(
    line: &'a str,
    line_offset: usize,
    field: &str,
) -> Option<(&'a str, usize)> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('[') {
        return None;
    }
    let key_start = line.len() - trimmed.len();
    let equals = line.find('=')?;
    let key = line.get(key_start..equals)?.trim();
    if key != field {
        return None;
    }
    let value = line.get(equals + 1..)?.trim_start();
    let value_start = line.len() - value.len();
    let quoted = quoted_value(value)?;
    Some((quoted, line_offset + value_start + 1))
}

fn inline_field<'a>(value: &'a str, field: &str) -> Option<&'a str> {
    let marker = format!("{field} =");
    let start = value.find(&marker)? + marker.len();
    let tail = value.get(start..)?.trim_start();
    quoted_value(tail)
}

#[cfg(test)]
mod tests;
