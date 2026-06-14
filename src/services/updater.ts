// eslint-disable-next-line @tauri-apps/cli/no-dynamic-import
import type { Update } from '@tauri-apps/plugin-updater';

/** 更新信息 */
export interface UpdateInfo {
  /** 当前版本号 */
  currentVersion: string;
  /** 可用的新版本号 */
  availableVersion: string;
  /** 更新说明 */
  notes?: string;
  /** 发布日期 */
  pubDate?: string;
}

/** 检查更新的结果：已是最新 / 有可用更新 */
export type CheckResult =
  | { status: 'up-to-date' }
  | { status: 'available'; info: UpdateInfo };

/**
 * 检查是否有可用更新
 * 使用动态 import 避免在浏览器环境或未安装插件时崩溃
 */
export async function checkForUpdate(): Promise<CheckResult> {
  // eslint-disable-next-line @tauri-apps/cli/no-dynamic-import
  const { check } = await import('@tauri-apps/plugin-updater');
  // eslint-disable-next-line @tauri-apps/cli/no-dynamic-import
  const { getVersion } = await import('@tauri-apps/api/app');

  const currentVersion = await getVersion();
  const update: Update | null = await check();

  if (update && update.available) {
    const info: UpdateInfo = {
      currentVersion,
      availableVersion: update.version,
      notes: update.body ?? undefined,
      pubDate: update.date ?? undefined,
    };
    return { status: 'available', info };
  }

  return { status: 'up-to-date' };
}

/**
 * 下载安装更新并重启应用
 * @returns 是否成功安装更新；无可用更新时返回 false
 */
export async function installUpdateAndRestart(): Promise<boolean> {
  // eslint-disable-next-line @tauri-apps/cli/no-dynamic-import
  const { check } = await import('@tauri-apps/plugin-updater');

  const update: Update | null = await check();

  if (update && update.available) {
    await update.downloadAndInstall();
    return true;
  }

  return false;
}
