import { useState, useMemo, useEffect } from 'react';
import { Modal } from './Modal';
import type { ImportPreviewResult, ImportConflictGroup, ImportVarDiff } from '../types';

interface ImportConflictDialogProps {
  isOpen: boolean;
  preview: ImportPreviewResult | null;
  onCancel: () => void;
  onConfirm: (overwriteNames: string[], mergeNames: string[], ignoreNames: string[]) => void;
}

function diffStatusText(diffType: string): string {
  switch (diffType) {
    case 'added_only_incoming':
      return '新增（导入中存在，现有无）';
    case 'missing_only_existing':
      return '删除（现有存在，导入中无）';
    case 'value_changed':
      return '值不同';
    default:
      return diffType;
  }
}

function diffStatusColor(diffType: string): string {
  switch (diffType) {
    case 'added_only_incoming':
      return 'text-green-600 dark:text-green-400';
    case 'missing_only_existing':
      return 'text-orange-600 dark:text-orange-400';
    case 'value_changed':
      return 'text-blue-600 dark:text-blue-400';
    default:
      return 'text-gray-600 dark:text-gray-400';
  }
}

function ConflictGroupSection({
  group,
  decision,
  onDecisionChange,
}: {
  group: ImportConflictGroup;
  decision: 'overwrite' | 'ignore' | 'merge';
  onDecisionChange: (decision: 'overwrite' | 'ignore' | 'merge') => void;
}) {
  return (
    <div className="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
      <div className="bg-gray-50 dark:bg-gray-700/40 px-4 py-3 flex items-start justify-between gap-3">
        <div className="flex-1">
          <div className="flex items-center gap-2">
            <span className="font-medium text-gray-900 dark:text-gray-100">{group.name}</span>
            {group.isIdentical && (
              <span className="text-xs px-2 py-0.5 bg-yellow-100 dark:bg-yellow-900/40 text-yellow-700 dark:text-yellow-300 rounded">
                内容完全相同
              </span>
            )}
          </div>
          <div className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            现有描述：{group.existingDescription || '(无)'}
            {group.existingDescription !== group.incomingDescription && (
              <>
                <span className="mx-2">→</span>
                <span>导入中：{group.incomingDescription || '(无)'}</span>
              </>
            )}
          </div>
          <div className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            差异：{group.varDiffs.length} 项
          </div>
        </div>
        <div className="flex items-center gap-3 shrink-0">
          <label className="flex items-center gap-1 text-sm cursor-pointer">
            <input
              type="radio"
              name={`decision-${group.name}`}
              checked={decision === 'ignore'}
              onChange={() => onDecisionChange('ignore')}
            />
            <span>忽略</span>
          </label>
          <label className="flex items-center gap-1 text-sm cursor-pointer">
            <input
              type="radio"
              name={`decision-${group.name}`}
              checked={decision === 'overwrite'}
              onChange={() => onDecisionChange('overwrite')}
            />
            <span>覆盖</span>
          </label>
          <label className="flex items-center gap-1 text-sm cursor-pointer">
            <input
              type="radio"
              name={`decision-${group.name}`}
              checked={decision === 'merge'}
              onChange={() => onDecisionChange('merge')}
            />
            <span>合并</span>
          </label>
        </div>
      </div>

      {group.varDiffs.length > 0 && (
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead className="text-xs text-gray-500 dark:text-gray-400 border-b border-gray-200 dark:border-gray-700">
              <tr>
                <th className="px-4 py-2 text-left font-medium w-52">变量名</th>
                <th className="px-4 py-2 text-left font-medium">现有值</th>
                <th className="px-4 py-2 text-left font-medium">导入值</th>
                <th className="px-4 py-2 text-left font-medium">状态</th>
              </tr>
            </thead>
            <tbody>
              {group.varDiffs.map((diff, i) => (
                <DiffRow key={`${diff.name}-${i}`} diff={diff} />
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function DiffRow({ diff }: { diff: ImportVarDiff }) {
  const existingShown =
    diff.existingValue !== null && diff.existingValue !== undefined
      ? diff.existingValue
      : diff.diffType === 'added_only_incoming'
        ? '—'
        : '';
  const incomingShown =
    diff.incomingValue !== null && diff.incomingValue !== undefined
      ? diff.incomingValue
      : diff.diffType === 'missing_only_existing'
        ? '—'
        : '';

  return (
    <tr className="border-b border-gray-100 dark:border-gray-700/50 last:border-0">
      <td className="px-4 py-2 font-mono text-gray-800 dark:text-gray-200">{diff.name}</td>
      <td className="px-4 py-2 font-mono text-gray-600 dark:text-gray-400 break-all">
        {existingShown}
      </td>
      <td className="px-4 py-2 font-mono text-gray-800 dark:text-gray-200 break-all">
        {incomingShown}
      </td>
      <td className={`px-4 py-2 ${diffStatusColor(diff.diffType)}`}>
        {diffStatusText(diff.diffType)}
      </td>
    </tr>
  );
}

export function ImportConflictDialog({
  isOpen,
  preview,
  onCancel,
  onConfirm,
}: ImportConflictDialogProps) {
  // 对每个冲突组默认选择"忽略"
  const defaultDecisions = useMemo(() => {
    const d: Record<string, 'overwrite' | 'ignore' | 'merge'> = {};
    preview?.conflictGroups.forEach((g) => {
      d[g.name] = 'ignore';
    });
    return d;
  }, [preview]);

  const [decisions, setDecisions] = useState<Record<string, 'overwrite' | 'ignore' | 'merge'>>(
    defaultDecisions,
  );

  useEffect(() => {
    if (preview) {
      const hasAll = preview.conflictGroups.every((g) =>
        Object.prototype.hasOwnProperty.call(decisions, g.name),
      );
      if (!hasAll) {
        setDecisions(defaultDecisions);
      }
    }
  }, [preview]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleDecisionChange = (groupName: string, decision: 'overwrite' | 'ignore' | 'merge') => {
    setDecisions((prev) => ({ ...prev, [groupName]: decision }));
  };

  const setAllTo = (decision: 'overwrite' | 'ignore' | 'merge') => {
    if (!preview) return;
    const next: Record<string, 'overwrite' | 'ignore' | 'merge'> = {};
    preview.conflictGroups.forEach((g) => {
      next[g.name] = decision;
    });
    setDecisions(next);
  };

  // 全部忽略且无新增 → 禁用确认按钮
  const isConfirmDisabled =
    !!preview &&
    preview.newGroups.length === 0 &&
    preview.conflictGroups.length > 0 &&
    preview.conflictGroups.every((g) => decisions[g.name] !== 'overwrite' && decisions[g.name] !== 'merge');

  const handleConfirm = () => {
    if (!preview) return;
    if (isConfirmDisabled) return;
    const overwrites: string[] = [];
    const merges: string[] = [];
    const ignores: string[] = [];
    preview.conflictGroups.forEach((g) => {
      if (decisions[g.name] === 'overwrite') {
        overwrites.push(g.name);
      } else if (decisions[g.name] === 'merge') {
        merges.push(g.name);
      } else {
        ignores.push(g.name);
      }
    });
    onConfirm(overwrites, merges, ignores);
  };

  return (
    <Modal isOpen={isOpen} onClose={onCancel} title="导入预览与冲突处理" maxWidth="max-w-3xl">
      {!preview ? null : (
        <div className="space-y-4">
          <div className="text-sm text-gray-600 dark:text-gray-300">
            共发现 <span className="font-semibold">{preview.newGroups.length + preview.conflictGroups.length}</span> 个变量组：
            <span className="ml-2 text-green-600 dark:text-green-400">{preview.newGroups.length} 个新增</span>，
            <span className="ml-2 text-blue-600 dark:text-blue-400">{preview.conflictGroups.length} 个名称冲突</span>
          </div>

          {preview.newGroups.length > 0 && (
            <details className="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
              <summary className="cursor-pointer px-4 py-3 text-sm font-medium bg-gray-50 dark:bg-gray-700/40 text-gray-800 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700/60">
                新增变量组（{preview.newGroups.length}）
              </summary>
              <div className="px-4 py-3 space-y-2 text-sm">
                {preview.newGroups.map((g, i) => (
                  <div key={`${g.name}-${i}`} className="flex items-start gap-2">
                    <span className="w-1.5 h-1.5 bg-green-500 rounded-full mt-1.5 shrink-0" />
                    <div>
                      <div className="font-medium text-gray-800 dark:text-gray-200">{g.name}</div>
                      {g.description && (
                        <div className="text-xs text-gray-500 dark:text-gray-400">{g.description}</div>
                      )}
                      <div className="text-xs text-gray-500 dark:text-gray-400">
                        {g.variables.length} 个变量
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </details>
          )}

          {preview.conflictGroups.length > 0 && (
            <>
              <div className="flex items-center justify-between">
                <div className="text-sm font-medium text-gray-800 dark:text-gray-200">
                  冲突变量组（{preview.conflictGroups.length}）
                </div>
                <div className="flex items-center gap-2 text-sm">
                  <button
                    onClick={() => setAllTo('ignore')}
                    className="px-3 py-1 text-xs rounded-md border border-gray-200 dark:border-gray-700 hover:bg-gray-100 dark:hover:bg-gray-700"
                  >
                    全部忽略
                  </button>
                  <button
                    onClick={() => setAllTo('overwrite')}
                    className="px-3 py-1 text-xs rounded-md border border-blue-200 dark:border-blue-800 text-blue-700 dark:text-blue-300 hover:bg-blue-50 dark:hover:bg-blue-900/40"
                  >
                    全部覆盖
                  </button>
                  <button
                    onClick={() => setAllTo('merge')}
                    className="px-3 py-1 text-xs rounded-md border border-purple-200 dark:border-purple-800 text-purple-700 dark:text-purple-300 hover:bg-purple-50 dark:hover:bg-purple-900/40"
                  >
                    全部合并
                  </button>
                </div>
              </div>
              <div className="space-y-3">
                {preview.conflictGroups.map((g) => (
                  <ConflictGroupSection
                    key={g.name}
                    group={g}
                    decision={decisions[g.name] ?? 'ignore'}
                    onDecisionChange={(d) => handleDecisionChange(g.name, d)}
                  />
                ))}
              </div>
            </>
          )}

          <div className="flex justify-end gap-3 pt-3 border-t border-gray-200 dark:border-gray-700">
            <button
              onClick={onCancel}
              className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600"
            >
              取消
            </button>
            <button
              onClick={handleConfirm}
              disabled={isConfirmDisabled}
              className={
                isConfirmDisabled
                  ? 'px-4 py-2 text-sm font-medium text-gray-400 dark:text-gray-500 bg-gray-200 dark:bg-gray-700 rounded-lg cursor-not-allowed'
                  : 'px-4 py-2 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg'
              }
              title='确认导入'
            >
               确认导入
            </button>
          </div>
        </div>
      )}
    </Modal>
  );
}
