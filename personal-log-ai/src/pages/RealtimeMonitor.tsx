import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  Activity,
  CheckCircle,
  XCircle,
  Keyboard,
  MousePointerClick,
  ClipboardCopy,
  AppWindow,
  Clock,
  Shield,
  Radio,
  EyeOff,
} from "lucide-react";
import { api } from "../services/tauri";
import { useAppStore } from "../stores/appStore";

interface EventItem {
  id: number;
  timestamp: number;
  event_type: string;
  app_name?: string;
  content?: string;
}

const EVENT_TYPE_LABELS: Record<string, { label: string; icon: React.ReactNode; color: string }> = {
  keydown: { label: "键盘", icon: <Keyboard size={14} />, color: "text-blue-600" },
  keyup: { label: "键盘", icon: <Keyboard size={14} />, color: "text-blue-600" },
  mouse_click: { label: "鼠标", icon: <MousePointerClick size={14} />, color: "text-green-600" },
  clipboard: { label: "剪贴板", icon: <ClipboardCopy size={14} />, color: "text-purple-600" },
  app_focus: { label: "应用切换", icon: <AppWindow size={14} />, color: "text-orange-600" },
  session_end: { label: "会话结束", icon: <Clock size={14} />, color: "text-gray-600" },
  ime_committed: { label: "中文输入", icon: <Keyboard size={14} />, color: "text-cyan-600" },
  ime_direct: { label: "直接输入", icon: <Keyboard size={14} />, color: "text-teal-600" },
};

