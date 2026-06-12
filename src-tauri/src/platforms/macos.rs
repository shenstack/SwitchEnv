use crate::models::{EnvVar, ShellConfigInfo};
use crate::platforms::{PlatformError, PlatformInfo, PlatformService};
use crate::platforms::shell_profile::ShellProfileManager;
use async_trait::async_trait;
use std::process::Command;

pub struct MacOSPlatformService {
    shell_profile: ShellProfileManager,
}

impl MacOSPlatformService {
    pub fn new() -> Self {
        Self {
            shell_profile: ShellProfileManager::new(),
        }
    }
}

#[async_trait]
impl PlatformService for MacOSPlatformService {
    async fn get_all_variables(&self, is_system_scope: bool) -> Result<Vec<EnvVar>, PlatformError> {
        let mut vars: Vec<EnvVar> = std::env::vars()
            .map(|(name, value)| EnvVar {
                name,
                value,
                is_system: is_system_scope,
                is_readonly: false,
                source: if is_system_scope {
                    Some("launchctl 会话环境".to_string())
                } else {
                    None
                },
            })
            .collect();

        if !is_system_scope {
            if let Ok(profile_vars) = self.shell_profile.read_managed_vars_with_source() {
                for (name, value, source_path) in profile_vars {
                    if !vars.iter().any(|v| v.name == name) {
                        vars.push(EnvVar {
                            name,
                            value,
                            is_system: false,
                            is_readonly: false,
                            source: Some(source_path),
                        });
                    }
                }
            }
        }

        Ok(vars)
    }

    async fn set_variable(&self, name: &str, value: &str, is_system_scope: bool) -> Result<(), PlatformError> {
        if is_system_scope {
            let output = Command::new("launchctl")
                .args(["setenv", name, value])
                .output()
                .map_err(|e| PlatformError::CommandFailed(e.to_string()))?;

            if !output.status.success() {
                return Err(PlatformError::ShellConfigError(
                    String::from_utf8_lossy(&output.stderr).to_string(),
                ));
            }
        } else {
            self.shell_profile.set_var(name, value)?;
        }
        Ok(())
    }

    async fn remove_variable(&self, name: &str, is_system_scope: bool) -> Result<(), PlatformError> {
        if is_system_scope {
            let output = Command::new("launchctl")
                .args(["unsetenv", name])
                .output()
                .map_err(|e| PlatformError::CommandFailed(e.to_string()))?;

            if !output.status.success() {
                return Err(PlatformError::ShellConfigError(
                    String::from_utf8_lossy(&output.stderr).to_string(),
                ));
            }
        } else {
            self.shell_profile.remove_var(name)?;
        }
        Ok(())
    }

    async fn get_variable(&self, name: &str) -> Result<Option<String>, PlatformError> {
        Ok(std::env::var(name).ok())
    }

    async fn can_modify_system(&self) -> Result<bool, PlatformError> {
        let output = Command::new("id")
            .arg("-u")
            .output()
            .map_err(|e| PlatformError::CommandFailed(e.to_string()))?;

        let uid: u32 = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .unwrap_or(1000);
        Ok(uid == 0)
    }

    async fn refresh_environment(&self) -> Result<(), PlatformError> {
        // macOS launchctl changes take effect immediately
        Ok(())
    }

    fn get_value_length_limit(&self) -> usize {
        usize::MAX
    }

    async fn open_system_settings(&self) -> Result<(), PlatformError> {
        Command::new("open")
            .args(["x-apple.systempreferences:com.apple.preference.security"])
            .spawn()
            .map_err(|e| PlatformError::CommandFailed(e.to_string()))?;
        Ok(())
    }

    async fn get_shell_config_info(&self) -> Result<ShellConfigInfo, PlatformError> {
        Ok(self.shell_profile.get_config_info())
    }

    fn get_platform_info(&self) -> PlatformInfo {
        PlatformInfo {
            os: "macos".to_string(),
            arch: std::env::consts::ARCH.to_string(),
        }
    }
}
