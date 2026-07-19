use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::sync::mpsc::Receiver;
use tauri::AppHandle;
use tauri::Emitter;

use crate::ime_ipc::{IMETextEvent, IMETextType};

// ============================================================
// 公开数据结构
// ============================================================

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct InputEvent {
    pub timestamp: i64,
    pub event_type: String,       // "keydown" | "keyup" | "mouse_click" | "mouse_move" | "clipboard" | "app_focus"
    pub app_bundle_id: Option<String>,
    pub app_name: Option<String>,
    pub window_title: Option<String>,
    pub content: Option<String>,
    pub is_sensitive: i32,          // 0=Normal 1=LocalOnly 2=Discarded
    pub key_code: Option<i64>,
    pub is_backspace: bool,
}

#[derive(Clone, Debug)]
pub struct SessionState {
    pub id: String,
    pub app_bundle_id: String,
    pub app_name: Option<String>,
    pub start_time: i64,
    pub last_input_time: Instant,
    pub buffer: String,
    pub pause_count: i32,
    pub backspace_count: i32,
    pub total_keystrokes: i32,
}

#[derive(Clone, Debug)]
pub struct AppInfo {
    pub bundle_id: String,
    pub name: String,
    pub window_title: Option<String>,
    pub pid: i32,
}

// ============================================================
// 全局状态
// ============================================================

static REALTIME_STATUS: Mutex<Option<Value>> = Mutex::new(None);
static SESSION_STATE: Mutex<Option<SessionState>> = Mutex::new(None);
static PAUSED: Mutex<bool> = Mutex::new(false);
static EVENT_COUNT: Mutex<i64> = Mutex::new(0);

// IME IPC 接收器
static IME_EVENT_RECEIVER: Mutex<Option<Receiver<IMETextEvent>>> = Mutex::new(None);

// IME 去重状态：记录最近一次 CGEventTap 按键时间和最近 IME Direct 文本
static LAST_CGEVENT_KEY_TIME: Mutex<Option<Instant>> = Mutex::new(None);
static LAST_IME_DIRECT_TEXT: Mutex<Option<(String, Instant)>> = Mutex::new(None);

// ============================================================
// 公开 API
// ============================================================

/// 启动输入采集（macOS CGEventTap + NSWorkspace 监听 + IME IPC）
pub fn start_input_capture(app: AppHandle) -> Result<()> {
    // 1. 检查辅助功能权限
    #[cfg(target_os = "macos")]
    if !check_accessibility_permission() {
        eprintln!("⚠️ 未获得辅助功能权限，正在请求...");
        request_accessibility_permission();
    }

    // 2. 启动 CGEventTap 键盘/鼠标监听线程
    #[cfg(target_os = "macos")]
    start_cgevent_tap()?;

    // 3. 启动 NSWorkspace 应用切换监听
    #[cfg(target_os = "macos")]
    start_app_switch_listener()?;

    // 4. 启动剪贴板监听
    #[cfg(target_os = "macos")]
    start_clipboard_watcher()?;

    // 5. 启动 IME IPC 服务端
    if let Err(e) = init_ime_receiver() {
        eprintln!("⚠️ IME IPC 初始化失败: {}", e);
    } else {
        eprintln!("✅ IME IPC 服务端已启动，监听 {}", crate::ime_ipc::SOCKET_PATH);
    }

    // 6. 启动定时器：每 500ms 更新实时状态 + 轮询 IME 事件
    let app_handle = app.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));

            // 轮询 IME 事件
            poll_ime_events();

            if let Err(e) = update_realtime_status_sync() {
                eprintln!("Status update error: {}", e);
            }
            // 向前端推送实时事件
            let _ = app_handle.emit("realtime_update", get_realtime_status().unwrap_or_default());
        }
    });

    Ok(())
}

/// 暂停/恢复记录
pub fn set_paused(paused: bool) {
    *PAUSED.lock().unwrap() = paused;
}

/// 获取实时状态
pub fn get_realtime_status() -> Result<Value> {
    let status = REALTIME_STATUS.lock().unwrap();
    let paused = *PAUSED.lock().unwrap();
    let mut base = status.clone().unwrap_or_else(|| json!({
        "recording": false,
        "app_name": null,
        "app_bundle_id": null,
        "today_events": 0,
        "paused": false,
    }));
    if let Some(obj) = base.as_object_mut() {
        obj.insert("paused".to_string(), json!(paused));
        obj.insert("recording".to_string(), json!(!paused));
    }
    Ok(base)
}

/// 保存当前会话（用于应用退出时）
pub fn save_current_session() {
    let mut session = SESSION_STATE.lock().unwrap();
    if let Some(ref state) = *session {
        let _ = save_session(state);
    }
    *session = None;
}



/// 打开系统设置中的辅助功能面板
pub fn open_accessibility_preferences() {
    #[cfg(target_os = "macos")]
    {
        let url = "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .ok();
    }
}

