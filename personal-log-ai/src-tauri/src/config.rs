use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;

static CONFIG_CACHE: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

/// 初始化配置：从数据库加载已有配置，填充默认值
pub fn init_config() -> Result<()> {
    let mut cache = HashMap::new();
    
    // 先设置默认值
    cache.insert("minimax_api_key".to_string(), "".to_string());
    cache.insert("minimax_group_id".to_string(), "".to_string());
    cache.insert("openai_api_key".to_string(), "".to_string());
    cache.insert("openai_base_url".to_string(), "https://api.openai.com/v1".to_string());
    cache.insert("openai_model".to_string(), "gpt-4o-mini".to_string());
    cache.insert("volcengine_api_key".to_string(), "".to_string());
    cache.insert("volcengine_base_url".to_string(), "https://ark.cn-beijing.volces.com/api/coding/v3".to_string());
    cache.insert("volcengine_model".to_string(), "doubao-pro-4k".to_string());
    cache.insert("ai_provider".to_string(), "minimax".to_string());
    cache.insert("data_retention_days".to_string(), "90".to_string());
    cache.insert("enable_clipboard".to_string(), "true".to_string());
    cache.insert("enable_mouse".to_string(), "true".to_string());
    cache.insert("pause_threshold".to_string(), "3".to_string());
    cache.insert("session_timeout".to_string(), "60".to_string());
    
    // 从数据库加载已有配置覆盖默认值
    if let Ok(conn) = crate::db::get_conn() {
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value FROM app_config")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
            ))
        })?;
        for row in rows {
            if let Ok((key, value)) = row {
                if let Some(v) = value {
                    cache.insert(key, v);
                }
            }
        }
    }
    
    let mut global = CONFIG_CACHE.lock().unwrap();
    *global = Some(cache);
    
    Ok(())
}

pub fn get_config() -> Result<Value> {
    let cache = CONFIG_CACHE.lock().unwrap();
    let config = cache.as_ref().cloned().unwrap_or_else(|| {
        let mut default = HashMap::new();
        default.insert("minimax_api_key".to_string(), "".to_string());
        default.insert("minimax_group_id".to_string(), "".to_string());
        default.insert("openai_api_key".to_string(), "".to_string());
        default.insert("openai_base_url".to_string(), "https://api.openai.com/v1".to_string());
        default.insert("openai_model".to_string(), "gpt-4o-mini".to_string());
        default.insert("volcengine_api_key".to_string(), "".to_string());
        default.insert(
            "volcengine_base_url".to_string(),
            "https://ark.cn-beijing.volces.com/api/coding/v3".to_string(),
        );
        default.insert("volcengine_model".to_string(), "doubao-pro-4k".to_string());
        default.insert("ai_provider".to_string(), "minimax".to_string());
        default.insert("data_retention_days".to_string(), "90".to_string());
        default.insert("enable_clipboard".to_string(), "true".to_string());
        default.insert("enable_mouse".to_string(), "true".to_string());
        default.insert("pause_threshold".to_string(), "3".to_string());
        default.insert("session_timeout".to_string(), "60".to_string());
        default
    });
    
    Ok(json!(config))
}

pub fn set_config(key: &str, value: &str) -> Result<()> {
    // 更新内存缓存
    {
        let mut cache = CONFIG_CACHE.lock().unwrap();
        if cache.is_none() {
            *cache = Some(HashMap::new());
        }
        if let Some(ref mut map) = *cache {
            map.insert(key.to_string(), value.to_string());
        }
    }
    
    // 同时保存到数据库
    let _ = crate::db::get_conn()?.lock().unwrap().execute(
        "INSERT OR REPLACE INTO app_config (key, value, updated_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![key, value, chrono::Utc::now().timestamp()],
    );
    
    Ok(())
}
