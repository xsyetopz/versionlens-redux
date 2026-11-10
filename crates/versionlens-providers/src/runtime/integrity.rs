use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::Value;
use sha2::{Digest, Sha224, Sha512};

use super::RuntimeSource;

impl RuntimeSource {
    pub fn verified_replacement(
        &self,
        body: &str,
        requirement: &str,
        selected: &str,
    ) -> Result<Option<String>, String> {
        let Some((current, prefix, algorithm, expected)) = integrity_pin(requirement)? else {
            return Ok(requirement.starts_with('v').then(|| format!("v{selected}")));
        };
        if matches!(self, Self::YarnModern) {
            return Err(
                "modern Yarn integrity verification requires hashing its published standalone binary"
                    .to_owned(),
            );
        }
        if !matches!(self, Self::PackageManager("npm" | "pnpm" | "yarn")) {
            return Err(
                "runtime source cannot verify the package-manager integrity pin".to_owned(),
            );
        }
        let metadata: Value = serde_json::from_str(body)
            .map_err(|error| format!("invalid package-manager metadata: {error}"))?;
        let digest = match algorithm {
            "1" => sha1_digest,
            "512" => sha512_digest,
            "224" => {
                return Err(
                    "package-manager SHA-224 verification requires the published release artifact"
                        .to_owned(),
                );
            }
            _ => {
                return Err(format!(
                    "package-manager SHA-{algorithm} pins are not supported"
                ));
            }
        };
        let current_digest = digest(&metadata, current)?;
        if !expected.eq_ignore_ascii_case(&current_digest) {
            return Err(
                "package-manager integrity pin does not match its published release".to_owned(),
            );
        }
        let selected_digest = digest(&metadata, selected)?;
        Ok(Some(format!(
            "{prefix}{selected}+sha{algorithm}.{selected_digest}"
        )))
    }

    pub fn integrity_artifact_urls(
        &self,
        body: &str,
        requirement: &str,
        selected: &str,
    ) -> Result<Option<(String, String)>, String> {
        let Some((current, _, algorithm, _)) = integrity_pin(requirement)? else {
            return Ok(None);
        };
        match self {
            Self::PackageManager("npm" | "pnpm" | "yarn") if algorithm == "224" => {
                let metadata: Value = serde_json::from_str(body)
                    .map_err(|error| format!("invalid package-manager metadata: {error}"))?;
                Ok(Some((
                    registry_artifact_url(&metadata, current)?,
                    registry_artifact_url(&metadata, selected)?,
                )))
            }
            Self::YarnModern if matches!(algorithm, "224" | "512") => Ok(Some((
                yarn_modern_artifact_url(current)?,
                yarn_modern_artifact_url(selected)?,
            ))),
            Self::YarnModern => Err(format!(
                "modern Yarn SHA-{algorithm} pins are not supported"
            )),
            Self::PackageManager("npm" | "pnpm" | "yarn") => Ok(None),
            _ if algorithm == "224" => {
                Err("runtime source cannot verify the package-manager integrity pin".to_owned())
            }
            _ => Ok(None),
        }
    }

    pub fn verify_artifact_integrity(
        &self,
        requirement: &str,
        artifact: &[u8],
    ) -> Result<(), String> {
        let Some((_, _, algorithm, expected)) = integrity_pin(requirement)? else {
            return Err("package-manager integrity pin is missing".to_owned());
        };
        let actual = artifact_digest(self, algorithm, artifact)?;
        if expected.eq_ignore_ascii_case(&actual) {
            Ok(())
        } else {
            Err("package-manager integrity pin does not match its published release".to_owned())
        }
    }

    pub fn artifact_integrity_replacement(
        &self,
        requirement: &str,
        selected: &str,
        artifact: &[u8],
    ) -> Result<String, String> {
        let Some((_, prefix, algorithm, _)) = integrity_pin(requirement)? else {
            return Err("package-manager integrity pin is missing".to_owned());
        };
        let digest = artifact_digest(self, algorithm, artifact)?;
        Ok(format!("{prefix}{selected}+sha{algorithm}.{digest}"))
    }
}

