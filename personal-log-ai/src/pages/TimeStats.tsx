import { useEffect, useState, useCallback, useMemo } from "react";
import {
  Clock,
  Monitor,
  Globe,
  Calendar,
  Repeat,
  Layers,
  TrendingUp,
  TrendingDown,
  Minus,
} from "lucide-react";
import { api } from "../services/tauri";
import {
  format,
  startOfDay,
  endOfDay,
  subDays,
  parseISO,
  isToday,
} from "date-fns";

interface UsageItem {
  target_type: string;
  target_id: string;
  target_name: string;
  duration: number;
}

interface SwitchingStats {
  total_switches: number;
  unique_apps: number;
  avg_session_minutes: number;
}

// Color palette for pie chart segments
const PIE_COLORS = [
  "#3b82f6",
  "#10b981",
  "#8b5cf6",
  "#f59e0b",
  "#ef4444",
  "#06b6d4",
  "#ec4899",
  "#84cc16",
  "#f97316",
  "#6366f1",
];

function formatDuration(seconds: number): string {
  if (seconds <= 0) return "0m";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

function getDayRange(date: Date): { start: number; end: number } {
  return {
    start: Math.floor(startOfDay(date).getTime() / 1000),
    end: Math.floor(endOfDay(date).getTime() / 1000),
  };
}

export default function TimeStats() {
  const [selectedDate, setSelectedDate] = useState(
    format(new Date(), "yyyy-MM-dd")
  );
  const [usage, setUsage] = useState<UsageItem[]>([]);
  const [yesterdayUsage, setYesterdayUsage] = useState<UsageItem[]>([]);
  const [switching, setSwitching] = useState<SwitchingStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadingYesterday, setLoadingYesterday] = useState(true);
  const [loadingSwitching, setLoadingSwitching] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const selectedDateObj = useMemo(() => {
    try {
      return parseISO(selectedDate);
    } catch {
      return new Date();
    }
  }, [selectedDate]);

  const loadData = useCallback(async (dateStr: string) => {
    const dateObj = parseISO(dateStr);
    const { start, end } = getDayRange(dateObj);

    // Load selected day's usage
    setLoading(true);
    setError(null);
    try {
      const data = await api.getAppUsage(start, end);
      setUsage((data as UsageItem[]) || []);
    } catch (e) {
      console.error("Failed to load usage:", e);
      setError("加载使用数据失败");
      setUsage([]);
    } finally {
      setLoading(false);
    }

    // Load yesterday's usage for comparison (only if selected date is today)
    if (isToday(dateObj)) {
      setLoadingYesterday(true);
      try {
        const yesterday = subDays(dateObj, 1);
        const yRange = getDayRange(yesterday);
        const yData = await api.getAppUsage(yRange.start, yRange.end);
        setYesterdayUsage((yData as UsageItem[]) || []);
      } catch (e) {
        console.error("Failed to load yesterday usage:", e);
        setYesterdayUsage([]);
      } finally {
        setLoadingYesterday(false);
      }
    } else {
      setYesterdayUsage([]);
      setLoadingYesterday(false);
    }

    // Load switching stats
    setLoadingSwitching(true);
    try {
      const swData = await api.getSwitchingStats(dateStr);
      setSwitching(swData as SwitchingStats);
    } catch (e) {
      console.error("Failed to load switching stats:", e);
      setSwitching(null);
    } finally {
      setLoadingSwitching(false);
    }
  }, []);

  useEffect(() => {
    loadData(selectedDate);
  }, [selectedDate, loadData]);

  // Derived data
  const appUsage = usage.filter((u) => u.target_type === "app");
  const domainUsage = usage.filter((u) => u.target_type === "domain");
  const totalSeconds = usage.reduce((sum, u) => sum + u.duration, 0);
  const yesterdayTotalSeconds = yesterdayUsage.reduce(
    (sum, u) => sum + u.duration,
    0
  );

  // Today vs yesterday comparison
  const diffSeconds = totalSeconds - yesterdayTotalSeconds;
  const diffPercent =
    yesterdayTotalSeconds > 0
      ? ((diffSeconds / yesterdayTotalSeconds) * 100)
      : 0;

  // Pie chart data - top apps by usage
  const pieData = useMemo(() => {
    const sorted = [...appUsage].sort((a, b) => b.duration - a.duration);
    const top = sorted.slice(0, 7);
    const otherSeconds = sorted.slice(7).reduce((s, a) => s + a.duration, 0);
    const pieTotal = top.reduce((s, a) => s + a.duration, 0) + otherSeconds;

    const segments = top.map((a, i) => ({
      name: a.target_name,
      duration: a.duration,
      color: PIE_COLORS[i % PIE_COLORS.length],
      pct: pieTotal > 0 ? (a.duration / pieTotal) * 100 : 0,
    }));

    if (otherSeconds > 0) {
      segments.push({
        name: "其他",
        duration: otherSeconds,
        color: "#9ca3af",
        pct: pieTotal > 0 ? (otherSeconds / pieTotal) * 100 : 0,
      });
    }

    return { segments, total: pieTotal };
  }, [appUsage]);

  // Build conic-gradient string
  const conicGradient = useMemo(() => {
    if (pieData.segments.length === 0 || pieData.total === 0) {
      return "conic-gradient(#e5e7eb 0% 100%)";
    }
    let acc = 0;
    const stops = pieData.segments.map((seg) => {
      const startPct = acc;
      acc += seg.pct;
      return `${seg.color} ${startPct}% ${acc}%`;
    });
    return `conic-gradient(${stops.join(", ")})`;
  }, [pieData]);

  // Max value for bar scaling
  const maxAppDuration = Math.max(...appUsage.map((a) => a.duration), 1);
  const maxDomainDuration = Math.max(...domainUsage.map((d) => d.duration), 1);

  return (
    <div className="p-8 max-w-7xl mx-auto">
      {/* Header with date picker */}
      <div className="mb-8 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">时长统计</h2>
          <p className="text-gray-500 mt-1">
            {format(selectedDateObj, "yyyy年MM月dd日 EEEE")} 使用情况
          </p>
        </div>
        <div className="flex items-center gap-2">
          <div className="relative">
            <Calendar
              size={18}
              className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 pointer-events-none z-10"
            />
            <input
              type="date"
              value={selectedDate}
              max={format(new Date(), "yyyy-MM-dd")}
              onChange={(e) => {
                if (e.target.value) setSelectedDate(e.target.value);
              }}
              className="pl-10 pr-3 py-2 rounded-lg border border-gray-200 bg-white text-sm text-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent shadow-sm"
            />
          </div>
        </div>
      </div>

      {error && (
        <div className="mb-6 rounded-lg bg-red-50 border border-red-200 px-4 py-3 text-sm text-red-600">
          {error}
        </div>
      )}

      {/* Top row: total time + comparison + switching stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
        {/* Total time card */}
        <div className="card flex items-center gap-4">
          <div className="p-4 bg-blue-100 rounded-xl shrink-0">
            <Clock size={28} className="text-blue-600" />
          </div>
          <div className="min-w-0">
            {loading ? (
              <div className="space-y-2">
                <div className="h-8 w-24 bg-gray-100 rounded animate-pulse" />
                <div className="h-4 w-16 bg-gray-100 rounded animate-pulse" />
              </div>
            ) : (
              <>
                <div className="text-2xl font-bold text-gray-900">
                  {formatDuration(totalSeconds)}
                </div>
                <div className="text-sm text-gray-500">总使用时长</div>
              </>
            )}
          </div>
        </div>

        {/* Today vs Yesterday comparison */}
        <div className="card flex items-center gap-4">
          <div
            className={`p-4 rounded-xl shrink-0 ${
              diffSeconds > 0
                ? "bg-orange-100"
                : diffSeconds < 0
                ? "bg-green-100"
                : "bg-gray-100"
            }`}
          >
            {diffSeconds > 0 ? (
              <TrendingUp
                size={28}
                className="text-orange-500"
              />
            ) : diffSeconds < 0 ? (
              <TrendingDown size={28} className="text-green-500" />
            ) : (
              <Minus size={28} className="text-gray-400" />
            )}
          </div>
          <div className="min-w-0">
            {loading || loadingYesterday ? (
              <div className="space-y-2">
                <div className="h-8 w-28 bg-gray-100 rounded animate-pulse" />
                <div className="h-4 w-20 bg-gray-100 rounded animate-pulse" />
              </div>
            ) : isToday(selectedDateObj) && yesterdayTotalSeconds > 0 ? (
              <>
                <div className="text-2xl font-bold text-gray-900">
                  {diffSeconds >= 0 ? "+" : ""}
                  {formatDuration(Math.abs(diffSeconds))}
                </div>
                <div className="text-sm text-gray-500">
                  vs 昨日{" "}
                  <span
                    className={
                      diffSeconds > 0
                        ? "text-orange-500"
                        : diffSeconds < 0
                        ? "text-green-500"
                        : "text-gray-400"
                    }
                  >
                    {diffPercent >= 0 ? "+" : ""}
                    {diffPercent.toFixed(1)}%
                  </span>
                </div>
              </>
            ) : (
              <>
                <div className="text-lg font-semibold text-gray-400">
                  {isToday(selectedDateObj)
                    ? "无昨日数据"
                    : "仅今日可对比"}
                </div>
                <div className="text-sm text-gray-400">vs 昨日</div>
              </>
            )}
          </div>
        </div>

        {/* Switching stats card */}
        <div className="card flex items-center gap-4">
          <div className="p-4 bg-purple-100 rounded-xl shrink-0">
            <Repeat size={28} className="text-purple-600" />
          </div>
          <div className="min-w-0 flex-1">
            {loadingSwitching ? (
              <div className="space-y-2">
                <div className="h-8 w-20 bg-gray-100 rounded animate-pulse" />
                <div className="h-4 w-16 bg-gray-100 rounded animate-pulse" />
              </div>
            ) : switching ? (
              <div className="flex items-baseline gap-4">
                <div>
                  <div className="text-2xl font-bold text-gray-900">
                    {switching.total_switches}
                  </div>
                  <div className="text-xs text-gray-500">切换次数</div>
                </div>
                <div className="w-px h-8 bg-gray-200" />
                <div>
                  <div className="text-2xl font-bold text-gray-900">
                    {switching.unique_apps}
                  </div>
                  <div className="text-xs text-gray-500">应用数</div>
                </div>
                <div className="w-px h-8 bg-gray-200" />
                <div>
                  <div className="text-2xl font-bold text-gray-900">
                    {switching.avg_session_minutes.toFixed(1)}
                    <span className="text-sm font-normal text-gray-400 ml-1">
                      min
                    </span>
                  </div>
                  <div className="text-xs text-gray-500">平均时长</div>
                </div>
              </div>
            ) : (
              <>
                <div className="text-lg font-semibold text-gray-400">
                  无切换数据
                </div>
                <div className="text-sm text-gray-400">应用切换统计</div>
              </>
            )}
          </div>
        </div>
      </div>

      {/* Middle row: bar charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        {/* App usage Top 10 */}
        <div className="card">
          <div className="flex items-center gap-2 mb-4">
            <Monitor size={20} className="text-blue-600" />
            <h3 className="text-lg font-semibold text-gray-900">
              应用时长 Top 10
            </h3>
          </div>
          {loading ? (
            <div className="space-y-3">
              {[...Array(5)].map((_, i) => (
                <div key={i} className="flex items-center gap-4">
                  <div className="w-6 h-4 bg-gray-100 rounded animate-pulse" />
                  <div className="flex-1 space-y-1">
                    <div className="flex justify-between">
                      <div className="h-4 w-24 bg-gray-100 rounded animate-pulse" />
                      <div className="h-4 w-10 bg-gray-100 rounded animate-pulse" />
                    </div>
                    <div className="h-2 w-full bg-gray-100 rounded-full animate-pulse" />
                  </div>
                </div>
              ))}
            </div>
          ) : appUsage.length > 0 ? (
            <div className="space-y-3">
              {appUsage.slice(0, 10).map((app, index) => (
                <div key={index} className="flex items-center gap-4">
                  <span className="text-sm text-gray-400 w-6 text-right">
                    {index + 1}
                  </span>
                  <div className="flex-1 min-w-0">
                    <div className="flex justify-between mb-1">
                      <span
                        className="text-sm font-medium text-gray-700 truncate"
                        title={app.target_name}
                      >
                        {app.target_name}
                      </span>
                      <span className="text-sm text-gray-500 ml-2 shrink-0">
                        {formatDuration(app.duration)}
                      </span>
                    </div>
                    <div className="w-full bg-gray-100 rounded-full h-2 overflow-hidden">
                      <div
                        className="bg-blue-600 h-2 rounded-full transition-all duration-500"
                        style={{
                          width: `${Math.min(
                            100,
                            (app.duration / maxAppDuration) * 100
                          )}%`,
                        }}
                      />
                    </div>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <EmptyState
              icon={<Monitor size={40} className="text-gray-300" />}
              text="暂无应用使用数据"
            />
          )}
        </div>

        {/* Website usage Top 10 */}
        <div className="card">
          <div className="flex items-center gap-2 mb-4">
            <Globe size={20} className="text-green-600" />
            <h3 className="text-lg font-semibold text-gray-900">
              网站时长 Top 10
            </h3>
          </div>
          {loading ? (
            <div className="space-y-3">
              {[...Array(5)].map((_, i) => (
                <div key={i} className="flex items-center gap-4">
                  <div className="w-6 h-4 bg-gray-100 rounded animate-pulse" />
                  <div className="flex-1 space-y-1">
                    <div className="flex justify-between">
                      <div className="h-4 w-24 bg-gray-100 rounded animate-pulse" />
                      <div className="h-4 w-10 bg-gray-100 rounded animate-pulse" />
                    </div>
                    <div className="h-2 w-full bg-gray-100 rounded-full animate-pulse" />
                  </div>
                </div>
              ))}
            </div>
          ) : domainUsage.length > 0 ? (
            <div className="space-y-3">
              {domainUsage.slice(0, 10).map((site, index) => (
                <div key={index} className="flex items-center gap-4">
                  <span className="text-sm text-gray-400 w-6 text-right">
                    {index + 1}
                  </span>
                  <div className="flex-1 min-w-0">
                    <div className="flex justify-between mb-1">
                      <span
                        className="text-sm font-medium text-gray-700 truncate"
                        title={site.target_name}
                      >
                        {site.target_name}
                      </span>
                      <span className="text-sm text-gray-500 ml-2 shrink-0">
                        {formatDuration(site.duration)}
                      </span>
                    </div>
                    <div className="w-full bg-gray-100 rounded-full h-2 overflow-hidden">
                      <div
                        className="bg-green-600 h-2 rounded-full transition-all duration-500"
                        style={{
                          width: `${Math.min(
                            100,
                            (site.duration / maxDomainDuration) * 100
                          )}%`,
                        }}
                      />
                    </div>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <EmptyState
              icon={<Globe size={40} className="text-gray-300" />}
              text="暂无网站使用数据"
            />
          )}
        </div>
      </div>

      {/* Bottom row: Pie chart */}
      <div className="card">
        <div className="flex items-center gap-2 mb-6">
          <Layers size={20} className="text-violet-600" />
          <h3 className="text-lg font-semibold text-gray-900">
            应用使用占比分布
          </h3>
        </div>
        {loading ? (
          <div className="flex flex-col md:flex-row items-center gap-8">
            <div
              className="w-48 h-48 rounded-full bg-gray-100 animate-pulse shrink-0"
            />
            <div className="flex-1 space-y-3 w-full">
              {[...Array(5)].map((_, i) => (
                <div key={i} className="flex items-center gap-3">
                  <div className="w-4 h-4 bg-gray-100 rounded animate-pulse" />
                  <div className="h-4 flex-1 bg-gray-100 rounded animate-pulse" />
                </div>
              ))}
            </div>
          </div>
        ) : pieData.segments.length > 0 && pieData.total > 0 ? (
          <div className="flex flex-col md:flex-row items-center gap-8">
            {/* Pie chart */}
            <div className="relative shrink-0">
              <div
                className="w-48 h-48 rounded-full shadow-lg"
                style={{ background: conicGradient }}
              />
              {/* Center label */}
              <div className="absolute inset-0 flex flex-col items-center justify-center">
                <div className="bg-white rounded-full w-24 h-24 flex flex-col items-center justify-center shadow-inner">
                  <span className="text-xs text-gray-400">总计</span>
                  <span className="text-sm font-bold text-gray-700">
                    {formatDuration(pieData.total)}
                  </span>
                </div>
              </div>
            </div>

            {/* Legend */}
            <div className="flex-1 space-y-2 w-full">
              {pieData.segments.map((seg, i) => (
                <div
                  key={i}
                  className="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-gray-50 transition-colors"
                >
                  <div
                    className="w-3 h-3 rounded-full shrink-0"
                    style={{ backgroundColor: seg.color }}
                  />
                  <span
                    className="text-sm text-gray-700 flex-1 truncate"
                    title={seg.name}
                  >
                    {seg.name}
                  </span>
                  <span className="text-sm font-medium text-gray-500 ml-2">
                    {seg.pct.toFixed(1)}%
                  </span>
                  <span className="text-xs text-gray-400 w-20 text-right">
                    {formatDuration(seg.duration)}
                  </span>
                </div>
              ))}
            </div>
          </div>
        ) : (
          <EmptyState
            icon={<Layers size={40} className="text-gray-300" />}
            text="暂无应用占比数据"
          />
        )}
      </div>
    </div>
  );
}

function EmptyState({
  icon,
  text,
}: {
  icon: React.ReactNode;
  text: string;
}) {
  return (
    <div className="flex flex-col items-center justify-center py-12 text-gray-400">
      {icon}
      <p className="mt-3 text-sm">{text}</p>
    </div>
  );
}
