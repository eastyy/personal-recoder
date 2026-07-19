import { useEffect, useState } from "react";
import { Target, Plus, Trash2, TrendingUp, Calendar, Loader2 } from "lucide-react";
import { api } from "../services/tauri";

interface Goal {
  id: number;
  title: string;
  metric_type: string;
  target_value: number;
  current_value: number;
  period: string;
  start_date: string | null;
  end_date: string | null;
  created_at: number;
}

const METRIC_TYPES = [
  { value: "char_count", label: "输入字数" },
  { value: "active_hours", label: "活跃时长(小时)" },
  { value: "focus_sessions", label: "专注次数" },
  { value: "todo_completed", label: "完成TODO数" },
];

const PERIODS = [
  { value: "daily", label: "每日" },
  { value: "weekly", label: "每周" },
  { value: "monthly", label: "每月" },
];

const METRIC_COLORS: Record<string, string> = {
  char_count: "text-blue-600",
  active_hours: "text-green-600",
  focus_sessions: "text-purple-600",
  todo_completed: "text-orange-600",
};

export default function Goals() {
  const [goals, setGoals] = useState<Goal[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAdd, setShowAdd] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [newMetric, setNewMetric] = useState("char_count");
  const [newTarget, setNewTarget] = useState("10000");
  const [newPeriod, setNewPeriod] = useState("daily");
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadGoals();
  }, []);

  async function loadGoals() {
    try {
      setLoading(true);
      const data = await api.getGoals();
      setGoals((data as Goal[]) || []);
    } catch (e) {
      console.error("Failed to load goals:", e);
    } finally {
      setLoading(false);
    }
  }

  async function handleAdd() {
    if (!newTitle.trim() || !newTarget) return;
    try {
      setSaving(true);
      await api.addGoal(newTitle.trim(), newMetric, parseInt(newTarget), newPeriod);
      setNewTitle("");
      setShowAdd(false);
      await loadGoals();
    } catch (e) {
      console.error("Failed to add goal:", e);
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete(id: number) {
    try {
      await api.deleteGoal(id);
      await loadGoals();
    } catch (e) {
      console.error("Failed to delete goal:", e);
    }
  }

  async function handleUpdateProgress(id: number, currentValue: number) {
    try {
      await api.updateGoalProgress(id, currentValue);
      await loadGoals();
    } catch (e) {
      console.error("Failed to update progress:", e);
    }
  }

  function getProgressPercent(goal: Goal): number {
    if (goal.target_value <= 0) return 0;
    return Math.min(100, (goal.current_value / goal.target_value) * 100);
  }

  function getPeriodLabel(period: string): string {
    return PERIODS.find((p) => p.value === period)?.label || period;
  }

  function getMetricLabel(metric: string): string {
    return METRIC_TYPES.find((m) => m.value === metric)?.label || metric;
  }

  return (
    <div className="p-8">
      <div className="mb-8 flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">目标追踪</h2>
          <p className="text-gray-500 mt-1">设定个人目标，追踪完成进度</p>
        </div>
        <button
          onClick={() => setShowAdd(!showAdd)}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-sm font-medium"
        >
          <Plus size={18} />
          新建目标
        </button>
      </div>

      {/* 添加目标表单 */}
      {showAdd && (
        <div className="card mb-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">新建目标</h3>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">目标名称</label>
              <input
                type="text"
                value={newTitle}
                onChange={(e) => setNewTitle(e.target.value)}
                placeholder="如：每日写5000字"
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div className="grid grid-cols-3 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">指标类型</label>
                <select
                  value={newMetric}
                  onChange={(e) => setNewMetric(e.target.value)}
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                >
                  {METRIC_TYPES.map((m) => (
                    <option key={m.value} value={m.value}>{m.label}</option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">目标值</label>
                <input
                  type="number"
                  value={newTarget}
                  onChange={(e) => setNewTarget(e.target.value)}
                  min="1"
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">周期</label>
                <select
                  value={newPeriod}
                  onChange={(e) => setNewPeriod(e.target.value)}
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                >
                  {PERIODS.map((p) => (
                    <option key={p.value} value={p.value}>{p.label}</option>
                  ))}
                </select>
              </div>
            </div>
            <div className="flex gap-3">
              <button
                onClick={handleAdd}
                disabled={saving || !newTitle.trim()}
                className="btn-primary flex items-center gap-2 disabled:opacity-50"
              >
                {saving ? <Loader2 size={16} className="animate-spin" /> : <Plus size={16} />}
                创建目标
              </button>
              <button
                onClick={() => setShowAdd(false)}
                className="btn-secondary"
              >
                取消
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 目标列表 */}
      {loading ? (
        <div className="text-center py-12 text-gray-400">加载中...</div>
      ) : goals.length > 0 ? (
        <div className="space-y-4">
          {goals.map((goal) => {
            const pct = getProgressPercent(goal);
            const isComplete = pct >= 100;
            return (
              <div key={goal.id} className="card">
                <div className="flex items-start justify-between mb-3">
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <Target size={18} className={isComplete ? "text-green-600" : "text-blue-600"} />
                      <h3 className="font-semibold text-gray-900">{goal.title}</h3>
                      {isComplete && (
                        <span className="text-xs px-2 py-0.5 bg-green-100 text-green-700 rounded-full">
                          已达成
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-3 mt-1 text-xs text-gray-500">
                      <span className={METRIC_COLORS[goal.metric_type] || "text-gray-600"}>
                        {getMetricLabel(goal.metric_type)}
                      </span>
                      <span className="flex items-center gap-1">
                        <Calendar size={12} />
                        {getPeriodLabel(goal.period)}
                      </span>
                      <span>创建于 {new Date(goal.created_at * 1000).toLocaleDateString("zh-CN")}</span>
                    </div>
                  </div>
                  <button
                    onClick={() => handleDelete(goal.id)}
                    className="text-gray-400 hover:text-red-500 transition-colors"
                  >
                    <Trash2 size={16} />
                  </button>
                </div>

                {/* 进度条 */}
                <div className="mb-3">
                  <div className="flex justify-between items-center mb-1">
                    <span className="text-sm text-gray-600">
                      {goal.current_value.toLocaleString()} / {goal.target_value.toLocaleString()}
                    </span>
                    <span className={`text-sm font-medium ${isComplete ? "text-green-600" : "text-blue-600"}`}>
                      {pct.toFixed(0)}%
                    </span>
                  </div>
                  <div className="w-full bg-gray-100 rounded-full h-3">
                    <div
                      className={`h-3 rounded-full transition-all ${isComplete ? "bg-green-500" : "bg-blue-600"}`}
                      style={{ width: `${pct}%` }}
                    />
                  </div>
                </div>

                {/* 手动更新进度 */}
                <div className="flex items-center gap-2">
                  <TrendingUp size={14} className="text-gray-400" />
                  <input
                    type="number"
                    value={goal.current_value}
                    onChange={(e) => {
                      const val = parseInt(e.target.value) || 0;
                      handleUpdateProgress(goal.id, val);
                    }}
                    className="w-32 px-2 py-1 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500"
                  />
                  <span className="text-xs text-gray-400">当前进度（可手动修改）</span>
                </div>
              </div>
            );
          })}
        </div>
      ) : (
        <div className="card text-center py-12">
          <Target size={48} className="text-gray-300 mx-auto mb-4" />
          <p className="text-gray-500">还没有目标</p>
          <p className="text-sm text-gray-400 mt-1">点击右上角「新建目标」开始追踪</p>
        </div>
      )}
    </div>
  );
}
