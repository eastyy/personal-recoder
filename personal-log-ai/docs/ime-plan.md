# Personal Log AI - 输入法版本规划文档

> 版本：v1.0 | 日期：2026-06-10 | 状态：规划中

---

## 1. 项目概述

### 1.1 目标
开发一个基于 InputMethodKit 的轻量级代理输入法（IME Proxy），解决当前 CGEventTap 无法捕获中文 IME 提交文本的问题。代理输入法以"透传"模式工作，用户几乎无感知，同时将所有输入文本（中文、英文、数字、符号）实时同步到 Personal Log AI 主应用。

### 1.2 核心挑战
- **CGEventTap 局限**：无法捕获 IME `commitComposition` 提交的汉字
- **用户体验**：不能改变用户原有的输入习惯和底层输入法行为
- **系统限制**：macOS 输入法必须以独立 .app 形式注册

### 1.3 方案选型结论

| 方案 | 说明 | 评估 |
|------|------|------|
| A. 代理输入法（推荐） | 基于 InputMethodKit 开发透传输入法 | 精确捕获、无感知、技术可行 |
| B. Accessibility API 轮询 | 定期读取焦点文本框内容 | 有延迟、不准确、依赖辅助功能权限 |
| C. 混合方案 | CGEventTap + 启发式检测 | 无法可靠捕获中文 |

**最终选择：方案 A（代理输入法）**

---

## 2. 系统架构

### 2.1 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                        用户层                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  系统拼音    │  │  搜狗输入法  │  │   其他第三方输入法   │  │
│  │  (底层IME)   │  │  (底层IME)   │  │                     │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
│         │                │                    │             │
│         └────────────────┼────────────────────┘             │
│                          ▼                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │           LogInputIME (代理输入法)                    │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │   │
│  │  │ IMKInputController │  │ 按键透传模块  │  │ IPC发送模块  │  │   │
│  │  │  - 接收按键事件    │  │  - 转发给底层 │  │  - Unix Socket│  │   │
│  │  │  - 接收提交文本    │  │  - 接收提交   │  │  - 发送文本   │  │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  │   │
│  └────────────────────────┬────────────────────────────┘   │
│                           │                                 │
│                           │ Unix Domain Socket              │
│                           ▼                                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │         Personal Log AI (主应用)                      │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │   │
│  │  │ IPC接收模块  │  │  数据融合层   │  │  数据库存储   │  │   │
│  │  │  - Unix Socket│  │  - 合并CGEvent│  │  - SQLite   │  │   │
│  │  │  - 解析文本   │  │  - 去重/排序  │  │  - 事件记录   │  │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 代理输入法工作原理

```
用户按键 ──► LogInputIME ──► 底层输入法处理 ──► 候选词/拼音
                              │
                              ▼
                         用户选择/空格提交
                              │
                              ▼
                    IMKInputController.commitComposition
                              │
                              ▼
                    提取提交的文本（如"你好"）
                              │
                              ├─► 通过 Unix Socket 发送给主应用
                              │
                              └─► 正常提交给目标应用（透传）
```

---

## 3. 技术设计

### 3.1 代理输入法（LogInputIME）

#### 3.1.1 项目结构

```
LogInputIME/
├── LogInputIME.xcodeproj/          # Xcode 项目
├── Sources/
│   ├── main.swift                  # 入口：注册输入法组件
│   ├── LogInputController.swift    # IMKInputController 子类
│   ├── KeyForwardingHandler.swift  # 按键透传逻辑
│   ├── IPCClient.swift             # Unix Socket 客户端
│   └── Info.plist                  # 输入法注册信息
├── Resources/
│   └── LogInputIME.icns            # 输入法图标
└── LogInputIME.entitlements        # 沙盒配置（无沙盒）
```

#### 3.1.2 核心类设计

**LogInputController.swift**

