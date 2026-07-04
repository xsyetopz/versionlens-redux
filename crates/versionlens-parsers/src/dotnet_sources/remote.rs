use super::model::DotnetSource;

pub fn filter_dotnet_remote_sources(sources: Vec<DotnetSource>) -> Vec<DotnetSource> {
    sources
        .into_iter()
        .filter(|source| matches!(source.protocol.as_str(), "http:" | "https:"))
        .collect()
}
