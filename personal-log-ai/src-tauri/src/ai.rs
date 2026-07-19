use anyhow::Result;
use serde_json::{json, Value};
use std::time::Duration;

/// 触发 AI 分析任务
pub async fn trigger_analysis(analysis_type: &str) -> Result<Value> {
    eprintln!("[ai] trigger_analysis called: type={}", analysis_type);
    let config = crate::config::get_config()?;
    let config_map = config.as_object().ok_or_else(|| anyhow::anyhow!("Invalid config"))?;

    let provider = config_map
        .get("ai_provider")
        .and_then(|v| v.as_str())
        .unwrap_or("minimax");
    eprintln!("[ai] provider={}", provider);

    let (api_key, is_configured) = match provider {
        "openai" | "custom" => {
            let key = config_map.get("openai_api_key").and_then(|v| v.as_str()).unwrap_or("");
            (key.to_string(), !key.is_empty())
        }
        "volcengine" => {
            let key = config_map.get("volcengine_api_key").and_then(|v| v.as_str()).unwrap_or("");
            (key.to_string(), !key.is_empty())
        }
        _ => {
            let key = config_map.get("minimax_api_key").and_then(|v| v.as_str()).unwrap_or("");
            (key.to_string(), !key.is_empty())
        }
    };

    if !is_configured {
        let hint = match provider {
            "volcengine" => "火山方舟 (volcengine)",
            "openai" => "OpenAI",
            "custom" => "自定义 (custom)",
            _ => "MiniMax",
        };
        let msg = format!(
            "⚠️ 演示模式：{} API Key 未配置。\n\n请在「设置 → AI 服务商」中选择服务商并填入 Key 后保存。",
            hint
        );
        // demo 也写库,这样前端刷新报告中心能看到"为什么没生成"
        let demo_id = format!("demo-{}", uuid::Uuid::new_v4());
        let _ = crate::db::insert_analysis_result(&demo_id, analysis_type, &msg);
        return Ok(json!({
            "id": demo_id,
            "analysis_type": analysis_type,
            "result_text": msg,
            "created_at": chrono::Utc::now().timestamp(),
        }));
    }

    // 获取分析数据
    let data = get_analysis_data_for_type(analysis_type).await?;

    if data.is_empty() {
        return Ok(json!({
            "id": "demo",
            "analysis_type": analysis_type,
            "result_text": "当前没有足够的输入数据进行分析，请稍后再试。",
            "created_at": chrono::Utc::now().timestamp(),
        }));
    }

    // 构建提示词
    let prompt = build_prompt(analysis_type, &data);
    eprintln!("[ai] prompt length={}, calling LLM...", prompt.len());

    // 调用 LLM
    let result = match provider {
        "openai" | "custom" => {
            let base_url = config_map
                .get("openai_base_url")
                .and_then(|v| v.as_str())
                .unwrap_or("https://api.openai.com/v1")
                .to_string();
            let model = config_map
                .get("openai_model")
                .and_then(|v| v.as_str())
                .unwrap_or("gpt-4o-mini")
                .to_string();
            eprintln!("[ai] calling openai: base_url={}, model={}", base_url, model);
            match call_openai(&api_key, &base_url, &model, &prompt).await {
                Ok(r) => { eprintln!("[ai] openai ok, len={}", r.len()); r }
                Err(e) => { eprintln!("[ai] openai error: {}", e); return Err(e); }
            }
        }
        "volcengine" => {
            let base_url = config_map
                .get("volcengine_base_url")
                .and_then(|v| v.as_str())
                .unwrap_or("https://ark.cn-beijing.volces.com/api/v3")
                .to_string();
            let model = config_map
                .get("volcengine_model")
                .and_then(|v| v.as_str())
                .unwrap_or("doubao-pro-4k")
                .to_string();
            eprintln!("[ai] calling volcengine: base_url={}, model={}", base_url, model);
            match call_openai(&api_key, &base_url, &model, &prompt).await {
                Ok(r) => { eprintln!("[ai] volcengine ok, len={}", r.len()); r }
                Err(e) => { eprintln!("[ai] volcengine error: {}", e); return Err(e); }
            }
        }
        _ => {
            let group_id = config_map
                .get("minimax_group_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            call_minimax(&api_key, &group_id, &prompt).await?
        }
    };

    // 保存结果
    let id = uuid::Uuid::new_v4().to_string();
    let _ = crate::db::insert_analysis_result(&id, analysis_type, &result);

    // 如果是 TODO 分析，自动提取 TODO
    if analysis_type == "todo" {
        if let Some(todo_text) = crate::todo::extract_todo_from_text(&result) {
            let _ = crate::db::insert_todo(&todo_text, None, chrono::Utc::now().timestamp());
        }
        // 也尝试从结果中批量提取
        let todos = crate::todo::extract_todos_from_session(&result);
        for todo_text in todos.iter().take(10) {
            let _ = crate::db::insert_todo(todo_text, None, chrono::Utc::now().timestamp());
        }
    }

    Ok(json!({
        "id": id,
        "analysis_type": analysis_type,
        "result_text": result,
        "created_at": chrono::Utc::now().timestamp(),
    }))
}

/// 根据分析类型获取对应的数据
async fn get_analysis_data_for_type(analysis_type: &str) -> Result<String> {
    let now = chrono::Utc::now().timestamp_millis();

    match analysis_type {
        "weekly" => {
            // 获取最近7天的数据
            let week_ago = now - 7 * 86400;
            let events = crate::db::get_recent_events(week_ago, now)?;

            let mut text = String::new();
            for event in events.iter().take(300) {
                if let Some(content) = event.get("content").and_then(|c| c.as_str()) {
                    if !content.is_empty() {
                        let ts = event.get("timestamp").and_then(|t| t.as_i64()).unwrap_or(0);
                        let date = chrono::DateTime::from_timestamp(ts, 0)
                            .map(|d| d.format("%m-%d %H:%M").to_string())
                            .unwrap_or_default();
                        text.push_str(&format!("[{}] {}\n", date, content));
                    }
                }
            }
            if text.len() > 8000 {
                text.truncate(8000);
                text.push_str("\n...[内容已截断]");
            }
            Ok(text)
        }
        "todo" => {
            // 获取最近1小时的数据
            let hour_ago = now - 3600;
            let events = crate::db::get_recent_events(hour_ago, now)?;
            let mut text = String::new();
            for event in events.iter().take(100) {
                if let Some(content) = event.get("content").and_then(|c| c.as_str()) {
                    if !content.is_empty() {
                        text.push_str(content);
                        text.push('\n');
                    }
                }
            }
            if text.len() > 3000 {
                text.truncate(3000);
                text.push_str("\n...[内容已截断]");
            }
            Ok(text)
        }
        _ => {
            // 默认获取最近24小时的数据
            let yesterday = now - 86400;
            let events = crate::db::get_recent_events(yesterday, now)?;
            let mut text = String::new();
            for event in events.iter().take(200) {
                if let Some(content) = event.get("content").and_then(|c| c.as_str()) {
                    if !content.is_empty() {
                        text.push_str(content);
                        text.push('\n');
                    }
                }
            }
            if text.len() > 5000 {
                text.truncate(5000);
                text.push_str("\n...[内容已截断]");
            }
            Ok(text)
        }
    }
}

/// 构建分析提示词
fn build_prompt(analysis_type: &str, data: &str) -> String {
    match analysis_type {
        "productivity" => format!(
            r#"你是一个个人生产力助手。请分析以下用户今日的电脑输入数据，给出生产力评估。

要求：
1. 评估时间分配是否合理
2. 识别可能的效率瓶颈
3. 给出3条具体改进建议
4. 用中文回答，控制在300字以内

以下是用户今日的输入数据（片段）：
{data}

请按以下格式输出：
## 生产力评估
[评分：X/100]

## 时间分析
[分析内容]

## 改进建议
1. [建议1]
2. [建议2]
3. [建议3]"#
        ),
        "topic" => format!(
            r#"你是一个个人助手。请从以下用户今日的输入数据中，提取3-5个核心话题/主题。

要求：
1. 识别用户主要在做什么工作
2. 提取讨论的关键话题
3. 用中文简洁回答

用户输入数据：
{data}

请按以下格式输出：
## 今日核心话题
1. [话题1]：[简要说明]
2. [话题2]：[简要说明]
3. [话题3]：[简要说明]"#
        ),
        "writing" => format!(
            r#"你是一个写作教练。请分析以下用户今日的文本输入，给出写作优化建议。

要求：
1. 检查是否有错别字或用词不当
2. 分析表达风格（简洁/冗长/正式/口语化）
3. 给出3条具体改进建议
4. 用中文回答

用户输入文本：
{data}

请按以下格式输出：
## 写作分析
[整体评价]

## 发现的问题
1. [问题1]
2. [问题2]

## 优化建议
1. [建议1]
2. [建议2]
3. [建议3]"#
        ),
        "todo" => format!(
            r#"你是一个任务管理助手。请从以下用户输入中，提取所有可能的待办事项（TODO）。

要求：
1. 识别明确的任务、计划、提醒
2. 过滤掉非任务性的陈述
3. 每条TODO简洁明确
4. 尝试提取截止日期（如果有）
5. 用中文回答

用户输入：
{data}

请按以下格式输出，如果没有找到TODO则回复"未发现待办事项"：
## 提取的 TODO
- [ ] [任务1] [截止日期（如有）]
- [ ] [任务2]
- [ ] [任务3]"#
        ),
        "weekly" => format!(
            r#"你是一个个人复盘助手。请综合分析以下用户过去一周的输入数据，生成一页纸周报。

要求：
1. 概括本周的主要活动和产出
2. 分析时间使用效率
3. 识别本周的习惯模式（如深夜工作、频繁切换应用等）
4. 与上周可能的对比趋势（基于数据推测）
5. 给出下周改进建议
6. 用中文回答，控制在500字以内

以下是用户本周的输入数据（片段）：
{data}

请按以下格式输出：
# 本周复盘报告

## 本周概览
[一段话总结]

## 产出统计
- 总输入字符数：约 X
- 活跃天数：X 天
- 主要应用：[应用名]

## 习惯分析
[分析内容]

## 下周建议
1. [建议1]
2. [建议2]
3. [建议3]"#
        ),
        _ => format!("请分析以下数据并给出洞察：\n{}", data),
    }
}

/// 调用 MiniMax API
async fn call_minimax(api_key: &str, group_id: &str, prompt: &str) -> Result<String> {
    let client = reqwest::Client::new();

    let body = json!({
        "model": "abab5.5-chat",
        "messages": [
            {
                "role": "system",
                "content": "你是一个个人生产力助手，帮助分析用户的工作数据。请用中文回答。"
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "tokens_to_generate": 2048,
        "temperature": 0.7,
    });

    let url = if group_id.is_empty() {
        "https://api.minimax.chat/v1/text/chatcompletion_v2".to_string()
    } else {
        format!("https://api.minimax.chat/v1/text/chatcompletion_v2?GroupId={}", group_id)
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(60))
        .json(&body)
        .send()
        .await?;

    let status = response.status();
    let response_text = response.text().await?;

    if !status.is_success() {
        return Err(anyhow::anyhow!("MiniMax API 请求失败: {} - {}", status, response_text));
    }

    let response_json: Value = serde_json::from_str(&response_text)?;
    let content = response_json
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("AI 分析完成，但返回内容为空");

    Ok(content.to_string())
}

/// 调用 OpenAI 兼容 API
async fn call_openai(api_key: &str, base_url: &str, model: &str, prompt: &str) -> Result<String> {
    let client = reqwest::Client::new();

    let body = json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": "你是一个个人生产力助手，帮助分析用户的工作数据。请用中文回答。"
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "max_tokens": 2048,
        "temperature": 0.7,
    });

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(60))
        .json(&body)
        .send()
        .await?;

    let status = response.status();
    let response_text = response.text().await?;

    if !status.is_success() {
        return Err(anyhow::anyhow!("OpenAI API 请求失败: {} - {}", status, response_text));
    }

    eprintln!("[ai] API response status={}, body_len={}", status, response_text.len());
    let response_json: Value = serde_json::from_str(&response_text)?;
    let content = response_json
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("AI 分析完成，但返回内容为空");

    Ok(content.to_string())
}