```swift
import InputMethodKit

class LogInputController: IMKInputController {
    
    // 底层输入法控制器（透传目标）
    private var underlyingInputController: IMKInputController?
    
    // IPC 客户端
    private let ipcClient = IPCClient.shared
    
    // 当前组合文本缓冲区
    private var compositionBuffer: String = ""
    
    // MARK: - 按键事件处理
    
    override func handle(_ event: NSEvent!, client sender: IMKTextInput!) -> Bool {
        // 1. 记录按键事件（用于分析打字节奏）
        logKeyEvent(event)
        
        // 2. 将按键转发给底层输入法处理
        // 底层输入法会处理拼音转换、候选词显示等
        let handled = forwardToUnderlyingIME(event, client: sender)
        
        return handled
    }
    
    // MARK: - 文本提交拦截
    
    override func commitComposition(_ sender: IMKTextInput!) {
        // 获取即将提交的文本
        if let committedText = sender.markedText {
            // 发送给主应用
            ipcClient.sendText(committedText, type: .committed)
        }
        
        // 调用底层输入法的提交
        underlyingInputController?.commitComposition(sender)
        
        // 清空缓冲区
        compositionBuffer = ""
    }
    
    // MARK: - 组合文本更新
    
    override func setMarkedText(_ string: Any!, selectionRange: NSRange, replacementRange: NSRange) {
        // 记录组合中的文本（拼音、候选状态）
        if let text = string as? String {
            compositionBuffer = text
            ipcClient.sendText(text, type: .composing)
        }
        
        // 转发给底层输入法
        underlyingInputController?.setMarkedText(string, selectionRange: selectionRange, replacementRange: replacementRange)
    }
    
    // MARK: - 直接输入（英文、数字、符号）
    
    override func inputText(_ string: String!, key keyCode: Int, modifiers flags: Int, client sender: IMKTextInput!) -> Bool {
        // 直接输入的文本（无需 IME 转换）
        if let text = string, !text.isEmpty {
            ipcClient.sendText(text, type: .direct)
        }
        
        // 转发给底层输入法或直接提交
        return underlyingInputController?.inputText(string, key: keyCode, modifiers: flags, client: sender) ?? false
    }
}
```

#### 3.1.3 按键透传逻辑

```swift
class KeyForwardingHandler {
    
    /// 判断按键是否应该由底层输入法处理
    func shouldForwardToUnderlyingIME(_ event: NSEvent) -> Bool {
        // 所有按键都转发给底层输入法
        // 代理输入法本身不做任何输入处理
        return true
    }
    
    /// 获取当前用户实际使用的底层输入法
    func getUnderlyingIME() -> IMKInputController? {
        // 方案 1：读取用户配置的首选底层输入法
        // 方案 2：自动检测当前系统默认中文输入法
        // 方案 3：让用户在代理输入法设置中选择
        
        // 当前采用方案 3：用户配置
        let preferredIME = UserDefaults.standard.string(forKey: "preferredUnderlyingIME")
        // ... 创建对应的 IMKInputController
        return nil
    }
}
```

#### 3.1.4 IPC 客户端（Unix Domain Socket）

