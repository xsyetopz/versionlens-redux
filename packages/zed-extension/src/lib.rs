use zed::settings::LspSettings;
use zed_extension_api::{
    self as zed, Architecture, DownloadedFileType, LanguageServerId,
    LanguageServerInstallationStatus, Os, Result, Worktree,
};

struct VersionLensExtension;

impl VersionLensExtension {
    fn server_binary() -> &'static str {
        let (os, _) = zed::current_platform();
        if os == zed::Os::Windows {
            "versionlens-lsp.exe"
        } else {
            "versionlens-lsp"
        }
    }

    fn release_target() -> Result<&'static str> {
        match zed::current_platform() {
            (Os::Linux, Architecture::X86) => Ok("linux-x64"),
            (Os::Linux, Architecture::Aarch64) => Ok("linux-arm64"),
            (Os::Mac, Architecture::X86) => Ok("darwin-x64"),
            (Os::Mac, Architecture::Aarch64) => Ok("darwin-arm64"),
            (Os::Windows, Architecture::X86) => Ok("win32-x64"),
            (Os::Windows, Architecture::Aarch64) => Ok("win32-arm64"),
            platform => Err(format!("unsupported Zed platform: {platform:?}")),
        }
    }

    fn release_binary(language_server_id: &LanguageServerId) -> Result<String> {
        let version = env!("CARGO_PKG_VERSION");
        let target = Self::release_target()?;
        let install_directory = format!("versionlens-lsp-{version}-{target}");
        let binary = format!("{install_directory}/bin/{}", Self::server_binary());
        if std::fs::metadata(&binary).is_ok_and(|metadata| metadata.is_file()) {
            return Ok(binary);
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::Downloading,
        );
        let tag = format!("v{version}");
        let release = zed::github_release_by_tag_name("xsyetopz/versionlens-redux", &tag)?;
        let asset_name = format!("versionlens-redux-zed-extension-{target}.tar.gz");
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("release {tag} does not contain {asset_name}"))?;
        zed::download_file(
            &asset.download_url,
            &install_directory,
            DownloadedFileType::GzipTar,
        )?;
        if zed::current_platform().0 != Os::Windows {
            zed::make_file_executable(&binary)?;
        }
        Ok(binary)
    }

    fn server_path(language_server_id: &LanguageServerId, worktree: &Worktree) -> Result<String> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;
        if let Some(binary) = settings.binary.and_then(|binary| binary.path) {
            return Ok(binary);
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
        Ok(zed::Command {
            command: Self::server_path(language_server_id, worktree)?,
            args: Vec::new(),
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(VersionLensExtension);
