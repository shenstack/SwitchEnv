import { useState } from 'react';
import { EnvVarManager } from './pages/EnvVarManager';
import { SystemVars } from './pages/SystemVars';
import { HistoryPage } from './pages/HistoryPage';
import { SettingsPage } from './pages/SettingsPage';
import { ToastProvider } from './components/ToastProvider';
import { useTheme } from './hooks/useTheme';
import { Layers, Monitor, History, Settings } from 'lucide-react';

type Tab = 'groups' | 'system-vars' | 'history' | 'settings';

export default function App() {
  const [activeTab, setActiveTab] = useState<Tab>('groups');
  useTheme();

  const tabs: { id: Tab; label: string; icon: React.ReactNode }[] = [
    { id: 'groups', label: '变量组', icon: <Layers size={18} /> },
    { id: 'system-vars', label: '系统变量', icon: <Monitor size={18} /> },
    { id: 'history', label: '历史记录', icon: <History size={18} /> },
    { id: 'settings', label: '设置', icon: <Settings size={18} /> },
  ];

  return (
    <ToastProvider>
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100">
        {/* Header */}
        <header className="sticky top-0 z-50 bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border-b border-gray-200 dark:border-gray-700">
          <div className="max-w-7xl mx-auto px-4">
            <div className="flex items-center justify-between h-14">
              <div className="flex items-center gap-2">
                <div className="w-8 h-8 bg-indigo-600 rounded-lg flex items-center justify-center">
                  <span className="text-white font-bold text-sm">E</span>
                </div>
                <h1 className="text-lg font-semibold">Env Assistant</h1>
              </div>
            </div>
            {/* Tab Bar */}
            <div className="flex gap-1 -mb-px">
              {tabs.map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={`flex items-center gap-1.5 px-4 py-2.5 text-sm font-medium border-b-2 transition-colors ${
                    activeTab === tab.id
                      ? 'border-indigo-600 text-indigo-600 dark:text-indigo-400 dark:border-indigo-400'
                      : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
                  }`}
                >
                  {tab.icon}
                  {tab.label}
                </button>
              ))}
            </div>
          </div>
        </header>

        {/* Content */}
        <main className="max-w-7xl mx-auto px-4 py-6">
          {activeTab === 'groups' && <EnvVarManager />}
          {activeTab === 'system-vars' && <SystemVars />}
          {activeTab === 'history' && <HistoryPage />}
          {activeTab === 'settings' && <SettingsPage />}
        </main>
      </div>
    </ToastProvider>
  );
}