```swift
import Foundation

class IPCClient {
    static let shared = IPCClient()
    
    private var socket: Int32 = -1
    private let socketPath = "/tmp/personal-log-ai-ime.sock"
    private let queue = DispatchQueue(label: "ipc-client")
    
    private init() {
        connect()
    }
    
    private func connect() {
        socket = Darwin.socket(AF_UNIX, SOCK_STREAM, 0)
        guard socket >= 0 else { return }
        
        var addr = sockaddr_un()
        addr.sun_family = sa_family_t(AF_UNIX)
        strncpy(&addr.sun_path.0, socketPath, MemoryLayout.size(ofValue: addr.sun_path) - 1)
        
        let addrLen = socklen_t(MemoryLayout<sockaddr_un>.size)
        let result = withUnsafePointer(to: &addr) {
            $0.withMemoryRebound(to: sockaddr.self, capacity: 1) {
                Darwin.connect(socket, $0, addrLen)
            }
        }
        
        if result < 0 {
            close(socket)
            socket = -1
        }
    }
    
    func sendText(_ text: String, type: TextType) {
        guard socket >= 0 else {
            connect() // 尝试重连
            return
        }
        
        let message = IPCMessage(
            type: type,
            text: text,
            timestamp: Date().timeIntervalSince1970,
            appBundleId: getCurrentAppBundleId(),
            appName: getCurrentAppName()
        )
        
        if let data = try? JSONEncoder().encode(message) {
            var payload = data
            payload.append(0x1E) // 记录分隔符 (RS)
            _ = payload.withUnsafeBytes { Darwin.write(socket, $0.baseAddress, $0.count) }
        }
    }
    
    enum TextType: String, Codable {
        case direct      // 直接输入（英文、数字、符号）
        case composing   // 组合中（拼音、候选）
        case committed   // 已提交（最终汉字）
    }
    
    struct IPCMessage: Codable {
        let type: TextType
        let text: String
        let timestamp: TimeInterval
        let appBundleId: String?
        let appName: String?
    }
}
```

### 3.2 主应用 IPC 服务端

#### 3.2.1 新增模块：`src/ime_ipc.rs`

```rust
use std::io::{BufRead, BufReader, Read};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::thread;

const SOCKET_PATH: &str = "/tmp/personal-log-ai-ime.sock";

#[derive(Debug, Clone)]
pub enum IMETextType {
    Direct,     // 直接输入
    Composing,  // 组合中
    Committed,  // 已提交
}

#[derive(Debug, Clone)]
pub struct IMETextEvent {
    pub text_type: IMETextType,
    pub text: String,
    pub timestamp: i64,
    pub app_bundle_id: Option<String>,
    pub app_name: Option<String>,
}

/// 启动 IPC 服务端，接收来自输入法的文本事件
pub fn start_ime_ipc_server(event_sender: Sender<IMETextEvent>) -> anyhow::Result<()> {
    // 清理旧 socket 文件
    if Path::new(SOCKET_PATH).exists() {
        std::fs::remove_file(SOCKET_PATH)?;
    }
    
    let listener = UnixListener::bind(SOCKET_PATH)?;
    
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
    
    Ok(())
}

fn handle_ime_connection(stream: UnixStream, sender: Sender<IMETextEvent>) {
    let reader = BufReader::new(stream);
    let mut buffer = String::new();
    
    for line in reader.split(0x1E) { // 以 RS 分隔
        match line {
            Ok(data) => {
                if let Ok(event) = parse_ime_message(&data) {
                    let _ = sender.send(event);
                }
            }
            Err(e) => eprintln!("IME IPC read error: {}", e),
        }
    }
}

fn parse_ime_message(data: &[u8]) -> anyhow::Result<IMETextEvent> {
    let json: serde_json::Value = serde_json::from_slice(data)?;
    
    Ok(IMETextEvent {
        text_type: match json["type"].as_str() {
            Some("direct") => IMETextType::Direct,
            Some("composing") => IMETextType::Composing,
            Some("committed") => IMETextType::Committed,
            _ => IMETextType::Direct,
        },
        text: json["text"].as_str().unwrap_or("").to_string(),
        timestamp: (json["timestamp"].as_f64().unwrap_or(0.0) * 1000.0) as i64,
        app_bundle_id: json["appBundleId"].as_str().map(|s| s.to_string()),
        app_name: json["appName"].as_str().map(|s| s.to_string()),
    })
}
```

#### 3.2.2 数据融合层

