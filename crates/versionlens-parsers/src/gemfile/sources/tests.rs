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
