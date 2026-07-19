use anyhow::Result;
use rusqlite::{Connection, params};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use chrono::Timelike;

static DB_CONN: Mutex<Option<Arc<Mutex<Connection>>>> = Mutex::new(None);

pub fn init_db(_app: AppHandle) -> Result<()> {
    let db_path = get_db_path()?;
    let conn = Connection::open(&db_path)?;

    conn.execute_batch(SCHEMA)?;

    let mut guard = DB_CONN.lock().unwrap();
    *guard = Some(Arc::new(Mutex::new(conn)));

    Ok(())
}

fn get_db_path() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
        let dir = home.join("Library/Application Support/PersonalLogAI");
        std::fs::create_dir_all(&dir)?;
        Ok(dir.join("data.db"))
    }
    #[cfg(target_os = "linux")]
    {
        let data_dir = dirs::data_dir().ok_or_else(|| anyhow::anyhow!("Cannot find data directory"))?;
        let dir = data_dir.join("PersonalLogAI");
        std::fs::create_dir_all(&dir)?;
        Ok(dir.join("data.db"))
    }
    #[cfg(target_os = "windows")]
    {
        let data_dir = dirs::data_dir().ok_or_else(|| anyhow::anyhow!("Cannot find data directory"))?;
        let dir = data_dir.join("PersonalLogAI");
        std::fs::create_dir_all(&dir)?;
        Ok(dir.join("data.db"))
    }
}