/// 记录按键事件（由 CGEventTap 回调调用）
pub fn record_key_event(key_code: i64, key_char: Option<&str>, is_backspace: bool) {
    if *PAUSED.lock().unwrap() {
        return;
    }

    let now = chrono::Utc::now().timestamp_millis();
    let app = get_active_app_sync();

    // 检查黑名单
    if is_blacklisted_sync(&app.bundle_id) {
        return;
    }

    // 检查是否为密码字段（isSecureTextField）
    #[cfg(target_os = "macos")]
    if is_secure_text_field() {
        return;
    }

    // 记录 CGEventTap 按键时间（用于 IME 去重判断）
    *LAST_CGEVENT_KEY_TIME.lock().unwrap() = Some(Instant::now());

    // === IME 去重：如果最近 500ms 内有 IME Direct 文本匹配，跳过 CGEventTap 记录 ===
    if let Some(ch) = key_char {
        if should_dedup_cgevent_key(ch) {
            return;
        }
    }

    // 写入原始事件
    let _ = crate::db::insert_raw_event(
        now,
        if is_backspace { "keyup" } else { "keydown" },
        Some(&app.bundle_id),
        Some(&app.name),
        app.window_title.as_deref(),
        key_char,
        0,
        None,
        &serde_json::to_string(&json!({
            "key_code": key_code,
            "is_backspace": is_backspace,
        })).unwrap_or_default(),
    );

    // 更新会话 buffer
    {
        let mut state = SESSION_STATE.lock().unwrap();
        if let Some(s) = state.as_mut() {
            if s.app_bundle_id == app.bundle_id {
                if is_backspace {
                    s.backspace_count += 1;
                    s.buffer.pop();
                } else if let Some(ch) = key_char {
                    s.buffer.push_str(ch);
                }
                s.total_keystrokes += 1;
                s.last_input_time = Instant::now();
            } else {
                // 应用切换，保存旧会话
                let _ = save_session(s);
                *s = create_new_session(&app, now);
                if let Some(ch) = key_char {
                    s.buffer.push_str(ch);
                    s.total_keystrokes += 1;
                }
            }
        } else {
            let mut new_session = create_new_session(&app, now);
            if let Some(ch) = key_char {
                new_session.buffer.push_str(ch);
                new_session.total_keystrokes += 1;
            }
            *state = Some(new_session);
        }
    }

    // 更新事件计数
    {
        let mut count = EVENT_COUNT.lock().unwrap();
        *count += 1;
    }
}

/// 记录鼠标点击事件
pub fn record_mouse_click(x: f64, y: f64) {
    if *PAUSED.lock().unwrap() {
        return;
    }

    // 检查是否启用了鼠标监听
    let enabled = crate::config::get_config()
        .ok()
        .and_then(|c| c.as_object().cloned())
        .and_then(|obj| obj.get("enable_mouse").and_then(|v| v.as_str()).map(|s| s == "true"))
        .unwrap_or(true);
    if !enabled {
        return;
    }

    let now = chrono::Utc::now().timestamp_millis();
    let app = get_active_app_sync();

    if is_blacklisted_sync(&app.bundle_id) {
        return;
    }

    let _ = crate::db::insert_raw_event(
        now,
        "mouse_click",
        Some(&app.bundle_id),
        Some(&app.name),
        app.window_title.as_deref(),
        None,
        0,
        None,
        &serde_json::to_string(&json!({
            "x": x,
            "y": y,
        })).unwrap_or_default(),
    );

    let mut count = EVENT_COUNT.lock().unwrap();
    *count += 1;
}

/// 记录剪贴板变化
pub fn record_clipboard_change(content: &str, source_app: &str) {
    if *PAUSED.lock().unwrap() {
        return;
    }

    // 剪贴板加强过滤
    if is_sensitive_clipboard_content(content) {
        return;
    }

    let now = chrono::Utc::now().timestamp_millis();
    let app = get_active_app_sync();

    let _ = crate::db::insert_raw_event(
        now,
        "clipboard",
        Some(&app.bundle_id),
        Some(&app.name),
        None,
        Some(content),
        0,
        None,
        &serde_json::to_string(&json!({
            "source_app": source_app,
            "length": content.len(),
        })).unwrap_or_default(),
    );

    let mut count = EVENT_COUNT.lock().unwrap();
    *count += 1;
}

/// 记录应用切换
pub fn record_app_switch(bundle_id: &str, name: &str, pid: i32) {
    if *PAUSED.lock().unwrap() {
        return;
    }

    let now = chrono::Utc::now().timestamp_millis();

    // 保存旧会话
    {
        let state = SESSION_STATE.lock().unwrap();
        if let Some(s) = state.as_ref() {
            if s.app_bundle_id != bundle_id {
                let _ = save_session(s);
            }
        }
    }

    // 创建新会话
    let app = AppInfo {
        bundle_id: bundle_id.to_string(),
        name: name.to_string(),
        window_title: get_window_title_sync(),
        pid,
    };
    {
        let mut state = SESSION_STATE.lock().unwrap();
        *state = Some(create_new_session(&app, now));
    }

    let _ = crate::db::insert_raw_event(
        now,
        "app_focus",
        Some(bundle_id),
        Some(name),
        None,
        None,
        0,
        None,
        &serde_json::to_string(&json!({
            "pid": pid,
        })).unwrap_or_default(),
    );
}

