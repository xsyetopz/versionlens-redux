use std::{fs, io::Cursor, path::Path};

use sha2::{Digest, Sha256};
use zed::settings::LspSettings;
use zed_extension_api::serde_json::{Value, json};
use zed_extension_api::{
    self as zed, Architecture, DownloadedFileType, LanguageServerId,
    LanguageServerInstallationStatus, Os, Result, Worktree,
};

include!(concat!(env!("OUT_DIR"), "/targets.rs"));

const REPOSITORY: &str = "xsyetopz/versionlens-redux";
const INSTALL_PREFIX: &str = "versionlens-lsp-";
const SETTINGS_SECTION: &str = "versionlens";

/// Rust target triple for a Zed platform, restricted to `targets.json` entries.
fn release_triple(os: Os, arch: Architecture) -> Result<&'static str> {
    let (arch_prefix, suffix) = match (os, arch) {
        (Os::Linux, Architecture::X8664) => ("x86_64", "unknown-linux-gnu"),
        (Os::Linux, Architecture::Aarch64) => ("aarch64", "unknown-linux-gnu"),
        (Os::Mac, Architecture::X8664) => ("x86_64", "apple-darwin"),
        (Os::Mac, Architecture::Aarch64) => ("aarch64", "apple-darwin"),
        (Os::Windows, Architecture::X8664) => ("x86_64", "pc-windows-msvc"),
        (Os::Windows, Architecture::Aarch64) => ("aarch64", "pc-windows-msvc"),
        platform => return Err(format!("unsupported Zed platform: {platform:?}")),
    };
    let triple = format!("{arch_prefix}-{suffix}");
    RELEASE_TARGETS
        .iter()
        .find(|target| **target == triple)
        .copied()
        .ok_or_else(|| format!("{triple} is not listed in targets.json"))
}

fn archive_name(version: &str, triple: &str) -> String {
    let extension = if triple.contains("windows") {
        "zip"
    } else {
        "tar.gz"
    };
    format!("{INSTALL_PREFIX}{version}-{triple}.{extension}")
}

/// Expected hex digest for `archive` from `sha256sum`-style `SHA256SUMS` text.
fn expected_checksum(sums: &str, archive: &str) -> Result<String> {
    sums.lines()
        .filter_map(|line| {
            let (hash, name) = line.trim().split_once(char::is_whitespace)?;
            let name = name.trim_start().trim_start_matches('*');
            (name == archive).then(|| hash.to_ascii_lowercase())
        })
        .next()
        .ok_or_else(|| format!("SHA256SUMS has no entry for {archive}"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn verify_checksum(archive: &str, bytes: &[u8], sums: &str) -> Result<()> {
    let expected = expected_checksum(sums, archive)?;
    let actual = sha256_hex(bytes);
    if expected == actual {
        Ok(())
    } else {
        Err(format!(
            "{archive} checksum mismatch: expected {expected}, got {actual}"
        ))
    }
}

/// Entries named `versionlens-lsp-*` other than `keep`.
fn stale_entries(entries: &[String], keep: &str) -> Vec<String> {
    entries
        .iter()
        .filter(|name| name.starts_with(INSTALL_PREFIX) && name.as_str() != keep)
        .cloned()
        .collect()
}

fn remove_stale(keep: &str) {
    let Ok(read) = fs::read_dir(".") else { return };
    let entries: Vec<String> = read
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    for name in stale_entries(&entries, keep) {
        let path = Path::new(&name);
        let _ = if path.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        };
    }
}

fn extract_binary(archive: &str, bytes: &[u8], binary_name: &str) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut out = Vec::new();
    if archive.ends_with(".zip") {
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
        let mut file = zip.by_name(binary_name).map_err(|e| e.to_string())?;
        file.read_to_end(&mut out).map_err(|e| e.to_string())?;
    } else {
        let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(bytes));
        for entry in tar.entries().map_err(|e| e.to_string())? {
            let mut entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path().map_err(|e| e.to_string())?;
            if path.file_name().is_some_and(|name| name == binary_name) {
                entry.read_to_end(&mut out).map_err(|e| e.to_string())?;
                return Ok(out);
            }
        }
        return Err(format!("{archive} does not contain {binary_name}"));
    }
    Ok(out)
}