pub fn get_conn() -> Result<Arc<Mutex<Connection>>> {
    let guard = DB_CONN.lock().unwrap();
    guard
        .as_ref()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS raw_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    app_bundle_id TEXT,
    app_name TEXT,
    window_title TEXT,
    content TEXT,
    is_sensitive INTEGER DEFAULT 0,
    session_id TEXT,
    metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_raw_events_ts ON raw_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_raw_events_app_ts ON raw_events(app_bundle_id, timestamp);

CREATE TABLE IF NOT EXISTS input_sessions (
    id TEXT PRIMARY KEY,
    app_bundle_id TEXT NOT NULL,
    app_name TEXT,
    start_time INTEGER NOT NULL,
    end_time INTEGER,
    char_count INTEGER DEFAULT 0,
    text_preview TEXT,
    pause_count INTEGER DEFAULT 0,
    context_tag TEXT,
    ai_analyzed INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS focus_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time INTEGER NOT NULL,
    end_time INTEGER NOT NULL,
    duration_sec INTEGER NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    target_name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS focus_daily (
    date TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    target_name TEXT NOT NULL,
    duration_sec INTEGER NOT NULL,
    session_count INTEGER DEFAULT 0,
    PRIMARY KEY (date, target_type, target_id)
);

CREATE TABLE IF NOT EXISTS focus_hourly (
    date TEXT NOT NULL,
    hour INTEGER NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    duration_sec INTEGER NOT NULL,
    PRIMARY KEY (date, hour, target_type, target_id)
);

CREATE TABLE IF NOT EXISTS analysis_results (
    id TEXT PRIMARY KEY,
    analysis_type TEXT NOT NULL,
    time_range_start INTEGER,
    time_range_end INTEGER,
    prompt_tokens INTEGER,
    completion_tokens INTEGER,
    result_text TEXT,
    result_json TEXT,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS todo_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    text TEXT NOT NULL,
    source_session TEXT,
    extracted_at INTEGER NOT NULL,
    due_date TEXT,
    status TEXT DEFAULT 'pending',
    completed_at INTEGER
);

CREATE TABLE IF NOT EXISTS flash_ideas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    text TEXT NOT NULL,
    source_session TEXT,
    captured_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS app_rules (
    bundle_id TEXT PRIMARY KEY,
    rule_type TEXT NOT NULL,
    enabled INTEGER DEFAULT 1,
    note TEXT
);

CREATE TABLE IF NOT EXISTS user_goals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    metric_type TEXT NOT NULL,
    target_value INTEGER NOT NULL,
    current_value INTEGER DEFAULT 0,
    period TEXT NOT NULL,
    start_date TEXT,
    end_date TEXT,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS app_config (
    key TEXT PRIMARY KEY,
    value TEXT,
    updated_at INTEGER
);

INSERT OR IGNORE INTO app_rules (bundle_id, rule_type, note) VALUES
('com.agilebits.onepassword7', 'blacklist', 'Password manager'),
('com.bitwarden.desktop', 'blacklist', 'Password manager'),
('org.keepassxc.keepassxc', 'blacklist', 'Password manager'),
('com.apple.keychainaccess', 'blacklist', 'Password manager');
"#;

pub fn insert_raw_event(
    timestamp: i64,
    event_type: &str,
    app_bundle_id: Option<&str>,
    app_name: Option<&str>,
    window_title: Option<&str>,
    content: Option<&str>,
    is_sensitive: i32,
    session_id: Option<&str>,
    metadata: &str,
) -> Result<()> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();
    conn.execute(
        "INSERT INTO raw_events (timestamp, event_type, app_bundle_id, app_name, window_title, content, is_sensitive, session_id, metadata) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![timestamp, event_type, app_bundle_id, app_name, window_title, content, is_sensitive, session_id, metadata],
    )?;
    Ok(())
}

pub fn insert_focus_session(start: i64, end: i64, duration: i64, target_type: &str, target_id: &str, target_name: &str) -> Result<()> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();
    conn.execute(
        "INSERT INTO focus_sessions (start_time, end_time, duration_sec, target_type, target_id, target_name) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![start, end, duration, target_type, target_id, target_name],
    )?;

    let date = chrono::DateTime::from_timestamp(start, 0).unwrap().format("%Y-%m-%d").to_string();
    let hour = chrono::DateTime::from_timestamp(start, 0).unwrap().hour() as i32;

    conn.execute(
        "INSERT INTO focus_daily (date, target_type, target_id, target_name, duration_sec, session_count) VALUES (?1, ?2, ?3, ?4, ?5, 1) ON CONFLICT(date, target_type, target_id) DO UPDATE SET duration_sec = duration_sec + excluded.duration_sec, session_count = session_count + 1",
        params![&date, target_type, target_id, target_name, duration],
    )?;

    conn.execute(
        "INSERT INTO focus_hourly (date, hour, target_type, target_id, duration_sec) VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(date, hour, target_type, target_id) DO UPDATE SET duration_sec = duration_sec + excluded.duration_sec",
        params![&date, hour, target_type, target_id, duration],
    )?;

    Ok(())
}

pub fn insert_todo(text: &str, source_session: Option<&str>, extracted_at: i64) -> Result<()> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();
    conn.execute(
        "INSERT INTO todo_items (text, source_session, extracted_at) VALUES (?1, ?2, ?3)",
        params![text, source_session, extracted_at],
    )?;
    Ok(())
}

pub fn insert_analysis_result(id: &str, analysis_type: &str, result_text: &str) -> Result<()> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();
    conn.execute(
        "INSERT INTO analysis_results (id, analysis_type, result_text, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, analysis_type, result_text, chrono::Utc::now().timestamp()],
    )?;
    Ok(())
}

pub fn get_daily_stats(date: &str) -> Result<Value> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();

    let total_input: i64 = conn.query_row(
        "SELECT COALESCE(SUM(char_count), 0) FROM input_sessions WHERE date(datetime(start_time, 'unixepoch')) = ?1",
        [date],
        |row| row.get(0),
    )?;

    let total_focus: i64 = conn.query_row(
        "SELECT COALESCE(SUM(duration_sec), 0) FROM focus_daily WHERE date = ?1",
        [date],
        |row| row.get(0),
    )?;

    let mut stmt = conn.prepare(
        "SELECT target_name, duration_sec FROM focus_daily WHERE date = ?1 AND target_type = 'app' ORDER BY duration_sec DESC LIMIT 5"
    )?;
    let rows = stmt.query_map([date], |row| {
        Ok(json!({
            "name": row.get::<_, String>(0)?,
            "duration": row.get::<_, i64>(1)?
        }))
    })?;
    let top_apps: Vec<Value> = rows.collect::<Result<Vec<_>, _>>()?;

    Ok(json!({
        "date": date,
        "total_input_chars": total_input,
        "total_focus_seconds": total_focus,
        "top_apps": top_apps
    }))
}

