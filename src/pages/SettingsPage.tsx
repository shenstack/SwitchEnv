import { useState, useEffect } from 'react';
import { useAppStore } from '../stores/useAppStore';
import type { AppSettings } from '../types';
import { getName, getVersion } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { appDataDir, appLogDir } from '@tauri-apps/api/path';
import { open as shellOpen } from '@tauri-apps/plugin-shell';
import * as ipc from '../services/ipc';
import { useUpdate } from '../contexts/UpdateContext';
import appLogo from '../assets/logo.png';

const REPO_URL = 'https://github.com/shenstack/SwitchEnv';

type SettingsTab = 'general' | 'about';

export function SettingsPage() {
  const { settings, updateSettings } = useAppStore();
  const [activeTab, setActiveTab] = useState<SettingsTab>('general');
  const [appInfo, setAppInfo] = useState<{ name: string; version: string }>({ name: '', version: '' });
  const [dataDir, setDataDir] = useState<string>('');
  const [logDir, setLogDir] = useState<string>('');
  const { status, info, checkUpdate, installUpdate } = useUpdate();
  const isChecking = status === 'checking';
  const isInstalling = status === 'installing';
  const isBusy = isChecking || isInstalling;

  useEffect(() => {
    (async () => {
      try {
        const [name, version] = await Promise.all([getName(), getVersion()]);
        setAppInfo({ name, version });
      } catch (err) {
        console.error('Failed to get app info:', err);
        setAppInfo({ name: 'SwitchEnv', version: '1.0.0' });
      }
    })();
  }, []);

  useEffect(() => {
    if (activeTab !== 'about') return;
    (async () => {
      try {
        const [dataPath, logPath] = await Promise.all([appDataDir(), appLogDir()]);
        setDataDir(dataPath);
        setLogDir(logPath);
      } catch (err) {
        console.error('Failed to get app directories:', err);
      }
    })();
  }, [activeTab]);

  const apply = (partial: Partial<AppSettings>) => {
    const next: AppSettings = { ...settings, ...partial };
    updateSettings(next);
  };

  return (
    <div>
      {/* ====== Tab Bar ====== */}
      <div className="flex gap-1 p-1 bg-gray-100 dark:bg-gray-700 rounded-full mb-6">
        <button
          onClick={() => setActiveTab('general')}
          className={`flex-1 px-4 py-1.5 text-sm rounded-full transition-colors ${
            activeTab === 'general'
              ? 'bg-white dark:bg-gray-800 text-indigo-600 dark:text-indigo-400 font-medium shadow-sm'
              : 'text-gray-600 dark:text-gray-300 hover:bg-gray-200/60 dark:hover:bg-gray-600'
          }`}
        >
          通用
        </button>
        <button
          onClick={() => setActiveTab('about')}
          className={`flex-1 px-4 py-1.5 text-sm rounded-full transition-colors ${
            activeTab === 'about'
              ? 'bg-white dark:bg-gray-800 text-indigo-600 dark:text-indigo-400 font-medium shadow-sm'
              : 'text-gray-600 dark:text-gray-300 hover:bg-gray-200/60 dark:hover:bg-gray-600'
          }`}
        >
          关于
        </button>
      </div>

      {/* ====== General Tab ====== */}
      {activeTab === 'general' && (
        <>
          {/* Theme */}
          <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 mb-4">
            <h3 className="text-sm font-semibold mb-4">主题</h3>
            <div className="flex gap-3">
              {[
                { value: 'system', label: '跟随系统' },
                { value: 'light', label: '浅色' },
                { value: 'dark', label: '深色' },
              ].map(opt => (
                <button
                  key={opt.value}
                  onClick={() => apply({ theme: { ...settings.theme, mode: opt.value } })}
                  className={`px-4 py-2 text-sm rounded-lg border ${
                    settings.theme.mode === opt.value
                      ? 'border-indigo-600 bg-indigo-50 text-indigo-700 dark:bg-indigo-900/30 dark:text-indigo-300'
                      : 'border-gray-200 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700'
                  }`}
                >
                  {opt.label}
                </button>
              ))}
            </div>
          </div>

          {/* History */}
          <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 mb-4">
            <h3 className="text-sm font-semibold mb-4">历史记录</h3>
            <label className="flex items-center gap-3 mb-3">
              <input
                type="checkbox"
                checked={settings.history.autoCleanup}
                onChange={(e) => apply({
                  history: {
                    ...settings.history,
                    autoCleanup: e.target.checked,
                  },
                })}
                className="w-4 h-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
              />
              <span className="text-sm">自动清理过期记录</span>
            </label>
            <div className="flex items-center gap-3">
              <span className="text-sm">保留天数:</span>
              <input
                type="number"
                min={1}
                max={365}
                value={settings.history.retentionDays}
                onChange={(e) => apply({
                  history: {
                    ...settings.history,
                    retentionDays: parseInt(e.target.value) || 30,
                  },
                })}
                className="w-20 px-3 py-1.5 bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg text-sm"
              />
            </div>
          </div>

          {/* Logs */}
          <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 mb-4">
            <h3 className="text-sm font-semibold mb-4">日志清理</h3>
            <label className="flex items-center gap-3 mb-3">
              <input
                type="checkbox"
                checked={settings.logs.autoCleanup}
                onChange={(e) => apply({
                  logs: {
                    ...settings.logs,
                    autoCleanup: e.target.checked,
                  },
                })}
                className="w-4 h-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
              />
              <span className="text-sm">启动时自动清理过期日志</span>
            </label>
            <div className="flex items-center gap-3">
              <span className="text-sm">保留天数:</span>
              <input
                type="number"
                min={1}
                max={365}
                value={settings.logs.retentionDays}
                onChange={(e) => apply({
                  logs: {
                    ...settings.logs,
                    retentionDays: parseInt(e.target.value) || 3,
                  },
                })}
                className="w-20 px-3 py-1.5 bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg text-sm"
              />
              <button
                onClick={async () => {
                  try {
                    const n = await ipc.cleanupLogs(settings.logs.retentionDays);
                    console.log(`已清理 ${n} 个日志文件`);
                  } catch (err) {
                    console.error('手动清理失败:', err);
                  }
                }}
                className="px-3 py-1.5 text-sm bg-indigo-50 text-indigo-700 dark:bg-indigo-900/30 dark:text-indigo-300 rounded-lg hover:bg-indigo-100 dark:hover:bg-indigo-900/50"
              >
                立即清理
              </button>
            </div>
          </div>
        </>
      )}

      {/* ====== About Tab ====== */}
      {activeTab === 'about' && (
        <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6">
          {/* Logo & App Name */}
          <div className="flex flex-col items-center text-center pb-6 mb-4 border-b border-gray-100 dark:border-gray-700">
            <img
              src={appLogo}
              alt="SwitchEnv Logo"
              className="w-20 h-20 rounded-2xl shadow-md mb-4 object-cover"
            />
            <button
              onClick={() => shellOpen(REPO_URL)}
              className="text-lg font-semibold text-gray-900 dark:text-gray-100 hover:text-indigo-600 dark:hover:text-indigo-400 hover:underline underline-offset-2 transition-colors cursor-pointer"
              title="在浏览器中打开项目仓库"
            >
              {appInfo.name || 'SwitchEnv'}
            </button>
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
              跨平台环境变量管理工具
            </p>
          </div>

          <div className="space-y-4">
            <div className="py-2 border-b border-gray-100 dark:border-gray-700">
              <div className="flex items-center justify-between gap-3">
                <span className="text-sm font-medium text-gray-500 dark:text-gray-400">版本</span>

                <div className="flex items-center gap-3">
                  <span className="text-sm">
                    {appInfo.version || '—'}
                  </span>

                  {status === 'available' ? (
                    <button
                      onClick={installUpdate}
                      disabled={isInstalling}
                      className="px-3 py-1.5 text-sm rounded-lg bg-indigo-600 text-white hover:bg-indigo-700 disabled:bg-indigo-400 disabled:cursor-not-allowed transition-colors"
                    >
                      {isInstalling ? '正在下载并安装…' : '升级到新版本'}
                    </button>
                  ) : (
                    <button
                      onClick={checkUpdate}
                      disabled={isBusy}
                      className="px-3 py-1.5 text-sm rounded-lg border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-60 disabled:cursor-not-allowed transition-colors"
                    >
                      {isChecking ? '正在检查…' : '检查更新'}
                    </button>
                  )}
                </div>
              </div>
              {status === 'available' && info && (
                <p className="mt-2 text-xs text-indigo-600 dark:text-indigo-400">
                  可升级到 v{info.availableVersion}
                </p>
              )}
              {status === 'up-to-date' && (
                <p className="mt-2 text-xs text-green-600 dark:text-green-400">
                  已是最新版本
                </p>
              )}
              {info?.notes && status === 'available' && (
                <p className="mt-2 text-xs text-gray-500 dark:text-gray-400 whitespace-pre-line">
                  {info.notes}
                </p>
              )}
            </div>
            <div className="flex items-start justify-between gap-4 py-2 border-b border-gray-100 dark:border-gray-700">
              <span className="text-sm font-medium text-gray-500 dark:text-gray-400 shrink-0">数据存储目录</span>
              <div className="flex items-start gap-2 text-right">
                <span className="text-sm break-all">{dataDir || '—'}</span>
                {dataDir && (
                  <button
                    onClick={async () => {
                      try {
                        await invoke('open_path', { path: dataDir });
                      } catch (e) {
                        console.error('打开数据目录失败:', e);
                      }
                    }}
                    className="text-sm text-indigo-600 dark:text-indigo-400 hover:underline shrink-0"
                  >
                    打开
                  </button>
                )}
              </div>
            </div>
            <div className="flex items-start justify-between gap-4 py-2">
              <span className="text-sm font-medium text-gray-500 dark:text-gray-400 shrink-0">日志目录</span>
              <div className="flex items-start gap-2 text-right">
                <span className="text-sm break-all">{logDir || '—'}</span>
                {logDir && (
                  <button
                    onClick={async () => {
                      try {
                        await invoke('open_path', { path: logDir });
                      } catch (e) {
                        console.error('打开日志目录失败:', e);
                      }
                    }}
                    className="text-sm text-indigo-600 dark:text-indigo-400 hover:underline shrink-0"
                  >
                    打开
                  </button>
                )}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
