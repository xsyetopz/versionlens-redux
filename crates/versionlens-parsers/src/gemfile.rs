use self::line::gem_name_range;
use versionlens_model::Ecosystem::Ruby;
mod github;
mod line;
mod sources;
mod standard;
mod syntax;

use crate::positions::line_range;
use github::{gem_github_default_dependency, gem_github_ref_dependency, gem_github_tag_dependency};
use line::{GemLineContext, GemNameSpan};
use sources::source_block_url_from_line;
use standard::standard_gem_dependency;
use syntax::{quoted_strings, strip_comment};
use versionlens_model::Dependency;

pub use sources::parse_gemfile_source_urls;

pub(crate) fn parse_gemfile(text: &str) -> Vec<Dependency> {
    let mut parser: GemfileParser<'_> = crate::default();

    for (line_index, line) in text.lines().enumerate() {
        parser.collect_line(line_index, line);
    }

    parser.dependencies
}

pub(crate) fn parse_gemspec(text: &str) -> Vec<Dependency> {
    text.lines()
        .enumerate()
        .filter_map(|(line_index, line)| parse_gemspec_line(line_index, line))
        .collect()
}

#[derive(Default)]
struct GemfileParser<'a> {
    block_stack: Vec<GemfileBlock<'a>>,
    dependencies: Vec<Dependency>,
}

enum GemfileBlock<'a> {
    Group(&'a str),
    Source(String),
    Git { url: String, group: String },
    Path { path: String, group: String },
}

impl<'a> GemfileParser<'a> {
    fn collect_line(&mut self, line_index: usize, line: &'a str) {
        use GemfileBlock::{Git, Group, Path, Source};
        let trimmed = line.trim_start();
        if trimmed.trim_end() == "end" {
            self.block_stack.pop();
            return;
        }
        if let Some(url) = source_block_url_from_line(trimmed) {
            self.block_stack.push(Source(url));
            return;
        }
        if let Some(path) = block_quoted_value(trimmed, "path") {
            self.block_stack.push(Path {
                path: path.to_owned(),
                group: format!("path {path}"),
            });
            return;
        }
        if let Some(url) = block_quoted_value(trimmed, "git") {
            self.block_stack.push(Git {
                url: url.to_owned(),
                group: format!("git {url}"),
            });
            return;
        }
        if trimmed.starts_with("group ") && trimmed.ends_with(" do") {
            self.block_stack
                .push(Group(trimmed.trim_end_matches(" do")));
            return;
        }

        if let Some(dependency) = parse_gem_line(
            line_index,
            line,
            self.current_group(),
            self.current_source(),
            self.current_block_requirement(),
        ) {
            self.dependencies.push(dependency);
        }
    }

    fn current_group(&self) -> &str {
        use GemfileBlock::{Git, Group, Path, Source};
        self.block_stack
            .iter()
            .rev()
            .find_map(|block| match block {
                Group(group) => Some(*group),
                Source(_) => None,
                Git { group, .. } | Path { group, .. } => Some(group.as_str()),
            })
            .unwrap_or("dependencies")
    }

    fn current_source(&self) -> Option<&str> {
        use GemfileBlock::{Git, Group, Path, Source};
        self.block_stack.iter().rev().find_map(|block| match block {
            Source(url) => Some(url.as_str()),
            Group(_) | Git { .. } | Path { .. } => None,
        })
    }

    fn current_block_requirement(&self) -> Option<&str> {
        use GemfileBlock::{Git, Group, Path, Source};
        self.block_stack.iter().rev().find_map(|block| match block {
            Git { url, .. } => Some(url.as_str()),
            Path { path, .. } => Some(path.as_str()),
            Group(_) | Source(_) => None,
        })
    }
}

fn parse_gem_line(
    line_index: usize,
    line: &str,
    group: &str,
    source_url: Option<&str>,
    block_requirement: Option<&str>,
) -> Option<Dependency> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("gem ") {
        return None;
    }

    let content = strip_comment(trimmed).trim_end();
    let inline_group = inline_group_name(content);
    let group_name = inline_group.as_deref().unwrap_or(group);
    let gem_context = GemLineContext {
        line_index,
        line,
        offset: line.len() - trimmed.len(),
        content,
        group: group_name,
    };
    let mut strings = quoted_strings(gem_context.content);
    let (name, _, end) = strings.next()?;
    let name = GemNameSpan { name, end };

    if let Some(requirement) = block_requirement {
        return Some(inherited_block_dependency(&gem_context, &name, requirement));
    }

    gem_github_tag_dependency(&gem_context, &name)
        .or_else(|| gem_github_ref_dependency(&gem_context, &name))
        .or_else(|| gem_github_default_dependency(&gem_context, &name))
        .or_else(|| {
            Some(standard_gem_dependency(
                &gem_context,
                &name,
                strings.next(),
                source_url,
            ))
        })
}

