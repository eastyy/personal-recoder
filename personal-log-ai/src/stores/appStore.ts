import { create } from "zustand";

interface AppState {
  // 权限引导
  permissionsChecked: boolean;

  // 实时状态
  isRecording: boolean;
  currentApp: string | null;
  todayEvents: number;

  // 统计数据
  dailyStats: Record<string, unknown> | null;
  focusSummary: Record<string, unknown> | null;

  // TODO
  todos: Array<Record<string, unknown>>;

  // 报告
  reports: Array<Record<string, unknown>>;

  // 配置
  config: Record<string, string>;

  // 动作
  setPermissionsChecked: (checked: boolean) => void;
  setRecording: (recording: boolean) => void;
  setCurrentApp: (app: string | null) => void;
  setTodayEvents: (count: number) => void;
  setDailyStats: (stats: Record<string, unknown>) => void;
  setFocusSummary: (summary: Record<string, unknown>) => void;
  setTodos: (todos: Array<Record<string, unknown>>) => void;
  setReports: (reports: Array<Record<string, unknown>>) => void;
  setConfig: (config: Record<string, string>) => void;
}

export const useAppStore = create<AppState>((set) => ({
  permissionsChecked: false,
  isRecording: true,
  currentApp: null,
  todayEvents: 0,
  dailyStats: null,
  focusSummary: null,
  todos: [],
  reports: [],
  config: {},

  setPermissionsChecked: (checked) => set({ permissionsChecked: checked }),
  setRecording: (recording) => set({ isRecording: recording }),
  setCurrentApp: (app) => set({ currentApp: app }),
  setTodayEvents: (count) => set({ todayEvents: count }),
  setDailyStats: (stats) => set({ dailyStats: stats }),
  setFocusSummary: (summary) => set({ focusSummary: summary }),
  setTodos: (todos) => set({ todos }),
  setReports: (reports) => set({ reports }),
  setConfig: (config) => set({ config }),
}));
