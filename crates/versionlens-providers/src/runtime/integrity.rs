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
mod tests;
