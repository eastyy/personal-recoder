use anyhow::Result;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::AppHandle;

#[derive(Clone, Debug)]
pub struct FocusTarget {
    pub target_type: String,  // "app" | "domain"
    pub target_id: String,
    pub target_name: String,
}

#[derive(Clone, Debug)]
pub struct FocusSession {
    pub start_time: i64,
    pub last_seen: Instant,
    pub duration: i64,
    pub target: FocusTarget,
}

static CURRENT_FOCUS: Mutex<Option<FocusSession>> = Mutex::new(None);

pub fn start_focus_tracker(_app: AppHandle) -> Result<()> {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(5));
            if let Err(e) = focus_tick() {
                eprintln!("Focus tracker error: {}", e);
            }
        }
    });
    Ok(())
}

fn focus_tick() -> Result<()> {
    let active = get_active_target()?;
    let now = chrono::Utc::now().timestamp();

    let mut current = CURRENT_FOCUS.lock().unwrap();

    match current.as_mut() {
        Some(session) if session.target.target_id == active.target_id => {
            // 同一目标，累加时长
            session.duration += 5;
            session.last_seen = Instant::now();
        }
        Some(session) => {
            // 切换目标，保存旧会话
            let elapsed = session.last_seen.elapsed().as_secs() as i64;
            if elapsed <= 60 {
                // 连续，保存
                save_focus_session(session)?;
            }
            *current = Some(create_new_session(&active, now));
        }
        None => {
            *current = Some(create_new_session(&active, now));
        }
    }

    Ok(())
}

fn get_active_target() -> Result<FocusTarget> {
    #[cfg(target_os = "macos")]
    {
        let app = crate::input::get_active_app_sync();
        let bundle_id = app.bundle_id.clone();
        let name = app.name.clone();

        // 如果是浏览器，尝试获取 URL 并提取域名
        if is_browser(&bundle_id) {
            if let Some(url) = get_browser_url(&bundle_id) {
                if let Some(domain) = extract_domain(&url) {
                    return Ok(FocusTarget {
                        target_type: "domain".to_string(),
                        target_id: format!("domain:{}", domain),
                        target_name: format!("{} ({})", name, domain),
                    });
                }
            }
        }

        Ok(FocusTarget {
            target_type: "app".to_string(),
            target_id: bundle_id,
            target_name: name,
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Linux 开发环境：模拟
        Ok(FocusTarget {
            target_type: "app".to_string(),
            target_id: "com.example.dev".to_string(),
            target_name: "Development".to_string(),
        })
    }
}

/// 判断是否为浏览器应用
fn is_browser(bundle_id: &str) -> bool {
    matches!(
        bundle_id,
        "com.apple.Safari"
            | "com.google.Chrome"
            | "com.google.Chrome.canary"
            | "org.mozilla.firefox"
            | "com.microsoft.edgemac"
            | "com.brave.Browser"
            | "com.apple.SafariTechnologyPreview"
            | "com.operasoftware.Opera"
            | "com.vivaldi.Vivaldi"
    )
}

/// 通过 AppleScript 获取浏览器当前标签页 URL
#[cfg(target_os = "macos")]
fn get_browser_url(bundle_id: &str) -> Option<String> {
    let script = match bundle_id {
        "com.apple.Safari" | "com.apple.SafariTechnologyPreview" => {
            r#"tell application "Safari" to get URL of current tab of front window"#
        }
        "com.google.Chrome" | "com.google.Chrome.canary" => {
            r#"tell application "Google Chrome" to get URL of active tab of front window"#
        }
        "org.mozilla.firefox" => {
            r#"tell application "Firefox" to get URL of active tab of front window"#
        }
        "com.microsoft.edgemac" => {
            r#"tell application "Microsoft Edge" to get URL of active tab of front window"#
        }
        "com.brave.Browser" => {
            r#"tell application "Brave Browser" to get URL of active tab of front window"#
        }
        "com.operasoftware.Opera" => {
            r#"tell application "Opera" to get URL of active tab of front window"#
        }
        "com.vivaldi.Vivaldi" => {
            r#"tell application "Vivaldi" to get URL of active tab of front window"#
        }
        _ => return None,
    };

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;

    if output.status.success() {
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if url.starts_with("http://") || url.starts_with("https://") {
            return Some(url);
        }
    }

    None
}

#[cfg(not(target_os = "macos"))]
fn get_browser_url(_bundle_id: &str) -> Option<String> {
    None
}

/// 从 URL 中提取域名
fn extract_domain(url: &str) -> Option<String> {
    // 去掉协议前缀
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;

    // 取第一个 / 之前的部分
    let host = without_scheme.split('/').next()?;

    // 去掉端口号
    let domain = host.split(':').next()?;

    // 去掉 www. 前缀
    let clean = domain.strip_prefix("www.").unwrap_or(domain);

    if clean.is_empty() {
        None
    } else {
        Some(clean.to_string())
    }
}

fn create_new_session(target: &FocusTarget, now: i64) -> FocusSession {
    FocusSession {
        start_time: now,
        last_seen: Instant::now(),
        duration: 5,
        target: target.clone(),
    }
}

fn save_focus_session(session: &FocusSession) -> Result<()> {
    let end_time = session.start_time + session.duration;

    crate::db::insert_focus_session(
        session.start_time,
        end_time,
        session.duration,
        &session.target.target_type,
        &session.target.target_id,
        &session.target.target_name,
    )?;

    Ok(())
}
