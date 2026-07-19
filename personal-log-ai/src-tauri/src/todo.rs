use chrono::{Datelike, Duration, Local, NaiveDate};
use regex::Regex;
use std::collections::HashSet;

lazy_static::lazy_static! {
    static ref TODO_PATTERNS: Vec<Regex> = vec![
        // Original patterns
        Regex::new(r"^(TODO|todo|待办|TODO:)\s*[:：]\s*(.+)$").unwrap(),
        Regex::new(r"^(记得|记得要)\s*(.+)$").unwrap(),
        // Need/must patterns
        Regex::new(r"^(需要|必须|应该|得|要)\s*(.+)$").unwrap(),
        // Planning patterns
        Regex::new(r"^(准备|计划|打算|想要|想)\s*(.+)$").unwrap(),
        // Imperative patterns
        Regex::new(r"^(提交|发送|回复|检查|确认|完成|处理|解决|安排|预约|订购|购买|联系|通知|提醒|整理)\s*(.+)$").unwrap(),
        // Reminder patterns
        Regex::new(r"^(别忘了|不要忘记|不要忘了|记着|记住)\s*(.+)$").unwrap(),
        // Time-based patterns
        Regex::new(r"^(这周|本周|下周|这月|下月|今天|明天|后天)\s*(要|需要|得|去|做)\s*(.+)$").unwrap(),
        // Original need/must with action verb at end
        Regex::new(r"^(需要|必须|应该|得)\s*(.+)(?:完成|做|处理|解决)$").unwrap(),
        // Time + 要 patterns (from original)
        Regex::new(r"^(明天要|今天要|下周要|这周要)\s*(.+)$").unwrap(),
    ];

    // Due date patterns
    static ref DUE_DATE_TODAY: Regex = Regex::new(r"今天").unwrap();
    static ref DUE_DATE_TOMORROW: Regex = Regex::new(r"明天").unwrap();
    static ref DUE_DATE_DAY_AFTER: Regex = Regex::new(r"后天").unwrap();
    static ref DUE_DATE_THIS_WEEK: Regex = Regex::new(r"(这周|本周)(周[一二三四五六日天])?").unwrap();
    static ref DUE_DATE_NEXT_WEEK: Regex = Regex::new(r"下周(周)?([一二三四五六日天])?").unwrap();
    static ref DUE_DATE_THIS_MONTH: Regex = Regex::new(r"这月").unwrap();
    static ref DUE_DATE_NEXT_MONTH: Regex = Regex::new(r"下月").unwrap();
    static ref DUE_DATE_MONTH_DAY: Regex = Regex::new(r"(\d{1,2})月(\d{1,2})[日号]").unwrap();
    static ref DUE_DATE_SLASH: Regex = Regex::new(r"(\d{1,2})/(\d{1,2})").unwrap();
    static ref DUE_DATE_NEXT_WEEK_DAY: Regex = Regex::new(r"下周(一|二|三|四|五|六|日|天)").unwrap();
    static ref DUE_DATE_THIS_WEEK_DAY: Regex = Regex::new(r"(这周|本周)(一|二|三|四|五|六|日|天)").unwrap();
}

/// Extract a single TODO item from a text line
pub fn extract_todo_from_text(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.len() < 3 || trimmed.len() > 200 {
        return None;
    }

    for pattern in TODO_PATTERNS.iter() {
        if let Some(caps) = pattern.captures(trimmed) {
            // For time-based patterns, the actual TODO is in capture group 3
            // For other patterns, it's in capture group 2
            if let Some(matched) = caps.get(3) {
                return Some(matched.as_str().trim().to_string());
            }
            if let Some(matched) = caps.get(2) {
                return Some(matched.as_str().trim().to_string());
            }
        }
    }

    // 祈使句模式检测
    if is_imperative_sentence(trimmed) {
        return Some(trimmed.to_string());
    }

    None
}

/// Extract multiple TODO items from a longer text (session buffer)
pub fn extract_todos_from_session(text: &str) -> Vec<String> {
    let mut todos: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for line in text.lines() {
        if let Some(todo) = extract_todo_from_text(line) {
            if seen.insert(todo.clone()) {
                todos.push(todo);
            }
        }
    }

    todos
}

