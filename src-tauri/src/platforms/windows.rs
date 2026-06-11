use crate::models::{EnvVar, ShellConfigInfo};
use crate::platforms::{PlatformError, PlatformInfo, PlatformService};
use async_trait::async_trait;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::process::Command;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE};
use winreg::RegKey;

/// 系统环境变量在注册表中的完整子路径
const SYSTEM_ENV_SUBKEY: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";
/// 用户环境变量在注册表中的子路径
const USER_ENV_SUBKEY: &str = "Environment";

/// 广播到所有顶层窗口（SendMessageTimeout 的目标句柄）
const HWND_BROADCAST: usize = 0xFFFF;
/// 设置变更消息（WM_SETTINGCHANGE / WM_WININICHANGE）
const WM_SETTINGCHANGE: u32 = 0x001A;
/// 如果接收消息的进程挂起，不要一直等待
const SMTO_ABORTIFHUNG: u32 = 0x0002;
/// 单次广播的超时时间（毫秒），5s 足以覆盖绝大多数场景
const BROADCAST_TIMEOUT_MS: u32 = 5000;

/// 通过 Win32 FFI 直接调用 SendMessageTimeoutW，避免每次 spawn PowerShell + C# Add-Type
/// 编译消耗 ~300-1500ms。
extern "system" {
    /// 发送消息到顶层窗口，并设置超时，防止因接收方挂起而阻塞。
    fn SendMessageTimeoutW(
        hWnd: usize,
        Msg: u32,
        wParam: usize,
        lParam: *const u16,
        fuFlags: u32,
        uTimeout: u32,
        lpdwResult: *mut usize,
    ) -> usize;
}

/// 将 UTF-8 Rust 字符串转换为以 NUL 结尾的 UTF-16 Vec<u16>，供 Win32 W 系列 API 使用
fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

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

    /// 向系统发送 WM_SETTINGCHANGE 广播，让其它进程感知环境变量变更。
    /// 使用 Win32 FFI 直接调用 SendMessageTimeoutW，取代原来的 PowerShell 方案，
    /// 将单次广播从 300-1500ms 降至 < 5ms。
    fn broadcast_setting_change() {
        let env_wide = to_wide("Environment");
        let mut result: usize = 0;
        // 忽略返回值，广播失败不应影响整体写入流程
        unsafe {
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                env_wide.as_ptr(),
                SMTO_ABORTIFHUNG,
                BROADCAST_TIMEOUT_MS,
                &mut result,
            );
        }
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

    /// 写入/更新环境变量。统一使用 winreg 直接写入注册表，
    /// 避免每变量 spawn 一次 setx.exe 进程的开销；
    /// 广播由调用方在整组写入完成后通过 refresh_environment 统一执行，
    /// 避免每变量广播一次导致的 N 倍放大延迟。
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

        // 统一走注册表 API（无论用户/系统变量、无论值长短），
        // 不再区分 setx 与 winreg，避免 spawn 外部进程。
        let key = Self::open_key_write(is_system_scope)?;
        key.set_value(name, &value)
            .map_err(|e| PlatformError::RegistryError(e.to_string()))?;

        Ok(())
    }

    /// 删除环境变量。广播由调用方整组完成后统一执行。
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

    /// 刷新环境变量：
    /// 仅执行一次 WM_SETTINGCHANGE 广播，让系统/新进程感知注册表变更。
    /// 不再全量重写当前进程的环境块 —— 当前应用无需从自身进程环境块读取业务变量，
    /// 全量 set_var 既耗时又不必要。
    async fn refresh_environment(&self) -> Result<(), PlatformError> {
        Self::broadcast_setting_change();
        Ok(())
    }

    fn get_value_length_limit(&self) -> usize {
        // 使用 winreg 直接写入，不再受 setx 1024 字符限制，
        // 这里返回一个大值保留兼容上层逻辑。
        32767
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
