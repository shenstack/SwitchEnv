import { EnvVarListPage } from './EnvVarListPage';

export function SystemVars() {
  return (
    <EnvVarListPage
      isSystem={true}
      title="系统级变量"
      showReadOnlyBanner={true}
      showSystemSettingsButton={true}
    />
  );
}
