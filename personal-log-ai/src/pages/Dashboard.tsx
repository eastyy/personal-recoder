import { useEffect, useState } from "react";
import {
  Zap,
  Clock,
  Target,
  Type,
  Radio,
  Keyboard,
  MousePointerClick,
  ClipboardCopy,
  AppWindow,
  Eye,
  EyeOff,
  TrendingUp,
  Activity,
} from "lucide-react";
import { api } from "../services/tauri";
import { useAppStore } from "../stores/appStore";
import { format } from "date-fns";
import { zhCN } from "date-fns/locale";

export default function Dashboard() {
  const { dailyStats, setDailyStats, focusSummary, setFocusSummary, isRecording } = useAppStore();
  const [loading, setLoading] = useState(true);
  const [typingStats, setTypingStats] = useState<Record<string, number> | null>(null);
  const [rhythm, setRhythm] = useState<Array<{ hour: number; char_count: number; key_count: number }>>([]);
  const today = format(new Date(), "yyyy-MM-dd");

  useEffect(() => {
    loadData();
  }, []);

  async function loadData() {
    try {
      setLoading(true);
      const now = Date.now();
      const startToday = new Date();
      startToday.setHours(0, 0, 0, 0);

      const [stats, focus, ts, rh] = await Promise.all([
        api.getDailyStats(today),
        api.getFocusSummary(today),
        api.getTypingStats(Math.floor(startToday.getTime() / 1000), Math.floor(now / 1000)).catch(() => null),
        api.getTypingRhythm(today).catch(() => null),
      ]);
      setDailyStats(stats as Record<string, unknown>);
      setFocusSummary(focus as Record<string, unknown>);
      if (ts) setTypingStats(ts as Record<string, number>);
      if (rh) setRhythm(rh as Array<{ hour: number; char_count: number; key_count: number }>);
    } catch (e) {
      console.error("Failed to load dashboard data:", e);
    } finally {
      setLoading(false);
    }
  }

  const stats = dailyStats || {};
  const focus = focusSummary || {};
  const topApps = (focus.items as Array<{ name: string; duration: number }>) || [];
  const totalSeconds = (focus.total_seconds as number) || 0;
  const totalHours = Math.floor(totalSeconds / 3600);
  const totalMinutes = Math.floor((totalSeconds % 3600) / 60);

  const wpm = typingStats?.avg_wpm ?? 0;
  const totalChars = typingStats?.total_chars ?? (stats.total_input_chars as number) ?? 0;
  const backspaceRate = typingStats?.backspace_rate ?? 0;
  const avgCpm = typingStats?.avg_cpm ?? 0;

  return (
    <div className="p-8">
      {/* 顶部标题 + 监控状态 */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">
            {format(new Date(), "M月d日 EEEE", { locale: zhCN })}
          </h2>
          <p className="text-gray-500 mt-1">今日数据概览</p>
        </div>

        <div className="flex items-center gap-3">
          <div
            className={`flex items-center gap-2 px-4 py-2 rounded-xl border ${
              isRecording
                ? "bg-green-50 border-green-200 text-green-700"
                : "bg-gray-50 border-gray-200 text-gray-500"
            }`}
          >
            {isRecording ? (
              <>
                <span className="relative flex h-2.5 w-2.5">
                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
                  <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-green-500"></span>
                </span>
                <Radio size={16} />
                <span className="text-sm font-medium">监控中</span>
              </>
            ) : (
              <>
                <EyeOff size={16} />
                <span className="text-sm font-medium">已暂停</span>
              </>
            )}
          </div>
        </div>
      </div>

      {/* 监控内容类型说明 */}
      <div className="mb-8 p-4 bg-white border border-gray-200 rounded-xl">
        <div className="flex items-center gap-2 mb-3">
          <Eye size={18} className="text-blue-600" />
          <h3 className="text-sm font-semibold text-gray-900">当前监控内容</h3>
        </div>
        <div className="flex flex-wrap gap-3">
          <MonitorTypeBadge icon={<Keyboard size={14} />} label="键盘输入" active={isRecording} />
          <MonitorTypeBadge icon={<MousePointerClick size={14} />} label="鼠标点击" active={isRecording} />
          <MonitorTypeBadge icon={<ClipboardCopy size={14} />} label="剪贴板变化" active={isRecording} />
          <MonitorTypeBadge icon={<AppWindow size={14} />} label="应用切换" active={isRecording} />
          <MonitorTypeBadge icon={<Clock size={14} />} label="窗口标题" active={isRecording} />
        </div>
        {!isRecording && (
          <p className="mt-3 text-xs text-amber-600 bg-amber-50 px-3 py-2 rounded-lg">
            监控已暂停，点击侧边栏「恢复记录」按钮重新开始采集
          </p>
        )}
      </div>

      {loading ? (
        <div className="flex items-center justify-center h-64">
          <div className="text-gray-500">加载中...</div>
        </div>
      ) : (
        <>
          {/* 统计卡片 */}
          <div className="grid grid-cols-4 gap-6 mb-8">
            <div className="stat-card">
              <div className="flex items-center gap-2 text-blue-600">
                <Type size={20} />
                <span className="stat-label">总输入字数</span>
              </div>
              <div className="stat-value">{totalChars}</div>
              <div className="text-xs text-gray-400">CPM: {avgCpm.toFixed(0)}</div>
            </div>
            <div className="stat-card">
              <div className="flex items-center gap-2 text-green-600">
                <Clock size={20} />
                <span className="stat-label">使用时长</span>
              </div>
              <div className="stat-value">
                {totalHours}h {totalMinutes}m
              </div>
              <div className="text-xs text-gray-400">
                {topApps.length} 个应用
              </div>
            </div>
            <div className="stat-card">
              <div className="flex items-center gap-2 text-purple-600">
                <Target size={20} />
                <span className="stat-label">退格率</span>
              </div>
              <div className="stat-value">{backspaceRate.toFixed(1)}%</div>
              <div className="text-xs text-gray-400">打字修正频率</div>
            </div>
            <div className="stat-card">
              <div className="flex items-center gap-2 text-orange-600">
                <Zap size={20} />
                <span className="stat-label">打字速度</span>
              </div>
              <div className="stat-value">{wpm.toFixed(0)} WPM</div>
              <div className="text-xs text-gray-400">每分钟词数</div>
            </div>
          </div>

          {/* 24h 热力图 */}
          <div className="card mb-6">
            <div className="flex items-center gap-2 mb-4">
              <Activity size={20} className="text-blue-600" />
              <h3 className="text-lg font-semibold text-gray-900">24小时活跃度热力图</h3>
            </div>
            <Heatmap24h rhythm={rhythm} />
          </div>

          {/* 应用时长排行 + AI 洞察 */}
          <div className="grid grid-cols-2 gap-6">
            <div className="card">
              <div className="flex items-center gap-2 mb-4">
                <TrendingUp size={20} className="text-blue-600" />
                <h3 className="text-lg font-semibold text-gray-900">今日应用时长 Top 5</h3>
              </div>
              {topApps.length > 0 ? (
                <div className="space-y-3">
                  {topApps.slice(0, 5).map((app, index) => (
                    <div key={index} className="flex items-center gap-4">
                      <span className="text-sm text-gray-500 w-6">{index + 1}</span>
                      <div className="flex-1">
                        <div className="flex justify-between mb-1">
                          <span className="text-sm font-medium text-gray-700">{app.name}</span>
                          <span className="text-sm text-gray-500">
                            {Math.floor(app.duration / 60)}m
                          </span>
                        </div>
                        <div className="w-full bg-gray-100 rounded-full h-2">
                          <div
                            className="bg-blue-600 h-2 rounded-full transition-all"
                            style={{
                              width: `${Math.min(100, totalSeconds > 0 ? (app.duration / totalSeconds) * 100 : 0)}%`,
                            }}
                          />
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="text-gray-400 text-center py-8">暂无数据</div>
              )}
            </div>

            <div className="card">
              <h3 className="text-lg font-semibold text-gray-900 mb-4">AI 洞察</h3>
              <div className="space-y-4">
                <div className="p-4 bg-blue-50 rounded-lg">
                  <p className="text-sm text-blue-800">
                    配置 AI API Key 后，系统将自动分析你的输入数据，生成生产力报告、主题提取和写作建议。
                  </p>
                </div>
                <div className="p-4 bg-gray-50 rounded-lg">
                  <p className="text-sm text-gray-600">
                    今日数据正在收集中。系统会在每日 03:00 自动生成分析报告，也可在报告中心手动触发。
                  </p>
                </div>
                {wpm > 0 && (
                  <div className="p-4 bg-green-50 rounded-lg">
                    <p className="text-sm text-green-800">
                      今日打字速度 {wpm.toFixed(0)} WPM，退格率 {backspaceRate.toFixed(1)}%。
                      {backspaceRate > 10 ? "退格率偏高，可能需要放慢打字节奏。" : "退格率正常，打字流畅。"}
                    </p>
                  </div>
                )}
              </div>
            </div>
          </div>
        </>
      )}
    </div>
  );
}

function MonitorTypeBadge({
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
      className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium ${
        active
          ? "bg-blue-50 text-blue-700 border border-blue-200"
          : "bg-gray-100 text-gray-400 border border-gray-200"
      }`}
    >
      {icon}
      {label}
    </div>
  );
}

/// 24小时热力图组件
function Heatmap24h({ rhythm }: { rhythm: Array<{ hour: number; char_count: number; key_count: number }> }) {
  const hours = Array.from({ length: 24 }, (_, h) => {
    const data = rhythm.find((r) => r.hour === h);
    return { hour: h, chars: data?.char_count ?? 0, keys: data?.key_count ?? 0 };
  });

  const maxChars = Math.max(...hours.map((h) => h.chars), 1);

  const getIntensity = (chars: number) => {
    if (chars === 0) return 0;
    return Math.min(1, chars / maxChars);
  };

  const getColor = (intensity: number) => {
    if (intensity === 0) return "bg-gray-100";
    if (intensity < 0.25) return "bg-blue-200";
    if (intensity < 0.5) return "bg-blue-400";
    if (intensity < 0.75) return "bg-blue-500";
    return "bg-blue-600";
  };

  return (
    <div>
      <div className="grid grid-cols-24 gap-1" style={{ gridTemplateColumns: "repeat(24, 1fr)" }}>
        {hours.map((h) => {
          const intensity = getIntensity(h.chars);
          return (
            <div key={h.hour} className="text-center">
              <div
                className={`h-12 rounded ${getColor(intensity)} hover:ring-2 hover:ring-blue-300 transition-all cursor-pointer flex items-center justify-center group relative`}
                title={`${h.hour}:00 - ${h.hour}:59\n字符: ${h.chars}\n按键: ${h.keys}`}
              >
                {h.chars > 0 && (
                  <span className="text-[10px] text-white font-medium">{h.chars}</span>
                )}
              </div>
              <div className="text-[9px] text-gray-400 mt-1">
                {h.hour % 3 === 0 ? `${h.hour}h` : ""}
              </div>
            </div>
          );
        })}
      </div>
      <div className="flex items-center justify-between mt-4 text-xs text-gray-500">
        <span>0:00</span>
        <div className="flex items-center gap-2">
          <span>低</span>
          <div className="flex gap-1">
            <div className="w-4 h-3 bg-gray-100 rounded"></div>
            <div className="w-4 h-3 bg-blue-200 rounded"></div>
            <div className="w-4 h-3 bg-blue-400 rounded"></div>
            <div className="w-4 h-3 bg-blue-500 rounded"></div>
            <div className="w-4 h-3 bg-blue-600 rounded"></div>
          </div>
          <span>高</span>
        </div>
        <span>23:59</span>
      </div>
    </div>
  );
}
