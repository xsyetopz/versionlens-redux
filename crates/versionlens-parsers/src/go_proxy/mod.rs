pub fn parse_go_proxy_urls(env: &[(String, String)]) -> Vec<String> {
    env.iter()
        .rev()
        .find_map(|(key, value)| (key == "GOPROXY").then_some(value))
        .map(|value| {
            value
                .split([',', '|'])
                .map(|value| value.trim())
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
mod tests;
