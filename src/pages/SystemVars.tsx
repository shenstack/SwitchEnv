import { useState, useEffect, useCallback } from 'react';
import { RefreshCw, Copy, ExternalLink } from 'lucide-react';
import * as ipc from '../services/ipc';
import { useToast } from '../components/ToastProvider';
import { SearchBar } from '../components/SearchBar';
import type { EnvVar } from '../types';

export function SystemVars() {
  const [userVars, setUserVars] = useState<EnvVar[]>([]);
  const [systemVars, setSystemVars] = useState<EnvVar[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');
  const [canModify, setCanModify] = useState(false);
  const { showToast } = useToast();

  const fetchVars = useCallback(async () => {
    setLoading(true);
    try {
      const [uVars, sVars, modify] = await Promise.all([
        ipc.getAllEnvVars(false),
        ipc.getAllEnvVars(true),
        ipc.canModifySystem(),
      ]);
      setUserVars(uVars);
      setSystemVars(sVars);
      setCanModify(modify);
    } catch (err) {
      showToast(`加载失败: ${err}`, 'error');
    } finally {
      setLoading(false);
    }
  }, [showToast]);

  useEffect(() => { fetchVars(); }, [fetchVars]);

  const handleCopy = async (text: string) => {
    try {
      await ipc.copyToClipboard(text);
      showToast('已复制', 'success');
    } catch {
      showToast('复制失败', 'error');
    }
  };

  const renderVarList = (vars: EnvVar[], title: string) => {
    const filtered = vars.filter(v =>
      v.name.toLowerCase().includes(search.toLowerCase()) ||
      v.value.toLowerCase().includes(search.toLowerCase())
    );

    return (
      <div className="mb-8">
        <h3 className="text-sm font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-3">
          {title} ({filtered.length})
        </h3>
        {filtered.length === 0 ? (
          <p className="text-sm text-gray-400">无匹配变量</p>
        ) : (
          <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 overflow-hidden">
            <div className="max-h-[400px] overflow-y-auto">
              {filtered.map((v, idx) => (
                <div key={v.name} className={`flex items-center gap-2 px-4 py-2 text-sm ${idx > 0 ? 'border-t border-gray-100 dark:border-gray-700' : ''}`}>
                  <span className="font-mono text-indigo-600 dark:text-indigo-400 min-w-[160px] truncate">{v.name}</span>
                  <span className="text-gray-300">=</span>
                  <span className="font-mono flex-1 truncate text-gray-600 dark:text-gray-300">{v.value}</span>
                  <button onClick={() => handleCopy(v.value)} className="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700" title="复制值">
                    <Copy size={12} />
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    );
  };

  if (loading) {
    return <div className="flex items-center justify-center py-20"><div className="text-gray-400">加载中...</div></div>;
  }

  return (
    <div>
      <div className="flex items-center gap-3 mb-6">
        <div className="flex-1">
          <SearchBar value={search} onChange={setSearch} placeholder="搜索环境变量..." />
        </div>
        <button onClick={fetchVars} className="p-2 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700" title="刷新">
          <RefreshCw size={18} />
        </button>
        <button onClick={() => ipc.openSystemSettings()} className="flex items-center gap-1.5 px-3 py-2 text-sm border border-gray-200 dark:border-gray-600 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700">
          <ExternalLink size={14} />
          系统设置
        </button>
      </div>

      {!canModify && (
        <div className="mb-4 px-4 py-3 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg text-sm text-yellow-800 dark:text-yellow-200">
          当前无管理员权限，系统级变量为只读模式。
        </div>
      )}

      {renderVarList(userVars, '用户级变量')}
      {renderVarList(systemVars, '系统级变量')}
    </div>
  );
}
