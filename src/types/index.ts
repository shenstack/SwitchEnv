export interface EnvVar {
  name: string;
  value: string;
  isSystem: boolean;
  isReadonly: boolean;
  /** 变量来源文件路径，仅在 macOS / Linux 平台填充 */
  source?: string;
}

export interface EnvVariable {
  name: string;
  value: string;
  isHidden: boolean;
}

export interface EnvGroup {
  id: string;
  name: string;
  description: string;
  variables: EnvVariable[];
  isActive: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface EnvVarConflict {
  name: string;
  existingValue: string;
  newValue: string;
  source: string;
  sourceGroupName?: string;
}

export interface ActivationResult {
  success: boolean;
  conflicts: EnvVarConflict[];
  errors: string[];
}

/**
 * 变量组导入流程中逐变量差异项。
 * diffType: "added_only_incoming" | "missing_only_existing" | "value_changed"
 */
export interface ImportVarDiff {
  name: string;
  diffType: string;
  existingValue?: string | null;
  incomingValue?: string | null;
  existingIsHidden?: boolean | null;
  incomingIsHidden?: boolean | null;
}

/** 导入流程中名称冲突的变量组信息 */
export interface ImportConflictGroup {
  name: string;
  existingDescription: string;
  incomingDescription: string;
  varDiffs: ImportVarDiff[];
  isIdentical: boolean;
}

/** 预检的整体结果 */
export interface ImportPreviewResult {
  newGroups: { name: string; description: string; variables: EnvVariable[]; createdAt?: number }[];
  conflictGroups: ImportConflictGroup[];
}

/**
 * 变量组模板：保存一组常用的变量名。
 */
export interface Template {
  id: string;
  name: string;
  keys: string[];
  createdAt: number;
  updatedAt: number;
}

export interface HistoryRecord {
  id: string;
  actionType: string;
  targetType: string;
  targetId: string;
  beforeData: string | null;
  afterData: string | null;
  timestamp: number;
}

export interface Backup {
  id: string;
  name: string;
  scope: string;
  dbSnapshot: string;
  envSnapshot: string;
  createdAt: number;
}

export interface AppSettings {
  theme: {
    mode: string;
    fontLevel: number;
  };
  history: {
    autoCleanup: boolean;
    retentionDays: number;
  };
  logs: {
    autoCleanup: boolean;
    retentionDays: number;
  };
}

export interface ShellConfigInfo {
  shellPath: string;
  configFile: string;
  managedVars: string[];
}
