use anyhow::Result;
use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, Timelike, Weekday};
use serde_json::Value;
use std::sync::Mutex;
use std::time::Duration;
use tauri::AppHandle;

/// Track the last time each scheduled task ran so we don't fire twice within
/// the same 60-second polling window.
static LAST_DAILY_RUN: Mutex<Option<NaiveDate>> = Mutex::new(None);
static LAST_WEEKLY_RUN: Mutex<Option<NaiveDate>> = Mutex::new(None);
static LAST_HOURLY_RUN: Mutex<Option<NaiveDateTime>> = Mutex::new(None);
static LAST_CLEANUP_DATE: Mutex<Option<NaiveDate>> = Mutex::new(None);

/// Start the background scheduler thread.
///
/// The thread loops every 60 seconds and checks whether any scheduled task
/// is due.  Each task is wrapped in error handling so a failure in one
/// analysis call never crashes the scheduler.
pub fn start_scheduler(_app: AppHandle) -> Result<()> {
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("[scheduler] Failed to create tokio runtime: {}", e);
                return;
            }
        };

        eprintln!("[scheduler] Started — polling every 60s");

        loop {
            std::thread::sleep(Duration::from_secs(60));

            let now = Local::now();
            let today = now.date_naive();
            let hour = now.hour();
            let minute = now.minute();
            let weekday = now.weekday();

            eprintln!(
                "[scheduler] tick — {} {:02}:{:02}:{:02} ({})",
                today, hour, minute, now.second(), weekday
            );

            // ---- Daily at 03:00 — productivity, topic, writing ----
            if hour == 3 && minute == 0 {
                let mut guard = LAST_DAILY_RUN.lock().unwrap();
                if *guard != Some(today) {
                    *guard = Some(today);
                    drop(guard);

                    eprintln!("[scheduler] Daily analysis triggered");
                    for analysis_type in &["productivity", "topic", "writing"] {
                        run_analysis_safely(&rt, analysis_type);
                    }
                }
            }

            // ---- Weekly on Sunday at 03:30 — weekly ----
            if weekday == Weekday::Sun && hour == 3 && minute == 30 {
                let mut guard = LAST_WEEKLY_RUN.lock().unwrap();
                if *guard != Some(today) {
                    *guard = Some(today);
                    drop(guard);

                    eprintln!("[scheduler] Weekly analysis triggered");
                    run_analysis_safely(&rt, "weekly");
                }
            }

            // ---- Hourly — todo ----
            if minute == 0 {
                let this_hour = now.with_minute(0).and_then(|d| d.with_second(0)).unwrap();
                let this_hour_n = this_hour.naive_local();
                let mut guard = LAST_HOURLY_RUN.lock().unwrap();
                if *guard != Some(this_hour_n) {
                    *guard = Some(this_hour_n);
                    drop(guard);

                    eprintln!("[scheduler] Hourly todo analysis triggered");
                    run_analysis_safely(&rt, "todo");
                }
            }

            // ---- Daily data cleanup (once per day, right after midnight) ----
            if hour == 0 && minute == 5 {
                let mut guard = LAST_CLEANUP_DATE.lock().unwrap();
                if *guard != Some(today) {
                    *guard = Some(today);
                    drop(guard);

                    run_cleanup_safely();
                }
            }
        }
    });

    Ok(())
}

/// Call `trigger_analysis` inside the scheduler's tokio runtime, catching
/// any errors so the scheduler thread never panics.
fn run_analysis_safely(rt: &tokio::runtime::Runtime, analysis_type: &str) {
    let result: Result<Value> = rt.block_on(crate::ai::trigger_analysis(analysis_type));

    match result {
        Ok(val) => {
            let id = val.get("id").and_then(|v| v.as_str()).unwrap_or("?");
            eprintln!(
                "[scheduler] analysis '{}' completed (id={})",
                analysis_type, id
            );
        }
        Err(e) => {
            eprintln!(
                "[scheduler] analysis '{}' failed: {}",
                analysis_type, e
            );
        }
    }
}

/// Read retention_days from config and invoke cleanup, catching errors.
fn run_cleanup_safely() {
    let retention_days: i32 = match crate::config::get_config() {
        Ok(config) => {
            config
                .get("data_retention_days")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(90)
        }
        Err(e) => {
            eprintln!("[scheduler] Failed to read config for cleanup: {}", e);
            90 // sensible default
        }
    };

    eprintln!(
        "[scheduler] Running data cleanup (retention_days={})",
        retention_days
    );

    match crate::export::cleanup_old_data(retention_days) {
        Ok((events, sessions)) => {
            eprintln!(
                "[scheduler] Cleanup done — events={}, sessions={}",
                events, sessions
            );
        }
        Err(e) => {
            eprintln!("[scheduler] Cleanup failed: {}", e);
        }
    }
}
