/// Reads the Go module proxy URLs from the `GOPROXY` entry of `env`.
pub fn parse_go_proxy_urls(env: &[(String, String)]) -> Vec<String> {
    env.iter()
        .rev()
        .find_map(|(key, value)| (key == "GOPROXY").then_some(value))
        .map(|value| {
            value
                .split([',', '|'])
                .map(str::trim)
                .take_while(|value| *value != "off")
                .filter_map(clean_go_proxy_url)
                .collect()
        })
        .unwrap_or_default()
}

fn clean_go_proxy_url(value: &str) -> Option<String> {
    let value = value.trim().trim_end_matches('/');
    if value.is_empty() || matches!(value, "direct" | "off") {
        return None;
    }

    Some(format!("{value}/{{base-module}}/@v/list"))
}

#[cfg(test)]
mod tests {
    use super::parse_go_proxy_urls;

    #[test]
    fn parses_go_proxy_urls_from_env() {
        let env = vec![
            (
                "GOPROXY".to_owned(),
                "https://proxy.example.test/,direct|https://fallback.example.test|off".to_owned(),
            ),
            ("OTHER".to_owned(), "ignored".to_owned()),
        ];

        assert_eq!(
            parse_go_proxy_urls(&env),
            vec![
                "https://proxy.example.test/{base-module}/@v/list",
                "https://fallback.example.test/{base-module}/@v/list",
            ]
        );
    }

    #[test]
    fn parses_go_proxy_urls_from_last_env_entry() {
        let env = vec![
            (
                "GOPROXY".to_owned(),
                "https://process-proxy.example.test".to_owned(),
            ),
            (
                "GOPROXY".to_owned(),
                "https://workspace-proxy.example.test".to_owned(),
            ),
        ];

        assert_eq!(
            parse_go_proxy_urls(&env),
            vec!["https://workspace-proxy.example.test/{base-module}/@v/list"]
        );
    }
}
