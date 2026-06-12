import { EnvVarListPage } from './EnvVarListPage';

export function UserVars() {
  return (
    <EnvVarListPage
      isSystem={false}
      title="用户级变量"
      showReadOnlyBanner={false}
      showSystemSettingsButton={false}
    />
  );
}