// ============================================================
// IME IPC 集成
// ============================================================

/// 初始化 IME IPC 接收器
pub fn init_ime_receiver() -> Result<()> {
    let receiver = crate::ime_ipc::start_ime_ipc_server()?;
    *IME_EVENT_RECEIVER.lock().unwrap() = Some(receiver);
    Ok(())
}

/// 轮询 IME 事件（在定时器线程中调用）
fn poll_ime_events() {
    let guard = IME_EVENT_RECEIVER.lock().unwrap();
    if let Some(ref receiver) = *guard {
        while let Ok(event) = receiver.try_recv() {
            process_ime_text_event(event);
        }
    }
}

/// 处理来自输入法的文本事件
fn process_ime_text_event(event: IMETextEvent) {
    if *PAUSED.lock().unwrap() {
        return;
    }

    // 获取当前应用信息用于黑名单检查
    let app = if let Some(ref bundle_id) = event.app_bundle_id {
        AppInfo {
            bundle_id: bundle_id.clone(),
            name: event.app_name.clone().unwrap_or_default(),
            window_title: get_window_title_sync(),
            pid: 0,
        }
    } else {
        get_active_app_sync()
    };

    // 检查黑名单
    if is_blacklisted_sync(&app.bundle_id) {
        return;
    }

    // 检查是否为密码字段
    #[cfg(target_os = "macos")]
    if is_secure_text_field() {
        return;
    }

    match event.text_type {
        IMETextType::Committed => {
            // 已提交的文本（如"你好"）
            record_ime_committed_text(&event);
        }
        IMETextType::Direct => {
            // 直接输入（英文、数字）
            // 记录去重标记，用于与 CGEventTap 去重
            *LAST_IME_DIRECT_TEXT.lock().unwrap() = Some((event.text.clone(), Instant::now()));
            record_ime_direct_text(&event);
        }
        IMETextType::Composing => {
            // 组合中的文本（拼音、候选状态）
            // 可选：记录用于分析输入过程，目前仅更新实时状态
            record_ime_composing_text(&event);
        }
    }
}

/// 判断 CGEventTap 的按键是否应与 IME Direct 去重
///
/// 逻辑：如果最近 500ms 内有 IME Direct 文本，且当前按键字符是该文本的一部分，
/// 则认为该按键已被 IME 覆盖，跳过 CGEventTap 记录。
fn should_dedup_cgevent_key(key_char: &str) -> bool {
    let guard = LAST_IME_DIRECT_TEXT.lock().unwrap();
    if let Some((ref text, timestamp)) = *guard {
        if timestamp.elapsed() < Duration::from_millis(500) {
            // 简单匹配：如果按键字符包含在 IME Direct 文本中
            return text.contains(key_char);
        }
    }
    false
}

/// 记录 IME 已提交文本
fn record_ime_committed_text(event: &IMETextEvent) {
    let now = event.timestamp;
    let app = if let Some(ref bundle_id) = event.app_bundle_id {
        AppInfo {
            bundle_id: bundle_id.clone(),
            name: event.app_name.clone().unwrap_or_default(),
            window_title: get_window_title_sync(),
            pid: 0,
        }
    } else {
        get_active_app_sync()
    };

    if is_blacklisted_sync(&app.bundle_id) {
        return;
    }

    // 写入原始事件
    let _ = crate::db::insert_raw_event(
        now,
        "ime_committed",
        Some(&app.bundle_id),
        Some(&app.name),
        app.window_title.as_deref(),
        Some(&event.text),
        0,
        None,
        &serde_json::to_string(&json!({
            "source": "ime",
            "text_type": "committed",
            "length": event.text.chars().count(),
        })).unwrap_or_default(),
    );

    // 更新会话 buffer
    {
        let mut state = SESSION_STATE.lock().unwrap();
        if let Some(s) = state.as_mut() {
            if s.app_bundle_id == app.bundle_id {
                s.buffer.push_str(&event.text);
                s.total_keystrokes += event.text.chars().count() as i32;
                s.last_input_time = Instant::now();
            } else {
                let _ = save_session(s);
                *s = create_new_session(&app, now);
                s.buffer.push_str(&event.text);
                s.total_keystrokes += event.text.chars().count() as i32;
            }
        } else {
            let mut new_session = create_new_session(&app, now);
            new_session.buffer.push_str(&event.text);
            new_session.total_keystrokes += event.text.chars().count() as i32;
            *state = Some(new_session);
        }
    }

    {
        let mut count = EVENT_COUNT.lock().unwrap();
        *count += 1;
    }
}

