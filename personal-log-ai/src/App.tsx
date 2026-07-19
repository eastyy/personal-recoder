import { Routes, Route, NavLink } from "react-router-dom";
import {
  LayoutDashboard,
  Clock,
  Activity,
  FileText,
  CheckSquare,
  Settings,
  Pause,
  Play,
  Radio,
  EyeOff,
  Keyboard,
  MousePointerClick,
  ClipboardCopy,
  AppWindow,
  Type,
  Target,
  Lightbulb,
} from "lucide-react";
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useAppStore } from "./stores/appStore";
import { api } from "./services/tauri";
import Dashboard from "./pages/Dashboard";
import TimeStats from "./pages/TimeStats";
import RealtimeMonitor from "./pages/RealtimeMonitor";
import ReportCenter from "./pages/ReportCenter";
import TodoList from "./pages/TodoList";
import SettingsPage from "./pages/Settings";
import IMESettings from "./pages/IMESettings";
import PermissionGuide from "./pages/PermissionGuide";
import ContentBrowser from "./pages/ContentBrowser";
import Goals from "./pages/Goals";
import FlashIdeas from "./pages/FlashIdeas";

function App() {
  const { isRecording, setRecording, permissionsChecked, setPermissionsChecked, setCurrentApp, setTodayEvents } = useAppStore();

  // 监听后端实时状态推送（必须在条件渲染之前，遵守 Hooks 规则）
  useEffect(() => {
    const unlisten = listen("realtime_update", (event) => {
      const payload = event.payload as Record<string, unknown>;
      if (payload.app_name) {
        setCurrentApp(payload.app_name as string);
      }
      if (typeof payload.today_events === "number") {
        setTodayEvents(payload.today_events);
      }
      if (typeof payload.recording === "boolean") {
        setRecording(payload.recording);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [setCurrentApp, setTodayEvents, setRecording]);

  // 权限引导流程
  if (!permissionsChecked) {
    return <PermissionGuide onAllGranted={() => setPermissionsChecked(true)} />;
  }

  // 暂停/恢复记录 — 调用后端
  async function toggleRecording() {
    const newState = !isRecording;
    try {
      await api.setRecordingPaused(!newState);
      setRecording(newState);
    } catch (e) {
      console.error("Failed to toggle recording:", e);
    }
  }

  return (
    <div className="flex h-screen bg-gray-50">
      {/* 侧边栏 */}
      <aside className="w-64 bg-white border-r border-gray-200 flex flex-col">
        {/* Logo + 监控状态 */}
        <div className="p-6">
          <h1 className="text-xl font-bold text-gray-900">个人输入统计助理</h1>
          <p className="text-xs text-gray-500 mt-1">个人输入记录与AI分析</p>

          {/* 全局监控状态指示器 */}
          <div className="mt-4 p-3 bg-gray-50 rounded-xl border border-gray-100">
            <div className="flex items-center gap-2 mb-2">
              {isRecording ? (
                <>
                  <span className="relative flex h-2 w-2">
                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
                    <span className="relative inline-flex rounded-full h-2 w-2 bg-green-500"></span>
                  </span>
                  <Radio size={14} className="text-green-600" />
                  <span className="text-xs font-semibold text-green-700">监控运行中</span>
                </>
              ) : (
                <>
                  <EyeOff size={14} className="text-gray-400" />
                  <span className="text-xs font-semibold text-gray-500">监控已暂停</span>
                </>
              )}
            </div>

            {/* 监控内容类型标签 */}
            <div className="flex flex-wrap gap-1">
              <SidebarMonitorBadge icon={<Keyboard size={10} />} label="键盘" active={isRecording} />
              <SidebarMonitorBadge icon={<MousePointerClick size={10} />} label="鼠标" active={isRecording} />
              <SidebarMonitorBadge icon={<ClipboardCopy size={10} />} label="剪贴板" active={isRecording} />
              <SidebarMonitorBadge icon={<AppWindow size={10} />} label="应用" active={isRecording} />
            </div>
          </div>
        </div>

        <nav className="flex-1 px-4 space-y-1">
          <NavLink to="/" className={({ isActive }) => `nav-item ${isActive ? "active" : ""}`} end>
            <LayoutDashboard size={20} />
            <span>总览</span>
          </NavLink>
          <NavLink to="/time" className={({ isActive }) => `nav-item ${isActive ? "active" : ""}`}>
            <Clock size={20} />
            <span>时长统计</span>
          </NavLink>
          <NavLink to="/realtime" className={({ isActive }) => `nav-item ${isActive ? "active" : ""}`}>
            <Activity size={20} />
            <span>实时监控</span>
          </NavLink>
          <NavLink to="/browser" className={({ isActive }) => `nav-item ${isActive ? "active" : ""}`}>
            <FileText size={20} />
            <span>内容浏览</span>
          </NavLink>
          <NavLink to="/reports" className={({ isActive }) => `nav-item ${isActive ? "active" : ""}`}>
            <FileText size={20} />
            <span>报告中心</span>
          </NavLink>
          <NavLink to="/todos" className={({ isActive }) => `nav-item ${isActive ? "active" : ""}`}>
            <CheckSquare size={20} />
            <span>TODO</span>
          </NavLink>
          <NavLink to="/goals" className={({ isActive }) => `nav-item ${isActive ? "active" : ""}`}>
            <Target size={20} />
            <span>目标</span>
          </NavLink>
          <NavLink to="/ideas" className={({ isActive }) => `nav-item ${isActive ? "active" : ""}`}>
            <Lightbulb size={20} />
            <span>闪念</span>
          </NavLink>
          <NavLink to="/settings" className={({ isActive }) => `nav-item ${isActive ? "active" : ""}`}>
            <Settings size={20} />
            <span>设置</span>
          </NavLink>
          <NavLink to="/ime" className={({ isActive }) => `nav-item ${isActive ? "active" : ""}`}>
            <Type size={20} />
            <span>输入法</span>
          </NavLink>
        </nav>

        <div className="p-4 border-t border-gray-200">
          <button
            onClick={toggleRecording}
            className={`w-full flex items-center justify-center gap-2 px-4 py-2 rounded-lg font-medium transition-colors ${
              isRecording
                ? "bg-red-50 text-red-600 hover:bg-red-100"
                : "bg-green-50 text-green-600 hover:bg-green-100"
            }`}
          >
            {isRecording ? <Pause size={18} /> : <Play size={18} />}
            {isRecording ? "暂停记录" : "恢复记录"}
          </button>
        </div>
      </aside>

      {/* 主内容区 */}
      <main className="flex-1 overflow-auto">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/time" element={<TimeStats />} />
          <Route path="/realtime" element={<RealtimeMonitor />} />
          <Route path="/browser" element={<ContentBrowser />} />
          <Route path="/reports" element={<ReportCenter />} />
          <Route path="/todos" element={<TodoList />} />
          <Route path="/goals" element={<Goals />} />
          <Route path="/ideas" element={<FlashIdeas />} />
          <Route path="/settings" element={<SettingsPage />} />
          <Route path="/ime" element={<IMESettings />} />
        </Routes>
      </main>
    </div>
  );
}

function SidebarMonitorBadge({
  icon,
  label,
  active,
}: {
  icon: React.ReactNode;
  label: string;
  active: boolean;
}) {
  return (
    <div
      className={`flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium ${
        active
          ? "bg-blue-100 text-blue-700"
          : "bg-gray-200 text-gray-400"
      }`}
    >
      {icon}
      {label}
    </div>
  );
}

export default App;
