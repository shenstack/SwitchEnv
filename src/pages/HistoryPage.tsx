import { useState, useEffect, useCallback } from 'react';
import { RotateCcw, Trash2, RefreshCw } from 'lucide-react';
import * as ipc from '../services/ipc';
import { useToast } from '../components/ToastProvider';
import { ConfirmDialog } from '../components/ConfirmDialog';
import type { HistoryRecord } from '../types';

const ACTION_LABELS: Record<string, string> = {
  create: '创建',
  edit: '编辑',
  delete: '删除',
  activate: '激活',
  deactivate: '停用',
};

const ACTION_COLORS: Record<string, string> = {
  create: 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300',
  edit: 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300',
  delete: 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300',
  activate: 'bg-indigo-100 text-indigo-700 dark:bg-indigo-900 dark:text-indigo-300',
  deactivate: 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300',
};

export function HistoryPage() {
  const [records, setRecords] = useState<HistoryRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [showClearConfirm, setShowClearConfirm] = useState(false);
  const { showToast } = useToast();

  const fetchHistory = useCallback(async () => {
    setLoading(true);
    try {
      const data = await ipc.getHistory(undefined, 100);
      setRecords(data);
    } catch (err) {
      showToast(`加载失败: ${err}`, 'error');
    } finally {
      setLoading(false);
    }
  }, [showToast]);

  useEffect(() => { fetchHistory(); }, [fetchHistory]);

  const handleRestore = async (id: string) => {
    try {
      await ipc.restoreHistory(id);
      showToast('已还原操作', 'success');
      fetchHistory();
    } catch (err) {
      showToast(`还原失败: ${err}`, 'error');
    }
  };

  const handleClear = async () => {
    try {
      await ipc.clearHistory();
      showToast('历史记录已清空', 'success');
      fetchHistory();
    } catch (err) {
      showToast(`清空失败: ${err}`, 'error');
    }
  };

  const formatTime = (ts: number) => {
    return new Date(ts * 1000).toLocaleString('zh-CN');
  };

  if (loading) {
    return <div className="flex items-center justify-center py-20"><div className="text-gray-400">加载中...</div></div>;
  }

  return (
    <div>
      <div className="flex items-center justify-end mb-6">
        <div className="flex items-center gap-2">
          <button onClick={fetchHistory} className="p-2 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700">
            <RefreshCw size={18} />
          </button>
          {records.length > 0 && (
            <button onClick={() => setShowClearConfirm(true)} className="flex items-center gap-1.5 px-3 py-2 text-sm text-red-600 border border-red-200 dark:border-red-800 rounded-lg hover:bg-red-50 dark:hover:bg-red-900/20">
              <Trash2 size={14} />
              清空全部
            </button>
          )}
        </div>
      </div>

      {records.length === 0 ? (
        <div className="text-center py-20 text-gray-400">暂无操作记录</div>
      ) : (
        <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 overflow-hidden">
          <div className="max-h-[600px] overflow-y-auto">
            {records.map((record, idx) => (
              <div key={record.id} className={`flex items-center gap-3 px-4 py-3 ${idx > 0 ? 'border-t border-gray-100 dark:border-gray-700' : ''}`}>
                <span className={`px-2 py-0.5 text-xs font-medium rounded ${ACTION_COLORS[record.actionType] || 'bg-gray-100 text-gray-700'}`}>
                  {ACTION_LABELS[record.actionType] || record.actionType}
                </span>
                <span className="text-sm text-gray-500 dark:text-gray-400">
                  {record.targetType === 'group' ? '变量组' : record.targetType}
                </span>
                <span className="text-sm font-mono text-gray-400 truncate max-w-[200px]">{record.targetId.slice(0, 8)}...</span>
                <span className="flex-1" />
                <span className="text-xs text-gray-400">{formatTime(record.timestamp)}</span>
                <button
                  onClick={() => handleRestore(record.id)}
                  className="p-1.5 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-400 hover:text-indigo-600"
                  title="还原此操作"
                >
                  <RotateCcw size={14} />
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      <ConfirmDialog
        isOpen={showClearConfirm}
        onClose={() => setShowClearConfirm(false)}
        onConfirm={handleClear}
        title="清空历史记录"
        message="确定要清空所有操作历史记录吗？此操作不可撤销。"
        confirmText="清空"
        variant="danger"
      />
    </div>
  );
}
