use super::*;

fn sums(archive: &str, bytes: &[u8]) -> String {
    format!(
        "{}  {archive}\n{}  other.tar.gz\n",
        sha256_hex(bytes),
        "0".repeat(64)
    )
}

#[test]
fn checksum_match() {
    let name = "versionlens-lsp-1.0.0-x86_64-apple-darwin.tar.gz";
    assert!(verify_checksum(name, b"payload", &sums(name, b"payload")).is_ok());
}

#[test]
fn checksum_mismatch_names_both_sums() {
    let name = "versionlens-lsp-1.0.0-x86_64-apple-darwin.tar.gz";
    let error = verify_checksum(name, b"tampered", &sums(name, b"payload")).unwrap_err();
    assert!(error.contains(&sha256_hex(b"payload")));
    assert!(error.contains(&sha256_hex(b"tampered")));
}

#[test]
fn checksum_missing_entry() {
    assert!(expected_checksum("abc  other.zip\n", "x.zip").is_err());
}

#[test]
fn stale_entries_exclude_current_and_unrelated() {
    let entries = [
        "versionlens-lsp-0.4.3-x86_64-apple-darwin",
        "versionlens-lsp-0.4.4-x86_64-apple-darwin",
        "versionlens-lsp-0.4.4-x86_64-apple-darwin.download",
        "other",
    ]
    .map(String::from);
    assert_eq!(
        stale_entries(&entries, "versionlens-lsp-0.4.4-x86_64-apple-darwin"),
        vec![
            "versionlens-lsp-0.4.3-x86_64-apple-darwin".to_owned(),
            "versionlens-lsp-0.4.4-x86_64-apple-darwin.download".to_owned()
        ]
    );
}

#[test]
fn triples_come_from_targets_json() {
    assert_eq!(
        release_triple(Os::Linux, Architecture::X8664).unwrap(),
        "x86_64-unknown-linux-gnu"
    );
    assert_eq!(
        archive_name("1.0.0", "aarch64-pc-windows-msvc"),
        "versionlens-lsp-1.0.0-aarch64-pc-windows-msvc.zip"
    );
    assert!(release_triple(Os::Linux, Architecture::X86).is_err());
}

#[test]
fn extracts_binary_from_tar_gz() {
    let mut builder = tar::Builder::new(Vec::new());
    let mut header = tar::Header::new_gnu();
    header.set_size(3);
    header.set_mode(0o755);
    header.set_cksum();
    builder
        .append_data(&mut header, "versionlens-lsp", &b"bin"[..])
        .unwrap();
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    std::io::Write::write_all(&mut gz, &builder.into_inner().unwrap()).unwrap();
    let bytes = gz.finish().unwrap();
    assert_eq!(
        extract_binary("x.tar.gz", &bytes, "versionlens-lsp").unwrap(),
        b"bin"
    );
}

#[test]
fn extracts_binary_from_zip() {
    use std::io::Write;
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file(
            "versionlens-lsp.exe",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    writer.write_all(b"exe").unwrap();
    let bytes = writer.finish().unwrap().into_inner();
    assert_eq!(
        extract_binary("x.zip", &bytes, "versionlens-lsp.exe").unwrap(),
        b"exe"
    );
}

#[test]
fn workspace_configuration_returns_configured_settings() {
    let settings = json!({"prereleases": true});
    assert_eq!(
        workspace_configuration(Some(settings.clone())),
        json!({"versionlens": settings})
    );
    assert_eq!(workspace_configuration(None), json!({"versionlens": {}}));
}

#[test]
fn initialization_options_ask_for_authentication_actions() {
    let settings = json!({"prereleases": true});
    assert_eq!(
        initialization_options(Some(settings)),
        json!({"versionlens": {"prereleases": true}, "authenticationActions": true})
    );
}
