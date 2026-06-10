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
  chainId: string | null;
  createdAt: number;
  updatedAt: number;
}

export interface ActivationResult {
  success: boolean;
  conflicts: EnvVarConflict[];
  deactivatedGroups: string[];
  errors: string[];
}

export interface EnvVarConflict {
  name: string;
  existingValue: string;
  newValue: string;
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
