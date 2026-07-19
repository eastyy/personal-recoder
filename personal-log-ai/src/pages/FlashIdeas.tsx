import { useEffect, useState } from "react";
import { Lightbulb, Plus, Trash2, Loader2, Clock } from "lucide-react";
import { api } from "../services/tauri";

interface FlashIdea {
  id: number;
  text: string;
  source_session: string | null;
  captured_at: number;
}

export default function FlashIdeas() {
  const [ideas, setIdeas] = useState<FlashIdea[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAdd, setShowAdd] = useState(false);
  const [newText, setNewText] = useState("");
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadIdeas();
  }, []);

  async function loadIdeas() {
    try {
      setLoading(true);
      const data = await api.getFlashIdeas(200);
      setIdeas((data as FlashIdea[]) || []);
    } catch (e) {
      console.error("Failed to load flash ideas:", e);
    } finally {
      setLoading(false);
    }
  }

  async function handleAdd() {
    if (!newText.trim()) return;
    try {
      setSaving(true);
      await api.addFlashIdea(newText.trim());
      setNewText("");
      setShowAdd(false);
      await loadIdeas();
    } catch (e) {
      console.error("Failed to add flash idea:", e);
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete(id: number) {
    try {
      await api.deleteFlashIdea(id);
      await loadIdeas();
    } catch (e) {
      console.error("Failed to delete flash idea:", e);
    }
  }

  function formatTime(ts: number): string {
    const date = new Date(ts * 1000);
    const now = new Date();
    const diff = Math.floor((now.getTime() - date.getTime()) / 1000);

    if (diff < 60) return "刚刚";
    if (diff < 3600) return `${Math.floor(diff / 60)}分钟前`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}小时前`;
    if (diff < 604800) return `${Math.floor(diff / 86400)}天前`;
    return date.toLocaleDateString("zh-CN", { month: "short", day: "numeric" });
  }

  return (
    <div className="p-8 max-w-3xl">
      <div className="mb-8 flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">闪念</h2>
          <p className="text-gray-500 mt-1">快速记录灵感与想法</p>
        </div>
        <button
          onClick={() => setShowAdd(!showAdd)}
          className="flex items-center gap-2 px-4 py-2 bg-amber-500 text-white rounded-lg hover:bg-amber-600 transition-colors text-sm font-medium"
        >
          <Plus size={18} />
          记录闪念
        </button>
      </div>

      {/* 添加闪念 */}
      {showAdd && (
        <div className="card mb-6 border-2 border-amber-200">
          <div className="flex items-start gap-3">
            <Lightbulb size={20} className="text-amber-500 mt-1 flex-shrink-0" />
            <div className="flex-1">
              <textarea
                value={newText}
                onChange={(e) => setNewText(e.target.value)}
                placeholder="记下你此刻的想法..."
                rows={3}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-amber-500 resize-none"
                autoFocus
              />
              <div className="flex gap-2 mt-3">
                <button
                  onClick={handleAdd}
                  disabled={saving || !newText.trim()}
                  className="flex items-center gap-2 px-4 py-2 bg-amber-500 text-white rounded-lg hover:bg-amber-600 transition-colors text-sm font-medium disabled:opacity-50"
                >
                  {saving ? <Loader2 size={16} className="animate-spin" /> : <Plus size={16} />}
                  保存
                </button>
                <button
                  onClick={() => {
                    setShowAdd(false);
                    setNewText("");
                  }}
                  className="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 text-sm"
                >
                  取消
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 闪念列表 */}
      {loading ? (
        <div className="text-center py-12 text-gray-400">加载中...</div>
      ) : ideas.length > 0 ? (
        <div className="space-y-3">
          {ideas.map((idea) => (
            <div
              key={idea.id}
              className="card border-l-4 border-l-amber-400 hover:shadow-md transition-shadow"
            >
              <div className="flex items-start gap-3">
                <Lightbulb size={18} className="text-amber-500 mt-0.5 flex-shrink-0" />
                <div className="flex-1 min-w-0">
                  <p className="text-sm text-gray-700 whitespace-pre-wrap">{idea.text}</p>
                  <div className="flex items-center gap-2 mt-2">
                    <Clock size={12} className="text-gray-400" />
                    <span className="text-xs text-gray-400">{formatTime(idea.captured_at)}</span>
                    {idea.source_session && (
                      <span className="text-xs text-gray-400">· 来自输入会话</span>
                    )}
                  </div>
                </div>
                <button
                  onClick={() => handleDelete(idea.id)}
                  className="text-gray-300 hover:text-red-500 transition-colors flex-shrink-0"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="card text-center py-12">
          <Lightbulb size={48} className="text-gray-300 mx-auto mb-4" />
          <p className="text-gray-500">还没有闪念</p>
          <p className="text-sm text-gray-400 mt-1">
            点击右上角「记录闪念」快速保存你的灵感
          </p>
        </div>
      )}
    </div>
  );
}
