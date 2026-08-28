use zed_extension_api as zed;
use zed_extension_api::settings::LspSettings;

struct SkillExtension {
    cached_binary_path: Option<String>,
}

impl SkillExtension {
    fn language_server_binary_path(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String, String> {
        // Check if user has configured a custom binary path
        if let Ok(lsp_settings) = LspSettings::for_worktree("skill-lsp", worktree) {
            if let Some(binary) = lsp_settings.binary {
                if let Some(path) = binary.path {
                    return Ok(path);
                }
            }
        }

        // Check if we have a cached path
        if let Some(path) = &self.cached_binary_path {
            if std::path::Path::new(path).exists() {
                return Ok(path.clone());
            }
        }

        // Try to find skill-lsp in PATH
        let output = std::process::Command::new("which")
            .arg("skill-lsp")
            .output()
            .map_err(|e| format!("Failed to find skill-lsp: {}", e))?;

        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            self.cached_binary_path = Some(path.clone());
            return Ok(path);
        }

        // Check common installation locations
        let common_paths = [
            "/usr/local/bin/skill-lsp",
            "/usr/bin/skill-lsp",
            &format!("{}/.local/bin/skill-lsp", std::env::var("HOME").unwrap_or_default()),
        ];

        for path in common_paths.iter() {
            if std::path::Path::new(path).exists() {
                self.cached_binary_path = Some(path.to_string());
                return Ok(path.to_string());
            }
        }

        Err(
            "SKILL LSP server not found. Please install skill-lsp or configure the path in settings."
                .to_string(),
        )
    }
}

impl zed::Extension for SkillExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command, String> {
        let path = self.language_server_binary_path(language_server_id, worktree)?;
        Ok(zed::Command {
            command: path,
            args: vec!["--stdio".to_string()],
            env: Default::default(),
        })
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>, String> {
        let settings = LspSettings::for_worktree("skill-lsp", worktree)?;
        Ok(settings.settings)
    }
}

zed::register_extension!(SkillExtension);
