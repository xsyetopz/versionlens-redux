use crate::positions::line_range;
use std::iter::from_fn;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::CocoaPods;

pub(crate) fn parse_cocoapods_podfile(text: &str) -> Vec<Dependency> {
    let mut parser: PodfileParser = crate::default();

    for (line_index, line) in text.lines().enumerate() {
        parser.collect_line(line_index, line);
    }

    parser.dependencies
}

#[derive(Default)]
struct PodfileParser {
    block_stack: Vec<String>,
    sources: Vec<String>,
    dependencies: Vec<Dependency>,
}

impl PodfileParser {
    fn collect_line(&mut self, line_index: usize, line: &str) {
        let trimmed = line.trim_start();
        if trimmed.trim_end() == "end" {
            self.block_stack.pop();
            return;
        }

        let content = strip_comment(trimmed).trim_end();
        if let Some(source) = source_line_url(content) {
            self.sources.push(trim_url(source));
            return;
        }
        if let Some(target) = block_name(content, "target") {
            self.block_stack.push(format!("target {target}"));
            return;
        }
        if let Some(target) = block_name(content, "abstract_target") {
            self.block_stack.push(format!("abstract_target {target}"));
            return;
        }

        if let Some(dependency) = parse_pod_line(PodLine {
            line_index,
            line,
            content,
            offset: line.len() - trimmed.len(),
            group: self.current_group(),
            global_source: self.sources.first().map(|value| value.as_str()),
        }) {
            self.dependencies.push(dependency);
        }
    }

    fn current_group(&self) -> &str {
        self.block_stack
            .last()
            .map(|value| value.as_str())
            .unwrap_or("dependencies")
    }
}

struct PodLine<'a> {
    line_index: usize,
    line: &'a str,
    content: &'a str,
    offset: usize,
    group: &'a str,
    global_source: Option<&'a str>,
}

fn parse_pod_line(pod_line: PodLine<'_>) -> Option<Dependency> {
    let after_pod = pod_line.content.strip_prefix("pod ")?.trim_start();
    let pod_start = pod_line.content.len() - after_pod.len();
    let mut strings = quoted_strings(after_pod);
    let (name, name_start, name_end) = strings.next()?;
    let name_quote_end = name_end + quote_len_after_name(after_pod, name_end);
    let args = &after_pod[name_quote_end..];

    let (requirement, requirement_start, requirement_end, requirement_prefix, requirement_suffix) =
        if let Some((value, start, end)) = first_version_requirement(args) {
            let absolute_start = pod_start + name_quote_end + start;
            let absolute_end = pod_start + name_quote_end + end;
            let (_, prefix) = split_operator_prefix(value);
            (
                value.to_owned(),
                absolute_start,
                absolute_end,
                prefix.to_owned(),
                "".to_owned(),
            )
        } else if let Some((key, value, start, end)) =
            source_option(args, &["tag", "branch", "commit"])
        {
            let absolute_start = pod_start + name_quote_end + start;
            let absolute_end = pod_start + name_quote_end + end;
            (
                value.to_owned(),
                absolute_start,
                absolute_end,
                format!(":{key} => '"),
                "'".to_owned(),
            )
        } else if let Some((_, value, start, end)) =
            source_option(args, &["path", "git", "podspec"])
        {
            (
                value.to_owned(),
                pod_start + name_quote_end + start,
                pod_start + name_quote_end + end,
                "".to_owned(),
                "".to_owned(),
            )
        } else {
            let insert_at = pod_start + name_end + quote_len_after_name(after_pod, name_end);
            (
                "latest".to_owned(),
                insert_at,
                insert_at,
                ", '".to_owned(),
                "'".to_owned(),
            )
        };

    let hosted_url = if source_option(args, &["path"]).is_some() {
        Some("path".to_owned())
    } else if source_option(args, &["git"]).is_some() {
        Some("git".to_owned())
    } else if source_option(args, &["podspec"]).is_some() {
        Some("podspec".to_owned())
    } else if let Some((_, source, _, _)) = source_option(args, &["source"]) {
        Some(trim_url(source))
    } else if requirement == "latest" {
        Some("latest".to_owned())
    } else {
        pod_line.global_source.map(trim_url)
    };

    Some(Dependency {
        name: name.to_owned(),
        requirement,
        ecosystem: CocoaPods,
        group: pod_line.group.to_owned(),
        hosted_url,
        hosted_name: None,
        range: line_range(
            pod_line.line_index,
            pod_line.line,
            pod_line.offset + pod_start + name_start,
            pod_line.offset + pod_start + name_end,
        ),
        requirement_range: line_range(
            pod_line.line_index,
            pod_line.line,
            pod_line.offset + requirement_start,
            pod_line.offset + requirement_end,
        ),
        requirement_prefix,
        requirement_suffix,
    })
}

