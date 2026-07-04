use super::syntax::{quoted_strings, strip_comment};

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
mod tests;
