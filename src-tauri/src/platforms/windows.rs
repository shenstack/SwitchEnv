use crate::models::{EnvVar, ShellConfigInfo};
use crate::platforms::{PlatformError, PlatformInfo, PlatformService};
use async_trait::async_trait;
use std::process::Command;

pub struct WindowsPlatformService;

impl WindowsPlatformService {
    const SETX_VALUE_LIMIT: usize = 1024;
}

#[async_trait]
impl PlatformService for WindowsPlatformService {
    async fn get_all_variables(&self, is_system_scope: bool) -> Result<Vec<EnvVar>, PlatformError> {
        let scope = if is_system_scope { "HKLM" } else { "HKCU" };
        let reg_path = if is_system_scope {
            r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment"
        } else {
            r"Environment"
        };

        let output = Command::new("reg")
            .args(["query", &format!("HKEY_{}_{}", scope, reg_path)])
            .output()
            .map_err(|e| PlatformError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(PlatformError::RegistryError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut vars = Vec::new();
        let mut current_name = String::new();

        for line in stdout.lines().skip(2) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if !line.starts_with("    ") {
                current_name = line.to_string();
            } else if let Some(value) = line.strip_prefix("    ") {
                let value = value
                    .strip_prefix("REG_")
                    .and_then(|v| v.strip_prefix("SZ"))
                    .and_then(|v| v.strip_prefix("_EXPAND_SZ"))
                    .and_then(|v| v.strip_prefix("    "))
                    .unwrap_or(value.trim_start());

                let name = current_name.trim().to_string();
                if !name.is_empty() {
                    vars.push(EnvVar {
                        name,
                        value: value.to_string(),
                        is_system: is_system_scope,
                        is_readonly: false,
                    });
                }
            }
        }

        Ok(vars)
    }

    async fn set_variable(&self, name: &str, value: &str, is_system_scope: bool) -> Result<(), PlatformError> {
        if value.len() > Self::SETX_VALUE_LIMIT {
            let reg_path = if is_system_scope {
                r"HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment"
            } else {
                r"HKCU\Environment"
            };
            let output = Command::new("reg")
                .args(["add", reg_path, "/v", name, "/t", "REG_SZ", "/d", value, "/f"])
                .output()
                .map_err(|e| PlatformError::CommandFailed(e.to_string()))?;

            if !output.status.success() {
                return Err(PlatformError::RegistryError(
                    String::from_utf8_lossy(&output.stderr).to_string(),
                ));
            }
        } else {
            let output = Command::new("setx")
                .args([name, value])
                .output()
                .map_err(|e| PlatformError::CommandFailed(e.to_string()))?;

            if !output.status.success() {
                return Err(PlatformError::RegistryError(
                    String::from_utf8_lossy(&output.stderr).to_string(),
                ));
            }
        }

        self.refresh_environment().await?;
        Ok(())
    }

    async fn remove_variable(&self, name: &str, is_system_scope: bool) -> Result<(), PlatformError> {
        let reg_path = if is_system_scope {
            r"HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment"
        } else {
            r"HKCU\Environment"
        };

        let output = Command::new("reg")
            .args(["delete", reg_path, "/v", name, "/f"])
            .output()
            .map_err(|e| PlatformError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(PlatformError::RegistryError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        self.refresh_environment().await?;
        Ok(())
    }

    async fn get_variable(&self, name: &str) -> Result<Option<String>, PlatformError> {
        Ok(std::env::var(name).ok())
    }

    async fn can_modify_system(&self) -> Result<bool, PlatformError> {
        let output = Command::new("net")
            .args(["session"])
            .output()
            .map_err(|e| PlatformError::CommandFailed(e.to_string()))?;

        // Simple heuristic: if we can query sessions, we likely have admin rights
        Ok(output.status.success())
    }

    async fn refresh_environment(&self) -> Result<(), PlatformError> {
        let _ = Command::new("rundll32.exe")
            .args(["user32.dll,UpdatePerUserSystemParameters"])
            .spawn();

        // Also broadcast via PowerShell
        let _ = Command::new("powershell")
            .args([
                "-Command",
                "[System.Environment]::SetEnvironmentVariable('__EA_REFRESH__', '1', 'User'); Remove-Item Env:\\__EA_REFRESH__",
            ])
            .output();
        Ok(())
    }

    fn get_value_length_limit(&self) -> usize {
        Self::SETX_VALUE_LIMIT
    }

    async fn open_system_settings(&self) -> Result<(), PlatformError> {
        Command::new("rundll32")
            .args(["sysdm.cpl,EditEnvironmentVariables"])
            .spawn()
            .map_err(|e| PlatformError::CommandFailed(e.to_string()))?;
        Ok(())
    }

    async fn get_shell_config_info(&self) -> Result<ShellConfigInfo, PlatformError> {
        Ok(ShellConfigInfo {
            shell_path: "N/A (Windows)".to_string(),
            config_file: "N/A (Windows uses Registry)".to_string(),
            managed_vars: vec![],
        })
    }

    fn get_platform_info(&self) -> PlatformInfo {
        PlatformInfo {
            os: "windows".to_string(),
            arch: std::env::consts::ARCH.to_string(),
        }
    }
}
