use crate::MavenNamedRepository;
use crate::{DocumentInput, parse_document};
use versionlens_model::Range;

pub(crate) fn parse_fixture(
    text: &str,
    uri: &str,
    language: &str,
) -> Vec<versionlens_model::Dependency> {
    parse_document(&DocumentInput::new(
        uri.to_owned(),
        language.to_owned(),
        text.to_owned(),
        None,
    ))
}

pub(crate) fn extract_range(text: &str, range: Range) -> &str {
    let line = text.lines().nth(range.start.line as usize).unwrap_or("");
    let start = utf16_character_to_byte(line, range.start.character);
    let end = utf16_character_to_byte(line, range.end.character);
    &line[start..end]
}

pub(crate) fn assert_parenthesized_pep508(
    text: &str,
    dependencies: &[versionlens_model::Dependency],
) {
    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "unfat");
    assert_eq!(dependencies[0].requirement, ">=0.0.13");
    assert_eq!(dependencies[0].requirement_suffix, ")");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        ">=0.0.13"
    );
    assert_eq!(dependencies[1].name, "httpx");
    assert_eq!(dependencies[1].requirement, ">=0.28,<1");
    assert_eq!(dependencies[1].requirement_suffix, ")");
}

pub(crate) fn assert_two_repositories(
    repositories: &[MavenNamedRepository],
    first: (&str, &str),
    second: (&str, &str),
) {
    assert_eq!(repositories.len(), 2);
    for (repository, expected) in repositories.iter().zip([first, second]) {
        assert_eq!(repository.id, expected.0);
        assert_eq!(repository.url, expected.1);
    }
}

fn utf16_character_to_byte(line: &str, character: u32) -> usize {
    let target = character as usize;
    let mut units = 0;
    for (offset, value) in line.char_indices() {
        if units >= target {
            return offset;
        }
        units += value.len_utf16();
    }
    line.len()
}