/// 记录 IME 直接输入文本（英文、数字、符号）
fn record_ime_direct_text(event: &IMETextEvent) {
    let now = event.timestamp;
    let app = if let Some(ref bundle_id) = event.app_bundle_id {
        AppInfo {
            bundle_id: bundle_id.clone(),
            name: event.app_name.clone().unwrap_or_default(),
            window_title: get_window_title_sync(),
            pid: 0,
        }
    } else {
        get_active_app_sync()
    };

    if is_blacklisted_sync(&app.bundle_id) {
        return;
    }

    // 写入原始事件
    let _ = crate::db::insert_raw_event(
        now,
        "ime_direct",
        Some(&app.bundle_id),
        Some(&app.name),
        app.window_title.as_deref(),
        Some(&event.text),
        0,
        None,
        &serde_json::to_string(&json!({
            "source": "ime",
            "text_type": "direct",
            "length": event.text.chars().count(),
        })).unwrap_or_default(),
    );

    // 更新会话 buffer
    {
        let mut state = SESSION_STATE.lock().unwrap();
        if let Some(s) = state.as_mut() {
            if s.app_bundle_id == app.bundle_id {
                s.buffer.push_str(&event.text);
                s.total_keystrokes += event.text.chars().count() as i32;
                s.last_input_time = Instant::now();
            } else {
                let _ = save_session(s);
                *s = create_new_session(&app, now);
                s.buffer.push_str(&event.text);
                s.total_keystrokes += event.text.chars().count() as i32;
            }
        } else {
            let mut new_session = create_new_session(&app, now);
            new_session.buffer.push_str(&event.text);
            new_session.total_keystrokes += event.text.chars().count() as i32;
            *state = Some(new_session);
        }
    }

    {
        let mut count = EVENT_COUNT.lock().unwrap();
        *count += 1;
    }
}

/// 记录 IME 组合中文本（拼音、候选状态）
fn record_ime_composing_text(event: &IMETextEvent) {
    // 组合中事件目前仅用于实时状态展示，不写入数据库
    // 未来可用于分析输入过程（如拼音纠错、候选选择等）
    let _ = event;
}

// ============================================================
// macOS CGEventTap FFI 声明
// ============================================================

#[cfg(target_os = "macos")]
use std::ffi::c_void;

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,              // CGEventTapLocation
        place: u32,            // CGEventTapPlacement
        options: u32,          // CGEventTapOptions
        events_of_interest: u64, // CGEventMask
        callback: extern "C" fn(*mut c_void, u32, *mut c_void, *mut c_void) -> *mut c_void,
        user_info: *mut c_void,
    ) -> *mut c_void; // CFMachPortRef

    fn CGEventGetType(event: *mut c_void) -> u32;
    fn CGEventGetIntegerValueField(event: *mut c_void, field: u32) -> i64;
    fn CGEventGetLocation(event: *mut c_void) -> CGPoint;
    fn CGEventGetFlags(event: *mut c_void) -> u64;
    fn CGEventKeyboardGetUnicodeString(
        event: *mut c_void,
        max_string_length: u32,
        actual_string_length: *mut u32,
        unicode_string: *mut u16,
    );

}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Copy, Clone, Default)]
struct CGPoint {
    x: f64,
    y: f64,
}

// macOS CGEvent constants (these are #define macros in C, not linkable symbols)
#[cfg(target_os = "macos")]
const KCG_EVENT_LEFT_MOUSE_DOWN: u64 = 1;
#[cfg(target_os = "macos")]
const KCG_EVENT_RIGHT_MOUSE_DOWN: u64 = 3;
#[cfg(target_os = "macos")]
const KCG_EVENT_KEY_DOWN: u64 = 10;
#[cfg(target_os = "macos")]
const KCG_EVENT_KEY_UP: u64 = 11;
#[cfg(target_os = "macos")]
const KCG_HID_EVENT_TAP: u32 = 0;
#[cfg(target_os = "macos")]
const KCG_HEAD_INSERT_EVENT_TAP: u32 = 0;
#[cfg(target_os = "macos")]
const KCG_EVENT_TAP_OPTION_LISTEN_ONLY: u32 = 1;
#[cfg(target_os = "macos")]
const KCG_EVENT_KEYBOARD_EVENT_KEYCODE: u32 = 9;

