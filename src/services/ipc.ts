import { invoke } from '@tauri-apps/api/core';
import type { EnvVar, EnvGroup, ActivationResult, HistoryRecord, Backup, AppSettings, ShellConfigInfo } from '../types';

// ===== 环境变量 =====
export async function getAllEnvVars(isSystem: boolean): Promise<EnvVar[]> {
  return invoke('get_all_env_vars', { isSystem });
}

export async function setEnvVar(name: string, value: string, isSystem: boolean): Promise<void> {
  return invoke('set_env_var', { name, value, isSystem });
}

export async function removeEnvVar(name: string, isSystem: boolean): Promise<void> {
  return invoke('remove_env_var', { name, isSystem });
}

export async function canModifySystem(): Promise<boolean> {
  return invoke('can_modify_system');
}

export async function refreshEnvironment(): Promise<void> {
  return invoke('refresh_environment');
}

export async function openSystemSettings(): Promise<void> {
  return invoke('open_system_settings');
}

export async function getShellConfigInfo(): Promise<ShellConfigInfo> {
  return invoke('get_shell_config_info');
}

// ===== 变量组 =====
export async function getAllGroups(): Promise<EnvGroup[]> {
  return invoke('get_all_groups');
}

export async function createGroup(name: string, description: string, variables: { name: string; value: string; isHidden: boolean }[], chainId: string | null): Promise<EnvGroup> {
  return invoke('create_group', { name, description, variables, chainId });
}

export async function updateGroup(id: string, name: string | null, description: string | null, variables: { name: string; value: string; isHidden: boolean }[] | null, chainId: string | null | undefined): Promise<EnvGroup> {
  return invoke('update_group', { id, name, description, variables, chainId });
}

export async function deleteGroup(id: string): Promise<void> {
  return invoke('delete_group', { id });
}

export async function activateGroup(id: string): Promise<ActivationResult> {
  return invoke('activate_group', { id });
}

export async function deactivateGroup(id: string): Promise<void> {
  return invoke('deactivate_group', { id });
}

// ===== 历史记录 =====
export async function getHistory(targetType?: string, limit?: number): Promise<HistoryRecord[]> {
  return invoke('get_history', { targetType, limit });
}

export async function restoreHistory(id: string): Promise<void> {
  return invoke('restore_history', { id });
}

export async function clearHistory(targetType?: string): Promise<void> {
  return invoke('clear_history', { targetType });
}

// ===== 备份 =====
export async function createBackup(name: string, scope: string): Promise<Backup> {
  return invoke('create_backup', { name, scope });
}

export async function getAllBackups(): Promise<Backup[]> {
  return invoke('get_all_backups');
}

export async function restoreBackup(id: string): Promise<void> {
  return invoke('restore_backup', { id });
}

export async function deleteBackup(id: string): Promise<void> {
  return invoke('delete_backup', { id });
}

export async function exportBackup(id: string, path: string): Promise<void> {
  return invoke('export_backup', { id, path });
}

export async function importBackup(path: string): Promise<Backup> {
  return invoke('import_backup', { path });
}

// ===== 设置 =====
export async function getAppSettings(): Promise<AppSettings> {
  return invoke('get_app_settings');
}

export async function setAppSettings(settings: AppSettings): Promise<void> {
  return invoke('set_app_settings', { settings });
}

// ===== 工具 =====
export async function copyToClipboard(text: string): Promise<void> {
  return invoke('copy_to_clipboard', { text });
}
