import { useState } from 'react';
import { EnvVarManager } from './pages/EnvVarManager';
import { UserVars } from './pages/UserVars';
import { SystemVars } from './pages/SystemVars';
import { HistoryPage } from './pages/HistoryPage';
import { SettingsPage } from './pages/SettingsPage';
import { ToastProvider } from './components/ToastProvider';
import { UpdateProvider } from './contexts/UpdateContext';
import { useTheme } from './hooks/useTheme';
import { Layers, User, Monitor, History, Settings, ArrowLeft } from 'lucide-react';

type MainTab = 'groups' | 'user-vars' | 'system-vars';
type SubPage = 'history' | 'settings' | null;

export default function App() {
  const [activeTab, setActiveTab] = useState<MainTab>('groups');
  const [subPage, setSubPage] = useState<SubPage>(null);
  useTheme();

  const mainTabs: { id: MainTab; label: string; icon: React.ReactNode }[] = [
    { id: 'groups', label: '变量组', icon: <Layers size={18} /> },
    { id: 'user-vars', label: '用户变量', icon: <User size={18} /> },
    { id: 'system-vars', label: '系统变量', icon: <Monitor size={18} /> },
  ];

  const rightIcons: { id: 'history' | 'settings'; icon: React.ReactNode; title: string }[] = [
    { id: 'history', icon: <History size={18} />, title: '历史记录' },
    { id: 'settings', icon: <Settings size={18} />, title: '设置' },
  ];

  const handleBack = () => {
    setSubPage(null);
  };

  const isRightIconActive = (id: 'history' | 'settings') => subPage === id;

  return (
    <ToastProvider>
      <UpdateProvider>
        <div className="min-h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100">
        {subPage === null && (
          <header className="sticky top-0 z-50 bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border-b border-gray-200 dark:border-gray-700">
            <div className="max-w-7xl mx-auto px-4">
              <div className="flex items-center justify-between">
                <div className="flex gap-1 -mb-px">
                  {mainTabs.map((tab) => {
                    const isActive = activeTab === tab.id && subPage === null;
                    return (
                      <button
                        key={tab.id}
                        onClick={() => {
                          setActiveTab(tab.id);
                          setSubPage(null);
                        }}
                        className={`flex items-center gap-1.5 px-4 py-2.5 text-sm font-medium border-b-2 transition-colors ${
                          isActive
                            ? 'border-indigo-600 text-indigo-600 dark:text-indigo-400 dark:border-indigo-400'
                            : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
                        }`}
                      >
                        {tab.icon}
                        {tab.label}
                      </button>
                    );
                  })}
                </div>

                <div className="flex items-center gap-1">
                  {rightIcons.map((item) => {
                    const active = isRightIconActive(item.id);
                    return (
                      <button
                        key={item.id}
                        onClick={() => setSubPage(item.id)}
                        title={item.title}
                        className={`flex items-center justify-center w-9 h-9 rounded-lg transition-colors ${
                          active
                            ? 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200'
                            : 'hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-500 dark:text-gray-400'
                        }`}
                      >
                        {item.icon}
                      </button>
                    );
                  })}
                </div>
              </div>
            </div>
          </header>
        )}

        <main className="max-w-7xl mx-auto px-4">
          {subPage === 'history' && (
            <>
              <div className="sticky top-0 z-40 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 -mx-4">
                <div className="max-w-7xl mx-auto px-4">
                  <div className="flex items-center gap-3 py-3">
                    <button
                      onClick={handleBack}
                      aria-label="返回"
                      className="p-2.5 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                    >
                      <ArrowLeft size={16} />
                    </button>
                    <h2 className="text-lg font-semibold">操作历史</h2>
                  </div>
                </div>
              </div>
              <div className="py-6">
                <HistoryPage />
              </div>
            </>
          )}
          {subPage === 'settings' && (
            <>
              <div className="sticky top-0 z-40 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 -mx-4">
                <div className="max-w-7xl mx-auto px-4">
                  <div className="flex items-center gap-3 py-3">
                    <button
                      onClick={handleBack}
                      aria-label="返回"
                      className="p-2.5 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                    >
                      <ArrowLeft size={16} />
                    </button>
                    <h2 className="text-lg font-semibold">设置</h2>
                  </div>
                </div>
              </div>
              <div className="py-6">
                <SettingsPage />
              </div>
            </>
          )}
          {subPage === null && (
            <div className="py-6">
              {activeTab === 'groups' && <EnvVarManager />}
              {activeTab === 'user-vars' && <UserVars />}
              {activeTab === 'system-vars' && <SystemVars />}
            </div>
          )}
        </main>
      </div>
      </UpdateProvider>
    </ToastProvider>
  );
}
