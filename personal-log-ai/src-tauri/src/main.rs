#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod input;
mod focus;
mod ai;
mod todo;
mod stats;
mod config;
mod ime_ipc;
mod scheduler;
mod export;

use tauri::{
    Manager,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{Menu, MenuItem, PredefinedMenuItem},
};

#[tauri::command]
async fn get_daily_stats(date: String) -> Result<serde_json::Value, String> {
    db::get_daily_stats(&date).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_app_usage(start: i64, end: i64) -> Result<Vec<serde_json::Value>, String> {
    db::get_app_usage(start, end).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_focus_summary(date: String) -> Result<serde_json::Value, String> {
    db::get_focus_summary(&date).map_err(|e| e.to_string())
}

#[tauri::command]
async fn query_events(params: serde_json::Value) -> Result<Vec<serde_json::Value>, String> {
    db::query_events(params).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_todos(status: Option<String>) -> Result<Vec<serde_json::Value>, String> {
    db::get_todos(status.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_todo(id: i64) -> Result<(), String> {
    db::toggle_todo(id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_reports(analysis_type: Option<String>) -> Result<Vec<serde_json::Value>, String> {
    db::get_reports(analysis_type.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn trigger_analysis(analysis_type: String) -> Result<serde_json::Value, String> {
    ai::trigger_analysis(&analysis_type).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_realtime_status() -> Result<serde_json::Value, String> {
    input::get_realtime_status().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_config() -> Result<serde_json::Value, String> {
    config::get_config().map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_config(key: String, value: String) -> Result<(), String> {
    config::set_config(&key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_recording_paused(paused: bool) -> Result<(), String> {
    input::set_paused(paused);
    Ok(())
}

#[tauri::command]
async fn check_permissions() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "accessibility": input::check_accessibility_permission(),
        "screen_recording": check_screen_recording_permission(),
    }))
}

#[tauri::command]
async fn open_accessibility_prefs() -> Result<(), String> {
    input::open_accessibility_preferences();
    Ok(())
}

#[tauri::command]
async fn open_screen_recording_prefs() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let url = "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenRecording";
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ============================================================
// IME 相关命令
// ============================================================

/// 安装 LogInputIME 代理输入法
/// 将内嵌的输入法 .app 复制到 ~/Library/Input Methods/ 目录
#[tauri::command]
async fn install_ime() -> Result<serde_json::Value, String> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
        let ime_dir = home.join("Library/Input Methods");
        let target_app = ime_dir.join("LogInputIME.app");

        // 检查是否已安装
        if target_app.exists() {
            return Ok(serde_json::json!({
                "success": true,
                "installed": true,
                "message": "输入法已安装"
            }));
        }

        // 确保目标目录存在
        if !ime_dir.exists() {
            std::fs::create_dir_all(&ime_dir).map_err(|e| e.to_string())?;
        }

        // 尝试从内嵌资源复制输入法（Tauri 资源目录）
        let resource_app = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("../Resources/LogInputIME.app"));

        if let Some(ref src) = resource_app {
            if src.exists() {
                // 使用 cp -R 复制 .app  bundle
                let status = std::process::Command::new("cp")
                    .args(["-R", src.to_str().unwrap_or(""), target_app.to_str().unwrap_or("")])
                    .status()
                    .map_err(|e| e.to_string())?;

                if status.success() {
                    return Ok(serde_json::json!({
                        "success": true,
                        "installed": true,
                        "message": "输入法安装成功，请在系统设置中启用"
                    }));
                }
            }
        }

        // 内嵌资源不存在，返回手动安装指引
        return Ok(serde_json::json!({
            "success": false,
            "installed": false,
            "message": "未找到内嵌输入法资源，请手动安装 LogInputIME.app"
        }));
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(serde_json::json!({
            "success": false,
            "message": "IME 安装仅支持 macOS"
        }))
    }
}

/// 检查输入法安装和启用状态
#[tauri::command]
async fn check_ime_status() -> Result<serde_json::Value, String> {
    #[cfg(target_os = "macos")]
    {
        use std::path::Path;

        let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
        let user_ime = home.join("Library/Input Methods/LogInputIME.app");
        let system_ime = Path::new("/Library/Input Methods/LogInputIME.app");

        let installed = user_ime.exists() || system_ime.exists();

        // 检查是否已启用（通过查询系统输入法列表）
        let enabled = check_ime_enabled();

        // 检查 IPC socket 是否活跃
        let socket_active = Path::new(crate::ime_ipc::SOCKET_PATH).exists();

        // 确定状态字符串
        let status = if enabled {
            "enabled"
        } else if installed {
            "installed"
        } else {
            "not_installed"
        };

        // 读取已保存的底层输入法设置
        let backend_ime = crate::config::get_config()
            .ok()
            .and_then(|c| c.as_object().cloned())
            .and_then(|obj| obj.get("backend_ime").and_then(|v| v.as_str()).map(|s| s.to_string()));

        Ok(serde_json::json!({
            "status": status,
            "installed": installed,
            "enabled": enabled,
            "socket_active": socket_active,
            "backend_ime": backend_ime,
            "capabilities": {
                "can_install": !installed,
                "can_open_settings": true,
            },
        }))
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(serde_json::json!({
            "installed": false,
            "enabled": false,
            "socket_active": false,
            "message": "IME 状态检查仅支持 macOS"
        }))
    }
}

/// 打开系统键盘设置面板
#[tauri::command]
async fn open_keyboard_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let url = "x-apple.systempreferences:com.apple.preference.keyboard?InputSources";
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 检查 Squirrel IME 集成状态
#[tauri::command]
async fn check_ime_integration() -> Result<serde_json::Value, String> {
    #[cfg(target_os = "macos")]
    {
        use std::path::Path;
        let home = dirs::home_dir().ok_or("无法获取用户主目录")?;

        let system_path = "/Library/Input Methods/Squirrel.app";
        let user_path = home.join("Library/Input Methods/Squirrel.app");

        let (squirrel_installed, squirrel_path) = if Path::new(system_path).exists() {
            (true, Some(system_path.to_string()))
        } else if user_path.exists() {
            (true, Some(user_path.to_string_lossy().to_string()))
        } else {
            (false, None)
        };

        let ipc_socket = Path::new(crate::ime_ipc::SOCKET_PATH).exists();

        // 检查二进制中是否包含 IPC hook
        let ipc_hook = if let Some(ref path) = squirrel_path {
            let bin = format!("{}/Contents/MacOS/Squirrel", path);
            std::process::Command::new("strings")
                .arg(&bin)
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.contains("personal-log-ai-ime.sock"))
                .unwrap_or(false)
        } else {
            false
        };

        // 读取输入方案列表
        let schemas = if let Some(ref path) = squirrel_path {
            let ss_dir = format!("{}/Contents/SharedSupport", path);
            std::fs::read_dir(&ss_dir)
                .ok()
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                        .filter(|s| s.ends_with(".schema.yaml"))
                        .map(|s| s.replace(".schema.yaml", ""))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        } else {
            vec![]
        };

        Ok(serde_json::json!({
            "squirrel_installed": squirrel_installed,
            "squirrel_path": squirrel_path,
            "ipc_socket": ipc_socket,
            "ipc_hook": ipc_hook,
            "schemas": schemas,
        }))
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(serde_json::json!({
            "squirrel_installed": false,
            "squirrel_path": null,
            "ipc_socket": false,
            "ipc_hook": false,
            "schemas": [],
        }))
    }
}

/// 重新部署 Rime（触发 Squirrel reload）
#[tauri::command]
async fn squirrel_reload() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let bin = "/Library/Input Methods/Squirrel.app/Contents/MacOS/Squirrel";
        let alt_bin = dirs::home_dir()
            .map(|h| h.join("Library/Input Methods/Squirrel.app/Contents/MacOS/Squirrel"))
            .unwrap_or_default();

        let path = if std::path::Path::new(bin).exists() {
            bin
        } else if alt_bin.exists() {
            alt_bin.to_str().unwrap_or(bin)
        } else {
            return Err("Squirrel 未安装".to_string());
        };

        std::process::Command::new(path)
            .arg("--reload")
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 设置底层输入法偏好
#[tauri::command]
async fn set_backend_ime(backend: String) -> Result<(), String> {
    crate::config::set_config("backend_ime", &backend).map_err(|e| e.to_string())
}

// ============================================================
// 数据导出与清理命令
// ============================================================

#[tauri::command]
async fn export_all_json() -> Result<String, String> {
    crate::export::export_all_json().map_err(|e| e.to_string())
}

#[tauri::command]
async fn export_events_csv(start: i64, end: i64) -> Result<String, String> {
    crate::export::export_events_csv(start, end).map_err(|e| e.to_string())
}

#[tauri::command]
async fn cleanup_old_data() -> Result<serde_json::Value, String> {
    let retention_days = crate::config::get_config()
        .ok()
        .and_then(|c| c.as_object().cloned())
        .and_then(|obj| obj.get("data_retention_days").and_then(|v| v.as_str()).and_then(|s| s.parse::<i32>().ok()))
        .unwrap_or(90);
    let (events, sessions) = crate::export::cleanup_old_data(retention_days).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "events_deleted": events,
        "sessions_deleted": sessions,
    }))
}

#[tauri::command]
async fn get_db_stats() -> Result<serde_json::Value, String> {
    crate::export::get_db_stats().map_err(|e| e.to_string())
}

// ============================================================
// 统计命令
// ============================================================

#[tauri::command]
async fn get_typing_stats(start: i64, end: i64) -> Result<serde_json::Value, String> {
    crate::stats::get_typing_stats(start, end).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_typing_rhythm(date: String) -> Result<serde_json::Value, String> {
    crate::stats::get_typing_rhythm(&date).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_switching_stats(date: String) -> Result<serde_json::Value, String> {
    crate::stats::get_switching_stats(&date).map_err(|e| e.to_string())
}

// ============================================================
// 闪念与目标命令
// ============================================================

#[tauri::command]
async fn get_flash_ideas(limit: Option<i64>) -> Result<Vec<serde_json::Value>, String> {
    crate::db::get_flash_ideas(limit).map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_flash_idea(text: String) -> Result<(), String> {
    crate::db::insert_flash_idea(&text, None).map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_flash_idea(id: i64) -> Result<(), String> {
    crate::db::delete_flash_idea(id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_goals() -> Result<Vec<serde_json::Value>, String> {
    crate::db::get_goals().map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_goal(title: String, metric_type: String, target_value: i64, period: String) -> Result<(), String> {
    crate::db::insert_goal(&title, &metric_type, target_value, &period).map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_goal(id: i64) -> Result<(), String> {
    crate::db::delete_goal(id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_goal_progress(id: i64, current_value: i64) -> Result<(), String> {
    crate::db::update_goal_progress(id, current_value).map_err(|e| e.to_string())
}

/// 检查 LogInputIME 是否在系统输入法列表中启用（macOS）
#[cfg(target_os = "macos")]
fn check_ime_enabled() -> bool {
    // 通过 Text Input Source Services 查询
    // 简化实现：检查进程是否运行或读取输入法配置
    // 实际生产环境应使用 TISCreateInputSourceList 等 API
    let output = std::process::Command::new("defaults")
        .args(["read", "com.apple.HIToolbox", "AppleEnabledInputSources"])
        .output();

    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        return text.contains("LogInputIME");
    }
    false
}

#[cfg(not(target_os = "macos"))]
fn check_ime_enabled() -> bool {
    false
}

/// 检查屏幕录制权限（macOS）
fn check_screen_recording_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        // macOS 10.15+ 需要屏幕录制权限才能获取窗口列表
        // Use CGWindowListCreateImage as a safer check, or just default to true
        // since the actual window listing is done separately
        // For now, return true to avoid blocking the UI
        true
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

fn main() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init());

    // 全局快捷键仅在 macOS 下启用
    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_plugin_global_shortcut::Builder::new().build());
    }

    builder
        .setup(|app| {
            // 初始化数据库
            db::init_db(app.handle().clone())?;

            // 初始化配置（从数据库加载）
            config::init_config()?;

            // 启动输入采集
            input::start_input_capture(app.handle().clone())?;

            // 启动专注追踪
            focus::start_focus_tracker(app.handle().clone())?;

            // 启动定时分析调度器
            if let Err(e) = scheduler::start_scheduler(app.handle().clone()) {
                eprintln!("⚠️ 调度器启动失败: {}", e);
            }

            // 注册全局快捷键: Cmd+Shift+L 显示/隐藏窗口（仅 macOS）
            #[cfg(target_os = "macos")]
            {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
                let show_hide_shortcut: Shortcut = "CmdOrCtrl+Shift+L".parse().map_err(|e| anyhow::anyhow!("Invalid shortcut: {}", e))?;
                app.global_shortcut().on_shortcuts(
                    vec![show_hide_shortcut],
                    move |app_handle, shortcut, event| {
                        if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                            if shortcut == &show_hide_shortcut {
                                if let Some(window) = app_handle.get_webview_window("main") {
                                    if window.is_visible().unwrap_or(false) {
                                        let _ = window.hide();
                                    } else {
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }
                                }
                            }
                        }
                    }
                )?;
            }

            // 设置系统托盘
            setup_tray(app)?;

            Ok(())
        })
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // 应用关闭前保存当前 session
                input::save_current_session();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_daily_stats,
            get_app_usage,
            get_focus_summary,
            query_events,
            get_todos,
            toggle_todo,
            get_reports,
            trigger_analysis,
            get_realtime_status,
            get_config,
            set_config,
            set_recording_paused,
            check_permissions,
            open_accessibility_prefs,
            open_screen_recording_prefs,
            install_ime,
            check_ime_status,
            check_ime_integration,
            squirrel_reload,
            open_keyboard_settings,
            set_backend_ime,
            export_all_json,
            export_events_csv,
            cleanup_old_data,
            get_db_stats,
            get_typing_stats,
            get_typing_rhythm,
            get_switching_stats,
            get_flash_ideas,
            add_flash_idea,
            delete_flash_idea,
            get_goals,
            add_goal,
            delete_goal,
            update_goal_progress,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // 创建托盘菜单
    let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "hide", "隐藏窗口", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let pause_item = MenuItem::with_id(app, "pause", "暂停记录", true, None::<&str>)?;
    let resume_item = MenuItem::with_id(app, "resume", "恢复记录", true, None::<&str>)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_item, &hide_item, &separator, &pause_item, &resume_item, &separator2, &quit_item])?;

    // 构建托盘图标
    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("个人输入统计助理")
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "hide" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }
                "pause" => {
                    input::set_paused(true);
                }
                "resume" => {
                    input::set_paused(false);
                }
                "quit" => {
                    // 退出前保存当前 session
                    input::save_current_session();
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            // 单击托盘图标显示窗口
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
