use anyhow::Result;
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub const SOCKET_PATH: &str = "/tmp/personal-log-ai-ime.sock";

// ============================================================
// 公开数据结构
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum IMETextType {
    Direct,    // 直接输入（英文、数字、符号）
    Composing, // 组合中（拼音、候选状态）
    Committed, // 已提交（最终汉字）
}

#[derive(Debug, Clone)]
pub struct IMETextEvent {
    pub text_type: IMETextType,
    pub text: String,
    pub timestamp: i64,
    pub app_bundle_id: Option<String>,
    pub app_name: Option<String>,
}

// ============================================================
// IME IPC 服务端
// ============================================================

/// 启动 IPC 服务端，接收来自输入法的文本事件
///
/// 监听 Unix Domain Socket `/tmp/personal-log-ai-ime.sock`，
/// 解析以 0x1E (RS) 分隔的 JSON 消息，通过 mpsc channel 发送事件。
/// 返回 Receiver，调用方可通过 try_recv() 轮询事件。
pub fn start_ime_ipc_server() -> Result<Receiver<IMETextEvent>> {
    // 清理旧 socket 文件
    if Path::new(SOCKET_PATH).exists() {
        std::fs::remove_file(SOCKET_PATH)?;
    }

    let listener = UnixListener::bind(SOCKET_PATH)?;
    let (event_sender, event_receiver) = channel::<IMETextEvent>();

    // 启动接收线程，将事件通过 channel 转发给调用方
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let sender = event_sender.clone();
                    thread::spawn(move || {
                        handle_ime_connection(stream, sender);
                    });
                }
                Err(e) => eprintln!("IME IPC connection error: {}", e),
            }
        }
    });

    Ok(event_receiver)
}

/// 处理单个 IME 客户端连接
fn handle_ime_connection(stream: UnixStream, sender: Sender<IMETextEvent>) {
    let mut reader = BufReader::new(stream);
    let mut buffer = Vec::new();

    loop {
        buffer.clear();
        // 以 0x1E (RS - Record Separator) 作为消息分隔符
        match reader.read_until(0x1E, &mut buffer) {
            Ok(0) => break, // 连接关闭
            Ok(_) => {
                // 移除末尾的 0x1E 分隔符
                if buffer.ends_with(&[0x1E]) {
                    buffer.pop();
                }
                if buffer.is_empty() {
                    continue;
                }
                if let Ok(event) = parse_ime_message(&buffer) {
                    if let Err(e) = sender.send(event) {
                        eprintln!("IME IPC send error (receiver dropped): {}", e);
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("IME IPC read error: {}", e);
                break;
            }
        }
    }
}

/// 解析 IME 发送的 JSON 消息
fn parse_ime_message(data: &[u8]) -> Result<IMETextEvent> {
    let json: Value = serde_json::from_slice(data)?;

    let text_type = match json.get("type").and_then(|v| v.as_str()) {
        Some("direct") => IMETextType::Direct,
        Some("composing") => IMETextType::Composing,
        Some("committed") => IMETextType::Committed,
        _ => IMETextType::Direct, // 默认回退
    };

    let timestamp = json
        .get("timestamp")
        .and_then(|v| v.as_f64())
        .map(|t| (t * 1000.0) as i64)
        .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

    Ok(IMETextEvent {
        text_type,
        text: json.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        timestamp,
        app_bundle_id: json
            .get("appBundleId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        app_name: json
            .get("appName")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    })
}
