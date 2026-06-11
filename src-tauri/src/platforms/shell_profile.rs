use crate::models::ShellConfigInfo;
use std::path::PathBuf;

const MARKER_START: &str = "# === Switch Env START ===";
const MARKER_END: &str = "# === Switch Env END ===";

#[derive(Debug, Clone)]
enum ShellType {
    Bash,
    Zsh,
    Fish,
}

pub struct ShellProfileManager {
    config_path: PathBuf,
    shell_type: ShellType,
}

impl ShellProfileManager {
    pub fn new() -> Self {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let shell_name = std::path::Path::new(&shell)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("bash");

        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));

        let (config_path, shell_type) = match shell_name {
            "zsh" => (home.join(".zshrc"), ShellType::Zsh),
            "fish" => (
                home.join(".config/fish/config.fish"),
                ShellType::Fish,
            ),
            _ => (home.join(".bashrc"), ShellType::Bash),
        };

        Self { config_path, shell_type }
    }

    pub fn read_managed_vars(&self) -> Result<Vec<(String, String)>, String> {
        let content = std::fs::read_to_string(&self.config_path).unwrap_or_default();
        let mut vars = Vec::new();
        let mut in_marker = false;

        for line in content.lines() {
            if line.trim() == MARKER_START {
                in_marker = true;
                continue;
            }
            if line.trim() == MARKER_END {
                in_marker = false;
                continue;
            }
            if in_marker {
                if let Some((name, value)) = self.parse_export_line(line) {
                    vars.push((name, value));
                }
            }
        }

        Ok(vars)
    }

    pub fn set_var(&self, name: &str, value: &str) -> Result<(), String> {
        let content = std::fs::read_to_string(&self.config_path).unwrap_or_default();
        let export_line = match self.shell_type {
            ShellType::Fish => format!("set -gx {} \"{}\"", name, value),
            _ => format!("export {}=\"{}\"", name, value),
        };

        let new_content = if content.contains(MARKER_START) {
            let mut result = String::new();
            let mut in_marker = false;
            let mut replaced = false;

            for line in content.lines() {
                if line.trim() == MARKER_START {
                    in_marker = true;
                    result.push_str(line);
                    result.push('\n');
                    continue;
                }
                if line.trim() == MARKER_END {
                    if !replaced {
                        result.push_str(&export_line);
                        result.push('\n');
                    }
                    in_marker = false;
                    result.push_str(line);
                    result.push('\n');
                    continue;
                }
                if in_marker && line.contains(&format!("{}=", name)) {
                    result.push_str(&export_line);
                    result.push('\n');
                    replaced = true;
                    continue;
                }
                result.push_str(line);
                result.push('\n');
            }
            if !replaced && !result.contains(&export_line) {
                // Insert before MARKER_END
                let idx = result.rfind(MARKER_END).unwrap_or(result.len());
                let mut new = String::new();
                new.push_str(&result[..idx]);
                new.push_str(&export_line);
                new.push('\n');
                new.push_str(&result[idx..]);
                new
            } else {
                result
            }
        } else {
            format!(
                "{}\n{}\n{}\n{}\n",
                content, MARKER_START, export_line, MARKER_END
            )
        };

        std::fs::write(&self.config_path, new_content)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;
        Ok(())
    }

    pub fn remove_var(&self, name: &str) -> Result<(), String> {
        let content = std::fs::read_to_string(&self.config_path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;

        let mut result = String::new();
        let mut in_marker = false;

        for line in content.lines() {
            if line.trim() == MARKER_START {
                in_marker = true;
            }
            if line.trim() == MARKER_END {
                in_marker = false;
            }
            if in_marker && line.contains(&format!("{}=", name)) {
                continue;
            }
            result.push_str(line);
            result.push('\n');
        }

        std::fs::write(&self.config_path, result)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;
        Ok(())
    }

    pub fn get_config_info(&self) -> ShellConfigInfo {
        let vars = self.read_managed_vars().unwrap_or_default();
        ShellConfigInfo {
            shell_path: std::env::var("SHELL").unwrap_or_default(),
            config_file: self.config_path.to_string_lossy().to_string(),
            managed_vars: vars.into_iter().map(|(n, _)| n).collect(),
        }
    }

    fn parse_export_line(&self, line: &str) -> Option<(String, String)> {
        match self.shell_type {
            ShellType::Fish => {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 && parts[0] == "set" && parts[1] == "-gx" {
                    let name = parts[2].to_string();
                    let value = parts[3..].join(" ").trim_matches('"').to_string();
                    return Some((name, value));
                }
                None
            }
            _ => {
                if let Some(stripped) = line.strip_prefix("export ") {
                    if let Some((name, value)) = stripped.split_once('=') {
                        let value = value.trim_matches('"').trim_matches('\'').to_string();
                        return Some((name.to_string(), value));
                    }
                }
                None
            }
        }
    }
}
