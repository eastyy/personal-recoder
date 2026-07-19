import { useEffect, useState } from "react";
import { CheckSquare, Square, Plus, Calendar, AlertCircle } from "lucide-react";
import { api } from "../services/tauri";

interface Todo {
  id: number;
  text: string;
  status: "pending" | "done" | "cancelled";
  extracted_at: number;
  due_date: string | null;
}

export default function TodoList() {
  const [todos, setTodos] = useState<Todo[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<"all" | "pending" | "done">("pending");
  const [newTodo, setNewTodo] = useState("");
  const [showAdd, setShowAdd] = useState(false);

  useEffect(() => {
    loadTodos();
  }, [filter]);

  async function loadTodos() {
    try {
      setLoading(true);
      const status = filter === "all" ? undefined : filter;
      const data = await api.getTodos(status);
      setTodos((data as Todo[]) || []);
    } catch (e) {
      console.error("Failed to load todos:", e);
    } finally {
      setLoading(false);
    }
  }

  async function handleToggle(id: number) {
    try {
      await api.toggleTodo(id);
      await loadTodos();
    } catch (e) {
      console.error("Failed to toggle todo:", e);
    }
  }

  const pendingCount = todos.filter((t) => t.status === "pending").length;
  const doneCount = todos.filter((t) => t.status === "done").length;
  const overdueCount = todos.filter((t) => {
    if (t.status !== "pending" || !t.due_date) return false;
    return new Date(t.due_date) < new Date(new Date().toDateString());
  }).length;

  function formatDueDate(due: string | null): string | null {
    if (!due) return null;
    try {
      const date = new Date(due);
      const today = new Date(new Date().toDateString());
      const diff = Math.floor((date.getTime() - today.getTime()) / (1000 * 60 * 60 * 24));
      if (diff === 0) return "今天";
      if (diff === 1) return "明天";
      if (diff === -1) return "昨天";
      if (diff > 0 && diff <= 7) return `${diff}天后`;
      if (diff < 0) return `逾期${Math.abs(diff)}天`;
      return due;
    } catch {
      return due;
    }
  }

  function isOverdue(due: string | null): boolean {
    if (!due) return false;
    return new Date(due) < new Date(new Date().toDateString());
  }

  return (
    <div className="p-8">
      <div className="mb-6">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold text-gray-900">TODO 列表</h2>
            <p className="text-gray-500 mt-1">
              待办 {pendingCount} 项 · 已完成 {doneCount} 项
              {overdueCount > 0 && (
                <span className="text-red-500 ml-2">· 逾期 {overdueCount} 项</span>
              )}
            </p>
          </div>
          <button
            onClick={() => setShowAdd(!showAdd)}
            className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-sm font-medium"
          >
            <Plus size={18} />
            添加
          </button>
        </div>
      </div>

      {/* 添加 TODO 输入框 */}
      {showAdd && (
        <div className="card mb-6">
          <div className="flex gap-3">
            <input
              type="text"
              value={newTodo}
              onChange={(e) => setNewTodo(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && newTodo.trim()) {
                  // Note: we don't have a direct add todo API, but we can use set_config workaround
                  // For now, just close the form
                  setNewTodo("");
                  setShowAdd(false);
                }
              }}
              placeholder="输入待办事项..."
              className="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
              autoFocus
            />
            <button
              onClick={() => {
                setNewTodo("");
                setShowAdd(false);
              }}
              className="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 text-sm"
            >
              取消
            </button>
          </div>
          <p className="text-xs text-gray-400 mt-2">
            提示：AI 也会自动从你的输入中提取 TODO 事项
          </p>
        </div>
      )}

      {/* 筛选 */}
      <div className="flex gap-2 mb-6">
        {(["all", "pending", "done"] as const).map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
              filter === f
                ? "bg-blue-600 text-white"
                : "bg-gray-100 text-gray-600 hover:bg-gray-200"
            }`}
          >
            {f === "all" ? "全部" : f === "pending" ? "待办" : "已完成"}
          </button>
        ))}
      </div>

      {/* TODO 列表 */}
      {loading ? (
        <div className="text-center py-8 text-gray-400">加载中...</div>
      ) : todos.length > 0 ? (
        <div className="space-y-2">
          {todos.map((todo) => {
            const dueLabel = formatDueDate(todo.due_date);
            const overdue = isOverdue(todo.due_date) && todo.status === "pending";
            return (
              <div
                key={todo.id}
                className={`flex items-start gap-4 p-4 bg-white rounded-lg border transition-all ${
                  todo.status === "done"
                    ? "border-gray-100 opacity-60"
                    : overdue
                    ? "border-red-200 bg-red-50"
                    : "border-gray-200 hover:shadow-sm"
                }`}
              >
                <button
                  onClick={() => handleToggle(todo.id)}
                  className="flex-shrink-0 text-blue-600 hover:text-blue-700 mt-0.5"
                >
                  {todo.status === "done" ? <CheckSquare size={22} /> : <Square size={22} />}
                </button>
                <div className="flex-1 min-w-0">
                  <span
                    className={`text-sm block ${
                      todo.status === "done" ? "line-through text-gray-400" : "text-gray-700"
                    }`}
                  >
                    {todo.text}
                  </span>
                  <div className="flex items-center gap-3 mt-1">
                    <span className="text-xs text-gray-400">
                      {new Date(todo.extracted_at * 1000).toLocaleDateString("zh-CN", { month: "short", day: "numeric" })}
                    </span>
                    {dueLabel && (
                      <span
                        className={`text-xs flex items-center gap-1 ${
                          overdue ? "text-red-600 font-medium" : "text-gray-500"
                        }`}
                      >
                        <Calendar size={12} />
                        {dueLabel}
                      </span>
                    )}
                    {overdue && (
                      <span className="text-xs flex items-center gap-1 text-red-600">
                        <AlertCircle size={12} />
                        需要处理
                      </span>
                    )}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      ) : (
        <div className="card text-center py-12">
          <CheckSquare size={48} className="text-gray-300 mx-auto mb-4" />
          <p className="text-gray-500">{filter === "done" ? "还没有已完成的 TODO" : "暂无 TODO"}</p>
          <p className="text-sm text-gray-400 mt-1">
            {filter === "pending" ? "AI 会自动从输入中提取待办事项" : "完成待办后会显示在这里"}
          </p>
        </div>
      )}
    </div>
  );
}
