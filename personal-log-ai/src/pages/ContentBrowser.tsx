import { useEffect, useState } from "react";
import { Search, FileText, ChevronRight, ChevronLeft, Filter, Calendar } from "lucide-react";
import { api } from "../services/tauri";
import { format } from "date-fns";

interface EventItem {
  id: number;
  timestamp: number;
  event_type: string;
  app_name: string;
  app_bundle_id: string;
  window_title: string;
  content: string;
  session_id: string;
}

const EVENT_TYPE_LABELS: Record<string, string> = {
  keydown: "键盘输入",
  keyup: "键盘松开",
  mouse_click: "鼠标点击",
  clipboard: "剪贴板",
  app_focus: "应用切换",
  paste: "粘贴",
  session_end: "会话结束",
  ime_committed: "IME提交",
  ime_direct: "IME直接",
};

const EVENT_TYPE_COLORS: Record<string, string> = {
  keydown: "text-blue-600 bg-blue-50",
  keyup: "text-gray-400 bg-gray-50",
  mouse_click: "text-green-600 bg-green-50",
  clipboard: "text-purple-600 bg-purple-50",
  app_focus: "text-orange-600 bg-orange-50",
  paste: "text-pink-600 bg-pink-50",
  session_end: "text-indigo-600 bg-indigo-50",
  ime_committed: "text-cyan-600 bg-cyan-50",
  ime_direct: "text-teal-600 bg-teal-50",
};

export default function ContentBrowser() {
  const [events, setEvents] = useState<EventItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [page, setPage] = useState(0);
  const [hasMore, setHasMore] = useState(true);
  const [filterType, setFilterType] = useState<string>("all");
  const [searchText, setSearchText] = useState("");
  const [selectedDate, setSelectedDate] = useState(format(new Date(), "yyyy-MM-dd"));

  const pageSize = 50;

  useEffect(() => {
    setPage(0);
    loadEvents();
  }, [filterType, selectedDate]);

  async function loadEvents(pageNum = page) {
    try {
      setLoading(true);
      const params: Record<string, unknown> = {
        limit: pageSize,
        offset: pageNum * pageSize,
        date: selectedDate,
      };
      if (filterType !== "all") {
        params.event_type = filterType;
      }
      if (searchText.trim()) {
        params.search = searchText.trim();
      }

      const data = await api.queryEvents(params);
      const result = (data as EventItem[]) || [];
      setEvents(result);
      setHasMore(result.length === pageSize);
    } catch (e) {
      console.error("Failed to load events:", e);
      setEvents([]);
    } finally {
      setLoading(false);
    }
  }

  function handleSearch() {
    setPage(0);
    loadEvents(0);
  }

  function handlePrevPage() {
    const newPage = Math.max(0, page - 1);
    setPage(newPage);
    loadEvents(newPage);
  }

  function handleNextPage() {
    if (!hasMore) return;
    const newPage = page + 1;
    setPage(newPage);
    loadEvents(newPage);
  }

  function formatTime(ts: number) {
    return format(new Date(ts), "HH:mm:ss");
  }

  function truncate(text: string, max: number = 100) {
    if (text.length <= max) return text;
    return text.substring(0, max) + "...";
  }

  return (
    <div className="p-8">
      <div className="mb-6">
        <h2 className="text-2xl font-bold text-gray-900">内容浏览器</h2>
        <p className="text-gray-500 mt-1">浏览所有记录的输入事件</p>
      </div>

      {/* 筛选栏 */}
      <div className="card mb-6">
        <div className="flex flex-wrap items-center gap-4">
          <div className="flex items-center gap-2">
            <Calendar size={18} className="text-gray-400" />
            <input
              type="date"
              value={selectedDate}
              onChange={(e) => setSelectedDate(e.target.value)}
              className="px-3 py-1.5 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <div className="flex items-center gap-2">
            <Filter size={18} className="text-gray-400" />
            <select
              value={filterType}
              onChange={(e) => setFilterType(e.target.value)}
              className="px-3 py-1.5 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-blue-500"
            >
              <option value="all">全部类型</option>
              <option value="keydown">键盘输入</option>
              <option value="mouse_click">鼠标点击</option>
              <option value="clipboard">剪贴板</option>
              <option value="app_focus">应用切换</option>
              <option value="paste">粘贴</option>
              <option value="session_end">会话结束</option>
              <option value="ime_committed">IME提交</option>
            </select>
          </div>

          <div className="flex items-center gap-2 flex-1 min-w-[200px]">
            <Search size={18} className="text-gray-400" />
            <input
              type="text"
              value={searchText}
              onChange={(e) => setSearchText(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSearch()}
              placeholder="搜索内容..."
              className="flex-1 px-3 py-1.5 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-blue-500"
            />
            <button onClick={handleSearch} className="btn-primary text-sm py-1.5">
              搜索
            </button>
          </div>
        </div>
      </div>

      {/* 事件列表 */}
      {loading ? (
        <div className="text-center py-12 text-gray-400">加载中...</div>
      ) : events.length > 0 ? (
        <>
          <div className="card overflow-hidden">
            <div className="divide-y divide-gray-100">
              {events.map((event) => (
                <div key={event.id} className="flex items-start gap-4 p-4 hover:bg-gray-50 transition-colors">
                  <div className="text-xs text-gray-400 font-mono w-20 flex-shrink-0 pt-1">
                    {formatTime(event.timestamp)}
                  </div>
                  <div className={`px-2 py-0.5 rounded text-xs font-medium flex-shrink-0 ${EVENT_TYPE_COLORS[event.event_type] || "text-gray-500 bg-gray-50"}`}>
                    {EVENT_TYPE_LABELS[event.event_type] || event.event_type}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="text-sm font-medium text-gray-700">
                      {event.app_name || "Unknown"}
                    </div>
                    {event.window_title && (
                      <div className="text-xs text-gray-400 mt-0.5">{event.window_title}</div>
                    )}
                    {event.content && (
                      <div className="text-sm text-gray-600 mt-1 font-mono bg-gray-50 px-2 py-1 rounded">
                        {truncate(event.content)}
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* 分页 */}
          <div className="flex items-center justify-between mt-4">
            <span className="text-sm text-gray-500">
              第 {page * pageSize + 1} - {page * pageSize + events.length} 条
            </span>
            <div className="flex gap-2">
              <button
                onClick={handlePrevPage}
                disabled={page === 0}
                className="flex items-center gap-1 px-3 py-1.5 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors disabled:opacity-30 text-sm"
              >
                <ChevronLeft size={16} />
                上一页
              </button>
              <button
                onClick={handleNextPage}
                disabled={!hasMore}
                className="flex items-center gap-1 px-3 py-1.5 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors disabled:opacity-30 text-sm"
              >
                下一页
                <ChevronRight size={16} />
              </button>
            </div>
          </div>
        </>
      ) : (
        <div className="card text-center py-12">
          <FileText size={48} className="text-gray-300 mx-auto mb-4" />
          <p className="text-gray-500">暂无事件数据</p>
          <p className="text-sm text-gray-400 mt-1">
            开始使用后，所有输入事件将显示在这里
          </p>
        </div>
      )}
    </div>
  );
}