export default function RealtimeMonitor() {
  const { isRecording } = useAppStore();
  const [status, setStatus] = useState<Record<string, unknown>>({});
  const [events, setEvents] = useState<EventItem[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // 初始加载
    loadData();

    // 监听后端实时推送事件
    const unlisten = listen("realtime_update", (event) => {
      const payload = event.payload as Record<string, unknown>;
      setStatus(payload);
      setLoading(false);
    });

    // 每 5 秒刷新一次事件列表（减少轮询频率）
    const interval = setInterval(() => {
      refreshEvents();
    }, 5000);

    return () => {
      unlisten.then((fn) => fn());
      clearInterval(interval);
    };
  }, []);

  async function loadData() {
    try {
      const [s, e] = await Promise.all([
        api.getRealtimeStatus(),
        api.queryEvents({ limit: 20 }),
      ]);
      setStatus(s as Record<string, unknown>);
      setEvents((e as EventItem[]) || []);
    } catch (err) {
      console.error("Failed to load realtime data:", err);
    } finally {
      setLoading(false);
    }
  }

  async function refreshEvents() {
    try {
      const e = await api.queryEvents({ limit: 20 });
      setEvents((e as EventItem[]) || []);
    } catch (err) {
      console.error("Failed to refresh events:", err);
    }
  }

  const appName = status.app_name as string;
  const todayEvents = status.today_events as number;
  const windowTitle = status.window_title as string | undefined;

  return (
    <div className="p-8">
      <div className="mb-8">
        <h2 className="text-2xl font-bold text-gray-900">实时监控</h2>
        <p className="text-gray-500 mt-1">查看当前记录状态与最近事件</p>
      </div>

      {/* 监控状态总览 */}
      <div className="mb-6 p-5 bg-white border border-gray-200 rounded-xl">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            {isRecording ? (
              <>
                <span className="relative flex h-3 w-3">
                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
                  <span className="relative inline-flex rounded-full h-3 w-3 bg-green-500"></span>
                </span>
                <Radio size={18} className="text-green-600" />
                <span className="font-semibold text-green-700">监控运行中</span>
              </>
            ) : (
              <>
                <EyeOff size={18} className="text-gray-400" />
                <span className="font-semibold text-gray-500">监控已暂停</span>
              </>
            )}
          </div>
          <div className="flex items-center gap-2 text-xs text-gray-500">
            <Shield size={14} />
            <span>所有数据本地存储，不上传服务器</span>
          </div>
        </div>

        {/* 监控内容类型 */}
        <div className="grid grid-cols-5 gap-3">
          <MonitorTypeCard
            icon={<Keyboard size={18} />}
            label="键盘输入"
            active={isRecording}
            description="记录按键字符与退格次数"
          />
          <MonitorTypeCard
            icon={<MousePointerClick size={18} />}
            label="鼠标点击"
            active={isRecording}
            description="记录点击位置与坐标"
          />
          <MonitorTypeCard
            icon={<ClipboardCopy size={18} />}
            label="剪贴板"
            active={isRecording}
            description="记录剪贴板内容变化"
          />
          <MonitorTypeCard
            icon={<AppWindow size={18} />}
            label="应用切换"
            active={isRecording}
            description="记录活跃应用与窗口"
          />
          <MonitorTypeCard
            icon={<Clock size={18} />}
            label="专注时长"
            active={isRecording}
            description="追踪各应用使用时长"
          />
        </div>
      </div>

      {/* 状态卡片 */}
      <div className="grid grid-cols-3 gap-6 mb-6">
        <div className="card flex items-center gap-4">
          <div className={`p-3 rounded-xl ${isRecording ? "bg-green-100" : "bg-red-100"}`}>
            <Activity size={24} className={isRecording ? "text-green-600" : "text-red-600"} />
          </div>
          <div>
            <div className="text-lg font-semibold text-gray-900">
              {isRecording ? "正在记录" : "已暂停"}
            </div>
            <div className="text-sm text-gray-500">采集状态</div>
          </div>
        </div>

        <div className="card flex items-center gap-4">
          <div className="p-3 bg-blue-100 rounded-xl">
            <CheckCircle size={24} className="text-blue-600" />
          </div>
          <div>
            <div className="text-lg font-semibold text-gray-900">{todayEvents}</div>
            <div className="text-sm text-gray-500">今日事件数</div>
          </div>
        </div>

        <div className="card flex items-center gap-4">
          <div className="p-3 bg-purple-100 rounded-xl">
            <AppWindow size={24} className="text-purple-600" />
          </div>
          <div className="min-w-0">
            <div className="text-lg font-semibold text-gray-900 truncate">{appName || "--"}</div>
            <div className="text-sm text-gray-500 truncate">
              {windowTitle || "当前应用"}
            </div>
          </div>
        </div>
      </div>

      {/* 事件列表 */}
      <div className="card">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">最近事件</h3>
        {loading ? (
          <div className="text-center py-8 text-gray-400">加载中...</div>
        ) : events.length > 0 ? (
          <div className="space-y-2 max-h-96 overflow-auto">
            {events.map((event) => {
              const typeInfo = EVENT_TYPE_LABELS[event.event_type] || {
                label: event.event_type,
                icon: <XCircle size={14} />,
                color: "text-gray-500",
              };
              return (
                <div
                  key={event.id}
                  className="flex items-center gap-4 p-3 bg-gray-50 rounded-lg"
                >
                  <div className={`flex-shrink-0 ${typeInfo.color}`}>
                    {typeInfo.icon}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="text-xs font-medium px-2 py-0.5 bg-white rounded border border-gray-200">
                        {typeInfo.label}
                      </span>
                      <span className="text-xs text-gray-500">
                        {new Date(event.timestamp).toLocaleTimeString()}
                      </span>
                      <span className="text-xs font-medium text-blue-600">
                        {event.app_name || "Unknown"}
                      </span>
                    </div>
                    {event.content && (
                      <div className="text-sm text-gray-700 truncate mt-1">{event.content}</div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        ) : (
          <div className="text-gray-400 text-center py-8">暂无事件数据</div>
        )}
      </div>
    </div>
  );
}

function MonitorTypeCard({
  icon,
  label,
  active,
  description,
}: {
  icon: React.ReactNode;
  label: string;
  active: boolean;
  description: string;
}) {
  return (
    <div
      className={`p-3 rounded-lg border text-center transition-all ${
        active
          ? "bg-blue-50 border-blue-200"
          : "bg-gray-50 border-gray-200 opacity-60"
      }`}
    >
      <div
        className={`inline-flex items-center justify-center w-8 h-8 rounded-lg mb-2 ${
          active ? "bg-blue-100 text-blue-600" : "bg-gray-200 text-gray-400"
        }`}
      >
        {icon}
      </div>
      <div className={`text-xs font-medium ${active ? "text-blue-700" : "text-gray-500"}`}>
        {label}
      </div>
      <div className="text-[10px] text-gray-400 mt-0.5">{description}</div>
    </div>
  );
}