#[cfg(target_os = "macos")]
fn start_cgevent_tap() -> Result<()> {
    std::thread::spawn(move || {
        unsafe {
            // CGEventMask 是位掩码：bit N 对应事件类型 N
            // 所以要监听事件类型 T，需要设置 1 << T
            let events_of_interest = (1u64 << KCG_EVENT_KEY_DOWN)
                | (1u64 << KCG_EVENT_KEY_UP)
                | (1u64 << KCG_EVENT_LEFT_MOUSE_DOWN)
                | (1u64 << KCG_EVENT_RIGHT_MOUSE_DOWN);

            eprintln!("🔧 正在创建 CGEventTap... (events mask: {:b})", events_of_interest);

            let tap = CGEventTapCreate(
                KCG_HID_EVENT_TAP,
                KCG_HEAD_INSERT_EVENT_TAP,
                KCG_EVENT_TAP_OPTION_LISTEN_ONLY,
                events_of_interest,
                event_tap_callback,
                std::ptr::null_mut(),
            );

            if tap.is_null() {
                eprintln!("⚠️ CGEventTap 创建失败，将使用降级模式");
                return;
            }

            eprintln!("✅ CGEventTap 创建成功，开始监听键盘/鼠标事件");

            // Create a CFRunLoopSource from the mach port
            let run_loop_source = CFMachPortCreateRunLoopSource(
                std::ptr::null_mut(),
                tap,
                0,
            );

            if run_loop_source.is_null() {
                eprintln!("⚠️ CFRunLoopSource 创建失败");
                return;
            }

            let run_loop = CFRunLoopGetCurrent();
            CFRunLoopAddSource(run_loop, run_loop_source, kCFRunLoopDefaultMode);
            eprintln!("✅ RunLoop 启动，等待事件...");
            CFRunLoopRun();
        }
    });

    Ok(())
}

#[cfg(target_os = "macos")]
extern "C" fn event_tap_callback(
    _proxy: *mut c_void,
    _event_type: u32,
    event: *mut c_void,
    _user_info: *mut c_void,
) -> *mut c_void {
    if event.is_null() {
        return event;
    }
    unsafe {
        let actual_type = CGEventGetType(event);

        // 调试：首次收到事件时打印
        static mut FIRST_EVENT: bool = true;
        if FIRST_EVENT {
            eprintln!("🎯 CGEventTap 回调被触发！event_type={}, CGEventGetType={}", _event_type, actual_type);
            FIRST_EVENT = false;
        }

        if actual_type == KCG_EVENT_KEY_DOWN as u32 {
            let key_code = CGEventGetIntegerValueField(event, KCG_EVENT_KEYBOARD_EVENT_KEYCODE) as i64;
            let flags = CGEventGetFlags(event);

            // kCGEventFlagMaskCommand = 1 << 20
            // kCGEventFlagMaskShift   = 1 << 17
            // kCGEventFlagMaskOption  = 1 << 19
            // kCGEventFlagMaskControl = 1 << 18
            let is_cmd = (flags & (1u64 << 20)) != 0;
            let _is_alt = (flags & (1u64 << 19)) != 0;
            let _is_ctrl = (flags & (1u64 << 18)) != 0;
            let is_fn = (flags & (1u64 << 23)) != 0;

            // === 排除与输入无关的快捷键 ===

            // Cmd+V 粘贴 → 单独处理（记录粘贴内容）
            if is_cmd && key_code == 9 {
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    record_paste_action();
                }));
                return event;
            }

            // Cmd+X 剪切 / Cmd+C 复制 → 不记录为键盘事件（剪贴板监听已覆盖）
            if is_cmd && (key_code == 7 || key_code == 8) {
                // key_code 7 = X, key_code 8 = C
                return event;
            }

            // Fn 系列键（亮度、音量、媒体控制等）
            if is_fn {
                return event;
            }

            // 功能键 F1-F12 (key_code 122-123, 99-101, etc.)
            if key_code >= 122 && key_code <= 145 {
                return event;
            }

            // PrintScreen (key_code 114), ScrollLock (key_code 107), Pause (key_code 113)
            if key_code == 114 || key_code == 107 || key_code == 113 {
                return event;
            }

            // Eject / Power (key_code 122)
            if key_code == 122 {
                return event;
            }

            // 纯修饰键按下（没有实际字符输出）
            // Cmd, Shift, Option, Control, Caps Lock, Fn
            if matches!(key_code, 55 | 56 | 58 | 59 | 57 | 63) {
                return event;
            }

            // Alt/Option + 字母（macOS 输入特殊字符的快捷方式，非功能性快捷键）
            // 这些实际上会输入字符（如 Alt+E = €），所以保留记录

            // Ctrl + 字母（终端快捷键等，通常不产生文本输入）
            // 但 Ctrl+C 在终端中是中断信号，Ctrl+V 是粘贴，保留记录

            // Cmd + 其他非输入快捷键（Cmd+Tab, Cmd+Q, Cmd+W, Cmd+H, Cmd+M, Cmd+N 等）
            // 这些不产生文本输入，排除
            if is_cmd {
                // 排除常见的非输入 Cmd 快捷键
                let non_input_cmd_keys = [
                    48, // Tab
                    12, // Q (退出)
                    13, // W (关闭窗口)
                    32, // [ (后退)
                    34, // I (反向标签)
                    36, // Return
                    37, // ] (前进)
                    38, // K (清除)
                    39, // L
                    40, // ; (查找)
                    41, // ' (不匹配)
                    42, // \ (后退)
                    43, // , (减少)
                    44, // . (增加)
                    45, // / (前进)
                    46, // M (最小化)
                    47, // N (新建窗口)
                    50, // ` (切换窗口)
                    53, // Escape
                ];
                if non_input_cmd_keys.contains(&key_code) {
                    return event;
                }
            }

            // === 记录有效键盘输入 ===

            // 使用 CGEventKeyboardGetUnicodeString 获取实际字符
            let key_char = get_unicode_string_from_event(event);
            let is_backspace = key_code == 51; // macOS backspace key code

            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                record_key_event(key_code, key_char.as_deref(), is_backspace);
            }));
        } else if actual_type == KCG_EVENT_LEFT_MOUSE_DOWN as u32
            || actual_type == KCG_EVENT_RIGHT_MOUSE_DOWN as u32
        {
            let location = CGEventGetLocation(event);
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                record_mouse_click(location.x, location.y);
            }));
        }
    }

    event
}

