use crate::positions::line_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Dotnet;

pub(crate) fn parse_paket_dependencies(text: &str) -> Vec<Dependency> {
    text.lines()
        .enumerate()
        .filter_map(|(line_index, line)| parse_paket_dependency_line(line_index, line))
        .collect()
}

pub(crate) fn parse_paket_references(text: &str) -> Vec<Dependency> {
    text.lines()
        .enumerate()
        .filter_map(|(line_index, line)| parse_paket_reference_line(line_index, line))
        .collect()
}

pub fn parse_paket_source_urls(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(paket_source_url)
        .filter(|url| url.starts_with("https://") || url.starts_with("http://"))
        .map(|value| value.to_owned())
        .collect()
}

fn parse_paket_dependency_line(line_index: usize, line: &str) -> Option<Dependency> {
    let (trimmed, offset) = paket_content(line)?;
    let after_keyword = trimmed.strip_prefix("nuget ")?;
    let name_start_in_trimmed = trimmed.len() - after_keyword.len();
    let name_end_in_after = after_keyword
        .find(char::is_whitespace)
        .unwrap_or(after_keyword.len());
    let name = after_keyword[..name_end_in_after].trim();
    if name.is_empty() {
        return None;
    }

    let after_name_start = name_start_in_trimmed + name_end_in_after;
    let after_name = &trimmed[after_name_start..];
    let requirement_trim_offset = after_name.len() - after_name.trim_start().len();
    let requirement_start_in_trimmed = after_name_start + requirement_trim_offset;
    let requirement_text = paket_requirement(after_name.trim_start());
    let requirement_end_in_trimmed = requirement_start_in_trimmed + requirement_text.len();

    Some(Dependency {
        name: name.to_owned(),
        requirement: requirement_text.to_owned(),
        ecosystem: Dotnet,
        group: "paket.dependencies".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: line_range(
            line_index,
            line,
            offset + name_start_in_trimmed,
            offset + name_start_in_trimmed + name.len(),
        ),
        requirement_range: line_range(
            line_index,
            line,
            offset + requirement_start_in_trimmed,
            offset + requirement_end_in_trimmed,
        ),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
        canonical_reference: None,
    })
}

fn parse_paket_reference_line(line_index: usize, line: &str) -> Option<Dependency> {
    let (trimmed, offset) = paket_content(line)?;

    let name = trimmed.split_whitespace().next()?;
    if name.is_empty() {
        return None;
    }

    let name_end = offset + name.len();
    Some(Dependency {
        name: name.to_owned(),
        requirement: "".to_owned(),
        ecosystem: Dotnet,
        group: "paket.references".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: line_range(line_index, line, offset, name_end),
        requirement_range: line_range(line_index, line, name_end, name_end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
        canonical_reference: None,
    })
}

fn paket_content(line: &str) -> Option<(&str, usize)> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
        return None;
    }
    Some((trimmed, line.len() - trimmed.len()))
}

fn paket_source_url(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let after_keyword = trimmed.strip_prefix("source ")?;
    let token = after_keyword.split_whitespace().next()?.trim();
    (!token.is_empty()).then_some(token)
}

fn paket_requirement(raw: &str) -> &str {
    if raw.is_empty() || starts_paket_option(raw) {
        return "";
    }

    let raw_without_comment = raw
        .find(" //")
        .map_or(raw, |comment_start| &raw[..comment_start])
        .trim_end();
    let mut end = raw_without_comment.len();
    for option in [
        " prerelease",
        " framework:",
        " restriction:",
        " copy_local:",
        " content:",
        " import_targets:",
        " redirects:",
        " strategy:",
        " lowest_matching:",
    ] {
        if let Some(index) = raw_without_comment.find(option) {
            end = end.min(index);
        }
    }
    raw_without_comment[..end].trim_end()
}

fn starts_paket_option(raw: &str) -> bool {
    raw.starts_with("prerelease")
        || raw.starts_with("framework:")
        || raw.starts_with("restriction:")
        || raw.starts_with("copy_local:")
        || raw.starts_with("content:")
        || raw.starts_with("import_targets:")
        || raw.starts_with("redirects:")
        || raw.starts_with("strategy:")
        || raw.starts_with("lowest_matching:")
}

#[cfg(test)]
mod tests {
use super::parse_paket_source_urls;
use crate::document::test_support::{extract_range, parse_fixture};
use versionlens_model::Ecosystem::Dotnet;

#[test]
fn parses_paket_dependencies_nuget_lines() {
    let text = package_file_fixture("parses-paket-dependencies-nuget-lines.txt");
    let dependencies = parse_fixture(text, "file:///work/paket.dependencies", "plaintext");

    assert_eq!(dependencies.len(), 3);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(
            0,
            Dotnet,
            "paket.dependencies",
            "Newtonsoft.Json",
            "13.0.3",
        ),
    );
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "13.0.3"
    );
    assert_eq!(dependencies[1].name, "FSharp.Core");
    assert_eq!(dependencies[1].requirement, ">= 8.0");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        ">= 8.0"
    );
    assert_eq!(dependencies[2].name, "Paket.Core");
    assert_eq!(dependencies[2].requirement, "");
    assert_eq!(extract_range(text, dependencies[2].requirement_range), "");
}

#[test]
fn parses_paket_references_as_unresolved_project_references_without_update_ranges() {
    let text = package_file_fixture(
        "parses-paket-references-as-unresolved-project-references-without-update-ranges.txt",
    );
    let dependencies = parse_fixture(text, "file:///work/paket.references", "plaintext");

    assert_eq!(dependencies.len(), 2);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(
            0,
            Dotnet,
            "paket.references",
            "Newtonsoft.Json",
            "",
        ),
    );
    crate::support::tests::assert_requirement_range_ends_at_dependency(&dependencies, 0);
}

#[test]
fn parses_paket_source_urls() {
    assert_eq!(
        parse_paket_source_urls("source https://api.nuget.org/v3/index.json\nsource ./local\n"),
        ["https://api.nuget.org/v3/index.json"]
    );
}

fn package_file_fixture(name: &str) -> &'static str {
    crate::support::tests::fixture("tests/fixtures/versionlens-parsers/src/paket/tests", name)
}
}
