import { invoke } from "@tauri-apps/api/core";

export const api = {
  // 统计数据
  getDailyStats: (date: string) => invoke("get_daily_stats", { date }),
  getAppUsage: (start: number, end: number) => invoke("get_app_usage", { start, end }),
  getFocusSummary: (date: string) => invoke("get_focus_summary", { date }),

  // 事件查询
  queryEvents: (params: Record<string, unknown>) => invoke("query_events", { params }),
  getRealtimeStatus: () => invoke("get_realtime_status"),

  // TODO
  getTodos: (status?: string) => invoke("get_todos", { status }),
  toggleTodo: (id: number) => invoke("toggle_todo", { id }),

  // 报告
  getReports: (analysisType?: string) => invoke("get_reports", { analysisType }),
  triggerAnalysis: (analysisType: string) => invoke("trigger_analysis", { analysisType }),

  // 配置
  getConfig: () => invoke("get_config"),
  setConfig: (key: string, value: string) => invoke("set_config", { key, value }),

  // 录制控制
  setRecordingPaused: (paused: boolean) => invoke("set_recording_paused", { paused }),

  // 权限
  checkPermissions: () => invoke<{ accessibility: boolean; screen_recording: boolean }>("check_permissions"),
  openAccessibilityPrefs: () => invoke("open_accessibility_prefs"),
  openScreenRecordingPrefs: () => invoke("open_screen_recording_prefs"),

  // IME
  installIME: () => invoke("install_ime"),
  checkIMEStatus: () => invoke("check_ime_status"),
  openKeyboardSettings: () => invoke("open_keyboard_settings"),
  setBackendIME: (backend: string) => invoke("set_backend_ime", { backend }),

  // 数据导出与清理
  exportAllJSON: () => invoke("export_all_json"),
  exportEventsCSV: (start: number, end: number) => invoke("export_events_csv", { start, end }),
  cleanupOldData: () => invoke("cleanup_old_data"),
  getDbStats: () => invoke("get_db_stats"),

  // 统计
  getTypingStats: (start: number, end: number) => invoke("get_typing_stats", { start, end }),
  getTypingRhythm: (date: string) => invoke("get_typing_rhythm", { date }),
  getSwitchingStats: (date: string) => invoke("get_switching_stats", { date }),

  // 闪念
  getFlashIdeas: (limit?: number) => invoke("get_flash_ideas", { limit }),
  addFlashIdea: (text: string) => invoke("add_flash_idea", { text }),
  deleteFlashIdea: (id: number) => invoke("delete_flash_idea", { id }),

  // 目标
  getGoals: () => invoke("get_goals"),
  addGoal: (title: string, metricType: string, targetValue: number, period: string) =>
    invoke("add_goal", { title, metricType, targetValue, period }),
  deleteGoal: (id: number) => invoke("delete_goal", { id }),
  updateGoalProgress: (id: number, currentValue: number) =>
    invoke("update_goal_progress", { id, currentValue }),
};

export type ApiType = typeof api;
