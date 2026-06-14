import React, { createContext, useCallback, useContext, useMemo, useRef, useState } from 'react';
import { type UpdateInfo, checkForUpdate, installUpdateAndRestart } from '../services/updater';
import { useToast } from '../components/ToastProvider';

/** 更新状态枚举 */
type UpdateStatus = 'idle' | 'checking' | 'installing' | 'up-to-date' | 'available' | 'error';

/** 更新上下文值 */
interface UpdateContextValue {
  status: UpdateStatus;
  info: UpdateInfo | null;
  errorMessage: string | null;
  checkUpdate: () => Promise<boolean>;
  installUpdate: () => Promise<boolean>;
}

const UpdateContext = createContext<UpdateContextValue | undefined>(undefined);

/** 更新状态管理 Provider */
export function UpdateProvider({ children }: { children: React.ReactNode }) {
  const { showToast } = useToast();
  const [status, setStatus] = useState<UpdateStatus>('idle');
  const [info, setInfo] = useState<UpdateInfo | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // 防止并发操作
  const busyRef = useRef(false);

  /** 检查更新 */
  const handleCheck = useCallback(async (): Promise<boolean> => {
    if (busyRef.current) return false;
    busyRef.current = true;
    setStatus('checking');
    setErrorMessage(null);

    try {
      const result = await checkForUpdate();

      if (result.status === 'available') {
        setInfo(result.info);
        setStatus('available');
        showToast(`发现新版本 v${result.info.availableVersion}`, 'info');
        return true;
      }

      setInfo(null);
      setStatus('up-to-date');
      showToast('当前已是最新版本', 'success');
      return false;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setErrorMessage(message);
      setStatus('error');
      showToast(message, 'error');
      return false;
    } finally {
      busyRef.current = false;
    }
  }, [showToast]);

  /** 安装更新并重启 */
  const handleInstall = useCallback(async (): Promise<boolean> => {
    if (busyRef.current) return false;
    busyRef.current = true;
    setStatus('installing');
    setErrorMessage(null);

    try {
      const installed = await installUpdateAndRestart();

      if (installed) {
        showToast('更新已下载，正在准备重启…', 'success');
      } else {
        setStatus('up-to-date');
        showToast('没有可更新的版本', 'info');
      }

      return installed;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setErrorMessage(message);
      setStatus('error');
      showToast(message, 'error');
      return false;
    } finally {
      busyRef.current = false;
    }
  }, [showToast]);

  const value = useMemo<UpdateContextValue>(
    () => ({
      status,
      info,
      errorMessage,
      checkUpdate: handleCheck,
      installUpdate: handleInstall,
    }),
    [status, info, errorMessage, handleCheck, handleInstall],
  );

  return <UpdateContext.Provider value={value}>{children}</UpdateContext.Provider>;
}

/** 获取更新上下文的 Hook */
export function useUpdate(): UpdateContextValue {
  const ctx = useContext(UpdateContext);
  if (ctx === undefined) {
    throw new Error('useUpdate 必须在 <UpdateProvider> 内部使用');
  }
  return ctx;
}