fn first_version_requirement<'a>(args: &'a str) -> Option<(&'a str, usize, usize)> {
    quoted_strings(args).find(|(_, start, _)| !args[..*start].contains(':'))
}

fn source_line_url(line: &str) -> Option<&str> {
    if !line.starts_with("source ") {
        return None;
    }
    quoted_strings(line).next().map(|(value, _, _)| value)
}

fn block_name<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    if !(line.starts_with(keyword) && line.ends_with(" do")) {
        return None;
    }
    quoted_strings(line).next().map(|(value, _, _)| value)
}

fn source_option<'a>(
    args: &'a str,
    keys: &[&'static str],
) -> Option<(&'static str, &'a str, usize, usize)> {
    keys.iter().find_map(|key| option_string(args, key))
}

fn option_string<'a>(
    args: &'a str,
    key: &'static str,
) -> Option<(&'static str, &'a str, usize, usize)> {
    [format!(":{key} =>"), format!("{key}:")]
        .into_iter()
        .find_map(|pattern| {
            let key_start = args.find(&pattern)?;
            let value_start = key_start + pattern.len();
            let tail = args[value_start..].trim_start();
            let whitespace = args[value_start..].len() - tail.len();
            let (value, start, end) = quoted_strings(tail).next()?;
            Some((
                key,
                value,
                value_start + whitespace + start,
                value_start + whitespace + end,
            ))
        })
}

fn split_operator_prefix(value: &str) -> (usize, &str) {
    for operator in ["~>", ">=", "<=", "==", "=", ">", "<"] {
        if let Some(rest) = value.strip_prefix(operator) {
            let whitespace = rest.len() - rest.trim_start().len();
            let prefix_len = operator.len() + whitespace;
            return (prefix_len, &value[..prefix_len]);
        }
    }
    (0, "")
}

fn quote_len_after_name(line: &str, name_end: usize) -> usize {
    line.as_bytes().get(name_end).map(|_| 1).unwrap_or_default()
}

fn trim_url(value: &str) -> String {
    value.trim_end_matches('/').to_owned()
}

fn strip_comment(line: &str) -> &str {
    let mut quote = None;
    let mut escaped = false;
    for (index, ch) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        match quote {
            Some(current) if ch == current => quote = None,
            None if ch == '\'' || ch == '"' => quote = Some(ch),
            None if ch == '#' => return &line[..index],
            Some(_) | None => {}
        }
    }
    line
}

fn quoted_strings(text: &str) -> impl Iterator<Item = (&str, usize, usize)> + '_ {
    let mut offset = 0usize;
    from_fn(move || {
        let bytes = text.as_bytes();
        let mut index = offset;
        while index < bytes.len() && bytes[index] != b'\'' && bytes[index] != b'"' {
            index += 1;
        }
        if index >= bytes.len() {
            offset = index;
            return None;
        }
        let quote = bytes[index];
        let start = index + 1;
        index = start;
        while index < bytes.len() {
            if bytes[index] == quote && bytes.get(index.wrapping_sub(1)) != Some(&b'\\') {
                let end = index;
                offset = index + 1;
                return Some((&text[start..end], start, end));
            }
            index += 1;
        }
        offset = index;
        None
    })
}
