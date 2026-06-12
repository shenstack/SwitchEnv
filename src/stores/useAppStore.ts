import { create } from 'zustand';
import type { AppSettings } from '../types';
import * as ipc from '../services/ipc';

interface AppStore {
  settings: AppSettings;
  loading: boolean;
  fetchSettings: () => Promise<void>;
  updateSettings: (settings: AppSettings) => Promise<void>;
}

export const useAppStore = create<AppStore>()((set) => ({
  settings: {
    theme: { mode: 'system', fontLevel: 2 },
    history: { autoCleanup: true, retentionDays: 30 },
    logs: { autoCleanup: true, retentionDays: 3 },
  },
  loading: false,

  fetchSettings: async () => {
    try {
      const settings = await ipc.getAppSettings();
      set({ settings });
    } catch {
      // Use defaults
    }
  },

  updateSettings: async (settings) => {
    try {
      await ipc.setAppSettings(settings);
      set({ settings });
    } catch (err) {
      console.error('Failed to save settings:', err);
    }
  },
}));
