use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Hex;

type ParsedRebarDependency = Option<Dependency>;

pub(crate) fn parse_rebar_config(text: &str) -> Vec<Dependency> {
    let mut dependencies = vec![];
    let mut in_deps = false;
    let mut in_profiles = false;
    let mut current_profile: Option<String> = None;
    let mut dependency_group = "deps".to_owned();
    let mut block: Option<RebarDependencyBlock> = None;
    let mut offset = 0;

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(mut pending) = block.take() {
            pending.text.push('\n');
            pending.text.push_str(line);
            pending.depth += rebar_brace_delta(line);
            if pending.depth <= 0 {
                if let Some(dependency) = parse_rebar_dependency_source(
                    text,
                    &pending.text,
                    pending.start_offset,
                    &pending.group,
                ) {
                    dependencies.push(dependency);
                }
            } else {
                block = Some(pending);
            }
            offset += line.len() + 1;
            continue;
        }

        if trimmed.starts_with("{profiles") {
            dependencies.extend(parse_rebar_compact_profile_dependencies(text, line, offset));
            in_profiles = true;
        } else if in_profiles && let Some(profile) = rebar_profile_name(trimmed) {
            current_profile = Some(profile.to_owned());
        }
        if let Some(section) = rebar_dependency_section_name(trimmed) {
            in_deps = true;
            dependency_group = current_profile
                .as_deref()
                .map(|profile| format!("{section}.{profile}"))
                .unwrap_or_else(|| section.to_owned());
        }
        if in_deps {
            if let Some(tuple_start) = rebar_multiline_tuple_start(line) {
                block = Some(RebarDependencyBlock {
                    start_offset: offset + tuple_start,
                    text: line[tuple_start..].to_owned(),
                    group: dependency_group.as_str().to_owned(),
                    depth: rebar_brace_delta(&line[tuple_start..]),
                });
                offset += line.len() + 1;
                continue;
            }
            for (atom_start, atom_end) in rebar_bare_dependency_spans(line) {
                if let Some(dependency) = parse_rebar_bare_dependency(
                    text,
                    RebarLineSpan {
                        line,
                        line_offset: offset,
                        start: atom_start,
                        end: atom_end,
                    },
                    &dependency_group,
                ) {
                    dependencies.push(dependency);
                }
            }
            for (tuple_start, tuple_end) in rebar_dependency_tuple_spans(line) {
                let Some(source) = line.get(tuple_start..tuple_end) else {
                    continue;
                };
                if let Some(dependency) = parse_rebar_dependency_source(
                    text,
                    source,
                    offset + tuple_start,
                    &dependency_group,
                ) {
                    dependencies.push(dependency);
                }
            }
        }
        if in_deps && (trimmed.starts_with("]}") || trimmed.ends_with("]}.") || trimmed == "]}") {
            in_deps = false;
            dependency_group = "deps".to_owned();
        }
        if in_profiles && trimmed.ends_with("]}.") {
            in_profiles = false;
            current_profile = None;
        }
        offset += line.len() + 1;
    }

    dependencies
}

fn parse_rebar_compact_profile_dependencies(
    text: &str,
    line: &str,
    line_offset: usize,
) -> Vec<Dependency> {
    let mut dependencies = vec![];
    let mut search_offset = 0usize;

    while let Some((section, relative_section_start)) = line
        .get(search_offset..)
        .and_then(rebar_next_dependency_section)
    {
        let section_start = search_offset + relative_section_start;
        let Some(profile_start) = line[..section_start].rfind('{') else {
            search_offset = section_start + section.len() + 1;
            continue;
        };
        let Some(profile) = rebar_profile_name(&line[profile_start..]) else {
            search_offset = section_start + section.len() + 1;
            continue;
        };
        let Some(section_list_start) = line
            .get(section_start..)
            .and_then(|tail| tail.find('['))
            .map(|index| section_start + index)
        else {
            search_offset = section_start + section.len() + 1;
            continue;
        };
        let Some(section_list_end) = rebar_matching_list_end(line, section_list_start) else {
            search_offset = section_start + section.len() + 1;
            continue;
        };

        let group = format!("{section}.{profile}");
        let source = &line[section_start..=section_list_end];
        for (atom_start, atom_end) in rebar_bare_dependency_spans(source) {
            if let Some(dependency) = parse_rebar_bare_dependency(
                text,
                RebarLineSpan {
                    line: source,
                    line_offset: line_offset + section_start,
                    start: atom_start,
                    end: atom_end,
                },
                &group,
            ) {
                dependencies.push(dependency);
            }
        }
        for (tuple_start, tuple_end) in rebar_dependency_tuple_spans(source) {
            let Some(tuple) = source.get(tuple_start..tuple_end) else {
                continue;
            };
            if let Some(dependency) = parse_rebar_dependency_source(
                text,
                tuple,
                line_offset + section_start + tuple_start,
                &group,
            ) {
                dependencies.push(dependency);
            }
        }

        search_offset = section_list_end + 1;
    }

    dependencies
}

