use std::fs;
use zed::LanguageServerId;
use zed_extension_api::{self as zed, Result};

struct EarthfileExtension {
    cached_binary_path: Option<String>,
}

impl EarthfileExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        if let Some(path) = worktree.which("earthlyls") {
            return Ok(path);
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = zed::latest_github_release(
            "glehmann/earthlyls",
            zed::GithubReleaseOptions {
                require_assets: false,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let archive_basename = format!(
            "earthlyls-{version}-{os}-{arch}",
            version = &release.version,
            os = match platform {
                zed::Os::Mac => "macos",
                zed::Os::Linux => "linux",
                zed::Os::Windows => "windows",
            },
            arch = match arch {
                zed::Architecture::Aarch64 => "arm64",
                zed::Architecture::X86 => "i686",
                zed::Architecture::X8664 => "amd64",
            },
        );
        let download_url = format!(
            "https://github.com/glehmann/earthlyls/releases/download/{version}/{archive_basename}.tar.gz",
            version = &release.version,
        );

        let version_dir = format!("earthlyls-{}", release.version);
        let mut binary_path = format!("{version_dir}/{archive_basename}/earthlyls");
        if platform == zed::Os::Windows {
            binary_path.push_str(".exe");
        }

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &download_url,
                &version_dir,
                zed::DownloadedFileType::GzipTar,
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(&entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for EarthfileExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: self.language_server_binary_path(language_server_id, worktree)?,
            args: vec![],
            env: Default::default(),
        })
    }
}

zed::register_extension!(EarthfileExtension);
