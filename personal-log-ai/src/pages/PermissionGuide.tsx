import { useEffect, useState } from "react";
import {
  Shield,
  CheckCircle2,
  XCircle,
  ExternalLink,
  RefreshCw,
  ChevronRight,
} from "lucide-react";
import { api } from "../services/tauri";

interface PermissionStatus {
  accessibility: boolean;
  screen_recording: boolean;
}

interface PermissionGuideProps {
  onAllGranted: () => void;
}

export default function PermissionGuide({ onAllGranted }: PermissionGuideProps) {
  const [permissions, setPermissions] = useState<PermissionStatus | null>(null);
  const [checking, setChecking] = useState(true);

  useEffect(() => {
    checkPermissions();
  }, []);

  async function checkPermissions() {
    try {
      setChecking(true);
      const result = await api.checkPermissions();
      setPermissions(result);

      // 如果所有权限都已授予，自动跳过引导
      if (result.accessibility && result.screen_recording) {
        setTimeout(() => onAllGranted(), 800);
      }
    } catch (e) {
      console.error("Failed to check permissions:", e);
      // 非 macOS 环境下默认全部通过
      setPermissions({ accessibility: true, screen_recording: true });
      setTimeout(() => onAllGranted(), 500);
    } finally {
      setChecking(false);
    }
  }

  const allGranted =
    permissions?.accessibility && permissions?.screen_recording;

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-white to-purple-50 flex items-center justify-center p-8">
      <div className="max-w-lg w-full">
        {/* 标题区域 */}
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-16 h-16 bg-blue-100 rounded-2xl mb-4">
            <Shield size={32} className="text-blue-600" />
          </div>
          <h1 className="text-2xl font-bold text-gray-900">
            个人输入统计助理
          </h1>
          <p className="text-gray-500 mt-2">
            首次使用需要授予以下系统权限，以确保应用正常运行
          </p>
        </div>

        {/* 权限卡片列表 */}
        <div className="space-y-4">
          {/* 辅助功能权限 */}
          <PermissionCard
            title="辅助功能权限"
            description="用于监听键盘输入、鼠标点击和应用切换。所有数据仅在本地处理，不会上传到任何服务器。"
            granted={permissions?.accessibility ?? false}
            loading={checking}
            onGrant={() => api.openAccessibilityPrefs()}
          />

          {/* 屏幕录制权限 */}
          <PermissionCard
            title="屏幕录制权限"
            description="用于获取当前窗口标题和浏览器 URL，以便更准确地记录你的工作上下文。"
            granted={permissions?.screen_recording ?? false}
            loading={checking}
            onGrant={() => api.openScreenRecordingPrefs()}
          />
        </div>

        {/* 操作按钮 */}
        <div className="mt-8 space-y-3">
          <button
            onClick={checkPermissions}
            disabled={checking}
            className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-blue-600 text-white rounded-xl font-medium hover:bg-blue-700 transition-colors disabled:opacity-50"
          >
            <RefreshCw size={18} className={checking ? "animate-spin" : ""} />
            {checking ? "检查中..." : "重新检查权限"}
          </button>

          {allGranted && (
            <button
              onClick={onAllGranted}
              className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-green-600 text-white rounded-xl font-medium hover:bg-green-700 transition-colors"
            >
              所有权限已授予，开始使用
              <ChevronRight size={18} />
            </button>
          )}

          {!allGranted && permissions && (
            <button
              onClick={onAllGranted}
              className="w-full px-4 py-3 text-gray-500 text-sm hover:text-gray-700 transition-colors"
            >
              稍后设置（部分功能可能不可用）
            </button>
          )}
        </div>

        {/* 底部隐私说明 */}
        <div className="mt-8 p-4 bg-gray-50 rounded-xl">
          <p className="text-xs text-gray-400 text-center leading-relaxed">
            🔒 你的隐私至关重要。所有输入记录均存储在本地 SQLite
            数据库中，不会上传到任何服务器。你可以随时在设置页面中关闭采集功能或删除历史数据。
          </p>
        </div>
      </div>
    </div>
  );
}

function PermissionCard({
  title,
  description,
  granted,
  loading,
  onGrant,
}: {
  title: string;
  description: string;
  granted: boolean;
  loading: boolean;
  onGrant: () => void;
}) {
  return (
    <div
      className={`p-5 rounded-xl border-2 transition-all ${
        granted
          ? "border-green-200 bg-green-50"
          : "border-gray-200 bg-white hover:border-blue-200"
      }`}
    >
      <div className="flex items-start gap-4">
        <div
          className={`flex-shrink-0 w-10 h-10 rounded-lg flex items-center justify-center ${
            granted ? "bg-green-100" : "bg-gray-100"
          }`}
        >
          {loading ? (
            <RefreshCw size={20} className="text-gray-400 animate-spin" />
          ) : granted ? (
            <CheckCircle2 size={20} className="text-green-600" />
          ) : (
            <XCircle size={20} className="text-gray-400" />
          )}
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h3 className="font-semibold text-gray-900">{title}</h3>
            {granted && (
              <span className="text-xs px-2 py-0.5 bg-green-100 text-green-700 rounded-full">
                已授予
              </span>
            )}
          </div>
          <p className="text-sm text-gray-500 mt-1">{description}</p>

          {!granted && !loading && (
            <button
              onClick={onGrant}
              className="mt-3 inline-flex items-center gap-1.5 text-sm text-blue-600 hover:text-blue-700 font-medium"
            >
              <ExternalLink size={14} />
              打开系统设置
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
