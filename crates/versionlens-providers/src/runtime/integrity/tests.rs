use super::*;

#[test]
fn integrity_requires_both_releases_to_publish_valid_digests() {
    let source = RuntimeSource::PackageManager("pnpm");
    let pin = format!("1.0.0+sha512.{}", "00".repeat(64));
    for target in [
        Value::Null,
        Value::String("sha512-YQ==".to_owned()),
        Value::String("sha512-???".to_owned()),
    ] {
        let body = serde_json::json!({"versions": {
            "1.0.0": {"dist": {"integrity": format!("sha512-{}==", "A".repeat(86))}},
            "2.0.0": {"dist": {"integrity": target}}
        }})
        .to_string();
        assert!(source.verified_replacement(&body, &pin, "2.0.0").is_err());
    }
}

#[test]
fn version_prefix_is_preserved_in_runtime_replacements() {
    assert_eq!(
        RuntimeSource::Node
            .verified_replacement("[]", "v20.0.0", "22.0.0")
            .unwrap(),
        Some("v22.0.0".to_owned())
    );
    assert_eq!(
        RuntimeSource::Node
            .verified_replacement("[]", "20.0.0", "22.0.0")
            .unwrap(),
        None
    );
}

#[test]
fn registry_sha1_pins_are_verified_and_replaced() {
    let source = RuntimeSource::PackageManager("yarn");
    let body = serde_json::json!({"versions": {
        "1.22.19": {"dist": {"shasum": "00112233445566778899aabbccddeeff00112233"}},
        "1.22.22": {"dist": {"shasum": "ffeeddccbbaa99887766554433221100ffeeddcc"}}
    }})
    .to_string();
    assert_eq!(
        source
            .verified_replacement(
                &body,
                "1.22.19+sha1.00112233445566778899aabbccddeeff00112233",
                "1.22.22"
            )
            .unwrap(),
        Some("1.22.22+sha1.ffeeddccbbaa99887766554433221100ffeeddcc".to_owned())
    );
}

#[test]
fn registry_integrity_replacements_preserve_version_prefixes() {
    let sha1_body = serde_json::json!({"versions": {
        "1.0.0": {"dist": {"shasum": "00112233445566778899aabbccddeeff00112233"}},
        "2.0.0": {"dist": {"shasum": "ffeeddccbbaa99887766554433221100ffeeddcc"}}
    }})
    .to_string();
    let source = RuntimeSource::PackageManager("npm");
    assert_eq!(
        source
            .verified_replacement(
                &sha1_body,
                "v1.0.0+sha1.00112233445566778899aabbccddeeff00112233",
                "2.0.0"
            )
            .unwrap(),
        Some("v2.0.0+sha1.ffeeddccbbaa99887766554433221100ffeeddcc".to_owned())
    );

    let sha512_body = serde_json::json!({"versions": {
        "1.0.0": {"dist": {"integrity": format!("sha512-{}==", "A".repeat(86))}},
        "2.0.0": {"dist": {"integrity": format!("sha512-{}w==", "/".repeat(85))}}
    }})
    .to_string();
    assert_eq!(
        source
            .verified_replacement(
                &sha512_body,
                &format!("v1.0.0+sha512.{}", "00".repeat(64)),
                "2.0.0"
            )
            .unwrap(),
        Some(format!("v2.0.0+sha512.{}", "ff".repeat(64)))
    );
}

#[test]
fn registry_sha1_pins_require_valid_digests_for_both_releases() {
    let source = RuntimeSource::PackageManager("yarn");
    let pin = "1.22.19+sha1.00112233445566778899aabbccddeeff00112233";
    for target in [Value::Null, Value::String("not-a-digest".to_owned())] {
        let body = serde_json::json!({"versions": {
            "1.22.19": {"dist": {"shasum": "00112233445566778899aabbccddeeff00112233"}},
            "1.22.22": {"dist": {"shasum": target}}
        }})
        .to_string();
        assert!(source.verified_replacement(&body, pin, "1.22.22").is_err());
    }
}

#[test]
fn sha224_requires_the_release_artifact() {
    let error = RuntimeSource::PackageManager("pnpm")
        .verified_replacement("{}", &format!("1.0.0+sha224.{}", "00".repeat(28)), "2.0.0")
        .unwrap_err();
    assert!(error.contains("published release artifact"));
}