pub fn get_app_usage(start: i64, end: i64) -> Result<Vec<Value>> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT target_type, target_id, target_name, SUM(duration_sec) as total FROM focus_sessions WHERE start_time >= ?1 AND end_time <= ?2 GROUP BY target_type, target_id ORDER BY total DESC"
    )?;

    let rows = stmt.query_map(params![start, end], |row| {
        Ok(json!({
            "target_type": row.get::<_, String>(0)?,
            "target_id": row.get::<_, String>(1)?,
            "target_name": row.get::<_, String>(2)?,
            "duration": row.get::<_, i64>(3)?
        }))
    })?;
    let result: Vec<Value> = rows.collect::<Result<Vec<_>, _>>()?;

    Ok(result)
}

pub fn get_focus_summary(date: &str) -> Result<Value> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();

    let total: i64 = conn.query_row(
        "SELECT COALESCE(SUM(duration_sec), 0) FROM focus_daily WHERE date = ?1",
        [date],
        |row| row.get(0),
    )?;

    let mut stmt = conn.prepare(
        "SELECT target_name, duration_sec, target_type FROM focus_daily WHERE date = ?1 ORDER BY duration_sec DESC LIMIT 10"
    )?;
    let rows = stmt.query_map([date], |row| {
        Ok(json!({
            "name": row.get::<_, String>(0)?,
            "duration": row.get::<_, i64>(1)?,
            "type": row.get::<_, String>(2)?
        }))
    })?;
    let items: Vec<Value> = rows.collect::<Result<Vec<_>, _>>()?;

    Ok(json!({
        "date": date,
        "total_seconds": total,
        "items": items
    }))
}

pub fn query_events(params: Value) -> Result<Vec<Value>> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();

    let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(100) as i64;
    let offset = params.get("offset").and_then(|v| v.as_i64()).unwrap_or(0) as i64;
    let event_type = params.get("event_type").and_then(|v| v.as_str());
    let search = params.get("search").and_then(|v| v.as_str());
    let date = params.get("date").and_then(|v| v.as_str());

    // Build dynamic query
    let mut where_clauses: Vec<String> = Vec::new();
    let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    let mut param_idx = 1;

    if let Some(et) = event_type {
        where_clauses.push(format!("event_type = ?{}", param_idx));
        param_values.push(Box::new(et.to_string()));
        param_idx += 1;
    }

    if let Some(s) = search {
        where_clauses.push(format!("content LIKE ?{}", param_idx));
        param_values.push(Box::new(format!("%{}%", s)));
        param_idx += 1;
    }

    if let Some(d) = date {
        // date is "YYYY-MM-DD", convert to start/end timestamps
        if let Ok(date) = chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d") {
            let start_ts = date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
            let end_ts = date.and_hms_opt(23, 59, 59).unwrap().and_utc().timestamp();
            where_clauses.push(format!("timestamp >= ?{}", param_idx));
            param_values.push(Box::new(start_ts));
            param_idx += 1;
            where_clauses.push(format!("timestamp <= ?{}", param_idx));
            param_values.push(Box::new(end_ts));
            param_idx += 1;
        }
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let query = format!(
        "SELECT id, timestamp, event_type, app_bundle_id, app_name, window_title, content, session_id FROM raw_events {} ORDER BY timestamp DESC LIMIT ?{} OFFSET ?{}",
        where_sql, param_idx, param_idx + 1
    );

    param_values.push(Box::new(limit));
    param_values.push(Box::new(offset));

    let params_ref: Vec<&dyn rusqlite::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&query)?;
    let mapped = stmt.query_map(params_ref.as_slice(), |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?,
            "timestamp": row.get::<_, i64>(1)?,
            "event_type": row.get::<_, String>(2)?,
            "app_bundle_id": row.get::<_, Option<String>>(3)?,
            "app_name": row.get::<_, Option<String>>(4)?,
            "window_title": row.get::<_, Option<String>>(5)?,
            "content": row.get::<_, Option<String>>(6)?,
            "session_id": row.get::<_, Option<String>>(7)?,
        }))
    })?;

    Ok(mapped.collect::<Result<Vec<_>, _>>()?)
}

