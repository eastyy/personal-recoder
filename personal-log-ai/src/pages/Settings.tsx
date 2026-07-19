import { useEffect, useState } from "react";
import { Save, Key, Database, Shield, Download, Trash2, BarChart3, Server } from "lucide-react";
import { api } from "../services/tauri";

export default function SettingsPage() {
  const [config, setLocalConfig] = useState<Record<string, string>>({});
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");
  const [exporting, setExporting] = useState(false);
  const [cleaning, setCleaning] = useState(false);
  const [dbStats, setDbStats] = useState<Record<string, number> | null>(null);

  useEffect(() => {
    loadConfig();
    loadDbStats();
  }, []);

  async function loadConfig() {
    try {
      const data = await api.getConfig();
      setLocalConfig((data as Record<string, string>) || {});
    } catch (e) {
      console.error("Failed to load config:", e);
    }
  }

  async function loadDbStats() {
    try {
      const stats = await api.getDbStats();
      setDbStats(stats as Record<string, number>);
    } catch (e) {
      console.error("Failed to load db stats:", e);
    }
  }

  async function handleSave() {
    try {
      setSaving(true);
      for (const [key, value] of Object.entries(config)) {
        await api.setConfig(key, value);
      }
      showMessage("保存成功！", "success");
    } catch (e) {
      console.error("Failed to save config:", e);
      showMessage("保存失败", "error");
    } finally {
      setSaving(false);
    }
  }

  async function handleExportJSON() {
    try {
      setExporting(true);
      const json = await api.exportAllJSON();
      const blob = new Blob([JSON.stringify(json, null, 2)], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `personal-log-export-${new Date().toISOString().split("T")[0]}.json`;
      a.click();
      URL.revokeObjectURL(url);
      showMessage("数据导出成功！", "success");
    } catch (e) {
      console.error("Export failed:", e);
      showMessage("导出失败，请稍后重试", "error");
    } finally {
      setExporting(false);
    }
  }

  async function handleExportCSV() {
    try {
      setExporting(true);
      const now = Date.now();
      const weekAgo = now - 7 * 24 * 3600 * 1000;
      const csv = await api.exportEventsCSV(Math.floor(weekAgo / 1000), Math.floor(now / 1000));
      const blob = new Blob([csv as string], { type: "text/csv;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `events-week-${new Date().toISOString().split("T")[0]}.csv`;
      a.click();
      URL.revokeObjectURL(url);
      showMessage("CSV 导出成功！", "success");
    } catch (e) {
      console.error("CSV export failed:", e);
      showMessage("导出失败", "error");
    } finally {
      setExporting(false);
    }
  }

  async function handleCleanup() {
    if (!confirm("确定清理过期数据吗？此操作不可撤销。")) return;
    try {
      setCleaning(true);
      await api.cleanupOldData();
      await loadDbStats();
      showMessage("数据清理完成！", "success");
    } catch (e) {
      console.error("Cleanup failed:", e);
      showMessage("清理失败", "error");
    } finally {
      setCleaning(false);
    }
  }

  function showMessage(text: string, _type: "success" | "error") {
    setMessage(text);
    setTimeout(() => setMessage(""), 4000);
  }

  const updateConfig = (key: string, value: string) => {
    setLocalConfig((prev) => ({ ...prev, [key]: value }));
  };

  const apiProvider = config.ai_provider || "minimax";

  return (
    <div className="p-8 max-w-2xl">
      <div className="mb-8">
        <h2 className="text-2xl font-bold text-gray-900">设置</h2>
        <p className="text-gray-500 mt-1">配置 AI 服务与采集选项</p>
      </div>

      {message && (
        <div className={`mb-6 p-4 rounded-lg text-sm ${message.includes("失败") ? "bg-red-50 text-red-700" : "bg-green-50 text-green-700"}`}>
          {message}
        </div>
      )}

      <div className="space-y-6">
        {/* AI 服务商选择 */}
        <div className="card">
          <div className="flex items-center gap-2 mb-4">
            <Server size={20} className="text-indigo-600" />
            <h3 className="text-lg font-semibold text-gray-900">AI 服务商</h3>
          </div>
          <div className="grid grid-cols-4 gap-3 mb-4">
            {[
              { value: "minimax", label: "MiniMax", desc: "国内服务" },
              { value: "openai", label: "OpenAI", desc: "GPT系列" },
              { value: "volcengine", label: "火山方舟", desc: "字节跳动" },
              { value: "custom", label: "自定义", desc: "兼容OpenAI API" },
            ].map((p) => (
              <button
                key={p.value}
                onClick={() => updateConfig("ai_provider", p.value)}
                className={`p-4 rounded-xl border-2 transition-all text-center ${
                  apiProvider === p.value
                    ? "border-blue-500 bg-blue-50"
                    : "border-gray-200 hover:border-gray-300"
                }`}
              >
                <div className="font-semibold text-gray-900">{p.label}</div>
                <div className="text-xs text-gray-500 mt-1">{p.desc}</div>
              </button>
            ))}
          </div>
        </div>

        {/* API 配置 */}
        <div className="card">
          <div className="flex items-center gap-2 mb-4">
            <Key size={20} className="text-blue-600" />
            <h3 className="text-lg font-semibold text-gray-900">
              {apiProvider === "minimax" ? "MiniMax API 配置" : apiProvider === "openai" ? "OpenAI API 配置" : apiProvider === "volcengine" ? "火山方舟 API 配置" : "自定义 API 配置"}
            </h3>
          </div>
          <div className="space-y-4">
            {apiProvider === "minimax" && (
              <>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">API Key</label>
                  <input
                    type="password"
                    value={config.minimax_api_key || ""}
                    onChange={(e) => updateConfig("minimax_api_key", e.target.value)}
                    placeholder="sk-..."
                    className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  />
                  <p className="text-xs text-gray-500 mt-1">在 MiniMax 开放平台获取 API Key</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Group ID</label>
                  <input
                    type="text"
                    value={config.minimax_group_id || ""}
                    onChange={(e) => updateConfig("minimax_group_id", e.target.value)}
                    placeholder="your-group-id"
                    className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  />
                </div>
              </>
            )}
            {(apiProvider === "openai" || apiProvider === "custom") && (
              <>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">API Key</label>
                  <input
                    type="password"
                    value={config.openai_api_key || ""}
                    onChange={(e) => updateConfig("openai_api_key", e.target.value)}
                    placeholder="sk-..."
                    className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">API Base URL</label>
                  <input
                    type="text"
                    value={config.openai_base_url || (apiProvider === "openai" ? "https://api.openai.com/v1" : "")}
                    onChange={(e) => updateConfig("openai_base_url", e.target.value)}
                    placeholder="https://api.openai.com/v1"
                    className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  />
                  <p className="text-xs text-gray-500 mt-1">
                    {apiProvider === "custom" ? "兼容 OpenAI API 格式的自定义端点 URL" : "OpenAI API 地址（可使用代理）"}
                  </p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">模型名称</label>
                  <input
                    type="text"
                    value={config.openai_model || (apiProvider === "openai" ? "gpt-4o-mini" : "")}
                    onChange={(e) => updateConfig("openai_model", e.target.value)}
                    placeholder="gpt-4o-mini"
                    className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  />
                </div>
              </>
            )}
            {apiProvider === "volcengine" && (
              <>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">API Key</label>
                  <input
                    type="password"
                    value={config.volcengine_api_key || ""}
                    onChange={(e) => updateConfig("volcengine_api_key", e.target.value)}
                    placeholder="ark-..."
                    className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  />
                  <p className="text-xs text-gray-500 mt-1">在火山方舟控制台获取 API Key</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">API Base URL</label>
                  <input
                    type="text"
                    value={config.volcengine_base_url || "https://ark.cn-beijing.volces.com/api/v3"}
                    onChange={(e) => updateConfig("volcengine_base_url", e.target.value)}
                    placeholder="https://ark.cn-beijing.volces.com/api/v3"
                    className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  />
                  <p className="text-xs text-gray-500 mt-1">火山方舟 OpenAI 兼容 API 地址</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">模型名称</label>
                  <input
                    type="text"
                    value={config.volcengine_model || "doubao-pro-4k"}
                    onChange={(e) => updateConfig("volcengine_model", e.target.value)}
                    placeholder="doubao-pro-4k"
                    className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  />
                  <p className="text-xs text-gray-500 mt-1">豆包模型，如 doubao-pro-4k、doubao-pro-32k 等</p>
                </div>
              </>
            )}
          </div>
        </div>

        {/* 采集设置 */}
        <div className="card">
          <div className="flex items-center gap-2 mb-4">
            <Database size={20} className="text-green-600" />
            <h3 className="text-lg font-semibold text-gray-900">采集设置</h3>
          </div>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <div className="text-sm font-medium text-gray-700">剪贴板监听</div>
                <div className="text-xs text-gray-500">记录剪贴板内容（已过滤敏感信息）</div>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={config.enable_clipboard === "true"}
                  onChange={(e) => updateConfig("enable_clipboard", e.target.checked.toString())}
                  className="sr-only peer"
                />
                <div className="w-11 h-6 bg-gray-200 peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
              </label>
            </div>
            <div className="flex items-center justify-between">
              <div>
                <div className="text-sm font-medium text-gray-700">鼠标活动</div>
                <div className="text-xs text-gray-500">记录鼠标点击与移动</div>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={config.enable_mouse === "true"}
                  onChange={(e) => updateConfig("enable_mouse", e.target.checked.toString())}
                  className="sr-only peer"
                />
                <div className="w-11 h-6 bg-gray-200 peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
              </label>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">输入停顿阈值（秒）</label>
                <input
                  type="number"
                  value={config.pause_threshold || "3"}
                  onChange={(e) => updateConfig("pause_threshold", e.target.value)}
                  min="1"
                  max="30"
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">会话超时（秒）</label>
                <input
                  type="number"
                  value={config.session_timeout || "60"}
                  onChange={(e) => updateConfig("session_timeout", e.target.value)}
                  min="10"
                  max="600"
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              </div>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">数据保留期（天）</label>
              <input
                type="number"
                value={config.data_retention_days || "90"}
                onChange={(e) => updateConfig("data_retention_days", e.target.value)}
                min="7"
                max="365"
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
          </div>
        </div>

        {/* 数据管理 */}
        <div className="card">
          <div className="flex items-center gap-2 mb-4">
            <BarChart3 size={20} className="text-blue-600" />
            <h3 className="text-lg font-semibold text-gray-900">数据管理</h3>
          </div>

          {dbStats && (
            <div className="grid grid-cols-4 gap-3 mb-4">
              <div className="text-center p-3 bg-gray-50 rounded-lg">
                <div className="text-2xl font-bold text-gray-900">{dbStats.total_events ?? 0}</div>
                <div className="text-xs text-gray-500">事件总数</div>
              </div>
              <div className="text-center p-3 bg-gray-50 rounded-lg">
                <div className="text-2xl font-bold text-gray-900">{dbStats.total_sessions ?? 0}</div>
                <div className="text-xs text-gray-500">会话数</div>
              </div>
              <div className="text-center p-3 bg-gray-50 rounded-lg">
                <div className="text-2xl font-bold text-gray-900">{dbStats.total_todos ?? 0}</div>
                <div className="text-xs text-gray-500">TODO</div>
              </div>
              <div className="text-center p-3 bg-gray-50 rounded-lg">
                <div className="text-2xl font-bold text-gray-900">
                  {dbStats.db_size_bytes ? (dbStats.db_size_bytes / 1024 / 1024).toFixed(1) : "0"} MB
                </div>
                <div className="text-xs text-gray-500">数据库大小</div>
              </div>
            </div>
          )}

          <div className="flex flex-wrap gap-3">
            <button
              onClick={handleExportJSON}
              disabled={exporting}
              className="flex items-center gap-2 px-4 py-2 bg-blue-50 text-blue-700 rounded-lg hover:bg-blue-100 transition-colors text-sm font-medium disabled:opacity-50"
            >
              <Download size={16} />
              {exporting ? "导出中..." : "导出 JSON"}
            </button>
            <button
              onClick={handleExportCSV}
              disabled={exporting}
              className="flex items-center gap-2 px-4 py-2 bg-green-50 text-green-700 rounded-lg hover:bg-green-100 transition-colors text-sm font-medium disabled:opacity-50"
            >
              <Download size={16} />
              导出 CSV (本周)
            </button>
            <button
              onClick={handleCleanup}
              disabled={cleaning}
              className="flex items-center gap-2 px-4 py-2 bg-red-50 text-red-700 rounded-lg hover:bg-red-100 transition-colors text-sm font-medium disabled:opacity-50"
            >
              <Trash2 size={16} />
              {cleaning ? "清理中..." : "清理过期数据"}
            </button>
            <button
              onClick={loadDbStats}
              className="flex items-center gap-2 px-4 py-2 bg-gray-50 text-gray-700 rounded-lg hover:bg-gray-100 transition-colors text-sm font-medium"
            >
              刷新统计
            </button>
          </div>
        </div>

        {/* 隐私 */}
        <div className="card">
          <div className="flex items-center gap-2 mb-4">
            <Shield size={20} className="text-purple-600" />
            <h3 className="text-lg font-semibold text-gray-900">隐私与安全</h3>
          </div>
          <div className="space-y-3 text-sm text-gray-600">
            <p>所有数据存储在本地 SQLite 数据库中，不会上传到任何服务器。</p>
            <p>AI 分析时仅发送需要分析的文本片段，不会发送完整数据库。</p>
            <p>API Key 存储在本地 SQLite 数据库中，与数据文件位于同一位置。</p>
            <p>密码管理器等敏感应用已默认加入黑名单，不会被记录。</p>
          </div>
        </div>

        {/* 保存按钮 */}
        <button
          onClick={handleSave}
          disabled={saving}
          className="w-full btn-primary flex items-center justify-center gap-2"
        >
          <Save size={18} />
          {saving ? "保存中..." : "保存设置"}
        </button>
      </div>
    </div>
  );
}
