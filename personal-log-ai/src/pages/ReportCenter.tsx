import { useEffect, useState } from "react";
import { FileText, Sparkles, Loader2, Calendar, ChevronDown, ChevronRight, RefreshCw } from "lucide-react";
import { api } from "../services/tauri";

interface Report {
  id: string;
  analysis_type: string;
  result_text: string;
  created_at: number;
}

const ANALYSIS_TYPES = [
  { key: "productivity", label: "生产力分析", icon: "📊", desc: "分析时间分配与效率" },
  { key: "topic", label: "主题提取", icon: "🧠", desc: "提取今日核心话题" },
  { key: "writing", label: "写作优化", icon: "✍️", desc: "错别字与表达建议" },
  { key: "todo", label: "TODO 提取", icon: "✅", desc: "从输入中提取待办" },
  { key: "weekly", label: "周报分析", icon: "📈", desc: "本周综合复盘" },
];

export default function ReportCenter() {
  const [reports, setReports] = useState<Report[]>([]);
  const [loading, setLoading] = useState(true);
  const [analyzing, setAnalyzing] = useState<string | null>(null);
  const [expandedId, setExpandedId] = useState<string | null>(null);

  useEffect(() => {
    loadReports();
  }, []);

  async function loadReports() {
    try {
      setLoading(true);
      const data = await api.getReports();
      setReports((data as Report[]) || []);
    } catch (e) {
      console.error("Failed to load reports:", e);
    } finally {
      setLoading(false);
    }
  }

  async function handleAnalyze(type: string) {
    try {
      setAnalyzing(type);
      await api.triggerAnalysis(type);
      await loadReports();
    } catch (e) {
      console.error("Analysis failed:", e);
      alert("分析失败，请检查 API Key 配置");
    } finally {
      setAnalyzing(null);
    }
  }

  const getTypeLabel = (type: string) => {
    return ANALYSIS_TYPES.find((t) => t.key === type)?.label || type;
  };

  const getTypeIcon = (type: string) => {
    return ANALYSIS_TYPES.find((t) => t.key === type)?.icon || "📄";
  };

  // Group reports by date
  const groupedReports: Record<string, Report[]> = {};
  const sortedReports = [...reports].sort((a, b) => b.created_at - a.created_at);
  for (const report of sortedReports) {
    const date = new Date(report.created_at * 1000).toLocaleDateString("zh-CN");
    if (!groupedReports[date]) groupedReports[date] = [];
    groupedReports[date].push(report);
  }

  return (
    <div className="p-8">
      <div className="mb-8">
        <h2 className="text-2xl font-bold text-gray-900">报告中心</h2>
        <p className="text-gray-500 mt-1">查看 AI 分析报告或手动触发分析</p>
      </div>

      {/* 分析按钮 */}
      <div className="grid grid-cols-5 gap-4 mb-8">
        {ANALYSIS_TYPES.map((type) => (
          <button
            key={type.key}
            onClick={() => handleAnalyze(type.key)}
            disabled={analyzing === type.key}
            className="card flex flex-col items-start gap-2 hover:shadow-md transition-shadow disabled:opacity-50"
          >
            <div className="flex items-center gap-2 w-full">
              {analyzing === type.key ? (
                <Loader2 size={18} className="animate-spin text-blue-600" />
              ) : (
                <Sparkles size={18} className="text-blue-600" />
              )}
              <span className="font-medium text-gray-900 text-sm">
                {type.icon} {type.label}
              </span>
            </div>
            <div className="text-xs text-gray-500">
              {analyzing === type.key ? "分析中..." : type.desc}
            </div>
          </button>
        ))}
      </div>

      {/* 刷新按钮 */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900">历史报告</h3>
        <button
          onClick={loadReports}
          className="flex items-center gap-1 text-sm text-gray-500 hover:text-gray-700"
        >
          <RefreshCw size={14} />
          刷新
        </button>
      </div>

      {/* 报告列表 - 按日期分组 */}
      {loading ? (
        <div className="text-center py-8 text-gray-400">加载中...</div>
      ) : reports.length > 0 ? (
        <div className="space-y-6">
          {Object.entries(groupedReports).map(([date, dateReports]) => (
            <div key={date}>
              <div className="flex items-center gap-2 mb-3 text-gray-500">
                <Calendar size={16} />
                <span className="text-sm font-medium">{date}</span>
                <span className="text-xs">({dateReports.length} 份报告)</span>
              </div>
              <div className="space-y-3">
                {dateReports.map((report) => (
                  <div key={report.id} className="card">
                    <button
                      onClick={() => setExpandedId(expandedId === report.id ? null : report.id)}
                      className="flex items-center gap-2 w-full text-left mb-2"
                    >
                      {expandedId === report.id ? (
                        <ChevronDown size={18} className="text-gray-400" />
                      ) : (
                        <ChevronRight size={18} className="text-gray-400" />
                      )}
                      <FileText size={18} className="text-blue-600" />
                      <span className="font-medium text-gray-900">
                        {getTypeIcon(report.analysis_type)} {getTypeLabel(report.analysis_type)}
                      </span>
                      <span className="text-xs text-gray-400 ml-auto">
                        {new Date(report.created_at * 1000).toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" })}
                      </span>
                    </button>
                    {expandedId === report.id && (
                      <div className="text-sm text-gray-700 whitespace-pre-wrap bg-gray-50 p-4 rounded-lg mt-2">
                        {report.result_text}
                      </div>
                    )}
                    {expandedId !== report.id && (
                      <div className="text-sm text-gray-500 ml-7 line-clamp-2">
                        {report.result_text.substring(0, 200)}
                        {report.result_text.length > 200 && "..."}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="card text-center py-12">
          <FileText size={48} className="text-gray-300 mx-auto mb-4" />
          <p className="text-gray-500">暂无报告</p>
          <p className="text-sm text-gray-400 mt-1">点击上方按钮触发 AI 分析</p>
          <p className="text-xs text-gray-400 mt-2">
            系统也会在每日 03:00 自动生成分析报告
          </p>
        </div>
      )}
    </div>
  );
}