pub fn get_todos(status: Option<&str>) -> Result<Vec<Value>> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();

    let rows = if let Some(s) = status {
        let mut stmt = conn.prepare("SELECT id, text, status, extracted_at FROM todo_items WHERE status = ?1 ORDER BY extracted_at DESC")?;
        let mapped = stmt.query_map([s], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "text": row.get::<_, String>(1)?,
                "status": row.get::<_, String>(2)?,
                "extracted_at": row.get::<_, i64>(3)?,
            }))
        })?;
        mapped.collect::<Result<Vec<_>, _>>()?
    } else {
        let mut stmt = conn.prepare("SELECT id, text, status, extracted_at FROM todo_items ORDER BY extracted_at DESC")?;
        let mapped = stmt.query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "text": row.get::<_, String>(1)?,
                "status": row.get::<_, String>(2)?,
                "extracted_at": row.get::<_, i64>(3)?,
            }))
        })?;
        mapped.collect::<Result<Vec<_>, _>>()?
    };

    Ok(rows)
}

pub fn toggle_todo(id: i64) -> Result<()> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();
    conn.execute(
        "UPDATE todo_items SET status = CASE WHEN status = 'pending' THEN 'done' ELSE 'pending' END, completed_at = CASE WHEN status = 'pending' THEN ?1 ELSE NULL END WHERE id = ?2",
        params![chrono::Utc::now().timestamp(), id],
    )?;
    Ok(())
}

pub fn get_reports(analysis_type: Option<&str>) -> Result<Vec<Value>> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();

    let rows = if let Some(t) = analysis_type {
        let mut stmt = conn.prepare("SELECT id, analysis_type, result_text, created_at FROM analysis_results WHERE analysis_type = ?1 ORDER BY created_at DESC LIMIT 50")?;
        let mapped = stmt.query_map([t], |row| {
            Ok(json!({
                "id": row.get::<_, String>(0)?,
                "analysis_type": row.get::<_, String>(1)?,
                "result_text": row.get::<_, String>(2)?,
                "created_at": row.get::<_, i64>(3)?,
            }))
        })?;
        mapped.collect::<Result<Vec<_>, _>>()?
    } else {
        let mut stmt = conn.prepare("SELECT id, analysis_type, result_text, created_at FROM analysis_results ORDER BY created_at DESC LIMIT 50")?;
        let mapped = stmt.query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, String>(0)?,
                "analysis_type": row.get::<_, String>(1)?,
                "result_text": row.get::<_, String>(2)?,
                "created_at": row.get::<_, i64>(3)?,
            }))
        })?;
        mapped.collect::<Result<Vec<_>, _>>()?
    };

    Ok(rows)
}

#[allow(dead_code)]
pub fn get_input_sessions_for_analysis(start: i64, end: i64) -> Result<Vec<Value>> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, app_bundle_id, app_name, start_time, end_time, char_count, text_preview, context_tag FROM input_sessions WHERE start_time >= ?1 AND start_time <= ?2 ORDER BY start_time"
    )?;

    let rows = stmt.query_map(params![start, end], |row| {
        Ok(json!({
            "id": row.get::<_, String>(0)?,
            "app_bundle_id": row.get::<_, String>(1)?,
            "app_name": row.get::<_, Option<String>>(2)?,
            "start_time": row.get::<_, i64>(3)?,
            "end_time": row.get::<_, Option<i64>>(4)?,
            "char_count": row.get::<_, i64>(5)?,
            "text_preview": row.get::<_, Option<String>>(6)?,
            "context_tag": row.get::<_, Option<String>>(7)?,
        }))
    })?;
    let result: Vec<Value> = rows.collect::<Result<Vec<_>, _>>()?;

    Ok(result)
}