fn inline_group_name(content: &str) -> Option<String> {
    inline_group_value(content, "groups")
        .map(|value| format!("groups {value}"))
        .or_else(|| inline_group_value(content, "group").map(|value| format!("group {value}")))
}

fn inline_group_value(content: &str, attr: &str) -> Option<String> {
    let marker = format!("{attr}:");
    let attr_start = content.find(&marker)?;
    let value = content[attr_start + marker.len()..].trim_start();
    if let Some(array_tail) = value.strip_prefix('[') {
        let array_end = array_tail.find(']')?;
        let array = &array_tail[..array_end];
        let quoted = quoted_strings(array)
            .map(|(value, _, _)| value.to_owned())
            .collect::<Vec<_>>();
        if !quoted.is_empty() {
            return Some(quoted.join(", "));
        }
        let symbols = array
            .split(',')
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        return (!symbols.is_empty()).then(|| symbols.join(", "));
    }

    let value = value
        .split([',', ' '])
        .next()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())?;
    Some(value.to_owned())
}

fn block_quoted_value<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    if !(line.starts_with(keyword) && line.ends_with(" do")) {
        return None;
    }
    let (value, _, _) = quoted_strings(line).next()?;
    Some(value)
}

fn inherited_block_dependency(
    context: &GemLineContext<'_>,
    name: &GemNameSpan<'_>,
    requirement: &str,
) -> Dependency {
    let quote = context
        .content
        .as_bytes()
        .get(name.end)
        .copied()
        .unwrap_or(b'"') as char;
    let insert_at = name.end + quote.len_utf8();
    Dependency {
        name: name.name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Ruby,
        group: context.group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: gem_name_range(context),
        requirement_range: line_range(
            context.line_index,
            context.line,
            context.offset + insert_at,
            context.offset + insert_at,
        ),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    }
}

fn parse_gemspec_line(line_index: usize, line: &str) -> Option<Dependency> {
    const METHODS: [&str; 3] = [
        "add_runtime_dependency",
        "add_development_dependency",
        "add_dependency",
    ];

    let trimmed = line.trim_start();
    let content = strip_comment(trimmed).trim_end();
    let (method, _method_start, method_end) = METHODS.iter().find_map(|method| {
        let start = content.find(method)?;
        let before = content[..start].trim_end();
        let method_start = start + content[start..].find(method)?;
        let valid_receiver = before.is_empty() || before.ends_with('.');
        valid_receiver.then_some((*method, method_start, method_start + method.len()))
    })?;

    let args = &content[method_end..];
    let args_offset = method_end;
    let mut strings = quoted_strings(args);
    let (name, name_start, name_end) = strings.next()?;
    let version = strings.next();
    let (requirement, requirement_start, requirement_end, requirement_prefix, requirement_suffix) =
        if let Some((requirement, start, end)) = version {
            (
                requirement.to_owned(),
                args_offset + start,
                args_offset + end,
                "".to_owned(),
                "".to_owned(),
            )
        } else {
            let quote = args.as_bytes().get(name_end).copied().unwrap_or(b'\"') as char;
            let insert_at = args_offset + name_end + quote.len_utf8();
            (
                "*".to_owned(),
                insert_at,
                insert_at,
                format!(", {quote}"),
                quote.to_string(),
            )
        };
    let offset = line.len() - trimmed.len();

    Some(Dependency {
        name: name.to_owned(),
        requirement,
        ecosystem: Ruby,
        group: method.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: line_range(
            line_index,
            line,
            offset + args_offset + name_start,
            offset + args_offset + name_end,
        ),
        requirement_range: line_range(
            line_index,
            line,
            offset + requirement_start,
            offset + requirement_end,
        ),
        requirement_prefix,
        requirement_suffix,
    })
}

#[cfg(test)]
mod tests;