/// Extract due date from TODO text
/// Checks for: 明天, 后天, 下周X, 这周X, 今天, X月X日, X/X
/// Returns a date string like '2026-07-12' or 'tomorrow', 'day_after', 'today' etc.
pub fn extract_due_date(text: &str) -> Option<String> {
    let now = Local::now();

    // 今天
    if DUE_DATE_TODAY.is_match(text) {
        return Some(now.format("%Y-%m-%d").to_string());
    }

    // 明天
    if DUE_DATE_TOMORROW.is_match(text) {
        let tomorrow = now + Duration::days(1);
        return Some(tomorrow.format("%Y-%m-%d").to_string());
    }

    // 后天
    if DUE_DATE_DAY_AFTER.is_match(text) {
        let day_after = now + Duration::days(2);
        return Some(day_after.format("%Y-%m-%d").to_string());
    }

    // 下周一/下周二/.../下周日
    if let Some(caps) = DUE_DATE_NEXT_WEEK_DAY.captures(text) {
        let day_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let target_weekday = chinese_weekday_to_num(day_str);
        if let Some(target) = target_weekday {
            let date = next_weekday_date(&now, target, 1);
            return Some(date.format("%Y-%m-%d").to_string());
        }
    }

    // 这周一/这周二/.../这周日
    if let Some(caps) = DUE_DATE_THIS_WEEK_DAY.captures(text) {
        let day_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let target_weekday = chinese_weekday_to_num(day_str);
        if let Some(target) = target_weekday {
            let date = this_weekday_date(&now, target);
            return Some(date.format("%Y-%m-%d").to_string());
        }
    }

    // 下周 (without specific day) -> next Monday
    if DUE_DATE_NEXT_WEEK.is_match(text) && !DUE_DATE_NEXT_WEEK_DAY.is_match(text) {
        let date = next_weekday_date(&now, chrono::Weekday::Mon, 1);
        return Some(date.format("%Y-%m-%d").to_string());
    }

    // 这周/本周 (without specific day) -> this Friday (end of week)
    if DUE_DATE_THIS_WEEK.is_match(text) && !DUE_DATE_THIS_WEEK_DAY.is_match(text) {
        let date = this_weekday_date(&now, chrono::Weekday::Fri);
        return Some(date.format("%Y-%m-%d").to_string());
    }

    // 这月
    if DUE_DATE_THIS_MONTH.is_match(text) {
        return Some(now.format("%Y-%m").to_string() + "-28");
    }

    // 下月
    if DUE_DATE_NEXT_MONTH.is_match(text) {
        let next_month = if now.month() == 12 {
            NaiveDate::from_ymd_opt(now.year() + 1, 1, 28)
        } else {
            NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 28)
        };
        if let Some(nm) = next_month {
            return Some(nm.format("%Y-%m-%d").to_string());
        }
    }

    // X月X日 format (e.g., 7月15日)
    if let Some(caps) = DUE_DATE_MONTH_DAY.captures(text) {
        let month: u32 = caps.get(1).map(|m| m.as_str()).unwrap_or("0").parse().unwrap_or(0);
        let day: u32 = caps.get(2).map(|m| m.as_str()).unwrap_or("0").parse().unwrap_or(0);
        let year = now.year();
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            // If the date has passed this year, use next year
            if date < now.date_naive() {
                if let Some(date) = NaiveDate::from_ymd_opt(year + 1, month, day) {
                    return Some(date.format("%Y-%m-%d").to_string());
                }
            }
            return Some(date.format("%Y-%m-%d").to_string());
        }
    }

    // X/X format (e.g., 7/15)
    if let Some(caps) = DUE_DATE_SLASH.captures(text) {
        let month: u32 = caps.get(1).map(|m| m.as_str()).unwrap_or("0").parse().unwrap_or(0);
        let day: u32 = caps.get(2).map(|m| m.as_str()).unwrap_or("0").parse().unwrap_or(0);
        let year = now.year();
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            if date < now.date_naive() {
                if let Some(date) = NaiveDate::from_ymd_opt(year + 1, month, day) {
                    return Some(date.format("%Y-%m-%d").to_string());
                }
            }
            return Some(date.format("%Y-%m-%d").to_string());
        }
    }

    None
}

