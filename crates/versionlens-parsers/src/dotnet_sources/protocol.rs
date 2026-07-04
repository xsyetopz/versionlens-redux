pub(super) fn protocol_from_url(url: &str) -> String {
    url.find(':')
        .map(|index| &url[..=index])
        .filter(|protocol| matches!(*protocol, "http:" | "https:"))
        .unwrap_or("file:")
        .to_owned()
}
