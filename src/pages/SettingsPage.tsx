import { useState, useEffect } from 'react';
import { useAppStore } from '../stores/useAppStore';
import { useToast } from '../components/ToastProvider';
import type { AppSettings } from '../types';

export function SettingsPage() {
  const { settings, updateSettings } = useAppStore();
  const { showToast } = useToast();
  const [localSettings, setLocalSettings] = useState<AppSettings>(settings);

  useEffect(() => {
    setLocalSettings(settings);
  }, [settings]);

  const handleSave = async () => {
    await updateSettings(localSettings);
    showToast('设置已保存', 'success');
  };

  return (
    <div className="max-w-2xl">
      <h2 className="text-lg font-semibold mb-6">设置</h2>

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
              onClick={() => setLocalSettings(prev => ({ ...prev, theme: { ...prev.theme, mode: opt.value } }))}
              className={`px-4 py-2 text-sm rounded-lg border ${
                localSettings.theme.mode === opt.value
                  ? 'border-indigo-600 bg-indigo-50 text-indigo-700 dark:bg-indigo-900/30 dark:text-indigo-300'
                  : 'border-gray-200 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700'
              }`}
            >
              {opt.label}
            </button>
          ))}
        </div>
      </div>

      {/* Notification */}
      <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 mb-4">
        <h3 className="text-sm font-semibold mb-4">通知</h3>
        <label className="flex items-center gap-3 mb-3">
          <input
            type="checkbox"
            checked={localSettings.notification.desktopEnabled}
            onChange={(e) => setLocalSettings(prev => ({
              ...prev,
              notification: { ...prev.notification, desktopEnabled: e.target.checked },
            }))}
            className="w-4 h-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
          />
          <span className="text-sm">桌面通知</span>
        </label>
        <label className="flex items-center gap-3">
          <input
            type="checkbox"
            checked={localSettings.notification.inAppEnabled}
            onChange={(e) => setLocalSettings(prev => ({
              ...prev,
              notification: { ...prev.notification, inAppEnabled: e.target.checked },
            }))}
            className="w-4 h-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
          />
          <span className="text-sm">应用内通知</span>
        </label>
      </div>

      {/* History */}
      <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 mb-6">
        <h3 className="text-sm font-semibold mb-4">历史记录</h3>
        <label className="flex items-center gap-3 mb-3">
          <input
            type="checkbox"
            checked={localSettings.history.autoCleanup}
            onChange={(e) => setLocalSettings(prev => ({
              ...prev,
              history: { ...prev.history, autoCleanup: e.target.checked },
            }))}
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
            value={localSettings.history.retentionDays}
            onChange={(e) => setLocalSettings(prev => ({
              ...prev,
              history: { ...prev.history, retentionDays: parseInt(e.target.value) || 30 },
            }))}
            className="w-20 px-3 py-1.5 bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg text-sm"
          />
        </div>
      </div>

      <button
        onClick={handleSave}
        className="px-6 py-2.5 bg-indigo-600 text-white text-sm font-medium rounded-lg hover:bg-indigo-700"
      >
        保存设置
      </button>
    </div>
  );
}
