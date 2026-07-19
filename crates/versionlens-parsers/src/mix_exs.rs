use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Hex;

pub(crate) fn parse_mix_exs(text: &str) -> Vec<Dependency> {
    let mut dependencies = vec![];
    let mut block: Option<MixDependencyBlock> = None;
    let mut offset = 0;

    for line in text.lines() {
        if let Some(mut pending) = block.take() {
            pending.text.push('\n');
            pending.text.push_str(line);
            if line.contains('}') {
                if let Some(dependency) =
                    parse_mix_dependency_source(text, &pending.text, pending.start_offset)
                {
                    dependencies.push(dependency);
                }
            } else {
                block = Some(pending);
            }
            offset += line.len() + 1;
            continue;
        }

        if line.contains("{:") && !line[line.find("{:").unwrap_or(0)..].contains('}') {
            block = Some(MixDependencyBlock {
                start_offset: offset,
                text: line.to_owned(),
            });
        } else {
            for (tuple_start, tuple_end) in mix_dependency_tuple_spans(line) {
                if let Some(dependency) = parse_mix_dependency_source(
                    text,
                    &line[tuple_start..tuple_end],
                    offset + tuple_start,
                ) {
                    dependencies.push(dependency);
                }
            }
        }
        offset += line.len() + 1;
    }

    dependencies
}

struct MixDependencyBlock {
    start_offset: usize,
    text: String,
}

fn mix_dependency_tuple_spans(line: &str) -> Vec<(usize, usize)> {
    let mut spans = vec![];
    let mut depth = 0usize;
    let mut tuple_start = None;

    for (index, ch) in line.char_indices() {
        match ch {
            '{' => {
                if depth == 0 {
                    tuple_start = Some(index);
                }
                depth += 1;
            }
            '}' if depth > 0 => {
                depth -= 1;
                if depth == 0
                    && let Some(start) = tuple_start.take()
                    && line.get(start..start + 2) == Some("{:")
                {
                    spans.push((start, index + ch.len_utf8()));
                }
            }
            _ => {}
        }
    }

    spans
}

fn parse_mix_dependency_source(
    text: &str,
    source: &str,
    source_offset: usize,
) -> Option<Dependency> {
    let tuple_start = source.find("{:")?;
    let tuple_end = source[tuple_start..]
        .find('}')
        .map(|index| tuple_start + index + 1)
        .unwrap_or(source.len());
    let tuple = &source[tuple_start..tuple_end];
    let name_start = tuple_start + 2;
    let name_end = source[name_start..]
        .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .map(|index| name_start + index)?;
    let name = source.get(name_start..name_end)?;

    let (package_name, hosted_name) = mix_package_name(tuple).unwrap_or((name, None));
    let source_requirement = quoted_requirement(tuple)
        .or_else(|| option_requirement(tuple, "git"))
        .or_else(|| option_requirement(tuple, "path"))
        .or_else(|| mix_umbrella_source_requirement(tuple));
    let requirement = quoted_requirement(tuple)
        .or_else(|| option_requirement(tuple, "git"))
        .or_else(|| option_requirement(tuple, "path"))
        .or_else(|| mix_umbrella_requirement(tuple));
    let hosted_url = option_requirement(tuple, "git")
        .map(|_| "git")
        .or_else(|| option_requirement(tuple, "path").map(|_| "path"))
        .or_else(|| mix_umbrella_requirement(tuple).map(|_| "umbrella"));
    let requirement = requirement?;
    let source_requirement = source_requirement?;
    let requirement_offset = source[tuple_start..tuple_end]
        .find(source_requirement)
        .map(|index| source_offset + tuple_start + index)?;

    Some(Dependency {
        name: package_name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Hex,
        group: mix_dependency_group(tuple),
        hosted_url: hosted_url.map(|value| value.to_owned()),
        hosted_name: hosted_name.map(|value| value.to_owned()),
        range: offset_range(text, source_offset + tuple_start, source_offset + tuple_end),
        requirement_range: offset_range(
            text,
            requirement_offset,
            requirement_offset + source_requirement.len(),
        ),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn quoted_requirement(tuple: &str) -> Option<&str> {
    let first_quote = tuple.find('"')?;
    let after_first = tuple.get(first_quote + 1..)?;
    let end = after_first.find('"')?;
    after_first.get(..end)
}

fn option_requirement<'a>(tuple: &'a str, option: &str) -> Option<&'a str> {
    let key = format!("{option}: ");
    let start = tuple.find(&key)? + key.len();
    let value = tuple.get(start..)?.trim_start();
    let value = value.strip_prefix('"')?;
    let end = value.find('"')?;
    value.get(..end)
}

fn mix_umbrella_requirement(tuple: &str) -> Option<&str> {
    let value = option_after_key(tuple, "in_umbrella")?;
    if value == "true" {
        Some("in_umbrella:true")
    } else {
        None
    }
}

fn mix_umbrella_source_requirement(tuple: &str) -> Option<&str> {
    let value = option_after_key(tuple, "in_umbrella")?;
    if value == "true" { Some("true") } else { None }
}

fn mix_dependency_group(tuple: &str) -> String {
    let Some(only_value) = option_after_key(tuple, "only") else {
        return "deps".to_owned();
    };

    let environments = only_value
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .filter_map(|part| part.trim().strip_prefix(':'))
        .map(|value| value.trim())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if environments.is_empty() {
        "deps".to_owned()
    } else {
        format!("deps.{}", environments.join(","))
    }
}

fn mix_package_name(tuple: &str) -> Option<(&str, Option<&str>)> {
    let app_name = tuple.strip_prefix("{:")?;
    let app_end = app_name.find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))?;
    let app_name = app_name.get(..app_end)?;
    let package_name = option_atom_or_string(tuple, "hex")?;
    Some((package_name, Some(app_name)))
}

fn option_atom_or_string<'a>(tuple: &'a str, option: &str) -> Option<&'a str> {
    let value = option_after_key(tuple, option)?;
    if let Some(atom) = value.strip_prefix(':') {
        let end = atom
            .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
            .unwrap_or(atom.len());
        return atom.get(..end);
    }

    let value = value.strip_prefix('"')?;
    let end = value.find('"')?;
    value.get(..end)
}

fn option_after_key<'a>(tuple: &'a str, option: &str) -> Option<&'a str> {
    let key = format!("{option}: ");
    let start = tuple.find(&key)? + key.len();
    let tail = tuple.get(start..)?;
    Some(tail.split('}').next()?.trim().trim_end_matches(','))
}

#[cfg(test)]
mod tests;
