use std::io::ErrorKind;

use versionlens_test_support::read_fixture;

#[test]
fn reports_the_fixture_path_and_io_source() {
    let error = read_fixture(env!("CARGO_MANIFEST_DIR"), "tests/fixtures", "missing.txt")
        .expect_err("missing fixtures must return an error");

    assert!(error.path.ends_with("tests/fixtures/missing.txt"));
    assert_eq!(error.source.kind(), ErrorKind::NotFound);
}

#[test]
fn reports_invalid_manifest_root_without_aborting() {
    let error = read_fixture("/", "tests/fixtures", "missing.txt")
        .expect_err("an invalid manifest root must return an error");

    assert_eq!(error.source.kind(), ErrorKind::InvalidInput);
}