```rust
// src/input.rs 中新增

use std::sync::mpsc::{channel, Receiver};
use std::collections::VecDeque;

static IME_EVENT_RECEIVER: Mutex<Option<Receiver<IMETextEvent>>> = Mutex::new(None);

/// 初始化 IME IPC 接收
pub fn init_ime_receiver() -> anyhow::Result<()> {
    let (sender, receiver) = channel::<IMETextEvent>();
    crate::ime_ipc::start_ime_ipc_server(sender)?;
    *IME_EVENT_RECEIVER.lock().unwrap() = Some(receiver);
    Ok(())
}

/// 处理来自输入法的文本事件
fn process_ime_text_event(event: IMETextEvent) {
    match event.text_type {
        IMETextType::Committed => {
            // 已提交的文本（如"你好"）
            // 合并到当前会话 buffer
            record_ime_committed_text(&event.text);
        }
        IMETextType::Direct => {
            // 直接输入（英文、数字）
            // 与 CGEventTap 捕获的做去重处理
            record_ime_direct_text(&event.text);
        }
        IMETextType::Composing => {
            // 组合中的文本（拼音、候选状态）
            // 可选：记录用于分析输入过程
            record_ime_composing_text(&event.text);
        }
    }
}

/// 在定时器循环中轮询 IME 事件
fn poll_ime_events() {
    if let Ok(mut guard) = IME_EVENT_RECEIVER.lock() {
        if let Some(ref receiver) = *guard {
            while let Ok(event) = receiver.try_recv() {
                process_ime_text_event(event);
            }
        }
    }
}
```

### 3.3 输入法安装与启用流程

#### 3.3.1 安装流程

```
用户点击"安装输入法"
    │
    ▼
┌─────────────────────┐
│ 1. 检查输入法是否已安装 │
│    - 检查 /Library/Input Methods/LogInputIME.app │
│    - 或 ~/Library/Input Methods/LogInputIME.app  │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ 2. 复制输入法到系统目录 │
│    - 需要管理员权限（/Library）│
│    - 或用户目录（~/Library）  │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ 3. 注册输入法到系统   │
│    - 执行 registerInputMethod: │
│    - 或调用 Text Input Source Services API │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ 4. 引导用户启用输入法 │
│    - 打开系统设置 > 键盘 > 输入法 │
│    - 勾选 LogInputIME │
│    - 提示切换到该输入法 │
└─────────────────────┘
```

#### 3.3.2 前端 UI 设计

**设置页面新增"输入法"板块：**

```
┌─────────────────────────────────────────┐
│  输入法集成                              │
├─────────────────────────────────────────┤
│                                         │
│  状态: ● 已安装  ○ 未安装  ○ 已启用      │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │  安装 LogInputIME 代理输入法     │   │
│  │                                 │   │
│  │  解决中文输入法文本捕获问题：      │   │
│  │  • 准确记录中文输入（如"你好"）   │   │
│  │  • 兼容系统拼音、搜狗等输入法      │   │
│  │  • 透传模式，不影响输入体验        │   │
│  │                                 │   │
│  │      [  安装输入法  ]            │   │
│  └─────────────────────────────────┘   │
│                                         │
│  安装后操作：                            │
│  1. 打开 系统设置 > 键盘 > 输入法        │
│  2. 勾选 "LogInputIME"                 │
│  3. 在菜单栏切换到该输入法              │
│                                         │
│  [ 打开键盘设置 ]  [ 检查安装状态 ]      │
│                                         │
│  底层输入法选择：                        │
│  ┌──────────────────────────────┐      │
│  │ 系统拼音 (Pinyin)            │ ▼    │
│  └──────────────────────────────┘      │
│  （代理输入法会将按键转发给此输入法处理）  │
│                                         │
└─────────────────────────────────────────┘
```

---

## 4. 开发里程碑

### Milestone 1: 代理输入法原型（Week 1）

**目标**：可独立运行的最小化代理输入法

| 任务 | 描述 | 预估工时 |
|------|------|---------|
| 1.1 | 创建 Xcode 项目，配置 InputMethodKit 框架 | 2h |
| 1.2 | 实现 IMKInputController 子类，处理基本按键 | 4h |
| 1.3 | 实现按键透传逻辑（转发给底层输入法） | 4h |
| 1.4 | 实现文本提交拦截（commitComposition） | 3h |
| 1.5 | 实现 IPC 客户端（Unix Socket） | 3h |
| 1.6 | 打包为 .app，测试安装到系统输入法列表 | 2h |

