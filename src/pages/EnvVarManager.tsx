import { useState, useEffect, useCallback } from 'react';
import { Plus, Power, PowerOff, Edit3, Trash2, Copy, ChevronDown, ChevronRight, RefreshCw, Download, Upload, FileText, FileJson, Eye, EyeOff, CheckSquare, Square } from 'lucide-react';
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
import { readTextFile } from '@tauri-apps/plugin-fs';
import * as ipc from '../services/ipc';
import { useToast } from '../components/ToastProvider';
import { Modal } from '../components/Modal';
import { ConfirmDialog } from '../components/ConfirmDialog';
import type { EnvGroup, EnvVariable, Template, EnvVarConflict } from '../types';

/**
 * 环境变量组管理主视图。
 * 功能：创建/编辑/删除/激活/停用、模板管理、导入导出、批量操作、搜索、排序。
 * 激活时会同时检测「系统环境变量」与「其他已激活变量组」中的同名变量，
 * 若存在差异则提示用户，用户确认后以 force=true 覆盖。
 */
export function EnvVarManager() {
  const [groups, setGroups] = useState<EnvGroup[]>([]);
  const [templates, setTemplates] = useState<Template[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');
  const [sortMode, setSortMode] = useState<'name' | 'time'>('time');
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set());
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

  // 对话框状态
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [editingGroup, setEditingGroup] = useState<EnvGroup | null>(null);
  const [deleteGroupId, setDeleteGroupId] = useState<string | null>(null);
  const [showBatchDelete, setShowBatchDelete] = useState(false);

  // 模板管理对话框
  const [showTemplateManager, setShowTemplateManager] = useState(false);

  // 冲突对话框
  const [conflictState, setConflictState] = useState<{
    group: EnvGroup;
    conflicts: EnvVarConflict[];
  } | null>(null);

  const { showToast } = useToast();

  // ---------- 数据加载 ----------
  const fetchAll = useCallback(async () => {
    setLoading(true);
    try {
      const [gs, ts] = await Promise.all([
        ipc.getAllGroups(),
        ipc.getAllTemplates(),
      ]);
      setGroups(gs);
      setTemplates(ts);
    } catch (err) {
      showToast(`加载失败: ${err}`, 'error');
    } finally {
      setLoading(false);
    }
  }, [showToast]);

  useEffect(() => {
    fetchAll();
  }, [fetchAll]);

  // ---------- 搜索与排序 ----------
  const filteredGroups = groups
    .filter((g) => {
      if (!search.trim()) return true;
      const q = search.toLowerCase();
      return (
        g.name.toLowerCase().includes(q) ||
        g.description.toLowerCase().includes(q) ||
        g.variables.some((v) => v.name.toLowerCase().includes(q) || v.value.toLowerCase().includes(q))
      );
    })
    .sort((a, b) => {
      if (sortMode === 'name') return a.name.localeCompare(b.name, 'zh-Hans-CN');
      return b.updatedAt - a.updatedAt;
    });

  // ---------- 创建 / 编辑 ----------
  const openCreate = () => {
    setEditingGroup(null);
    setShowCreateModal(true);
  };

  const openEdit = (g: EnvGroup) => {
    setEditingGroup(g);
    setShowCreateModal(true);
  };

  const handleSaveGroup = async (
    name: string,
    description: string,
    variables: EnvVariable[],
  ) => {
    try {
      if (editingGroup) {
        await ipc.updateGroup(editingGroup.id, name, description, variables);
        showToast(`变量组 "${name}" 已更新`, 'success');
      } else {
        await ipc.createGroup(name, description, variables);
        showToast(`变量组 "${name}" 已创建`, 'success');
      }
      setShowCreateModal(false);
      setEditingGroup(null);
      fetchAll();
    } catch (err) {
      showToast(`保存失败: ${err}`, 'error');
    }
  };

  // ---------- 激活 / 停用 ----------
  const handleToggleActive = async (g: EnvGroup) => {
    try {
      if (g.isActive) {
        await ipc.deactivateGroup(g.id);
        showToast(`已停用 "${g.name}"`, 'info');
        fetchAll();
        return;
      }
      // 一次 RPC 同时完成「冲突检测 + 返回 conflicts」
      const result = await ipc.activateGroup(g.id, false);
      if (!result.success && result.conflicts.length > 0) {
        setConflictState({ group: g, conflicts: result.conflicts });
        return;
      }
      showToast(`已激活 "${g.name}"`, 'success');
      fetchAll();
    } catch (err) {
      showToast(`操作失败: ${err}`, 'error');
    }
  };

  const handleConflictOverride = async () => {
    if (!conflictState) return;
    try {
      // force=true 跳过冲突检测，直接写入系统变量
      const result = await ipc.activateGroup(conflictState.group.id, true);
      if (!result.success) {
        const errMsg = result.errors.length > 0
          ? result.errors.join('; ')
          : '激活失败';
        showToast(errMsg, 'error');
        return;
      }
      showToast(`已激活 "${conflictState.group.name}"（覆盖已有变量）`, 'success');
      setConflictState(null);
      fetchAll();
    } catch (err) {
      showToast(`激活失败: ${err}`, 'error');
    }
  };

  // ---------- 删除 ----------
  const handleDelete = async (id: string) => {
    try {
      await ipc.deleteGroup(id);
      showToast('变量组已删除', 'success');
      setDeleteGroupId(null);
      fetchAll();
    } catch (err) {
      showToast(`删除失败: ${err}`, 'error');
    }
  };

  // ---------- 批量操作 ----------
  const toggleSelected = (id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const toggleSelectAll = () => {
    if (filteredGroups.every((g) => selectedIds.has(g.id))) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(filteredGroups.map((g) => g.id)));
    }
  };

  const handleBatchDelete = async () => {
    if (selectedIds.size === 0) return;
    try {
      const n = await ipc.batchDeleteGroups([...selectedIds]);
      showToast(`已删除 ${n} 个变量组`, 'success');
      setSelectedIds(new Set());
      setShowBatchDelete(false);
      fetchAll();
    } catch (err) {
      showToast(`批量删除失败: ${err}`, 'error');
    }
  };

  // ---------- 导入导出 ----------
  const handleExport = async () => {
    // 必须有勾选才能导出。无勾选时按钮本身是 disabled，这里作为二次兜底。
    const ids = [...selectedIds];
    if (ids.length === 0) {
      showToast('请先勾选要导出的变量组', 'warning');
      return;
    }
    try {
      const stamp = new Date().toISOString().slice(0, 10);
      const filePath = await saveDialog({
        title: '导出变量组',
        defaultPath: `env-groups-${stamp}.json`,
        filters: [{ name: 'JSON 文件', extensions: ['json'] }],
      });
      if (!filePath) return;
      const result = await ipc.exportGroups(ids, filePath);
      showToast(`已导出 ${ids.length} 个变量组到 ${result}`, 'success');
    } catch (err) {
      showToast(`导出失败: ${err}`, 'error');
    }
  };

  const handleImportClick = async () => {
    try {
      const selected = await openDialog({
        title: '导入变量组',
        multiple: false,
        filters: [{ name: 'JSON 文件', extensions: ['json'] }],
      });
      if (!selected) return;
      const filePath = Array.isArray(selected) ? selected[0] : selected;
      if (!filePath) return;
      const text = await readTextFile(filePath);
      const count = await ipc.importGroups(text);
      showToast(`成功导入 ${count} 个变量组`, 'success');
      fetchAll();
    } catch (err) {
      showToast(`导入失败: ${err}`, 'error');
    }
  };

  // ---------- 复制 ----------
  const handleCopy = async (text: string) => {
    try {
      await ipc.copyToClipboard(text);
      showToast('已复制到剪贴板', 'success');
    } catch {
      showToast('复制失败', 'error');
    }
  };

  // ---------- 渲染 ----------
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
      <div className="flex flex-wrap items-center gap-3 mb-6">
        <div className="flex-1 min-w-[200px]">
          <input
            type="text"
            placeholder="搜索变量组 / 变量名 / 值..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
          />
        </div>
        {/* 批量 / 管理按钮 */}
        <button
          onClick={() => setSortMode(sortMode === 'name' ? 'time' : 'name')}
          className="flex items-center gap-1.5 px-3 py-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg text-sm hover:bg-gray-50 dark:hover:bg-gray-700"
          title="切换排序方式"
        >
          <RefreshCw size={14} /> {sortMode === 'name' ? '按名称' : '按时间'}
        </button>
        <button
          onClick={fetchAll}
          className="p-2 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700"
          title="刷新"
        >
          <RefreshCw size={18} />
        </button>
        <button
          onClick={handleExport}
          disabled={selectedIds.size === 0}
          className={`flex items-center gap-1.5 px-3 py-2 text-sm bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg ${selectedIds.size > 0 ? 'hover:bg-gray-50 dark:hover:bg-gray-700' : 'opacity-40 cursor-not-allowed'}`}
          title={selectedIds.size > 0 ? '导出已选的变量组' : '请先勾选要导出的变量组'}
        >
          <Download size={14} /> 导出
        </button>
        <button
          onClick={handleImportClick}
          className="flex items-center gap-1.5 px-3 py-2 text-sm bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700"
        >
          <Upload size={14} /> 导入
        </button>
        <button
          onClick={() => setShowTemplateManager(true)}
          className="flex items-center gap-1.5 px-3 py-2 text-sm bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700"
        >
          <FileText size={14} /> 模板
        </button>

        {/* 根据是否有勾选，在末尾切换：新建分组 / 批量删除 */}
        {selectedIds.size > 0 ? (
          <button
            onClick={() => setShowBatchDelete(true)}
            className="flex items-center gap-1.5 px-4 py-2 bg-red-600 text-white text-sm font-medium rounded-lg hover:bg-red-700"
          >
            <Trash2 size={16} /> 批量删除
          </button>
        ) : (
          <button
            onClick={openCreate}
            className="flex items-center gap-1.5 px-4 py-2 bg-indigo-600 text-white text-sm font-medium rounded-lg hover:bg-indigo-700"
          >
            <Plus size={16} /> 新建分组
          </button>
        )}
      </div>

      {/* 全选提示 */}
      {filteredGroups.length > 0 && (
        <div className="flex items-center gap-2 mb-3 text-sm">
          <button
            onClick={toggleSelectAll}
            className="flex items-center gap-1.5 px-2 py-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700"
          >
            {filteredGroups.every((g) => selectedIds.has(g.id)) ? (
              <CheckSquare size={14} />
            ) : (
              <Square size={14} />
            )} 全选
          </button>
          {selectedIds.size > 0 && (
            <span className="text-gray-500">已选 {selectedIds.size} / {filteredGroups.length}</span>
          )}
        </div>
      )}

      {/* Group List */}
      {filteredGroups.length === 0 ? (
        <div className="text-center py-20 text-gray-400">
          {search ? '没有匹配的变量组' : '暂无变量组，点击"新建分组"开始'}
        </div>
      ) : (
        <div className="grid gap-4">
          {filteredGroups.map((group) => {
            const isExpanded = expandedIds.has(group.id);
            const isSelected = selectedIds.has(group.id);
            return (
              <div
                key={group.id}
                className={`bg-white dark:bg-gray-800 rounded-xl border overflow-hidden transition-all ${
                  isSelected
                    ? 'border-indigo-400 ring-2 ring-indigo-400/30'
                    : 'border-gray-200 dark:border-gray-700'
                }`}
              >
                {/* 标题栏 */}
                <div className="flex items-center gap-3 px-4 py-3">
                  <button
                    onClick={() => toggleSelected(group.id)}
                    className="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700"
                    title="选择"
                  >
                    {isSelected ? <CheckSquare size={16} className="text-indigo-600" /> : <Square size={16} />}
                  </button>
                  <button
                    onClick={() => handleToggleActive(group)}
                    className={`p-1.5 rounded-lg transition-colors ${
                      group.isActive
                        ? 'bg-green-100 text-green-600 dark:bg-green-900 dark:text-green-400'
                        : 'bg-gray-100 text-gray-400 dark:bg-gray-700'
                    }`}
                    title={group.isActive ? '点击停用' : '点击激活'}
                  >
                    {group.isActive ? <Power size={16} /> : <PowerOff size={16} />}
                  </button>

                  <button
                    onClick={() =>
                      setExpandedIds((prev) => {
                        const next = new Set(prev);
                        if (next.has(group.id)) next.delete(group.id);
                        else next.add(group.id);
                        return next;
                      })
                    }
                    className="flex-1 text-left min-w-0"
                  >
                    <div className="flex items-center gap-2 flex-wrap">
                      {isExpanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
                      <span className="font-medium">{group.name}</span>
                      {group.isActive && (
                        <span className="px-1.5 py-0.5 text-xs bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300 rounded">
                          已激活
                        </span>
                      )}
                    </div>
                    {group.description && (
                      <p className="text-sm text-gray-500 dark:text-gray-400 ml-6 mt-0.5 truncate">
                        {group.description}
                      </p>
                    )}
                  </button>

                  <span className="text-xs text-gray-400">{group.variables.length} 个变量</span>

                  <button
                    onClick={() => openEdit(group)}
                    className="p-1.5 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
                    title="编辑"
                  >
                    <Edit3 size={16} />
                  </button>
                  <button
                    onClick={() => setDeleteGroupId(group.id)}
                    className="p-1.5 rounded-lg hover:bg-red-50 dark:hover:bg-red-900/30 text-red-500"
                    title="删除"
                  >
                    <Trash2 size={16} />
                  </button>
                </div>

                {/* 展开内容 */}
                {isExpanded && (
                  <div className="border-t border-gray-100 dark:border-gray-700 px-4 py-2 bg-gray-50/50 dark:bg-gray-800/50">
                    {group.variables.length === 0 ? (
                      <p className="text-sm text-gray-400 py-2">暂无变量</p>
                    ) : (
                      <div className="space-y-1">
                        {group.variables.map((v, idx) => (
                          <VariableRow key={idx} variable={v} onCopy={handleCopy} />
                        ))}
                      </div>
                    )}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}

      {/* 创建 / 编辑对话框 */}
      {showCreateModal && (
        <EditGroupModal
          initial={editingGroup}
          templates={templates}
          onCancel={() => {
            setShowCreateModal(false);
            setEditingGroup(null);
          }}
          onSave={handleSaveGroup}
          onAfterTemplatesChange={fetchAll}
        />
      )}

      {/* 删除确认 */}
      <ConfirmDialog
        isOpen={!!deleteGroupId}
        onClose={() => setDeleteGroupId(null)}
        onConfirm={() => deleteGroupId && handleDelete(deleteGroupId)}
        title="删除变量组"
        message="确定要删除此变量组吗？如果已激活，将自动停用并清理系统环境变量。"
        confirmText="删除"
        variant="danger"
      />

      {/* 批量删除确认 */}
      <ConfirmDialog
        isOpen={showBatchDelete}
        onClose={() => setShowBatchDelete(false)}
        onConfirm={handleBatchDelete}
        title="批量删除"
        message={`确定要删除选中的 ${selectedIds.size} 个变量组吗？已激活的组会先自动停用。`}
        confirmText="删除"
        variant="danger"
      />

      {/* 冲突对话框 */}
      {conflictState && (
        <Modal
          isOpen={true}
          onClose={() => setConflictState(null)}
          title="检测到变量冲突"
          maxWidth="max-w-xl"
        >
          <p className="text-sm text-gray-600 dark:text-gray-300 mb-4">
            激活 "{conflictState.group.name}" 将覆盖以下变量（系统环境变量或其他已激活变量组中存在同名变量且值不同）：
          </p>
          <div className="space-y-2 max-h-64 overflow-y-auto border border-gray-200 dark:border-gray-700 rounded-lg p-3 bg-gray-50 dark:bg-gray-900/30">
            {conflictState.conflicts.map((c, idx) => (
              <div key={idx} className="text-sm font-mono">
                <div className="text-indigo-600 dark:text-indigo-300">
                  {c.name}
                  <span className="ml-2 text-xs font-sans text-gray-500 dark:text-gray-400">
                    （来源: {c.sourceGroupName ? `变量组 "${c.sourceGroupName}"` : '系统环境变量'}）
                  </span>
                </div>
                <div className="text-gray-500 dark:text-gray-400 pl-4 mt-0.5">
                  当前: <span className="text-red-500">{c.existingValue || '(空)'}</span>
                </div>
                <div className="text-gray-500 dark:text-gray-400 pl-4">
                  新值: <span className="text-green-600 dark:text-green-400">{c.newValue || '(空)'}</span>
                </div>
              </div>
            ))}
          </div>
          <div className="flex justify-end gap-3 pt-4 mt-4 border-t border-gray-200 dark:border-gray-700">
            <button
              onClick={() => setConflictState(null)}
              className="px-4 py-2 text-sm bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600"
            >
              取消
            </button>
            <button
              onClick={handleConflictOverride}
              className="px-4 py-2 text-sm bg-indigo-600 text-white rounded-lg hover:bg-indigo-700"
            >
              仍然激活（覆盖）
            </button>
          </div>
        </Modal>
      )}

      {/* 模板管理 */}
      {showTemplateManager && (
        <TemplateManager
          templates={templates}
          onClose={() => {
            setShowTemplateManager(false);
            fetchAll();
          }}
        />
      )}
    </div>
  );
}

// ---------- 子组件：单个变量行 ----------
function VariableRow({
  variable,
  onCopy,
}: {
  variable: EnvVariable;
  onCopy: (text: string) => void;
}) {
  const [hidden, setHidden] = useState(variable.isHidden);
  const displayValue = hidden ? '••••••••' : variable.value;
  return (
    <div className="flex items-center gap-2 py-1.5 px-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700/50 text-sm">
      <span className="font-mono text-indigo-600 dark:text-indigo-300 min-w-[140px] truncate">
        {variable.name}
      </span>
      <span className="text-gray-300">=</span>
      <span className="font-mono flex-1 truncate">{displayValue}</span>
      <button
        onClick={() => setHidden(!hidden)}
        className="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-600"
        title={hidden ? '显示' : '隐藏'}
      >
        {hidden ? <Eye size={12} /> : <EyeOff size={12} />}
      </button>
      <button
        onClick={() => onCopy(variable.name)}
        className="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-600"
        title="复制变量名"
      >
        <Copy size={12} />
      </button>
      <button
        onClick={() => onCopy(variable.value)}
        className="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-600"
        title="复制值"
      >
        <FileJson size={12} />
      </button>
    </div>
  );
}

// ---------- 子组件：编辑 / 创建组对话框 ----------
function EditGroupModal({
  initial,
  templates,
  onCancel,
  onSave,
  onAfterTemplatesChange,
}: {
  initial: EnvGroup | null;
  templates: Template[];
  onCancel: () => void;
  onSave: (name: string, description: string, variables: EnvVariable[]) => void;
  onAfterTemplatesChange: () => void;
}) {
  const [name, setName] = useState(initial?.name ?? '');
  const [description, setDescription] = useState(initial?.description ?? '');
  const [variables, setVariables] = useState<EnvVariable[]>(
    initial?.variables ?? [{ name: '', value: '', isHidden: false }],
  );
  const [showCreateTemplate, setShowCreateTemplate] = useState(false);
  const [newTemplateName, setNewTemplateName] = useState('');
  const { showToast } = useToast();

  const addVar = () => setVariables((v) => [...v, { name: '', value: '', isHidden: false }]);
  const removeVar = (idx: number) =>
    setVariables((v) => (v.length > 1 ? v.filter((_, i) => i !== idx) : v));
  const updateVar = (idx: number, field: keyof EnvVariable, value: string | boolean) =>
    setVariables((v) => v.map((x, i) => (i === idx ? { ...x, [field]: value } : x)));

  const applyTemplate = (tpl: Template) => {
    const existing = new Set(variables.filter((v) => v.name.trim()).map((v) => v.name.trim()));
    const toAdd = tpl.keys
      .filter((k) => k.trim() && !existing.has(k.trim()))
      .map((k) => ({ name: k.trim(), value: '', isHidden: false }));
    if (toAdd.length === 0) {
      showToast('模板中的变量名已全部存在', 'info');
      return;
    }
    setVariables((v) => [...v.filter((x) => x.name.trim() || x.value.trim()), ...toAdd]);
    showToast(`已添加 ${toAdd.length} 个变量名`, 'success');
  };

  const handleCreateTemplate = async () => {
    if (!newTemplateName.trim()) {
      showToast('请输入模板名称', 'warning');
      return;
    }
    const keys = variables
      .map((v) => v.name.trim())
      .filter((n) => n.length > 0);
    if (keys.length === 0) {
      showToast('请先填写至少一个变量名', 'warning');
      return;
    }
    try {
      await ipc.createTemplate(newTemplateName.trim(), keys);
      showToast(`模板 "${newTemplateName}" 已创建`, 'success');
      setNewTemplateName('');
      setShowCreateTemplate(false);
      onAfterTemplatesChange();
    } catch (err) {
      showToast(`创建模板失败: ${err}`, 'error');
    }
  };

  // 仅保留 name 与 value 都非空的变量，避免写入无效条目
  const validVariables = variables.filter(
    (v) => v.name.trim().length > 0 && v.value.trim().length > 0,
  );
  const canSave = name.trim().length > 0 && validVariables.length > 0;

  return (
    <Modal
      isOpen={true}
      onClose={onCancel}
      title={initial ? `编辑变量组：${initial.name}` : '新建分组'}
      maxWidth="max-w-2xl"
    >
      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium mb-1">名称</label>
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="例如: Node.js Dev"
            className="w-full px-3 py-2 bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">描述</label>
          <input
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="可选"
            className="w-full px-3 py-2 bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
          />
        </div>

        <div>
          <div className="flex items-center justify-between mb-2">
            <label className="block text-sm font-medium">环境变量</label>
            <div className="flex items-center gap-2">
              {templates.length > 0 && (
                <select
                  onChange={(e) => {
                    const tpl = templates.find((t) => t.id === e.target.value);
                    if (tpl) applyTemplate(tpl);
                    e.currentTarget.value = '';
                  }}
                  className="px-2 py-1 bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded text-sm"
                  defaultValue=""
                >
                  <option value="">应用模板...</option>
                  {templates.map((t) => (
                    <option key={t.id} value={t.id}>
                      {t.name}
                    </option>
                  ))}
                </select>
              )}
              <button
                onClick={() => setShowCreateTemplate((s) => !s)}
                className="text-sm text-indigo-600 hover:text-indigo-700"
              >
                {showCreateTemplate ? '取消' : '保存为模板'}
              </button>
              <button
                onClick={addVar}
                className="text-sm text-indigo-600 hover:text-indigo-700 flex items-center gap-1"
              >
                <Plus size={14} /> 添加变量
              </button>
            </div>
          </div>

          {showCreateTemplate && (
            <div className="mb-2 p-2 bg-indigo-50 dark:bg-indigo-900/30 rounded-lg flex items-center gap-2">
              <input
                value={newTemplateName}
                onChange={(e) => setNewTemplateName(e.target.value)}
                placeholder="输入模板名称"
                className="flex-1 px-2 py-1 text-sm bg-white dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded"
              />
              <button
                onClick={handleCreateTemplate}
                className="px-2 py-1 text-sm bg-indigo-600 text-white rounded hover:bg-indigo-700"
              >
                创建
              </button>
            </div>
          )}

          <div className="space-y-2 max-h-80 overflow-y-auto pr-1">
            {variables.map((v, idx) => {
              const nameEmpty = v.name.trim().length === 0;
              const valueEmpty = v.value.trim().length === 0;
              return (
                <div key={idx} className="flex items-center gap-2">
                  <input
                    value={v.name}
                    onChange={(e) => updateVar(idx, 'name', e.target.value)}
                    placeholder="变量名"
                    className={`flex-1 px-3 py-2 bg-gray-100 dark:bg-gray-700 border rounded-lg text-sm font-mono focus:outline-none focus:ring-2 focus:ring-indigo-500 ${
                      nameEmpty
                        ? 'border-red-400 dark:border-red-500'
                        : 'border-gray-200 dark:border-gray-600'
                    }`}
                  />
                  <span className="text-gray-300">=</span>
                  <input
                    value={v.value}
                    onChange={(e) => updateVar(idx, 'value', e.target.value)}
                    placeholder="变量值"
                    className={`flex-[2] px-3 py-2 bg-gray-100 dark:bg-gray-700 border rounded-lg text-sm font-mono focus:outline-none focus:ring-2 focus:ring-indigo-500 ${
                      valueEmpty
                        ? 'border-red-400 dark:border-red-500'
                        : 'border-gray-200 dark:border-gray-600'
                    }`}
                  />
                <button
                  onClick={() => updateVar(idx, 'isHidden', !v.isHidden)}
                  className="p-2 rounded hover:bg-gray-100 dark:hover:bg-gray-600"
                  title={v.isHidden ? '取消隐藏' : '隐藏值'}
                >
                  {v.isHidden ? <EyeOff size={14} /> : <Eye size={14} />}
                </button>
                <button
                  onClick={() => removeVar(idx)}
                  className="p-2 rounded hover:bg-red-50 dark:hover:bg-red-900/30 text-red-500"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            );
            })}
          </div>
        </div>

        <div className="flex justify-end gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
          <button
            onClick={onCancel}
            className="px-4 py-2 text-sm font-medium bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600"
          >
            取消
          </button>
          <button
            disabled={!canSave}
            onClick={() =>
              onSave(
                name.trim(),
                description.trim(),
                validVariables.map((v) => ({
                  ...v,
                  name: v.name.trim(),
                  value: v.value.trim(),
                })),
              )
            }
            className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 rounded-lg hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            保存
          </button>
        </div>
      </div>
    </Modal>
  );
}

// ---------- 子组件：模板管理 ----------
function TemplateManager({ templates, onClose }: { templates: Template[]; onClose: () => void }) {
  const [local, setLocal] = useState<Template[]>(templates);
  const [editing, setEditing] = useState<Template | null>(null);
  const [name, setName] = useState('');
  const [keysText, setKeysText] = useState('');
  const { showToast } = useToast();

  const startCreate = () => {
    setEditing(null);
    setName('');
    setKeysText('');
  };

  const startEdit = (t: Template) => {
    setEditing(t);
    setName(t.name);
    setKeysText(t.keys.join('\n'));
  };

  const handleSave = async () => {
    if (!name.trim()) {
      showToast('请输入模板名称', 'warning');
      return;
    }
    const ks = keysText
      .split('\n')
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
    if (ks.length === 0) {
      showToast('请至少输入一个变量名', 'warning');
      return;
    }
    try {
      if (editing) {
        await ipc.updateTemplate(editing.id, name.trim(), ks);
        showToast('模板已更新', 'success');
      } else {
        await ipc.createTemplate(name.trim(), ks);
        showToast('模板已创建', 'success');
      }
      setLocal(await ipc.getAllTemplates());
      setEditing(null);
      setName('');
      setKeysText('');
    } catch (err) {
      showToast(`保存失败: ${err}`, 'error');
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await ipc.deleteTemplate(id);
      showToast('模板已删除', 'success');
      setLocal(await ipc.getAllTemplates());
    } catch (err) {
      showToast(`删除失败: ${err}`, 'error');
    }
  };

  return (
    <Modal isOpen={true} onClose={onClose} title="变量组模板管理" maxWidth="max-w-2xl">
      <div className="space-y-4">
        <div className="border border-gray-200 dark:border-gray-700 rounded-lg p-3">
          <label className="block text-sm font-medium mb-1">{editing ? '编辑模板' : '新建模板'}</label>
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="模板名称"
            className="w-full px-3 py-2 mb-2 bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
          />
          <textarea
            value={keysText}
            onChange={(e) => setKeysText(e.target.value)}
            placeholder="变量名列表，每行一个"
            rows={4}
            className="w-full px-3 py-2 bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg text-sm font-mono focus:outline-none focus:ring-2 focus:ring-indigo-500"
          />
          <div className="flex gap-2 justify-end mt-2">
            {editing && (
              <button
                onClick={startCreate}
                className="px-3 py-1.5 text-sm bg-gray-100 dark:bg-gray-700 rounded hover:bg-gray-200 dark:hover:bg-gray-600"
              >
                新建
              </button>
            )}
            <button
              onClick={handleSave}
              className="px-3 py-1.5 text-sm bg-indigo-600 text-white rounded hover:bg-indigo-700"
            >
              {editing ? '更新' : '创建'}
            </button>
          </div>
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">现有模板 ({local.length})</label>
          {local.length === 0 ? (
            <p className="text-sm text-gray-400 py-4 text-center">暂无模板</p>
          ) : (
            <div className="space-y-2 max-h-64 overflow-y-auto pr-1">
              {local.map((t) => (
                <div
                  key={t.id}
                  className="flex items-start gap-3 p-2 bg-gray-50 dark:bg-gray-900/30 rounded-lg border border-gray-200 dark:border-gray-700"
                >
                  <div className="flex-1 min-w-0">
                    <div className="font-medium text-sm">{t.name}</div>
                    <div className="text-xs text-gray-500 mt-1">
                      {t.keys.length} 个变量名: {t.keys.slice(0, 3).join(', ')}
                      {t.keys.length > 3 ? `... 等` : ''}
                    </div>
                  </div>
                  <button
                    onClick={() => startEdit(t)}
                    className="p-1.5 rounded hover:bg-gray-100 dark:hover:bg-gray-700"
                    title="编辑"
                  >
                    <Edit3 size={14} />
                  </button>
                  <button
                    onClick={() => handleDelete(t.id)}
                    className="p-1.5 rounded hover:bg-red-50 dark:hover:bg-red-900/30 text-red-500"
                    title="删除"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="flex justify-end pt-3 border-t border-gray-200 dark:border-gray-700">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm bg-indigo-600 text-white rounded-lg hover:bg-indigo-700"
          >
            完成
          </button>
        </div>
      </div>
    </Modal>
  );
}

// ---------- 子组件：模板管理 ----------