#[test]
fn modern_yarn_integrity_requires_the_standalone_binary() {
    let error = RuntimeSource::YarnModern
        .verified_replacement(
            "{}",
            &format!("4.14.1+sha224.{}", "00".repeat(28)),
            "4.18.0",
        )
        .unwrap_err();
    assert!(error.contains("standalone binary"));
}

#[test]
fn sha224_artifacts_verify_current_before_replacing_the_target() {
    let source = RuntimeSource::PackageManager("pnpm");
    let requirement = "1.0.0+sha224.e8c69735a0b5c2320a34aa2234434719d06d2b71a18d74904ebe874c";
    let body = r#"{"versions":{"1.0.0":{"dist":{"tarball":"https://registry.example/pnpm-1.0.0.tgz"}},"2.0.0":{"dist":{"tarball":"https://registry.example/pnpm-2.0.0.tgz"}}}}"#;
    assert_eq!(
        source
            .integrity_artifact_urls(body, requirement, "2.0.0")
            .unwrap(),
        Some((
            "https://registry.example/pnpm-1.0.0.tgz".to_owned(),
            "https://registry.example/pnpm-2.0.0.tgz".to_owned(),
        ))
    );
    source
        .verify_artifact_integrity(requirement, b"current")
        .unwrap();
    assert_eq!(
        source
            .artifact_integrity_replacement(requirement, "2.0.0", b"selected")
            .unwrap(),
        "2.0.0+sha224.1bb4223297e1f5140bf49830b53cd9e844a5142a3dfa5f8a2b024aa9"
    );
    assert!(
        source
            .verify_artifact_integrity(requirement, b"tampered")
            .is_err()
    );
}

#[test]
fn modern_yarn_sha224_uses_the_standalone_binary_urls() {
    let requirement = "4.14.1+sha224.88b7a7244bbd9040380c417f7eb556d85c67640b651f113cb4c72113";
    assert_eq!(
        RuntimeSource::YarnModern
            .integrity_artifact_urls("{}", requirement, "4.18.0")
            .unwrap(),
        Some((
            "https://repo.yarnpkg.com/4.14.1/packages/yarnpkg-cli/bin/yarn.js".to_owned(),
            "https://repo.yarnpkg.com/4.18.0/packages/yarnpkg-cli/bin/yarn.js".to_owned(),
        ))
    );
}

#[test]
fn modern_yarn_sha512_hashes_its_standalone_binary() {
    let requirement = "v4.14.1+sha512.b537be33c31741f9d6343c37446fa62c89faf3ebbf9e21061dcbc9ca87fdda16a43efaaa984607d8d8e0980d35af8ed215f264ea7b81bf1b272c8c8c168bb212";
    let source = RuntimeSource::YarnModern;
    assert_eq!(
        source
            .integrity_artifact_urls("{}", requirement, "4.18.0")
            .unwrap(),
        Some((
            "https://repo.yarnpkg.com/4.14.1/packages/yarnpkg-cli/bin/yarn.js".to_owned(),
            "https://repo.yarnpkg.com/4.18.0/packages/yarnpkg-cli/bin/yarn.js".to_owned(),
        ))
    );
    source
        .verify_artifact_integrity(requirement, b"current")
        .unwrap();
    assert!(
        source
            .verify_artifact_integrity(requirement, b"tampered")
            .is_err()
    );
    assert_eq!(
        source
            .artifact_integrity_replacement(requirement, "4.18.0", b"selected")
            .unwrap(),
        "v4.18.0+sha512.2ed4e329579f3f078c7c8cd654bb84d0cb378b9f59e5dacf0d58385cd7a2a0ccdbb9f87ca8e751d6425d8d966977781a8de74c55b499ddeb56b5811cc7610293"
    );
}

#[test]
fn sha224_requires_valid_pins_and_artifact_metadata() {
    let source = RuntimeSource::PackageManager("npm");
    assert!(
        source
            .integrity_artifact_urls("{}", "1.0.0+sha224.1234", "2.0.0")
            .is_err()
    );
    let requirement = format!("1.0.0+sha224.{}", "00".repeat(28));
    for body in [
        r#"{"versions":{"1.0.0":{"dist":{}},"2.0.0":{"dist":{"tarball":"https://registry.example/target.tgz"}}}}"#,
        r#"{"versions":{"1.0.0":{"dist":{"tarball":"file:///tmp/current.tgz"}},"2.0.0":{"dist":{"tarball":"https://registry.example/target.tgz"}}}}"#,
    ] {
        assert!(
            source
                .integrity_artifact_urls(body, &requirement, "2.0.0")
                .is_err()
        );
    }
}
