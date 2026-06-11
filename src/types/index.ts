export interface EnvVar {
  name: string;
  value: string;
  isSystem: boolean;
  isReadonly: boolean;
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
  source: string; // 'system' 或某个已激活组的 id
  sourceGroupName?: string;
}

export interface ActivationResult {
  success: boolean;
  conflicts: EnvVarConflict[];
  errors: string[];
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
  notification: {
    desktopEnabled: boolean;
    inAppEnabled: boolean;
  };
  history: {
    autoCleanup: boolean;
    retentionDays: number;
  };
}

export interface ShellConfigInfo {
  shellPath: string;
  configFile: string;
  managedVars: string[];
}
