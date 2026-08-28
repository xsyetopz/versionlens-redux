use crate::positions::offset_range;
use crate::requirement_range::operator_requirement_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Cpan;

pub(crate) fn parse_cpanfile(text: &str) -> Vec<Dependency> {
    let mut dependencies = vec![];
    let mut scope_stack = Vec::<Scope>::new();
    let mut offset = 0usize;

    for line in text.lines() {
        let trimmed = line.trim_start();
        push_scope(trimmed, &mut scope_stack);

        if let Some((kind, call_start)) = dependency_call(trimmed) {
            let group = cpanfile_group(kind, &scope_stack);
            if let Some(dependency) = parse_dependency_call(CpanDependencyCall {
                text,
                line,
                line_offset: offset,
                call_start: offset + line.len() - trimmed.len() + call_start,
                kind,
                group: &group,
            }) {
                dependencies.push(dependency);
            }
        }

        pop_scopes(trimmed, &mut scope_stack);
        offset += line.len() + 1;
    }

    dependencies
}

#[derive(Clone)]
enum Scope {
    Phase(String),
    Feature(String),
}

type ScopeStack = Vec<Scope>;
use Scope::{Feature as ScopeFeature, Phase as ScopePhase};

fn push_scope(line: &str, scope_stack: &mut ScopeStack) {
    if let Some(name) = quoted_arg_after(line, "on ") {
        scope_stack.push(ScopePhase(name.to_owned()));
    } else if let Some(name) = quoted_arg_after(line, "feature ") {
        scope_stack.push(ScopeFeature(name.to_owned()));
    }
}

fn pop_scopes(line: &str, scope_stack: &mut ScopeStack) {
    if line.starts_with("};") || line == "}" || line == "}," {
        scope_stack.pop();
    }
}

fn quoted_arg_after<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = line.strip_prefix(prefix)?.trim_start();
    quoted_string(rest).map(|(_, value, _)| value)
}

fn dependency_call(line: &str) -> Option<(&str, usize)> {
    for kind in [
        "requires",
        "recommends",
        "suggests",
        "conflicts",
        "configure_requires",
        "build_requires",
        "test_requires",
        "author_requires",
    ] {
        if line.starts_with(kind) {
            return Some((kind, 0));
        }
    }
    None
}

fn cpanfile_group(kind: &str, scope_stack: &[Scope]) -> String {
    let kind = match kind {
        "configure_requires" | "build_requires" | "test_requires" | "author_requires" => "requires",
        other => other,
    };
    let prefix = scope_stack.last().map(|scope| match scope {
        ScopePhase(phase) => phase.to_owned(),
        ScopeFeature(feature) => format!("feature.{feature}"),
    });
    prefix.map_or_else(|| kind.to_owned(), |prefix| format!("{prefix}.{kind}"))
}

struct CpanDependencyCall<'a> {
    text: &'a str,
    line: &'a str,
    line_offset: usize,
    call_start: usize,
    kind: &'a str,
    group: &'a str,
}

fn parse_dependency_call(call: CpanDependencyCall<'_>) -> Option<Dependency> {
    let local_call_start = call.call_start.checked_sub(call.line_offset)?;
    let local_after_call = local_call_start + call.kind.len();
    let (_, name, name_relative_start) = quoted_string(&call.line[local_after_call..])?;
    let local_name_start = local_after_call + name_relative_start;
    let name_start = call.line_offset + local_name_start;
    let local_after_name = local_name_start + name.len() + 1;
    let (requirement, requirement_start) =
        if let Some((_, requirement, requirement_relative_start)) =
            quoted_string(&call.line[local_after_name..])
        {
            (
                requirement.to_owned(),
                call.line_offset + local_after_name + requirement_relative_start,
            )
        } else {
            ("0".to_owned(), name_start + name.len())
        };
    let range = operator_requirement_range(&requirement, &["==", "!=", "<=", ">=", "<", ">", "~"]);
    let requirement_absolute_start = requirement_start + range.start;
    let requirement_absolute_end = requirement_start + range.end;

    Some(Dependency {
        name: name.to_owned(),
        requirement,
        ecosystem: Cpan,
        group: cpanfile_shortcut_group(call.kind)
            .unwrap_or(call.group)
            .to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(call.text, name_start, name_start + name.len()),
        requirement_range: offset_range(
            call.text,
            requirement_absolute_start,
            requirement_absolute_end,
        ),
        requirement_prefix: range.prefix,
        requirement_suffix: "".to_owned(),
    })
}

fn cpanfile_shortcut_group(kind: &str) -> Option<&'static str> {
    match kind {
        "configure_requires" => Some("configure.requires"),
        "build_requires" => Some("build.requires"),
        "test_requires" => Some("test.requires"),
        "author_requires" => Some("develop.requires"),
        _ => None,
    }
}

fn quoted_string(text: &str) -> Option<(char, &str, usize)> {
    let quote_start = text.find(['\'', '"'])?;
    let quote = text[quote_start..].chars().next()?;
    let start = quote_start + quote.len_utf8();
    let end = start + text[start..].find(quote)?;
    Some((quote, &text[start..end], start))
}
