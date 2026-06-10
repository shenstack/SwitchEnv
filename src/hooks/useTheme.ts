import { useEffect } from 'react';
import { useAppStore } from '../stores/useAppStore';

export function useTheme() {
  const theme = useAppStore((s) => s.settings.theme);

  useEffect(() => {
    const root = document.documentElement;

    if (theme.mode === 'system') {
      const mq = window.matchMedia('(prefers-color-scheme: dark)');
      const update = () => root.classList.toggle('dark', mq.matches);
      update();
      mq.addEventListener('change', update);
      return () => mq.removeEventListener('change', update);
    } else {
      root.classList.toggle('dark', theme.mode === 'dark');
    }
  }, [theme.mode]);
}