fn integrity_pin(requirement: &str) -> Result<Option<(&str, &str, &str, &str)>, String> {
    let Some((version, integrity)) = requirement.split_once("+sha") else {
        return Ok(None);
    };
    let (algorithm, expected) = integrity
        .split_once('.')
        .ok_or_else(|| "package-manager integrity pin has an invalid digest format".to_owned())?;
    let expected_length = match algorithm {
        "1" => 40,
        "224" => 56,
        "512" => 128,
        _ => {
            return Err(format!(
                "package-manager SHA-{algorithm} pins are not supported"
            ));
        }
    };
    if expected.len() != expected_length || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!(
            "package-manager SHA-{algorithm} pin has an invalid digest"
        ));
    }
    let prefix = if version.starts_with('v') { "v" } else { "" };
    Ok(Some((
        version.trim_start_matches('v'),
        prefix,
        algorithm,
        expected,
    )))
}

fn registry_artifact_url(metadata: &Value, version: &str) -> Result<String, String> {
    metadata["versions"][version]["dist"]["tarball"]
        .as_str()
        .filter(|url| url.starts_with("https://") || url.starts_with("http://"))
        .map(str::to_owned)
        .ok_or_else(|| {
            format!("package-manager release {version} has no valid published tarball URL")
        })
}

fn yarn_modern_artifact_url(version: &str) -> Result<String, String> {
    let version = semver::Version::parse(version)
        .map_err(|_| format!("modern Yarn release {version} is not a valid version"))?;
    if version.major < 2 {
        return Err(format!(
            "modern Yarn release {version} does not use the standalone binary source"
        ));
    }
    Ok(format!(
        "https://repo.yarnpkg.com/{version}/packages/yarnpkg-cli/bin/yarn.js"
    ))
}

fn artifact_digest(
    source: &RuntimeSource,
    algorithm: &str,
    artifact: &[u8],
) -> Result<String, String> {
    match (source, algorithm) {
        (_, "224") => Ok(hex_digest(&Sha224::digest(artifact))),
        (RuntimeSource::YarnModern, "512") => Ok(hex_digest(&Sha512::digest(artifact))),
        _ => Err(format!(
            "package-manager SHA-{algorithm} does not require artifact verification"
        )),
    }
}

fn sha1_digest(metadata: &Value, version: &str) -> Result<String, String> {
    let digest = metadata["versions"][version]["dist"]["shasum"]
        .as_str()
        .filter(|digest| digest.len() == 40 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or_else(|| format!("package-manager release {version} has no valid SHA-1 digest"))?;
    Ok(digest.to_ascii_lowercase())
}

fn sha512_digest(metadata: &Value, version: &str) -> Result<String, String> {
    let encoded = metadata["versions"][version]["dist"]["integrity"]
        .as_str()
        .and_then(|integrity| {
            integrity
                .split_whitespace()
                .find_map(|part| part.strip_prefix("sha512-"))
        })
        .ok_or_else(|| format!("package-manager release {version} has no SHA-512 integrity"))?;
    let digest = STANDARD
        .decode(encoded)
        .map_err(|_| format!("package-manager release {version} has invalid SHA-512 integrity"))?;
    if digest.len() != 64 {
        return Err(format!(
            "package-manager release {version} has invalid SHA-512 integrity"
        ));
    }
    Ok(hex_digest(&digest))
}

fn hex_digest(digest: &[u8]) -> String {
    let mut hex = String::with_capacity(digest.len() * 2);
    for &byte in digest {
        hex.push(char::from(b"0123456789abcdef"[usize::from(byte >> 4)]));
        hex.push(char::from(b"0123456789abcdef"[usize::from(byte & 15)]));
    }
    hex
}

#[cfg(test)]
mod tests {
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
}
