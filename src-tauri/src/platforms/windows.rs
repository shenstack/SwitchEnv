use crate::models::{EnvVar, ShellConfigInfo};
use crate::platforms::{PlatformError, PlatformInfo, PlatformService};
use async_trait::async_trait;
use std::process::Command;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE};
use winreg::RegKey;

/// 系统环境变量在注册表中的完整子路径
const SYSTEM_ENV_SUBKEY: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";
/// 用户环境变量在注册表中的子路径
const USER_ENV_SUBKEY: &str = "Environment";

/// setx 对单个值长度的限制（微软文档），超过时使用注册表 API 写入
const SETX_VALUE_LIMIT: usize = 1024;

pub struct WindowsPlatformService;

impl WindowsPlatformService {
    /// 打开指定作用域的注册表键（只读）
    fn open_key_read(is_system: bool) -> Result<RegKey, PlatformError> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let hkcu = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let (root, subkey) = if is_system {
            (hklm, SYSTEM_ENV_SUBKEY)
        } else {
            (hkcu, USER_ENV_SUBKEY)
        };
        root.open_subkey_with_flags(subkey, KEY_READ)
            .map_err(|e| PlatformError::RegistryError(e.to_string()))
    }

    /// 打开指定作用域的注册表键（可写）
    fn open_key_write(is_system: bool) -> Result<RegKey, PlatformError> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let hkcu = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let (root, subkey) = if is_system {
            (hklm, SYSTEM_ENV_SUBKEY)
        } else {
            (hkcu, USER_ENV_SUBKEY)
        };
        root.open_subkey_with_flags(subkey, KEY_SET_VALUE)
            .map_err(|e| PlatformError::RegistryError(e.to_string()))
    }

    /// 检测当前进程是否具有管理员权限（通过尝试以写入权限打开系统环境变量键判断）
    fn is_elevated() -> bool {
        RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey_with_flags(SYSTEM_ENV_SUBKEY, KEY_SET_VALUE)
            .is_ok()
    }

    /// 向系统发送 WM_SETTINGCHANGE 广播，让其它进程感知环境变量变更
    fn broadcast_setting_change() {
        // 通过 PowerShell 调用 Win32 API 广播设置变更
        let _ = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "$code = @'\nusing System;\nusing System.Runtime.InteropServices;\npublic static class Win32 {\n    [DllImport(\"user32.dll\", CharSet=CharSet.Auto)]\n    public static extern bool SendMessageTimeout(IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam, uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);\n}\n'@\nAdd-Type -TypeDefinition $code -Language CSharp;\n$HWND_BROADCAST = [IntPtr]0xffff;\n$WM_SETTINGCHANGE = 0x1a;\n$result = [UIntPtr]::Zero;\n[Win32]::SendMessageTimeout($HWND_BROADCAST, $WM_SETTINGCHANGE, [UIntPtr]::Zero, \"Environment\", 0x0002, 5000, [ref]$result) | Out-Null;",
            ])
            .output();
    }
}

#[async_trait]
impl PlatformService for WindowsPlatformService {
    /// 读取指定作用域下所有环境变量（直接走 winreg，避免 reg.exe 的 GBK 编码与解析问题）
    async fn get_all_variables(
        &self,
        is_system_scope: bool,
    ) -> Result<Vec<EnvVar>, PlatformError> {
        let key = Self::open_key_read(is_system_scope)?;
        let mut vars = Vec::new();

        for entry in key.enum_values() {
            let (name, _reg_value) = entry
                .map_err(|e| PlatformError::RegistryError(e.to_string()))?;

            // 只解析字符串类型（REG_SZ / REG_EXPAND_SZ），其它类型忽略
            match key.get_value::<String, _>(&name) {
                Ok(value) => {
                    vars.push(EnvVar {
                        name,
                        value,
                        is_system: is_system_scope,
                        is_readonly: false,
                    });
                }
                Err(_) => continue,
            }
        }

        Ok(vars)
    }

    /// 写入/更新环境变量。用户变量用 setx（Windows 标准持久化），长值/系统变量走 winreg
    async fn set_variable(
        &self,
        name: &str,
        value: &str,
        is_system_scope: bool,
    ) -> Result<(), PlatformError> {
        if is_system_scope && !Self::is_elevated() {
            return Err(PlatformError::PermissionDenied(
                "修改系统变量需要以管理员身份运行，请右键以管理员身份运行本程序".to_string(),
            ));
        }

        if value.len() > SETX_VALUE_LIMIT || is_system_scope {
            let key = Self::open_key_write(is_system_scope)?;
            key.set_value(name, &value)
                .map_err(|e| PlatformError::RegistryError(e.to_string()))?;
        } else {
            // 用户变量且长度在 setx 范围内
            let output = Command::new("setx")
                .args([name, value])
                .output()
                .map_err(|e| PlatformError::CommandFailed(e.to_string()))?;

            if !output.status.success() {
                return Err(PlatformError::RegistryError(format!(
                    "setx 写入失败: {}",
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
        }

        Self::broadcast_setting_change();
        Ok(())
    }

    /// 删除环境变量
    async fn remove_variable(
        &self,
        name: &str,
        is_system_scope: bool,
    ) -> Result<(), PlatformError> {
        if is_system_scope && !Self::is_elevated() {
            return Err(PlatformError::PermissionDenied(
                "删除系统变量需要以管理员身份运行，请右键以管理员身份运行本程序".to_string(),
            ));
        }

        let key = Self::open_key_write(is_system_scope)?;
        key.delete_value(name)
            .map_err(|e| PlatformError::RegistryError(e.to_string()))?;

        Self::broadcast_setting_change();
        Ok(())
    }

    /// 读取单个变量（从当前进程的环境块读取）
    async fn get_variable(&self, name: &str) -> Result<Option<String>, PlatformError> {
        Ok(std::env::var(name).ok())
    }

    /// 是否拥有修改系统变量的权限
    async fn can_modify_system(&self) -> Result<bool, PlatformError> {
        Ok(Self::is_elevated())
    }

    /// 刷新环境变量（广播 + 同步到当前进程）
    async fn refresh_environment(&self) -> Result<(), PlatformError> {
        Self::broadcast_setting_change();

        // 从注册表重新合并用户 + 系统变量到当前进程的环境块
        let user_vars = self.get_all_variables(false).await.unwrap_or_default();
        let system_vars = self.get_all_variables(true).await.unwrap_or_default();

        let mut merged: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for v in system_vars {
            merged.insert(v.name, v.value);
        }
        for v in user_vars {
            merged.insert(v.name, v.value);
        }

        for (k, v) in merged {
            std::env::set_var(&k, &v);
        }

        Ok(())
    }

    fn get_value_length_limit(&self) -> usize {
        SETX_VALUE_LIMIT
    }

    /// 打开系统环境变量设置面板
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
            config_file: "N/A (Windows 使用注册表)".to_string(),
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