fn rebar_next_dependency_section(source: &str) -> Option<(&'static str, usize)> {
    [
        ("project_plugins", "{project_plugins"),
        ("deps", "{deps"),
        ("plugins", "{plugins"),
    ]
    .into_iter()
    .filter_map(|(section, marker)| source.find(marker).map(|index| (section, index)))
    .min_by_key(|(_, index)| *index)
}

fn rebar_matching_list_end(source: &str, list_start: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (relative_index, ch) in source.get(list_start..)?.char_indices() {
        let index = list_start + relative_index;
        match ch {
            '[' => depth += 1,
            ']' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

struct RebarDependencyBlock {
    start_offset: usize,
    text: String,
    group: String,
    depth: isize,
}

struct RebarLineSpan<'a> {
    line: &'a str,
    line_offset: usize,
    start: usize,
    end: usize,
}

fn parse_rebar_bare_dependency(
    text: &str,
    span: RebarLineSpan<'_>,
    group: &str,
) -> ParsedRebarDependency {
    let name = span.line.get(span.start..span.end)?;
    Some(Dependency {
        name: name.to_owned(),
        requirement: "latest".to_owned(),
        ecosystem: Hex,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(
            text,
            span.line_offset + span.start,
            span.line_offset + span.end,
        ),
        requirement_range: offset_range(
            text,
            span.line_offset + span.start,
            span.line_offset + span.end,
        ),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn parse_rebar_dependency_source(
    text: &str,
    source: &str,
    source_offset: usize,
    group: &str,
) -> ParsedRebarDependency {
    let tuple_start = source.find('{')?;
    let tuple_end = source
        .rfind('}')
        .map(|index| index + 1)
        .unwrap_or(source.len());
    let tuple = source.get(tuple_start..tuple_end)?;
    let name = rebar_tuple_atom(tuple)?;

    let (package_name, hosted_name) = rebar_package_name(tuple).unwrap_or((name, None));
    let (requirement, hosted_url) = rebar_requirement(tuple)?;
    let requirement_offset = source[tuple_start..tuple_end]
        .find(requirement)
        .map(|index| source_offset + tuple_start + index)
        .unwrap_or(source_offset + tuple_start + 1);

    Some(Dependency {
        name: package_name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Hex,
        group: group.to_owned(),
        hosted_url: hosted_url.map(|value| value.to_owned()),
        hosted_name: hosted_name.map(|value| value.to_owned()),
        range: offset_range(text, source_offset + tuple_start, source_offset + tuple_end),
        requirement_range: offset_range(
            text,
            requirement_offset,
            requirement_offset + requirement.len(),
        ),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn rebar_multiline_tuple_start(line: &str) -> Option<usize> {
    let start = line.find('{')?;
    if rebar_dependency_section_name(&line[start..]).is_some()
        || rebar_brace_delta(&line[start..]) <= 0
    {
        None
    } else {
        Some(start)
    }
}

fn rebar_brace_delta(source: &str) -> isize {
    source.chars().fold(0, |depth, ch| match ch {
        '{' => depth + 1,
        '}' => depth - 1,
        _ => depth,
    })
}

fn rebar_bare_dependency_spans(line: &str) -> Vec<(usize, usize)> {
    let scan_start = rebar_dependency_section_list_start(line).unwrap_or(0);
    let mut spans = vec![];
    let mut depth = 0usize;
    let mut atom_start = None;

    for (relative_index, ch) in line[scan_start..].char_indices() {
        let index = scan_start + relative_index;
        match ch {
            '{' => {
                depth += 1;
                atom_start = None;
            }
            '}' if depth > 0 => {
                depth -= 1;
                atom_start = None;
            }
            ch if depth == 0 && (ch.is_ascii_alphanumeric() || ch == '_') => {
                atom_start.get_or_insert(index);
            }
            _ if depth == 0 => {
                if let Some(start) = atom_start.take() {
                    spans.push((start, index));
                }
            }
            _ => {}
        }
    }
    if let Some(start) = atom_start {
        spans.push((start, line.len()));
    }

    spans
}

fn rebar_profile_name(trimmed: &str) -> Option<&str> {
    let tail = trimmed.strip_prefix('{')?;
    if tail.starts_with("deps")
        || tail.starts_with("plugins")
        || tail.starts_with("project_plugins")
        || tail.starts_with("profiles")
    {
        return None;
    }
    let end = tail.find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))?;
    tail.get(..end)
}

fn rebar_dependency_tuple_spans(line: &str) -> Vec<(usize, usize)> {
    let scan_start = rebar_dependency_section_list_start(line).unwrap_or(0);
    let mut spans = vec![];
    let mut depth = 0usize;
    let mut tuple_start = None;

    for (relative_index, ch) in line[scan_start..].char_indices() {
        let index = scan_start + relative_index;
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
                {
                    spans.push((start, index + ch.len_utf8()));
                }
            }
            _ => {}
        }
    }

    spans
}

fn rebar_dependency_section_name(source: &str) -> Option<&'static str> {
    let tail = source.trim_start();
    if tail.starts_with("{deps") {
        Some("deps")
    } else if tail.starts_with("{project_plugins") {
        Some("project_plugins")
    } else if tail.starts_with("{plugins") {
        Some("plugins")
    } else {
        None
    }
}

fn rebar_dependency_section_list_start(line: &str) -> Option<usize> {
    for section in ["{project_plugins", "{deps", "{plugins"] {
        if let Some(section_start) = line.find(section)
            && let Some(list_start) = line
                .get(section_start..)?
                .find('[')
                .map(|index| section_start + index + 1)
        {
            return Some(list_start);
        }
    }
    None
}

fn rebar_tuple_atom(tuple: &str) -> Option<&str> {
    let tail = tuple.strip_prefix('{')?.trim_start();
    let end = tail.find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))?;
    tail.get(..end)
}

fn rebar_requirement(tuple: &str) -> Option<(&str, Option<&'static str>)> {
    if let Some(url) = rebar_source_url(tuple, "git_subdir") {
        return Some((url, Some("git")));
    }
    if let Some(url) = rebar_source_url(tuple, "git") {
        return Some((url, Some("git")));
    }
    if let Some(url) = rebar_source_url(tuple, "hg") {
        return Some((url, Some("hg")));
    }
    if tuple.contains("{pkg,") {
        return rebar_quoted_requirement(tuple)
            .map(|requirement| (requirement, None))
            .or(Some(("latest", None)));
    }
    rebar_quoted_requirement(tuple).map(|requirement| (requirement, None))
}

fn rebar_quoted_requirement(tuple: &str) -> Option<&str> {
    let first_quote = tuple.find('"')?;
    let value = tuple.get(first_quote + 1..)?;
    let end = value.find('"')?;
    value.get(..end)
}

fn rebar_source_url<'a>(tuple: &'a str, source: &str) -> Option<&'a str> {
    let marker = format!("{{{source},");
    let start = tuple.find(&marker)? + marker.len();
    let value = tuple.get(start..)?.trim_start();
    let value = value.strip_prefix('"')?;
    let end = value.find('"')?;
    value.get(..end)
}

fn rebar_package_name(tuple: &str) -> Option<(&str, Option<&str>)> {
    let app_name = rebar_tuple_atom(tuple)?;
    let marker = "{pkg,";
    let start = tuple.find(marker)? + marker.len();
    let tail = tuple.get(start..)?.trim_start();
    let end = tail.find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))?;
    let package_name = tail.get(..end)?;
    Some((package_name, Some(app_name)))
}

#[cfg(test)]
mod tests;