/// Convert Chinese weekday string to chrono Weekday
fn chinese_weekday_to_num(day_str: &str) -> Option<chrono::Weekday> {
    match day_str {
        "一" => Some(chrono::Weekday::Mon),
        "二" => Some(chrono::Weekday::Tue),
        "三" => Some(chrono::Weekday::Wed),
        "四" => Some(chrono::Weekday::Thu),
        "五" => Some(chrono::Weekday::Fri),
        "六" => Some(chrono::Weekday::Sat),
        "日" | "天" => Some(chrono::Weekday::Sun),
        _ => None,
    }
}

/// Calculate the date for the next occurrence of a given weekday
/// weeks_ahead: 0 = this week, 1 = next week
fn next_weekday_date(
    now: &chrono::DateTime<Local>,
    target: chrono::Weekday,
    weeks_ahead: i64,
) -> NaiveDate {
    let current = now.weekday();
    let current_num = current.num_days_from_monday() as i64;
    let target_num = target.num_days_from_monday() as i64;

    let mut days_ahead = target_num - current_num;
    if days_ahead <= 0 {
        days_ahead += 7;
    }
    days_ahead += weeks_ahead * 7;

    now.date_naive() + Duration::days(days_ahead)
}

/// Calculate the date for a given weekday in the current week
fn this_weekday_date(now: &chrono::DateTime<Local>, target: chrono::Weekday) -> NaiveDate {
    let current = now.weekday();
    let current_num = current.num_days_from_monday() as i64;
    let target_num = target.num_days_from_monday() as i64;

    let days_diff = target_num - current_num;

    now.date_naive() + Duration::days(days_diff)
}