/// Initialization options: the settings, and a request for the server's URL authentication code
/// actions, since Zed lets an extension add neither commands nor a text prompt of its own.
fn initialization_options(settings: Option<Value>) -> Value {
    let mut options = workspace_configuration(settings);
    options["authenticationActions"] = json!(true);
    options
}

/// Workspace configuration answering the server's `versionlens` section pull.
fn workspace_configuration(settings: Option<Value>) -> Value {
    json!({ SETTINGS_SECTION: settings.unwrap_or_else(|| json!({})) })
}

struct VersionLensExtension;

impl VersionLensExtension {
    fn server_binary() -> &'static str {
        if zed::current_platform().0 == Os::Windows {
            "versionlens-lsp.exe"
        } else {
            "versionlens-lsp"
        }
    }

    fn release_binary(language_server_id: &LanguageServerId) -> Result<String> {
        let version = env!("CARGO_PKG_VERSION");
        let (os, arch) = zed::current_platform();
        let triple = release_triple(os, arch)?;
        let install_directory = format!("{INSTALL_PREFIX}{version}-{triple}");
        let binary = format!("{install_directory}/{}", Self::server_binary());
        if fs::metadata(&binary).is_ok_and(|metadata| metadata.is_file()) {
            return Ok(binary);
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::Downloading,
        );
        let tag = format!("v{version}");
        let release = zed::github_release_by_tag_name(REPOSITORY, &tag)?;
        let archive = archive_name(version, triple);
        let asset_url = |name: &str| {
            release
                .assets
                .iter()
                .find(|asset| asset.name == name)
                .map(|asset| asset.download_url.clone())
                .ok_or_else(|| format!("release {tag} does not contain {name}"))
        };
        let archive_url = asset_url(&archive)?;
        let sums_url = asset_url("SHA256SUMS")?;

        // Download uncompressed so the bytes can be hashed before anything runs,
        // then extract in the extension. Scratch files are always removed.
        let archive_path = format!("{install_directory}.download");
        let sums_path = format!("{install_directory}.SHA256SUMS");
        let result = (|| {
            zed::download_file(
                &archive_url,
                &archive_path,
                DownloadedFileType::Uncompressed,
            )?;
            zed::download_file(&sums_url, &sums_path, DownloadedFileType::Uncompressed)?;
            let bytes = fs::read(&archive_path).map_err(|e| e.to_string())?;
            let sums = fs::read_to_string(&sums_path).map_err(|e| e.to_string())?;
            verify_checksum(&archive, &bytes, &sums)?;
            let executable = extract_binary(&archive, &bytes, Self::server_binary())?;
            fs::create_dir_all(&install_directory).map_err(|e| e.to_string())?;
            let partial = format!("{binary}.part");
            fs::write(&partial, executable).map_err(|e| e.to_string())?;
            if os != Os::Windows {
                zed::make_file_executable(&partial)?;
            }
            fs::rename(&partial, &binary).map_err(|e| e.to_string())
        })();
        let _ = fs::remove_file(&archive_path);
        let _ = fs::remove_file(&sums_path);
        if let Err(error) = result {
            let _ = fs::remove_dir_all(&install_directory);
            return Err(error);
        }
        remove_stale(&install_directory);
        Ok(binary)
    }

    /// `lsp.versionlens.binary.path`, then `VERSIONLENS_LSP`, then `PATH`, then the release.
    fn server_path(
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
        configured: Option<String>,
    ) -> Result<String> {
        if let Some(path) = configured {
            return Ok(path);
        }
        let env = worktree.shell_env();
        if let Some((_, path)) = env
            .iter()
            .find(|(key, value)| key == "VERSIONLENS_LSP" && !value.is_empty())
        {
            return Ok(path.clone());
        }
        if let Some(path) = worktree.which(Self::server_binary()) {
            return Ok(path);
        }
        Self::release_binary(language_server_id)
    }
}

impl zed::Extension for VersionLensExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<zed::Command> {
        let binary = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?.binary;
        let (path, args) = binary.map_or((None, None), |binary| (binary.path, binary.arguments));
        Ok(zed::Command {
            command: Self::server_path(language_server_id, worktree, path)?,
            args: args.unwrap_or_else(|| vec!["--stdio".to_owned()]),
            env: worktree.shell_env(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<Value>> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?.settings;
        Ok(Some(initialization_options(settings)))
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<Value>> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?.settings;
        Ok(Some(workspace_configuration(settings)))
    }
}

zed::register_extension!(VersionLensExtension);

#[cfg(test)]
mod tests;