/// 从 CGEvent 获取 Unicode 字符串
#[cfg(target_os = "macos")]
unsafe fn get_unicode_string_from_event(event: *mut c_void) -> Option<String> {
    let mut buffer = [0u16; 128];
    let mut length: u32 = 0;
    CGEventKeyboardGetUnicodeString(event, 128, &mut length, buffer.as_mut_ptr());

    if length > 0 {
        let chars: String = buffer[..length as usize]
            .iter()
            .filter_map(|&c| char::from_u32(c as u32))
            .collect();
        if chars.is_empty() { None } else { Some(chars) }
    } else {
        None
    }
}

/// 记录粘贴操作（Cmd+V）
#[cfg(target_os = "macos")]
fn record_paste_action() {
    use objc2_app_kit::NSPasteboard;

    let pb = NSPasteboard::generalPasteboard();
    if let Some(content) = unsafe { pb.stringForType(objc2_app_kit::NSPasteboardTypeString) } {
        let text = content.to_string();
        if !text.is_empty() && !is_sensitive_clipboard_content(&text) {
            let now = chrono::Utc::now().timestamp_millis();
            let app = get_active_app_sync();

            let _ = crate::db::insert_raw_event(
                now,
                "paste",
                Some(&app.bundle_id),
                Some(&app.name),
                app.window_title.as_deref(),
                Some(&text),
                0,
                None,
                &serde_json::to_string(&json!({
                    "length": text.len(),
                    "trigger": "cmd_v",
                })).unwrap_or_default(),
            );

            let mut count = EVENT_COUNT.lock().unwrap();
            *count += 1;
        }
    }
}

// ============================================================
// macOS NSWorkspace 应用切换监听
// ============================================================

#[cfg(target_os = "macos")]
fn start_app_switch_listener() -> Result<()> {
    // Use polling instead of NSWorkspace notifications to avoid objc2 API issues
    std::thread::spawn(move || {
        let mut last_bundle_id = String::new();
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let app = get_active_app_sync();
            if app.bundle_id != last_bundle_id && !last_bundle_id.is_empty() {
                record_app_switch(&app.bundle_id, &app.name, app.pid);
            }
            last_bundle_id = app.bundle_id;
        }
    });

    Ok(())
}

// ============================================================
// macOS 剪贴板监听
// ============================================================

#[cfg(target_os = "macos")]
fn start_clipboard_watcher() -> Result<()> {
    use objc2_app_kit::NSPasteboard;

    std::thread::spawn(move || {
        let mut last_change_count: i64 = {
            let pb = NSPasteboard::generalPasteboard();
            pb.changeCount() as i64
        };

        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));

            // 检查是否启用了剪贴板监听
            let enabled = crate::config::get_config()
                .ok()
                .and_then(|c| c.as_object().cloned())
                .and_then(|obj| obj.get("enable_clipboard").and_then(|v| v.as_str()).map(|s| s == "true"))
                .unwrap_or(true);
            if !enabled {
                continue;
            }

            let pb = NSPasteboard::generalPasteboard();
            let current_count = pb.changeCount() as i64;

            if current_count != last_change_count {
                last_change_count = current_count;

                // 读取剪贴板内容
                if let Some(content) = unsafe {
                    let string = pb.stringForType(
                        objc2_app_kit::NSPasteboardTypeString
                    );
                    string.map(|s| s.to_string())
                } {
                    let source = "unknown";
                    record_clipboard_change(&content, source);
                }
            }
        }
    });

    Ok(())
}

// ============================================================
// macOS 窗口标题获取
// ============================================================

#[cfg(target_os = "macos")]
fn get_window_title_sync() -> Option<String> {
    // Use AppleScript to get the focused window title
    let script = r#"
        tell application "System Events"
            tell (first process whose frontmost is true)
                tell window 1
                    return name
                end tell
            end tell
        end tell
    "#;

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;

    if output.status.success() {
        let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if title.is_empty() { None } else { Some(title) }
    } else {
        None
    }
}

#[cfg(not(target_os = "macos"))]
fn get_window_title_sync() -> Option<String> {
    None
}

// ============================================================
// macOS 密码字段检测
// ============================================================

