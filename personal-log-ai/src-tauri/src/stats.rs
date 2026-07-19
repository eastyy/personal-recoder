use anyhow::Result;
use rusqlite::params;
use serde_json::{json, Value};

// ============================================================================
// Existing functions (kept as-is)
// ============================================================================

pub fn calculate_typing_speed(char_count: i64, duration_seconds: i64) -> f64 {
    if duration_seconds <= 0 {
        return 0.0;
    }
    let minutes = duration_seconds as f64 / 60.0;
    char_count as f64 / minutes
}

pub fn calculate_wpm(char_count: i64, duration_seconds: i64) -> f64 {
    // 中文：每字算一个词；英文：每5个字符算一个词
    let words = char_count as f64 / 5.0;
    let minutes = duration_seconds as f64 / 60.0;
    if minutes <= 0.0 {
        return 0.0;
    }
    words / minutes
}

pub fn calculate_backspace_rate(backspace_count: i64, total_chars: i64) -> f64 {
    if total_chars <= 0 {
        return 0.0;
    }
    (backspace_count as f64 / total_chars as f64) * 100.0
}

pub fn calculate_focus_score(pause_count: i64, duration_seconds: i64) -> f64 {
    if duration_seconds <= 0 {
        return 0.0;
    }
    let pauses_per_minute = pause_count as f64 / (duration_seconds as f64 / 60.0);
    // 停顿越少，专注度越高
    let score = 100.0 - (pauses_per_minute * 5.0);
    score.max(0.0).min(100.0)
}

#[allow(dead_code)]
pub fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

// ============================================================================
// New functions
// ============================================================================

/// Get comprehensive typing statistics for a time range
pub fn get_typing_stats(start_ts: i64, end_ts: i64) -> Result<Value> {
    let conn = crate::db::get_conn()?;
    let conn = conn.lock().unwrap();

    // Count keydown events in range
    let total_keystrokes: i64 = conn.query_row(
        "SELECT COUNT(*) FROM raw_events WHERE event_type = 'keydown' AND timestamp >= ?1 AND timestamp <= ?2",
        params![start_ts, end_ts],
        |row| row.get(0),
    )?;

    // Sum char_count from input_sessions in range
    let total_chars: i64 = conn.query_row(
        "SELECT COALESCE(SUM(char_count), 0) FROM input_sessions WHERE start_time >= ?1 AND start_time <= ?2",
        params![start_ts, end_ts],
        |row| row.get(0),
    )?;

    // Count backspace events
    let backspace_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM raw_events WHERE event_type = 'keydown' AND content = 'Backspace' AND timestamp >= ?1 AND timestamp <= ?2",
        params![start_ts, end_ts],
        |row| row.get(0),
    )?;

    let duration_minutes = (end_ts - start_ts) as f64 / 60.0;

    let avg_wpm = if duration_minutes > 0.0 {
        calculate_wpm(total_chars, (end_ts - start_ts).max(1))
    } else {
        0.0
    };

    let avg_cpm = if duration_minutes > 0.0 {
        calculate_typing_speed(total_chars, (end_ts - start_ts).max(1))
    } else {
        0.0
    };

    let backspace_rate = calculate_backspace_rate(backspace_count, total_chars);

    Ok(json!({
        "total_chars": total_chars,
        "total_keystrokes": total_keystrokes,
        "backspace_count": backspace_count,
        "avg_wpm": (avg_wpm * 100.0).round() / 100.0,
        "avg_cpm": (avg_cpm * 100.0).round() / 100.0,
        "backspace_rate": (backspace_rate * 100.0).round() / 100.0,
        "duration_seconds": end_ts - start_ts
    }))
}

/// Get daily typing rhythm (per-hour breakdown)
pub fn get_typing_rhythm(date: &str) -> Result<Value> {
    let conn = crate::db::get_conn()?;
    let conn = conn.lock().unwrap();

    let mut rhythm: Vec<Value> = Vec::with_capacity(24);

    for hour in 0..24i64 {
        // Count keydown events for this hour
        let key_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM raw_events WHERE event_type = 'keydown' AND date(datetime(timestamp, 'unixepoch')) = ?1 AND strftime('%H', datetime(timestamp, 'unixepoch')) = ?2",
            params![date, format!("{:02}", hour)],
            |row| row.get(0),
        )?;

        // Sum char_count from input_sessions for this hour
        let char_count: i64 = conn.query_row(
            "SELECT COALESCE(SUM(char_count), 0) FROM input_sessions WHERE date(datetime(start_time, 'unixepoch')) = ?1 AND CAST(strftime('%H', datetime(start_time, 'unixepoch')) AS INTEGER) = ?2",
            params![date, hour],
            |row| row.get(0),
        )?;

        rhythm.push(json!({
            "hour": hour,
            "char_count": char_count,
            "key_count": key_count
        }));
    }

    Ok(Value::Array(rhythm))
}

