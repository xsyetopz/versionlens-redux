pub(crate) fn fixture(base: &str, name: &str) -> &'static str {
    versionlens_test_support::static_fixture!(base, name).expect("test fixture must be readable")
}

pub(crate) fn range(
    start_line: u32,
    start_character: u32,
    end_line: u32,
    end_character: u32,
) -> versionlens_model::Range {
    versionlens_model::Range {
        start: versionlens_model::Position {
            line: start_line,
            character: start_character,
        },
        end: versionlens_model::Position {
            line: end_line,
            character: end_character,
        },
    }
}