/// Check if text starts with an imperative verb phrase
fn is_imperative_sentence(text: &str) -> bool {
    let imperative_starts = [
        "请", "去", "把", "将", "给", "帮", "联系", "通知", "提醒",
        "准备", "整理", "提交", "发送", "回复", "检查", "确认",
        "完成", "处理", "解决", "安排", "预约", "订购", "购买",
    ];

    for start in &imperative_starts {
        if text.starts_with(start) && text.len() > start.len() + 3 {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_todo_basic() {
        assert_eq!(
            extract_todo_from_text("TODO: 买牛奶"),
            Some("买牛奶".to_string())
        );
        assert_eq!(
            extract_todo_from_text("待办：写报告"),
            Some("写报告".to_string())
        );
    }

    #[test]
    fn test_extract_todo_need_patterns() {
        assert_eq!(
            extract_todo_from_text("需要完成项目文档"),
            Some("完成项目文档".to_string())
        );
        assert_eq!(
            extract_todo_from_text("必须提交申请表"),
            Some("提交申请表".to_string())
        );
        assert_eq!(
            extract_todo_from_text("应该回复邮件"),
            Some("回复邮件".to_string())
        );
        assert_eq!(
            extract_todo_from_text("得去超市"),
            Some("去超市".to_string())
        );
        assert_eq!(
            extract_todo_from_text("要写周报"),
            Some("写周报".to_string())
        );
    }

    #[test]
    fn test_extract_todo_planning_patterns() {
        assert_eq!(
            extract_todo_from_text("准备面试"),
            Some("面试".to_string())
        );
        assert_eq!(
            extract_todo_from_text("计划去旅行"),
            Some("去旅行".to_string())
        );
        assert_eq!(
            extract_todo_from_text("打算学Rust"),
            Some("学Rust".to_string())
        );
        assert_eq!(
            extract_todo_from_text("想买新书"),
            Some("买新书".to_string())
        );
    }

    #[test]
    fn test_extract_todo_imperative_patterns() {
        assert_eq!(
            extract_todo_from_text("提交代码"),
            Some("代码".to_string())
        );
        assert_eq!(
            extract_todo_from_text("发送邮件给客户"),
            Some("邮件给客户".to_string())
        );
        assert_eq!(
            extract_todo_from_text("检查部署状态"),
            Some("部署状态".to_string())
        );
        assert_eq!(
            extract_todo_from_text("购买办公用品"),
            Some("办公用品".to_string())
        );
        assert_eq!(
            extract_todo_from_text("联系供应商"),
            Some("供应商".to_string())
        );
    }

    #[test]
    fn test_extract_todo_reminder_patterns() {
        assert_eq!(
            extract_todo_from_text("别忘了开会"),
            Some("开会".to_string())
        );
        assert_eq!(
            extract_todo_from_text("不要忘记交报告"),
            Some("交报告".to_string())
        );
        assert_eq!(
            extract_todo_from_text("记着买菜"),
            Some("买菜".to_string())
        );
        assert_eq!(
            extract_todo_from_text("记住密码"),
            Some("密码".to_string())
        );
    }

    #[test]
    fn test_extract_todo_time_based_patterns() {
        assert_eq!(
            extract_todo_from_text("明天需要交报告"),
            Some("交报告".to_string())
        );
        assert_eq!(
            extract_todo_from_text("今天要完成文档"),
            Some("完成文档".to_string())
        );
        assert_eq!(
            extract_todo_from_text("下周得去开会"),
            Some("去开会".to_string())
        );
        assert_eq!(
            extract_todo_from_text("这周要提交总结"),
            Some("提交总结".to_string())
        );
    }

    #[test]
    fn test_extract_todo_too_short() {
        assert_eq!(extract_todo_from_text("好"), None);
        assert_eq!(extract_todo_from_text("ab"), None);
    }

    #[test]
    fn test_extract_todo_too_long() {
        let long = "a".repeat(201);
        assert_eq!(extract_todo_from_text(&long), None);
    }

    #[test]
    fn test_extract_todos_from_session() {
        let session = "今天写代码\nTODO: 测试功能\n随便一行\n需要修复bug\n别忘了买咖啡";
        let todos = extract_todos_from_session(session);
        assert_eq!(todos.len(), 3);
        assert!(todos.contains(&"测试功能".to_string()));
        assert!(todos.contains(&"修复bug".to_string()));
        assert!(todos.contains(&"买咖啡".to_string()));
    }

    #[test]
    fn test_extract_todos_from_session_dedup() {
        let session = "TODO: 买牛奶\nTODO: 买牛奶\n需要买牛奶";
        let todos = extract_todos_from_session(session);
        // "买牛奶" should appear only once even though it's matched by both patterns
        // Note: the need pattern returns "买牛奶" and the TODO pattern also returns "买牛奶"
        assert!(todos.contains(&"买牛奶".to_string()));
        let milk_count = todos.iter().filter(|t| t == &"买牛奶").count();
        assert_eq!(milk_count, 1);
    }

    #[test]
    fn test_extract_todos_empty() {
        let todos = extract_todos_from_session("");
        assert!(todos.is_empty());

        let todos = extract_todos_from_session("随便写的\n不是待办\n只是记录");
        assert!(todos.is_empty());
    }

    #[test]
    fn test_extract_due_date_relative() {
        // 今天
        let due = extract_due_date("今天要开会");
        assert!(due.is_some());

        // 明天
        let due = extract_due_date("明天需要交报告");
        assert!(due.is_some());

        // 后天
        let due = extract_due_date("后天去检查");
        assert!(due.is_some());
    }

    #[test]
    fn test_extract_due_date_weekday() {
        let due = extract_due_date("下周一开会");
        assert!(due.is_some());

        let due = extract_due_date("这周五提交");
        assert!(due.is_some());

        let due = extract_due_date("下周去出差");
        assert!(due.is_some());
    }

    #[test]
    fn test_extract_due_date_month_day() {
        let due = extract_due_date("7月15日截止");
        assert!(due.is_some());

        let due = extract_due_date("12/25买礼物");
        assert!(due.is_some());
    }

    #[test]
    fn test_extract_due_date_none() {
        let due = extract_due_date("随便一句话");
        assert!(due.is_none());
    }

    #[test]
    fn test_is_imperative() {
        assert!(is_imperative_sentence("请帮我买个东西"));
        assert!(!is_imperative_sentence("今天天气不错"));
    }
}
