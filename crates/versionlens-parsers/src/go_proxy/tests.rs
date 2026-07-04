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
