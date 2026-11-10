use super::syntax::{quoted_strings, strip_comment};

/// Lists the `source` URLs of a Gemfile.
pub fn parse_gemfile_source_urls(text: &str) -> Vec<String> {
    text.lines().filter_map(source_url_from_line).collect()
}

pub(super) fn source_block_url_from_line(line: &str) -> Option<String> {
    let trimmed = strip_comment(line.trim_start()).trim_end();
    if !trimmed.starts_with("source ") || !trimmed.ends_with(" do") {
        return None;
    }

    source_url(trimmed)
}

fn source_url_from_line(line: &str) -> Option<String> {
    let trimmed = strip_comment(line.trim_start()).trim_end();
    if !trimmed.starts_with("source ") || trimmed.ends_with(" do") {
        return None;
    }

    source_url(trimmed)
}

fn source_url(value: &str) -> Option<String> {
    quoted_strings(value)
        .next()
        .map(|(url, _, _)| url.trim_end_matches('/').to_owned())
        .filter(|url| !url.is_empty())
}

#[cfg(test)]
mod tests {
    use super::parse_gemfile_source_urls;

    #[test]
    fn parses_top_level_gemfile_source_urls() {
        let urls = parse_gemfile_source_urls(
            r#"
source "https://gems.example.test/"
source 'https://ignored.example.test' do
  gem "inside", "1.0.0"
end
source "https://mirror.example.test" # comment
"#,
        );

        assert_eq!(
            urls,
            vec!["https://gems.example.test", "https://mirror.example.test"]
        );
    }
}
