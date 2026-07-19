use super::protocol::protocol_from_url;
use super::schema::DotnetSource;

pub fn parse_dotnet_sources(text: &str) -> Vec<DotnetSource> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_line)
        .collect()
}

pub fn parse_dotnet_enabled_sources(text: &str, feed_urls: &[String]) -> Vec<DotnetSource> {
    feed_urls
        .iter()
        .filter(|url| !url.trim().is_empty())
        .map(|url| source_from_url(url))
        .chain(
            parse_dotnet_sources(text)
                .into_iter()
                .filter(|source| source.enabled),
        )
        .collect()
}

fn parse_line(line: &str) -> DotnetSource {
    let machine_wide = line.as_bytes().get(1) == Some(&b'M');
    let url = line
        .get(if machine_wide { 3 } else { 2 }..)
        .unwrap_or_default()
        .trim();
    let protocol = protocol_from_url(url);

    DotnetSource {
        enabled: line.starts_with('E'),
        machine_wide,
        url: url.to_owned(),
        protocol,
    }
}

fn source_from_url(url: &str) -> DotnetSource {
    let url = url.trim();
    let protocol = protocol_from_url(url);

    DotnetSource {
        enabled: true,
        machine_wide: protocol == "file:",
        url: url.to_owned(),
        protocol,
    }
}
