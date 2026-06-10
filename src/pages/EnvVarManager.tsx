import { useState, useEffect, useCallback } from 'react';
import { Plus, Power, PowerOff, Edit3, Trash2, Copy, ChevronDown, ChevronRight, RefreshCw } from 'lucide-react';
import * as ipc from '../services/ipc';
import { useToast } from '../components/ToastProvider';
import { Modal } from '../components/Modal';
import { SearchBar } from '../components/SearchBar';
import { ConfirmDialog } from '../components/ConfirmDialog';
import type { EnvGroup, EnvVariable } from '../types';

export function EnvVarManager() {
  const [groups, setGroups] = useState<EnvGroup[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [newGroup, setNewGroup] = useState({ name: '', description: '', variables: [{ name: '', value: '', isHidden: false }] });
  const { showToast } = useToast();

  const fetchGroups = useCallback(async () => {
    setLoading(true);
    try {
      const data = await ipc.getAllGroups();
      setGroups(data);
    } catch (err) {
      showToast(`加载失败: ${err}`, 'error');
    } finally {
      setLoading(false);
    }
  }, [showToast]);

  useEffect(() => { fetchGroups(); }, [fetchGroups]);

  const handleCreate = async () => {
    try {
      const vars = newGroup.variables.filter(v => v.name.trim());
      await ipc.createGroup(newGroup.name, newGroup.description, vars, null);
      setShowCreateModal(false);
      setNewGroup({ name: '', description: '', variables: [{ name: '', value: '', isHidden: false }] });
      showToast('变量组创建成功', 'success');
      fetchGroups();
    } catch (err) {
      showToast(`创建失败: ${err}`, 'error');
    }
  };

  const handleToggleActive = async (group: EnvGroup) => {
    try {
      if (group.isActive) {
        await ipc.deactivateGroup(group.id);
        showToast(`已停用 "${group.name}"`, 'info');
      } else {
        const result = await ipc.activateGroup(group.id);
        if (result.conflicts.length > 0) {
          showToast(`发现 ${result.conflicts.length} 个变量冲突`, 'warning');
        }
        if (result.errors.length > 0) {
          showToast(`部分变量设置失败`, 'warning');
        }
        showToast(`已激活 "${group.name}"`, 'success');
      }
      fetchGroups();
    } catch (err) {
      showToast(`操作失败: ${err}`, 'error');
    }
  };

  const handleDelete = async () => {
    if (!deleteId) return;
    try {
      await ipc.deleteGroup(deleteId);
      showToast('变量组已删除', 'success');
      fetchGroups();
    } catch (err) {
      showToast(`删除失败: ${err}`, 'error');
    }
  };

  const handleCopy = async (text: string) => {
    try {
      await ipc.copyToClipboard(text);
      showToast('已复制到剪贴板', 'success');
    } catch {
      showToast('复制失败', 'error');
    }
  };

  const addVariable = () => {
    setNewGroup(prev => ({
      ...prev,
      variables: [...prev.variables, { name: '', value: '', isHidden: false }],
    }));
  };

  const removeVariable = (index: number) => {
    setNewGroup(prev => ({
      ...prev,
      variables: prev.variables.filter((_, i) => i !== index),
    }));
  };

  const updateVariable = (index: number, field: keyof EnvVariable, value: string | boolean) => {
    setNewGroup(prev => ({
      ...prev,
      variables: prev.variables.map((v, i) => i === index ? { ...v, [field]: value } : v),
    }));
  };

  const filteredGroups = groups.filter(g =>
    g.name.toLowerCase().includes(search.toLowerCase()) ||
    g.description.toLowerCase().includes(search.toLowerCase())
  );

  if (loading) {
    return (
      <div className="flex items-center justify-center py-20">
        <div className="text-gray-400">加载中...</div>
      </div>
    );
  }

  return (
    <div>
      {/* Toolbar */}
      <div className="flex items-center gap-3 mb-6">
        <div className="flex-1">
          <SearchBar value={search} onChange={setSearch} placeholder="搜索变量组..." />
        </div>
        <button onClick={fetchGroups} className="p-2 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700" title="刷新">
          <RefreshCw size={18} />
        </button>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center gap-1.5 px-4 py-2 bg-indigo-600 text-white text-sm font-medium rounded-lg hover:bg-indigo-700"
        >
          <Plus size={16} />
          新建变量组
        </button>
      </div>

      {/* Group List */}
      {filteredGroups.length === 0 ? (
        <div className="text-center py-20 text-gray-400">
          {search ? '没有匹配的变量组' : '暂无变量组，点击"新建变量组"开始'}
        </div>
      ) : (
        <div className="grid gap-4">
          {filteredGroups.map(group => (
            <div key={group.id} className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 overflow-hidden">
              {/* Group Header */}
              <div className="flex items-center gap-3 px-4 py-3">
                <button
                  onClick={() => handleToggleActive(group)}
                  className={`p-1.5 rounded-lg transition-colors ${
                    group.isActive
                      ? 'bg-green-100 text-green-600 dark:bg-green-900 dark:text-green-400'
                      : 'bg-gray-100 text-gray-400 dark:bg-gray-700 dark:text-gray-500'
                  }`}
                  title={group.isActive ? '点击停用' : '点击激活'}
                >
                  {group.isActive ? <Power size={16} /> : <PowerOff size={16} />}
                </button>

                <button
                  onClick={() => setExpandedId(expandedId === group.id ? null : group.id)}
                  className="flex-1 text-left"
                >
                  <div className="flex items-center gap-2">
                    {expandedId === group.id ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
                    <span className="font-medium">{group.name}</span>
                    {group.isActive && (
                      <span className="px-1.5 py-0.5 text-xs bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300 rounded">
                        已激活
                      </span>
                    )}
                  </div>
                  {group.description && (
                    <p className="text-sm text-gray-500 dark:text-gray-400 ml-6 mt-0.5">{group.description}</p>
                  )}
                </button>

                <span className="text-xs text-gray-400">{group.variables.length} 个变量</span>

                <div className="flex items-center gap-1">
                  <button className="p-1.5 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-400" title="编辑（开发中）">
                    <Edit3 size={14} />
                  </button>
                  <button onClick={() => setDeleteId(group.id)} className="p-1.5 rounded-lg hover:bg-red-50 dark:hover:bg-red-900/30 text-gray-400 hover:text-red-500" title="删除">
                    <Trash2 size={14} />
                  </button>
                </div>
              </div>

              {/* Expanded Variables */}
              {expandedId === group.id && (
                <div className="border-t border-gray-100 dark:border-gray-700 px-4 py-2 bg-gray-50/50 dark:bg-gray-800/50">
                  {group.variables.length === 0 ? (
                    <p className="text-sm text-gray-400 py-2">暂无变量</p>
                  ) : (
                    <div className="space-y-1">
                      {group.variables.map((variable, idx) => (
                        <div key={idx} className="flex items-center gap-2 py-1.5 px-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700/50 text-sm">
                          <span className="font-mono text-indigo-600 dark:text-indigo-400 min-w-[120px]">{variable.name}</span>
                          <span className="text-gray-300">=</span>
                          <span className="font-mono flex-1 truncate">
                            {variable.isHidden ? '••••••••' : variable.value}
                          </span>
                          <button onClick={() => handleCopy(variable.name)} className="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-600" title="复制变量名">
                            <Copy size={12} />
                          </button>
                          <button onClick={() => handleCopy(variable.value)} className="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-600" title="复制值">
                            <Copy size={12} />
                          </button>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Create Modal */}
      <Modal isOpen={showCreateModal} onClose={() => setShowCreateModal(false)} title="创建变量组" maxWidth="max-w-2xl">
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-1">组名称</label>
            <input
              type="text"
              value={newGroup.name}
              onChange={(e) => setNewGroup(prev => ({ ...prev, name: e.target.value }))}
              className="w-full px-3 py-2 bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
              placeholder="例如: Node.js Dev"
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">描述</label>
            <input
              type="text"
              value={newGroup.description}
              onChange={(e) => setNewGroup(prev => ({ ...prev, description: e.target.value }))}
              className="w-full px-3 py-2 bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
              placeholder="可选描述"
            />
          </div>
          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="block text-sm font-medium">环境变量</label>
              <button onClick={addVariable} className="text-sm text-indigo-600 hover:text-indigo-700 flex items-center gap-1">
                <Plus size={14} /> 添加变量
              </button>
            </div>
            <div className="space-y-2">
              {newGroup.variables.map((variable, idx) => (
                <div key={idx} className="flex items-center gap-2">
                  <input
                    type="text"
                    value={variable.name}
                    onChange={(e) => updateVariable(idx, 'name', e.target.value)}
                    className="flex-1 px-3 py-2 bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg text-sm font-mono focus:outline-none focus:ring-2 focus:ring-indigo-500"
                    placeholder="变量名"
                  />
                  <span className="text-gray-400">=</span>
                  <input
                    type="text"
                    value={variable.value}
                    onChange={(e) => updateVariable(idx, 'value', e.target.value)}
                    className="flex-[2] px-3 py-2 bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg text-sm font-mono focus:outline-none focus:ring-2 focus:ring-indigo-500"
                    placeholder="变量值"
                  />
                  <button onClick={() => removeVariable(idx)} className="p-2 text-gray-400 hover:text-red-500">
                    <Trash2 size={14} />
                  </button>
                </div>
              ))}
            </div>
          </div>
          <div className="flex justify-end gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
            <button onClick={() => setShowCreateModal(false)} className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600">
              取消
            </button>
            <button onClick={handleCreate} disabled={!newGroup.name.trim()} className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 rounded-lg hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed">
              创建
            </button>
          </div>
        </div>
      </Modal>

      {/* Delete Confirm */}
      <ConfirmDialog
        isOpen={!!deleteId}
        onClose={() => setDeleteId(null)}
        onConfirm={handleDelete}
        title="删除变量组"
        message="确定要删除此变量组吗？如果已激活，将自动停用并清理所有环境变量。"
        confirmText="删除"
        variant="danger"
      />
    </div>
  );
}