#[cfg(target_os = "macos")]
fn is_secure_text_field() -> bool {
    // 密码字段检测：通过检查当前应用是否在密码管理器/安全应用黑名单中
    // 以及窗口标题是否包含密码相关关键词
    // 注意：由于 Tauri 应用无法直接访问其他应用的 UI 元素树，
    // 这里采用基于应用和窗口标题的启发式检测
    let app = get_active_app_sync();

    // 密码管理器应用始终视为安全字段
    let secure_apps = [
        "com.agilebits.onepassword7",
        "com.agilebits.onepassword-osx",
        "com.bitwarden.desktop",
        "org.keepassxc.keepassxc",
        "com.apple.keychainaccess",
        "com.apple.SecurityAgent",
        "com.1password.1password",
        "com.lastpass.lastpass",
    ];
    if secure_apps.contains(&app.bundle_id.as_str()) {
        return true;
    }

    // 窗口标题包含密码相关关键词
    if let Some(title) = &app.window_title {
        let lower = title.to_lowercase();
        if lower.contains("password") || lower.contains("login") || lower.contains("sign in")
            || lower.contains("密码") || lower.contains("登录") || lower.contains("认证") {
            return true;
        }
    }

    false
}

#[cfg(not(target_os = "macos"))]
fn is_secure_text_field() -> bool {
    false
}

// ============================================================
// 敏感内容检测
// ============================================================

fn is_sensitive_clipboard_content(content: &str) -> bool {
    // JWT Token
    if content.starts_with("eyJ") && content.contains('.') && content.matches('.').count() >= 2 {
        return true;
    }
    // AWS Access Key
    if content.starts_with("AKIA") && content.len() >= 16 && content.len() <= 24 {
        return true;
    }
    // 私钥
    if content.contains("-----BEGIN") && content.contains("PRIVATE KEY-----") {
        return true;
    }
    // 信用卡号（Luhn 算法验证）
    let digits: String = content.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 13 && digits.len() <= 19 && is_luhn_valid(&digits) {
        return true;
    }
    // 过长内容
    if content.len() > 1000 {
        return true;
    }
    false
}

/// Luhn 算法验证信用卡号
fn is_luhn_valid(digits: &str) -> bool {
    let nums: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();
    if nums.len() < 13 {
        return false;
    }
    let mut sum = 0;
    let mut double = false;
    for &n in nums.iter().rev() {
        let mut val = n;
        if double {
            val *= 2;
            if val > 9 {
                val -= 9;
            }
        }
        sum += val;
        double = !double;
    }
    sum % 10 == 0
}

// ============================================================
// 黑名单检查
// ============================================================

fn is_blacklisted_sync(bundle_id: &str) -> bool {
    let blacklist = [
        // 密码管理器
        "com.agilebits.onepassword7",
        "com.agilebits.onepassword-osx",
        "com.bitwarden.desktop",
        "org.keepassxc.keepassxc",
        "com.lastpass.lastpass",
        "com.1password.1password",
        // 系统安全
        "com.apple.keychainaccess",
        "com.apple.SecurityAgent",
        "com.apple.coreservices.uiagent",
        // 银行/金融
        "com.icbc.icbcclient",
        "com.ccb.ccbclient",
        "com.boc.bocclient",
        "com.cmb.cmbclient",
        // 浏览器隐私模式（按需添加）
        // VPN/安全工具
        "com.tunnelbear.mac.TunnelBear",
        "com.openvpn.openvpn",
    ];
    blacklist.contains(&bundle_id)
}

// ============================================================
// 获取当前活跃应用（同步版本）
// ============================================================

#[cfg(target_os = "macos")]
pub fn get_active_app_sync() -> AppInfo {
    use objc2_app_kit::NSWorkspace;

    {
        let workspace = NSWorkspace::sharedWorkspace();
        if let Some(front_app) = workspace.frontmostApplication() {
            let bundle_id = front_app.bundleIdentifier()
                .map(|s| s.to_string())
                .unwrap_or_default();
            let name = front_app.localizedName()
                .map(|s| s.to_string())
                .unwrap_or_default();
            let pid = front_app.processIdentifier() as i32;
            let window_title = get_window_title_sync();

            return AppInfo {
                bundle_id,
                name,
                window_title,
                pid,
            };
        }
    }

    AppInfo {
        bundle_id: "com.apple.unknown".to_string(),
        name: "Unknown".to_string(),
        window_title: None,
        pid: 0,
    }
}

#[cfg(not(target_os = "macos"))]
pub fn get_active_app_sync() -> AppInfo {
    AppInfo {
        bundle_id: "com.example.dev".to_string(),
        name: "Development".to_string(),
        window_title: Some("Editor".to_string()),
        pid: 0,
    }
}

// ============================================================
// 实时状态更新
// ============================================================