pub fn get_recent_events(start: i64, end: i64) -> Result<Vec<Value>> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, event_type, app_bundle_id, app_name, content FROM raw_events WHERE timestamp >= ?1 AND timestamp <= ?2 AND event_type IN ('keydown', 'clipboard', 'ime_committed') AND is_sensitive = 0 ORDER BY timestamp DESC LIMIT 200"
    )?;

    let rows = stmt.query_map(params![start, end], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?,
            "timestamp": row.get::<_, i64>(1)?,
            "event_type": row.get::<_, String>(2)?,
            "app_bundle_id": row.get::<_, Option<String>>(3)?,
            "app_name": row.get::<_, Option<String>>(4)?,
            "content": row.get::<_, Option<String>>(5)?,
        }))
    })?;
    let result: Vec<Value> = rows.collect::<Result<Vec<_>, _>>()?;

    Ok(result)
}

// ============================================================
// Flash Ideas (闪念)
// ============================================================

pub fn insert_flash_idea(text: &str, source_session: Option<&str>) -> Result<()> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();
    conn.execute(
        "INSERT INTO flash_ideas (text, source_session, captured_at) VALUES (?1, ?2, ?3)",
        params![text, source_session, chrono::Utc::now().timestamp()],
    )?;
    Ok(())
}

pub fn get_flash_ideas(limit: Option<i64>) -> Result<Vec<Value>> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();
    let lim = limit.unwrap_or(100);
    let mut stmt = conn.prepare(
        "SELECT id, text, source_session, captured_at FROM flash_ideas ORDER BY captured_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![lim], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?,
            "text": row.get::<_, String>(1)?,
            "source_session": row.get::<_, Option<String>>(2)?,
            "captured_at": row.get::<_, i64>(3)?,
        }))
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn delete_flash_idea(id: i64) -> Result<()> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();
    conn.execute("DELETE FROM flash_ideas WHERE id = ?1", params![id])?;
    Ok(())
}

// ============================================================
// User Goals (目标追踪)
// ============================================================

pub fn insert_goal(title: &str, metric_type: &str, target_value: i64, period: &str) -> Result<()> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();
    conn.execute(
        "INSERT INTO user_goals (title, metric_type, target_value, current_value, period, created_at) VALUES (?1, ?2, ?3, 0, ?4, ?5)",
        params![title, metric_type, target_value, period, chrono::Utc::now().timestamp()],
    )?;
    Ok(())
}

pub fn get_goals() -> Result<Vec<Value>> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, title, metric_type, target_value, current_value, period, start_date, end_date, created_at FROM user_goals ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?,
            "title": row.get::<_, String>(1)?,
            "metric_type": row.get::<_, String>(2)?,
            "target_value": row.get::<_, i64>(3)?,
            "current_value": row.get::<_, i64>(4)?,
            "period": row.get::<_, String>(5)?,
            "start_date": row.get::<_, Option<String>>(6)?,
            "end_date": row.get::<_, Option<String>>(7)?,
            "created_at": row.get::<_, i64>(8)?,
        }))
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn delete_goal(id: i64) -> Result<()> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();
    conn.execute("DELETE FROM user_goals WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn update_goal_progress(id: i64, current_value: i64) -> Result<()> {
    let conn = get_conn()?;
    let conn = conn.lock().unwrap();
    conn.execute(
        "UPDATE user_goals SET current_value = ?1 WHERE id = ?2",
        params![current_value, id],
    )?;
    Ok(())
}
