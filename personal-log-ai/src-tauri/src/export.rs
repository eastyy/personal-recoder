use anyhow::Result;
use rusqlite::params;
use serde_json::{json, Value};
use std::path::PathBuf;

/// Export all data as JSON.
///
/// Queries every major table (raw_events, input_sessions, focus_sessions,
/// focus_daily, todo_items, analysis_results) and returns them as a single
/// JSON object with arrays.
pub fn export_all_json() -> Result<String> {
    let conn = crate::db::get_conn()?;
    let conn = conn.lock().unwrap();

    // -- raw_events --
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, event_type, app_bundle_id, app_name, window_title, content, is_sensitive, session_id, metadata FROM raw_events ORDER BY timestamp ASC",
    )?;
    let raw_events: Vec<Value> = stmt
        .query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "timestamp": row.get::<_, i64>(1)?,
                "event_type": row.get::<_, String>(2)?,
                "app_bundle_id": row.get::<_, Option<String>>(3)?,
                "app_name": row.get::<_, Option<String>>(4)?,
                "window_title": row.get::<_, Option<String>>(5)?,
                "content": row.get::<_, Option<String>>(6)?,
                "is_sensitive": row.get::<_, i64>(7)?,
                "session_id": row.get::<_, Option<String>>(8)?,
                "metadata": row.get::<_, Option<String>>(9)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);

    // -- input_sessions --
    let mut stmt = conn.prepare(
        "SELECT id, app_bundle_id, app_name, start_time, end_time, char_count, text_preview, pause_count, context_tag, ai_analyzed FROM input_sessions ORDER BY start_time ASC",
    )?;
    let input_sessions: Vec<Value> = stmt
        .query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, String>(0)?,
                "app_bundle_id": row.get::<_, String>(1)?,
                "app_name": row.get::<_, Option<String>>(2)?,
                "start_time": row.get::<_, i64>(3)?,
                "end_time": row.get::<_, Option<i64>>(4)?,
                "char_count": row.get::<_, i64>(5)?,
                "text_preview": row.get::<_, Option<String>>(6)?,
                "pause_count": row.get::<_, i64>(7)?,
                "context_tag": row.get::<_, Option<String>>(8)?,
                "ai_analyzed": row.get::<_, i64>(9)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);

    // -- focus_sessions --
    let mut stmt = conn.prepare(
        "SELECT id, start_time, end_time, duration_sec, target_type, target_id, target_name FROM focus_sessions ORDER BY start_time ASC",
    )?;
    let focus_sessions: Vec<Value> = stmt
        .query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "start_time": row.get::<_, i64>(1)?,
                "end_time": row.get::<_, i64>(2)?,
                "duration_sec": row.get::<_, i64>(3)?,
                "target_type": row.get::<_, String>(4)?,
                "target_id": row.get::<_, String>(5)?,
                "target_name": row.get::<_, String>(6)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);

    // -- focus_daily --
    let mut stmt = conn.prepare(
        "SELECT date, target_type, target_id, target_name, duration_sec, session_count FROM focus_daily ORDER BY date ASC",
    )?;
    let focus_daily: Vec<Value> = stmt
        .query_map([], |row| {
            Ok(json!({
                "date": row.get::<_, String>(0)?,
                "target_type": row.get::<_, String>(1)?,
                "target_id": row.get::<_, String>(2)?,
                "target_name": row.get::<_, String>(3)?,
                "duration_sec": row.get::<_, i64>(4)?,
                "session_count": row.get::<_, i64>(5)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);

    // -- todo_items --
    let mut stmt = conn.prepare(
        "SELECT id, text, source_session, extracted_at, due_date, status, completed_at FROM todo_items ORDER BY extracted_at ASC",
    )?;
    let todo_items: Vec<Value> = stmt
        .query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "text": row.get::<_, String>(1)?,
                "source_session": row.get::<_, Option<String>>(2)?,
                "extracted_at": row.get::<_, i64>(3)?,
                "due_date": row.get::<_, Option<String>>(4)?,
                "status": row.get::<_, String>(5)?,
                "completed_at": row.get::<_, Option<i64>>(6)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);

    // -- analysis_results --
    let mut stmt = conn.prepare(
        "SELECT id, analysis_type, time_range_start, time_range_end, prompt_tokens, completion_tokens, result_text, result_json, created_at FROM analysis_results ORDER BY created_at ASC",
    )?;
    let analysis_results: Vec<Value> = stmt
        .query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, String>(0)?,
                "analysis_type": row.get::<_, String>(1)?,
                "time_range_start": row.get::<_, Option<i64>>(2)?,
                "time_range_end": row.get::<_, Option<i64>>(3)?,
                "prompt_tokens": row.get::<_, Option<i64>>(4)?,
                "completion_tokens": row.get::<_, Option<i64>>(5)?,
                "result_text": row.get::<_, Option<String>>(6)?,
                "result_json": row.get::<_, Option<String>>(7)?,
                "created_at": row.get::<_, i64>(8)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);

    let export = json!({
        "exported_at": chrono::Utc::now().timestamp(),
        "raw_events": raw_events,
        "input_sessions": input_sessions,
        "focus_sessions": focus_sessions,
        "focus_daily": focus_daily,
        "todo_items": todo_items,
        "analysis_results": analysis_results,
    });

    Ok(serde_json::to_string_pretty(&export)?)
}

/// Export raw events in a timestamp range as CSV.
///
/// Columns: timestamp, event_type, app_name, content, session_id
pub fn export_events_csv(start_ts: i64, end_ts: i64) -> Result<String> {
    let conn = crate::db::get_conn()?;
    let conn = conn.lock().unwrap();

    let mut stmt = conn.prepare(
        "SELECT timestamp, event_type, app_name, content, session_id FROM raw_events WHERE timestamp >= ?1 AND timestamp <= ?2 ORDER BY timestamp ASC",
    )?;

    let rows = stmt.query_map(params![start_ts, end_ts], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<String>>(4)?,
        ))
    })?;

    let mut csv = String::from("timestamp,event_type,app_name,content,session_id\n");
    for row in rows {
        let (timestamp, event_type, app_name, content, session_id) = row?;
        csv.push_str(&format!(
            "{},{},{},{},{}\n",
            timestamp,
            csv_escape(&event_type),
            csv_escape(&app_name.unwrap_or_default()),
            csv_escape(&content.unwrap_or_default()),
            csv_escape(&session_id.unwrap_or_default()),
        ));
    }

    Ok(csv)
}