/// Calculate productivity score (0-100) based on focus sessions and typing
pub fn calculate_productivity_score(
    focus_minutes: i64,
    typing_minutes: i64,
    switching_count: i64,
) -> f64 {
    // Focus component: up to 50 points (120 min focus = full 50)
    let focus_component = (focus_minutes as f64 / 120.0 * 50.0).min(50.0);

    // Typing component: up to 35 points (60 min typing = full 35)
    let typing_component = (typing_minutes as f64 / 60.0 * 35.0).min(35.0);

    // Switching penalty: each switch reduces score, up to 15 points deducted
    // 15+ switches = max penalty
    let switching_penalty = (switching_count as f64 * 1.0).min(15.0);

    let score = focus_component + typing_component - switching_penalty;

    score.max(0.0).min(100.0)
}

/// Get app switching statistics for a date
pub fn get_switching_stats(date: &str) -> Result<Value> {
    let conn = crate::db::get_conn()?;
    let conn = conn.lock().unwrap();

    // Count app_focus events for the day
    let total_switches: i64 = conn.query_row(
        "SELECT COUNT(*) FROM raw_events WHERE event_type = 'app_focus' AND date(datetime(timestamp, 'unixepoch')) = ?1",
        params![date],
        |row| row.get(0),
    )?;

    // Count unique apps
    let unique_apps: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT app_bundle_id) FROM raw_events WHERE event_type = 'app_focus' AND date(datetime(timestamp, 'unixepoch')) = ?1 AND app_bundle_id IS NOT NULL",
        params![date],
        |row| row.get(0),
    )?;

    // Calculate average session duration from focus_sessions for the day
    let avg_session: f64 = conn.query_row(
        "SELECT COALESCE(AVG(duration_sec), 0) FROM focus_sessions WHERE date(datetime(start_time, 'unixepoch')) = ?1",
        params![date],
        |row| row.get(0),
    )?;

    let avg_session_minutes = (avg_session / 60.0 * 100.0).round() / 100.0;

    Ok(json!({
        "date": date,
        "total_switches": total_switches,
        "unique_apps": unique_apps,
        "avg_session_minutes": avg_session_minutes
    }))
}

/// Format seconds to human readable (enhanced)
/// Like '2h 15m' or '45m 30s' or '30s'
pub fn format_duration_human(seconds: i64) -> String {
    if seconds <= 0 {
        return "0s".to_string();
    }

    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        if minutes > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}h", hours)
        }
    } else if minutes > 0 {
        if secs > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}m", minutes)
        }
    } else {
        format!("{}s", secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_human() {
        assert_eq!(format_duration_human(0), "0s");
        assert_eq!(format_duration_human(30), "30s");
        assert_eq!(format_duration_human(90), "1m 30s");
        assert_eq!(format_duration_human(60), "1m");
        assert_eq!(format_duration_human(3600), "1h");
        assert_eq!(format_duration_human(8100), "2h 15m");
    }

    #[test]
    fn test_calculate_productivity_score() {
        // No activity
        let score = calculate_productivity_score(0, 0, 0);
        assert_eq!(score, 0.0);

        // Full focus + typing, no switching
        let score = calculate_productivity_score(120, 60, 0);
        assert_eq!(score, 85.0);

        // High switching penalty
        let score = calculate_productivity_score(120, 60, 20);
        assert_eq!(score, 70.0);

        // Score capped at 100
        let score = calculate_productivity_score(200, 100, 0);
        assert_eq!(score, 85.0); // 50 + 35 = 85
    }

    #[test]
    fn test_wpm_and_speed() {
        assert_eq!(calculate_wpm(300, 60), 60.0);
        assert_eq!(calculate_wpm(0, 60), 0.0);
        assert_eq!(calculate_wpm(100, 0), 0.0);
        assert_eq!(calculate_typing_speed(300, 60), 300.0);
    }

    #[test]
    fn test_backspace_rate() {
        assert_eq!(calculate_backspace_rate(10, 100), 10.0);
        assert_eq!(calculate_backspace_rate(0, 0), 0.0);
    }

    #[test]
    fn test_focus_score() {
        let score = calculate_focus_score(0, 60);
        assert_eq!(score, 100.0);

        // 5 pauses in 1 minute => 100 - (5 * 5) = 75
        let score = calculate_focus_score(5, 60);
        assert_eq!(score, 75.0);

        // Very high pauses => 0
        let score = calculate_focus_score(100, 60);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m");
    }
}