fn update_realtime_status_sync() -> Result<()> {
    let app = get_active_app_sync();
    let paused = *PAUSED.lock().unwrap();
    let count = *EVENT_COUNT.lock().unwrap();
    let now = chrono::Utc::now().timestamp_millis();

    // 检查会话超时：如果会话超过 session_timeout 秒无输入，自动保存
    let session_timeout_secs: u64 = crate::config::get_config()
        .ok()
        .and_then(|c| c.as_object().cloned())
        .and_then(|obj| obj.get("session_timeout").and_then(|v| v.as_str()).and_then(|s| s.parse::<u64>().ok()))
        .unwrap_or(60);

    {
        let mut state = SESSION_STATE.lock().unwrap();
        let should_save = state.as_ref().map(|s| s.last_input_time.elapsed().as_secs() > session_timeout_secs).unwrap_or(false);
        if should_save {
            if let Some(old_session) = state.take() {
                let _ = save_session(&old_session);
            }
        }
    }

    let mut status = REALTIME_STATUS.lock().unwrap();
    *status = Some(json!({
        "timestamp": now,
        "app_name": app.name,
        "app_bundle_id": app.bundle_id,
        "window_title": app.window_title,
        "recording": !paused,
        "paused": paused,
        "today_events": count,
    }));

    Ok(())
}

// ============================================================
// 会话管理
// ============================================================

fn create_new_session(app: &AppInfo, now: i64) -> SessionState {
    SessionState {
        id: uuid::Uuid::new_v4().to_string(),
        app_bundle_id: app.bundle_id.clone(),
        app_name: Some(app.name.clone()),
        start_time: now,
        last_input_time: Instant::now(),
        buffer: String::new(),
        pause_count: 0,
        backspace_count: 0,
        total_keystrokes: 0,
    }
}

fn save_session(state: &SessionState) -> Result<()> {
    if state.buffer.is_empty() && state.total_keystrokes == 0 {
        return Ok(()); // 空会话不保存
    }

    let duration_ms = state.last_input_time.elapsed().as_millis() as i64;
    let end_time = state.start_time + duration_ms;
    let char_count = state.buffer.chars().count() as i64;
    let duration_sec = duration_ms / 1000;

    // 1. 保存到 raw_events（原始事件流）
    let _ = crate::db::insert_raw_event(
        state.start_time,
        "session_end",
        Some(&state.app_bundle_id),
        state.app_name.as_deref(),
        None,
        if state.buffer.is_empty() { None } else { Some(&state.buffer) },
        0,
        Some(&state.id),
        &serde_json::to_string(&json!({
            "pause_count": state.pause_count,
            "backspace_count": state.backspace_count,
            "char_count": char_count,
            "total_keystrokes": state.total_keystrokes,
            "duration_ms": duration_ms,
            "typing_speed": crate::stats::calculate_typing_speed(char_count, duration_sec.max(1)),
            "wpm": crate::stats::calculate_wpm(char_count, duration_sec.max(1)),
            "backspace_rate": crate::stats::calculate_backspace_rate(state.backspace_count as i64, char_count.max(1)),
            "focus_score": crate::stats::calculate_focus_score(state.pause_count as i64, duration_sec.max(1)),
        })).unwrap_or_default(),
    );

    // 2. 保存到 input_sessions（会话聚合表）
    let _ = crate::db::get_conn()?.lock().unwrap().execute(
        "INSERT INTO input_sessions (id, app_bundle_id, app_name, start_time, end_time, char_count, text_preview, pause_count, context_tag) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            &state.id,
            &state.app_bundle_id,
            state.app_name.as_deref(),
            state.start_time,
            end_time,
            char_count,
            state.buffer.chars().take(200).collect::<String>(),
            state.pause_count,
            None::<&str>,
        ],
    );

    // 3. 自动提取 TODO
    if let Some(todo_text) = crate::todo::extract_todo_from_text(&state.buffer) {
        let _ = crate::db::insert_todo(&todo_text, Some(&state.id), chrono::Utc::now().timestamp());
    }

    Ok(())
}

// ============================================================
// macOS Accessibility Permission Check
// ============================================================

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: *mut c_void) -> bool;
}

/// 请求辅助功能权限（弹出系统对话框）
#[cfg(target_os = "macos")]
pub fn request_accessibility_permission() {
    // AXIsProcessTrustedWithOptions with NULL just checks without prompting.
    // The system dialog for accessibility permission is triggered automatically
    // by CGEventTapCreate when the app tries to use accessibility features.
    // Users can also manually add the app in System Settings.
    let _ = unsafe { AXIsProcessTrustedWithOptions(std::ptr::null_mut()) };
}

#[cfg(target_os = "macos")]
pub fn check_accessibility_permission() -> bool {
    unsafe {
        AXIsProcessTrustedWithOptions(std::ptr::null_mut())
    }
}

#[cfg(not(target_os = "macos"))]
pub fn check_accessibility_permission() -> bool {
    true
}

// ============================================================
// macOS CoreFoundation FFI
// ============================================================

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRunLoopGetCurrent() -> *mut c_void;
    fn CFRunLoopAddSource(rl: *mut c_void, source: *mut c_void, mode: *mut c_void);
    fn CFRunLoopRun();
    static kCFRunLoopDefaultMode: *mut c_void;
    fn CFMachPortCreateRunLoopSource(
        allocator: *mut c_void,
        port: *mut c_void,
        order: i32,
    ) -> *mut c_void;
}