/// Escape a field for CSV output (RFC 4180 style: quote if it contains
/// comma, quote, or newline; double internal quotes).
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}

/// Delete data older than `retention_days`.
///
/// Removes rows from `raw_events`, `input_sessions`, and `focus_sessions`
/// whose timestamp / start_time falls before the cutoff.  Analysis results
/// and app_config are preserved.
///
/// Returns `(events_deleted, sessions_deleted)`.
pub fn cleanup_old_data(retention_days: i32) -> Result<(i64, i64)> {
    let cutoff = chrono::Utc::now().timestamp() - (retention_days as i64 * 86400);

    let conn = crate::db::get_conn()?;
    let conn = conn.lock().unwrap();

    let events_deleted = conn.execute(
        "DELETE FROM raw_events WHERE timestamp < ?1",
        params![cutoff],
    )? as i64;

    let sessions_deleted = conn.execute(
        "DELETE FROM input_sessions WHERE start_time < ?1",
        params![cutoff],
    )? as i64;

    // Also clean up old focus_sessions
    let _ = conn.execute(
        "DELETE FROM focus_sessions WHERE start_time < ?1",
        params![cutoff],
    );

    eprintln!(
        "[cleanup] cutoff={} events_deleted={} sessions_deleted={}",
        cutoff, events_deleted, sessions_deleted
    );

    Ok((events_deleted, sessions_deleted))
}

/// Get database statistics as a JSON object.
///
/// Returns counts for total_events, total_sessions, total_todos,
/// total_reports, and db_size_bytes.
pub fn get_db_stats() -> Result<Value> {
    let conn = crate::db::get_conn()?;
    let conn = conn.lock().unwrap();

    let total_events: i64 = conn.query_row(
        "SELECT COUNT(*) FROM raw_events",
        [],
        |row| row.get(0),
    )?;

    let total_sessions: i64 = conn.query_row(
        "SELECT COUNT(*) FROM input_sessions",
        [],
        |row| row.get(0),
    )?;

    let total_todos: i64 = conn.query_row(
        "SELECT COUNT(*) FROM todo_items",
        [],
        |row| row.get(0),
    )?;

    let total_reports: i64 = conn.query_row(
        "SELECT COUNT(*) FROM analysis_results",
        [],
        |row| row.get(0),
    )?;

    let db_size_bytes = get_db_size()?;

    Ok(json!({
        "total_events": total_events,
        "total_sessions": total_sessions,
        "total_todos": total_todos,
        "total_reports": total_reports,
        "db_size_bytes": db_size_bytes,
    }))
}

/// Get the SQLite database file size in bytes.
fn get_db_size() -> Result<i64> {
    let db_path = get_db_path()?;
    if db_path.exists() {
        let metadata = std::fs::metadata(&db_path)?;
        Ok(metadata.len() as i64)
    } else {
        Ok(0)
    }
}

/// Resolve the database file path, mirroring `db::get_db_path()`.
fn get_db_path() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
        Ok(home.join("Library/Application Support/PersonalLogAI/data.db"))
    }
    #[cfg(target_os = "linux")]
    {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find data directory"))?;
        Ok(data_dir.join("PersonalLogAI/data.db"))
    }
    #[cfg(target_os = "windows")]
    {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find data directory"))?;
        Ok(data_dir.join("PersonalLogAI/data.db"))
    }
}