**验收标准**：
- [ ] 输入法可安装到 `/Library/Input Methods/`
- [ ] 可在系统设置中启用
- [ ] 切换到该输入法后，按键可透传给底层输入法
- [ ] 文本提交时可通过 socket 发送测试消息

### Milestone 2: 主应用 IPC 接收（Week 1-2）

**目标**：主应用可接收并存储输入法发送的文本

| 任务 | 描述 | 预估工时 |
|------|------|---------|
| 2.1 | 创建 `src/ime_ipc.rs` 模块，实现 Unix Socket 服务端 | 3h |
| 2.2 | 在 `input.rs` 中集成 IME 事件处理 | 2h |
| 2.3 | 实现数据融合：CGEventTap + IME 事件去重 | 4h |
| 2.4 | 数据库表扩展：标记 IME 来源的事件 | 2h |
| 2.5 | 实时状态显示 IME 输入统计 | 2h |

**验收标准**：
- [ ] 主应用启动时创建 Unix Socket 监听
- [ ] 输入法提交的文本正确存入数据库
- [ ] 实时监控页面显示 IME 输入事件
- [ ] 与 CGEventTap 捕获的英文输入不重复

### Milestone 3: 安装与配置流程（Week 2）

**目标**：用户可一键安装和配置输入法

| 任务 | 描述 | 预估工时 |
|------|------|---------|
| 3.1 | 主应用内嵌输入法 .app 资源 | 2h |
| 3.2 | 实现输入法安装命令（复制到系统目录） | 3h |
| 3.3 | 实现输入法状态检测命令 | 2h |
| 3.4 | 前端"输入法设置"页面 | 4h |
| 3.5 | 引导用户启用输入法的提示流程 | 2h |
| 3.6 | 底层输入法选择配置 | 2h |

**验收标准**：
- [ ] 前端有完整的输入法安装/配置 UI
- [ ] 可检测输入法是否已安装、已启用
- [ ] 一键安装按钮工作正常
- [ ] 安装后正确引导用户到系统设置启用

### Milestone 4: 透传优化与稳定性（Week 3）

**目标**：输入法透传稳定，用户体验无感知

| 任务 | 描述 | 预估工时 |
|------|------|---------|
| 4.1 | 优化底层输入法切换逻辑 | 4h |
| 4.2 | 处理特殊按键（方向键、回车、退格） | 3h |
| 4.3 | 处理组合键（Cmd+C/V，Ctrl+Space） | 3h |
| 4.4 | 长文本输入的性能优化 | 2h |
| 4.5 | IPC 连接断线重连机制 | 2h |
| 4.6 | 内存泄漏检查和修复 | 2h |

**验收标准**：
- [ ] 连续输入 1000 字无卡顿
- [ ] 切换应用后输入法正常工作
- [ ] 主应用重启后输入法自动重连
- [ ] 无内存泄漏（24 小时运行测试）

### Milestone 5: 测试与发布（Week 4）

| 任务 | 描述 | 预估工时 |
|------|------|---------|
| 5.1 | 单元测试：IPC 协议编解码 | 2h |
| 5.2 | 集成测试：输入法 + 主应用端到端 | 4h |
| 5.3 | 兼容性测试：不同 macOS 版本 | 3h |
| 5.4 | 兼容性测试：不同底层输入法 | 3h |
| 5.5 | 编写用户文档 | 2h |
| 5.6 | 打包发布 | 2h |

---

## 5. 数据流与事件处理

### 5.1 事件类型定义

