use crate::models::{EnvVar, PlatformInfo, ShellConfigInfo};
use async_trait::async_trait;
use std::process::Command;

use super::shell_profile::ShellProfileManager;
use super::{PlatformError, PlatformService};

const ETC_ENVIRONMENT: &str = "/etc/environment";

pub struct LinuxPlatformService {
    shell_profile: ShellProfileManager,
}

impl LinuxPlatformService {
    pub fn new() -> Self {
        Self {
            shell_profile: ShellProfileManager::new(),
        }
    }
}

#[async_trait]
impl PlatformService for LinuxPlatformService {
    async fn get_all_variables(&self, is_system_scope: bool) -> Result<Vec<EnvVar>, PlatformError> {
        if is_system_scope {
            let content = std::fs::read_to_string(ETC_ENVIRONMENT)
                .unwrap_or_default();
            let mut vars = Vec::new();

            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((name, value)) = line.split_once('=') {
                    let value = value.trim_matches('"').trim_matches('\'');
                    vars.push(EnvVar {
                        name: name.trim().to_string(),
                        value: value.to_string(),
                        is_system: true,
                        is_readonly: false,
                    });
                }
            }
            Ok(vars)
        } else {
            let mut vars: Vec<EnvVar> = std::env::vars()
                .map(|(name, value)| EnvVar {
                    name,
                    value,
                    is_system: false,
                    is_readonly: false,
                })
                .collect();

            if let Ok(profile_vars) = self.shell_profile.read_managed_vars() {
                for (name, value) in profile_vars {
                    if !vars.iter().any(|v| v.name == name) {
                        vars.push(EnvVar {
                            name,
                            value,
                            is_system: false,
                            is_readonly: false,
                        });
                    }
                }
            }
            Ok(vars)
        }
    }

    async fn set_variable(&self, name: &str, value: &str, is_system_scope: bool) -> Result<(), PlatformError> {
        if is_system_scope {
            let script = format!(
                r#"#!/bin/bash
if grep -q '^{}=' {}; then
    sed -i 's|^{}=.*|{}="{}"|' {}
else
    echo '{}="{}"' >> {}
fi
"#,
                name, ETC_ENVIRONMENT, name, name, value, ETC_ENVIRONMENT, name, value, ETC_ENVIRONMENT
            );

            let temp_file = std::env::temp_dir().join(format!("SwitchEnv-{}.sh", uuid::Uuid::new_v4()));
            std::fs::write(&temp_file, &script)
                .map_err(|e| PlatformError::PermissionDenied(e.to_string()))?;

            let output = Command::new("pkexec")
                .arg("/bin/bash")
                .arg(&temp_file)
                .output()
                .map_err(|e| PlatformError::PermissionDenied(e.to_string()))?;

            let _ = std::fs::remove_file(&temp_file);

            if !output.status.success() {
                return Err(PlatformError::PermissionDenied(
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
            let script = format!(
                r#"#!/bin/bash
sed -i '/^{}=/d' {}
"#,
                name, ETC_ENVIRONMENT
            );

            let temp_file = std::env::temp_dir().join(format!("SwitchEnv-{}.sh", uuid::Uuid::new_v4()));
            std::fs::write(&temp_file, &script)
                .map_err(|e| PlatformError::PermissionDenied(e.to_string()))?;

            let output = Command::new("pkexec")
                .arg("/bin/bash")
                .arg(&temp_file)
                .output()
                .map_err(|e| PlatformError::PermissionDenied(e.to_string()))?;

            let _ = std::fs::remove_file(&temp_file);

            if !output.status.success() {
                return Err(PlatformError::PermissionDenied(
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
        Ok(())
    }

    fn get_value_length_limit(&self) -> usize {
        usize::MAX
    }

    async fn open_system_settings(&self) -> Result<(), PlatformError> {
        Command::new("xdg-open")
            .arg("environment://")
            .spawn()
            .map_err(|e| PlatformError::CommandFailed(e.to_string()))?;
        Ok(())
    }

    async fn get_shell_config_info(&self) -> Result<ShellConfigInfo, PlatformError> {
        Ok(self.shell_profile.get_config_info())
    }

    fn get_platform_info(&self) -> PlatformInfo {
        PlatformInfo {
            os: "linux".to_string(),
            arch: std::env::consts::ARCH.to_string(),
        }
    }
}
