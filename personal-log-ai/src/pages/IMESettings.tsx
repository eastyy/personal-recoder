import { useState, useEffect } from "react";
import {
  Keyboard,
  RefreshCw,
  CheckCircle2,
  XCircle,
  Loader2,
  Settings as SettingsIcon,
  Zap,
  FileText,
  FolderOpen,
  Info,
  Plug,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

interface StatusItem {
  checked: boolean;
  ok: boolean;
  label: string;
  detail?: string;
}

interface SchemaInfo {
  name: string;
}

interface CheckResult {
  squirrel_installed: boolean;
  squirrel_path: string | null;
  ipc_socket: boolean;
  ipc_hook: boolean;
  schemas: string[];
}

export default function IMESettings() {
  const [checking, setChecking] = useState(false);
  const [reloading, setReloading] = useState(false);
  const [message, setMessage] = useState("");
  const [messageType, setMessageType] = useState<"success" | "error" | "info">("info");

  const [result, setResult] = useState<CheckResult | null>(null);

  useEffect(() => {
    checkAll();
  }, []);

  async function checkAll() {
    setChecking(true);
    try {
      const r = await invoke<CheckResult>("check_ime_integration");
      setResult(r);
    } catch (e) {
      console.error("检查失败:", e);
      showMessage("检查失败，请确保后端服务正常运行", "error");
    } finally {
      setChecking(false);
    }
  }

  async function reloadRime() {
    try {
      setReloading(true);
      await invoke("squirrel_reload");
      showMessage("Rime 重新部署已触发", "success");
    } catch (e) {
      console.error("重新部署失败:", e);
      showMessage("重新部署失败，请确认 Squirrel 正在运行", "error");
    } finally {
      setReloading(false);
    }
  }

  async function openKeyboardSettings() {
    try {
      await invoke("open_keyboard_settings");
      showMessage("已打开系统键盘设置", "info");
    } catch (e) {
      console.error("打开键盘设置失败:", e);
      showMessage("无法打开系统设置", "error");
    }
  }

  function showMessage(text: string, type: "success" | "error" | "info") {
    setMessage(text);
    setMessageType(type);
    setTimeout(() => setMessage(""), 4000);
  }

  function getStatus(checked: boolean, ok: boolean, okLabel: string, failLabel: string, detail?: string): StatusItem {
    return { checked, ok, label: ok ? okLabel : failLabel, detail };
  }

  const squirrelStatus = result
    ? getStatus(true, result.squirrel_installed, "Squirrel 已安装", "Squirrel 未安装", result.squirrel_path || undefined)
    : { checked: false, ok: false, label: "检查中…" };

  const ipcStatus = result
    ? getStatus(true, result.ipc_socket, "IPC socket 已建立", "IPC socket 未建立", "/tmp/personal-log-ai-ime.sock")
    : { checked: false, ok: false, label: "检查中…" };

  const hookStatus = result
    ? getStatus(true, result.ipc_hook, "IPC Hook 已注入", "IPC Hook 未注入", "二进制中包含 personal-log-ai-ime.sock")
    : { checked: false, ok: false, label: "检查中…" };

  const schemas: SchemaInfo[] = (result?.schemas || []).map((s) => ({ name: s }));

  function renderStatus(item: StatusItem) {
    if (!item.checked) {
      return (
        <div className="flex items-center gap-3 px-4 py-3 rounded-xl border border-gray-200 bg-gray-50">
          <Loader2 size={20} className="animate-spin text-gray-400" />
          <span className="font-medium text-gray-400">检查中…</span>
        </div>
      );
    }
    return (
      <div className={`flex items-start gap-3 px-4 py-3 rounded-xl border ${item.ok ? "bg-green-50 border-green-200" : "bg-red-50 border-red-200"}`}>
        <span className={item.ok ? "text-green-600" : "text-red-600"}>
          {item.ok ? <CheckCircle2 size={20} /> : <XCircle size={20} />}
        </span>
        <div className="flex-1 min-w-0">
          <span className={`font-medium ${item.ok ? "text-green-700" : "text-red-700"}`}>{item.label}</span>
          {item.detail && <div className="text-xs text-gray-500 mt-0.5 break-all">{item.detail}</div>}
        </div>
      </div>
    );
  }

  return (
    <div className="p-8 max-w-2xl">
      <div className="mb-8">
        <h2 className="text-2xl font-bold text-gray-900">Squirrel 鼠须管</h2>
        <p className="text-gray-500 mt-1">输入法集成状态与 Rime 配置</p>
      </div>

      {message && (
        <div className={`mb-6 p-4 rounded-lg text-sm ${messageType === "success" ? "bg-green-50 text-green-700" : messageType === "error" ? "bg-red-50 text-red-700" : "bg-blue-50 text-blue-700"}`}>
          {message}
        </div>
      )}

      <div className="space-y-6">
        <div className="card">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-2">
              <Keyboard size={20} className="text-blue-600" />
              <h3 className="text-lg font-semibold text-gray-900">集成状态</h3>
            </div>
            <button onClick={checkAll} disabled={checking} className="flex items-center gap-1.5 px-3 py-1.5 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors text-sm font-medium disabled:opacity-50">
              {checking ? <Loader2 size={16} className="animate-spin" /> : <RefreshCw size={16} />}
              {checking ? "检查中…" : "刷新"}
            </button>
          </div>
          <div className="space-y-3">
            {renderStatus(squirrelStatus)}
            {renderStatus(ipcStatus)}
            {renderStatus(hookStatus)}
          </div>
        </div>

        <div className="card">
          <div className="flex items-center gap-2 mb-4">
            <FolderOpen size={20} className="text-purple-600" />
            <h3 className="text-lg font-semibold text-gray-900">Rime 配置</h3>
          </div>
          <div className="mb-5">
            <div className="text-sm font-medium text-gray-700 mb-1">配置目录</div>
            <div className="px-3 py-2 bg-gray-50 rounded-lg text-sm text-gray-600 font-mono">~/Library/Rime/</div>
          </div>
          <div>
            <div className="flex items-center gap-2 mb-2">
              <FileText size={16} className="text-gray-400" />
              <span className="text-sm font-medium text-gray-700">输入方案（{schemas.length}）</span>
            </div>
            {schemas.length > 0 ? (
              <div className="flex flex-wrap gap-2">
                {schemas.map((s) => (
                  <span key={s.name} className="inline-flex items-center gap-1.5 px-3 py-1.5 bg-blue-50 text-blue-700 rounded-lg text-sm font-medium">
                    <FileText size={14} />
                    {s.name}
                  </span>
                ))}
              </div>
            ) : (
              <p className="text-sm text-gray-400 px-3 py-2 bg-gray-50 rounded-lg">未找到方案文件</p>
            )}
          </div>
        </div>

        <div className="card">
          <div className="flex items-center gap-2 mb-3">
            <Info size={20} className="text-amber-600" />
            <h3 className="text-lg font-semibold text-gray-900">使用提示</h3>
          </div>
          <div className="space-y-2.5 text-sm text-gray-600">
            <div className="flex items-start gap-2">
              <Plug size={16} className="text-gray-400 mt-0.5 flex-shrink-0" />
              <span>
                切换输入方案：在 Squirrel 激活状态下按{" "}
                <kbd className="px-1.5 py-0.5 bg-gray-100 border border-gray-300 rounded text-xs font-mono">Ctrl</kbd>{" "}+{" "}
                <kbd className="px-1.5 py-0.5 bg-gray-100 border border-gray-300 rounded text-xs font-mono">`</kbd>
              </span>
            </div>
            <div className="flex items-start gap-2">
              <Zap size={16} className="text-gray-400 mt-0.5 flex-shrink-0" />
              <span>修改 Rime 配置后需点击「重新部署」使配置生效。</span>
            </div>
            <div className="flex items-start gap-2">
              <SettingsIcon size={16} className="text-gray-400 mt-0.5 flex-shrink-0" />
              <span>首次使用需在系统键盘设置中添加 Squirrel 输入法。</span>
            </div>
            <p className="text-gray-400 mt-2 pl-6">所有输入数据仅保存在本地，不会上传到任何服务器。</p>
          </div>
        </div>

        <div className="flex flex-col gap-3">
          <button onClick={reloadRime} disabled={reloading} className="w-full btn-primary flex items-center justify-center gap-2">
            {reloading ? <Loader2 size={18} className="animate-spin" /> : <Zap size={18} />}
            {reloading ? "部署中…" : "重新部署 Rime"}
          </button>
          <button onClick={openKeyboardSettings} className="w-full btn-secondary flex items-center justify-center gap-2">
            <SettingsIcon size={18} />
            打开系统键盘设置
          </button>
        </div>
      </div>
    </div>
  );
}
