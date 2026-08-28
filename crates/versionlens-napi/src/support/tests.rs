pub(crate) fn fixture(base: &str, name: &str) -> &'static str {
    versionlens_test_support::static_fixture!(base, name).expect("test fixture must be readable")
}
