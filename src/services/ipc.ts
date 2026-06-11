import { invoke } from '@tauri-apps/api/core';
import type {
  EnvVar,
  EnvGroup,
  ActivationResult,
  HistoryRecord,
  Backup,
  AppSettings,
  ShellConfigInfo,
  Template,
  EnvVarConflict,
} from '../types';

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

/**
 * 导出用户级或系统级环境变量为 key=value 格式文本。
 * isSystem=false 导出用户级变量；isSystem=true 导出系统级变量。
 * savePath 提供时写入文件并返回绝对路径；
 * savePath 缺省时直接返回文本内容。
 */
export async function exportEnvVars(
  isSystem: boolean,
  savePath?: string,
): Promise<string> {
  return invoke('export_env_vars', { isSystem, savePath });
}

// ===== 变量组 =====
export async function getAllGroups(): Promise<EnvGroup[]> {
  return invoke('get_all_groups');
}

export async function createGroup(
  name: string,
  description: string,
  variables: { name: string; value: string; isHidden: boolean }[],
): Promise<EnvGroup> {
  return invoke('create_group', { name, description, variables });
}

export async function updateGroup(
  id: string,
  name: string | null,
  description: string | null,
  variables: { name: string; value: string; isHidden: boolean }[] | null,
): Promise<EnvGroup> {
  return invoke('update_group', { id, name, description, variables });
}

export async function deleteGroup(id: string): Promise<void> {
  return invoke('delete_group', { id });
}

/**
 * 激活变量组。
 * force=false（默认）：仅检测冲突，若存在 conflicts 则返回 success=false，
 *   不做任何写入。
 * force=true：忽略冲突，直接写入系统变量并刷新环境。
 */
export async function activateGroup(
  id: string,
  force: boolean = false,
): Promise<ActivationResult> {
  return invoke('activate_group', { id, force });
}

export async function deactivateGroup(id: string): Promise<void> {
  return invoke('deactivate_group', { id });
}

// ===== 模板 =====
export async function getAllTemplates(): Promise<Template[]> {
  return invoke('get_all_templates');
}

export async function createTemplate(
  name: string,
  keys: string[],
): Promise<Template> {
  return invoke('create_template', { name, keys });
}

export async function updateTemplate(
  id: string,
  name: string,
  keys: string[],
): Promise<Template> {
  return invoke('update_template', { id, name, keys });
}

export async function deleteTemplate(id: string): Promise<void> {
  return invoke('delete_template', { id });
}

// ===== 导入导出 / 批量 / 冲突检测 =====
export async function exportGroups(
  groupIds?: string[],
  savePath?: string,
): Promise<string> {
  return invoke('export_groups', { groupIds, savePath });
}

export async function importGroups(json: string): Promise<number> {
  return invoke('import_groups', { json });
}

export async function batchDeleteGroups(ids: string[]): Promise<number> {
  return invoke('batch_delete_groups', { ids });
}

/**
 * 检测变量组与当前系统环境（用户环境变量）的冲突。
 * 激活流程中通常直接使用 activateGroup(id, force=false) 获取 conflicts，
 * 本接口用于详情页预览冲突。
 */
export async function detectConflicts(
  groupId: string,
): Promise<EnvVarConflict[]> {
  return invoke('detect_conflicts', { groupId });
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