```rust
// 扩展现有事件类型
pub enum EventType {
    KeyDown,
    KeyUp,
    MouseClick,
    MouseMove,
    Clipboard,
    AppFocus,
    // 新增 IME 相关
    IMEDirect,      // 代理输入法：直接输入（英文、数字）
    IMEComposing,   // 代理输入法：组合中（拼音）
    IMECommitted,   // 代理输入法：已提交（汉字）
}
```

### 5.2 去重策略

当同时开启 CGEventTap 和代理输入法时，英文输入会被两边都捕获：

```
CGEventTap 捕获: "hello"（逐个字母）
代理输入法捕获: "hello"（完整字符串）

去重逻辑：
1. 同一应用内，500ms 窗口期内
2. CGEventTap 的字母序列拼接后与 IME Direct 文本匹配
3. 丢弃 CGEventTap 的字母事件，保留 IME Direct 的完整文本
```

### 5.3 会话 Buffer 融合

```
会话 Buffer 构建：

时间线 ──────────────────────────────────────────►

CGEventTap:    h  e  l  l  o     [空格]
IME Direct:         "hello"              
IME Committed:                          "世界"

融合结果:      "hello 世界"

存储到数据库：
- 事件1: type=ime_direct, content="hello"
- 事件2: type=ime_committed, content="世界"
```

---

## 6. 风险评估与缓解

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|---------|
| 透传逻辑不完善导致输入体验差 | 中 | 高 | 充分测试各种输入法场景；提供快速禁用开关 |
| 底层输入法 API 变更 | 低 | 中 | 关注 macOS 更新；保持代码简洁易于修改 |
| 用户不愿安装额外输入法 | 中 | 高 | 清晰的安装引导；强调透传无感知；提供备选方案说明 |
| IPC 通信被安全软件拦截 | 低 | 中 | 使用标准 Unix Socket；提供故障排查指南 |
| 与某些应用兼容性差 | 中 | 中 | 测试主流应用；提供应用级禁用配置 |
| App Store 审核问题 | 中 | 高 | 输入法作为独立组件；主应用不直接包含输入法二进制 |

---

## 7. 参考资源

### 7.1 开源项目参考

| 项目 | 用途 | 关键文件 |
|------|------|---------|
| [McBopomofo](https://github.com/openvanilla/McBopomofo) | InputMethodKit 架构参考 | `Source/InputMethodController.swift` |
| [azooKey-Desktop](https://github.com/azooKey/azooKey-Desktop) | 现代 Swift IME 实现 | `azooKey-macOS/` |
| [Squirrel](https://github.com/rime/squirrel) | 输入法生命周期管理 | `macos/SquirrelInputController.m` |

### 7.2 Apple 官方文档

- [InputMethodKit Framework](https://developer.apple.com/documentation/inputmethodkit)
- [IMKInputController](https://developer.apple.com/documentation/inputmethodkit/imkinputcontroller)
- [Text Input Sources](https://developer.apple.com/documentation/coreservices/text_input_sources)

---

## 8. 附录

### 8.1 IPC 协议规范

```json
{
  "version": 1,
  "type": "committed",
  "text": "你好世界",
  "timestamp": 1718000000.123,
  "appBundleId": "com.apple.TextEdit",
  "appName": "TextEdit"
}
```

**TextType 枚举：**
- `direct` - 直接输入的字符（英文、数字、符号）
- `composing` - 组合中的文本（拼音、未确认的候选）
- `committed` - 已确认提交的文本（最终汉字）

**分隔符：** 0x1E (RS - Record Separator)

### 8.2 文件清单

**新增文件：**
- `src/ime_ipc.rs` - IPC 服务端
- `src-tauri/ime-proxy/` - 代理输入法 Xcode 项目
- `src/pages/IMESettings.tsx` - 输入法设置页面

**修改文件：**
- `src/input.rs` - 集成 IME 事件处理
- `src/main.rs` - 添加 IME 相关命令
- `src/db.rs` - 扩展事件类型枚举
- `src/App.tsx` - 添加输入法设置入口

---

*文档结束*
