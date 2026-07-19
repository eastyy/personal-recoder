# 个人输入统计助理（原 Personal Log AI）- 项目交接记录

> 创建时间：2026-07-12
> 上次更新：2026-07-12（改名「个人输入统计助理」+ Rime精简到拼音/五笔98 + 报告生成bug修复 + 输入法签名修复）

## 项目概述

「个人输入统计助理」（英文名占位 Personal Input Stats Assistant）是一个 macOS 桌面工具（Tauri 2.0 + React + Rust），持续记录用户在电脑上的输入行为（键盘、鼠标、剪贴板、应用切换、IME），通过云端 LLM 进行多维度分析，生成个人化洞察报告。

**命名状态（2026-07-12 保守改名）：**
- Tauri productName / window title / 托盘 tooltip：✅ 已改为「个人输入统计助理」
- Tauri identifier（macOS bundle id）：✅ `com.personallogai.app` → `com.pisa.app`
- 前端品牌字符串（index.html / App.tsx / PermissionGuide.tsx）：✅ 已改
- **未改**（按用户要求保留，避免签名与数据迁移风险）：
  - Rust crate name `personal-log-ai` / npm package name `personal-log-ai`（仅源码标识，不影响用户）
  - IPC socket 路径 `/tmp/personal-log-ai-ime.sock`（rust ime_ipc.rs + swift LogIPCClient.swift 共 2 处）— IME 中文输入捕获的通信链路，改动会断链
  - 数据库目录 `~/Library/Application Support/PersonalLogAI/data.db`（db.rs + export.rs 共 3 处）— 改路径会孤立历史 9544 条数据
  - Squirrel 输入法二进制、`LogInputIME.app`、Swift 源码 ID — 不重新签名

## 路径

- 项目根目录：`/Users/yy/Documents/trae_projects/recoder/personal-log-ai`
- Squirrel IME 源码：`/Users/yy/Documents/trae_projects/recoder/squirrel-ime`
- 数据库：`~/Library/Application Support/PersonalLogAI/data.db`
- Rime 用户配置：`~/Library/Rime/`
- 已安装输入法：`/Library/Input Methods/Squirrel.app`
- 官方 Squirrel pkg 缓存：`~/Library/Caches/Homebrew/downloads/*Squirrel-1.1.2.pkg`

## 技术栈

- 前端：React 18 + TypeScript + TailwindCSS + Zustand + Vite
- 后端：Rust (Tauri 2.0) + SQLite (rusqlite)
- IME：Squirrel（鼠须管）官方版 + 自编译含 IPC hook 的二进制
- AI：MiniMax / OpenAI / 自定义 / 火山方舟（VolcEngine）

## 构建命令

- 前端：`cd personal-log-ai && npm run build`
- Rust 检查：`cd personal-log-ai/src-tauri && cargo check`
- 完整运行：`cd personal-log-ai && npm run tauri dev`
- Squirrel 编译：`cd squirrel-ime && xcodebuild -project Squirrel.xcodeproj -scheme Squirrel -configuration Release -derivedDataPath build -arch arm64 -arch x86_64 ONLY_ACTIVE_ARCH=NO clean build`
- Squirrel 一键安装：`cd squirrel-ime && ./build-and-install.sh`

## 核心架构：IME 中文输入捕获

### 工作原理

用户用鼠须管输入中文 → Squirrel 的 Rime 引擎做拼音/五笔转汉字 → 在 `commit(string:)` 方法提交汉字到目标应用的同时 → `LogIPCClient` 通过 Unix Socket 把同样的汉字发给 Personal Log AI → Rust 端 `ime_ipc.rs` 接收并存入数据库。

### 关键文件

**Squirrel 侧（squirrel-ime/）：**
- `sources/LogIPCClient.swift` — 轻量 Unix Socket 客户端（fire-and-forget，不阻塞输入法主线程）
- `sources/SquirrelInputController.swift` 第 551 行 `commit(string:)` — IPC hook 点，在文本提交时同步发送 IPC 消息
- `sources/Main.swift` 第 18 行 `appDir` — 输入法注册路径（改为 `~/Library/Input Methods/Squirrel.app`）
- `build-and-install.sh` — 一键编译+安装脚本

**Personal Log AI 侧（personal-log-ai/src-tauri/src/）：**
- `ime_ipc.rs` — IPC 服务端，监听 `/tmp/personal-log-ai-ime.sock`，解析 JSON+0x1E 分隔的消息
- `input.rs` — `process_ime_text_event()` 处理 IME 事件，`is_blacklisted_sync()` 隐私黑名单，`is_secure_text_field()` 密码框检测
- `db.rs` — `get_recent_events()` 查询包含 ime_committed 事件（注意：timestamp 是毫秒级）
- `ai.rs` — AI 分析，支持 4 个服务商（minimax/openai/custom/volcengine）
- `config.rs` — 配置管理，包含 volcengine 默认值，内存缓存 + 数据库持久化
- `main.rs` — Tauri 命令注册，包含 `check_ime_integration`、`squirrel_reload`、`open_keyboard_settings`

### IPC 协议

- Socket：`/tmp/personal-log-ai-ime.sock`（Unix Domain Socket）
- 消息格式：JSON + 0x1E (RS) 分隔符
- JSON 字段：type(committed/direct), text, timestamp(float), appBundleId, appName
- Rust 端 `parse_ime_message()` 解析，存为 event_type=`ime_committed`

## 签名方案（最终版，重要）

### 系统级 Squirrel.app 的部署方式

1. **从官方 pkg 安装**（保留 Team ID `28HU5A7B46` 签名）
2. **只替换主二进制**：`Contents/MacOS/Squirrel` → 含 IPC hook 的版本
3. **只签主二进制**（adhoc + entitlements），绝对不签 bundle，不用 --deep
4. **保留官方 dylib 签名**：`Contents/Frameworks/librime.1.dylib` 等

### 签名命令

```bash
# 主二进制签名（带 entitlements）
sudo codesign --force --sign - --entitlements squirrel-ime/resources/Squirrel.entitlements \
  /Library/Input\ Methods/Squirrel.app/Contents/MacOS/Squirrel

# 如果添加了新文件到 SharedSupport，需要先更新 bundle 签名（更新 CodeResources）
sudo codesign --force --sign - /Library/Input\ Methods/Squirrel.app
sudo codesign --force --sign - --entitlements squirrel-ime/resources/Squirrel.entitlements \
  /Library/Input\ Methods/Squirrel.app/Contents/MacOS/Squirrel
```

### 官方 pkg 恢复方法

```bash
# brew 缓存路径
pkg=~/Library/Caches/Homebrew/downloads/*Squirrel-1.1.2.pkg

# 解压官方 pkg
mkdir -p /tmp/squirrel-pkg && cd /tmp/squirrel-pkg
xar -xf $pkg
cat Payload | gunzip -dc | cpio -i

# 用官方 Squirrel.app 覆盖系统级（保留 dylib 官方签名）
sudo rm -rf "/Library/Input Methods/Squirrel.app"
sudo cp -R /tmp/squirrel-pkg/Squirrel.app "/Library/Input Methods/Squirrel.app"

# 替换主二进制为含 IPC hook 的版本
sudo cp /path/to/custom/Squirrel "/Library/Input Methods/Squirrel.app/Contents/MacOS/Squirrel"

# 恢复官方辅助二进制（rime-install 等）
sudo cp /tmp/squirrel-pkg/Squirrel.app/Contents/MacOS/rime-install "/Library/Input Methods/Squirrel.app/Contents/MacOS/rime-install"
sudo cp /tmp/squirrel-pkg/Squirrel.app/Contents/MacOS/rime_deployer "/Library/Input Methods/Squirrel.app/Contents/MacOS/rime_deployer"
sudo cp /tmp/squirrel-pkg/Squirrel.app/Contents/MacOS/rime_dict_manager "/Library/Input Methods/Squirrel.app/Contents/MacOS/rime_dict_manager"
```

⚠️ macOS 26 (Tahoe) 拒绝加载 adhoc 签名的 dylib，所以必须保留官方版的 dylib 签名。
⚠️ 绝对不能用 `codesign --deep` 签 Squirrel.app，会把官方 dylib 的 Team ID 签名覆盖成 adhoc。

## 当前状态

### 已完成的功能

1. **Squirrel 输入法集成** — 已安装、注册、启用，中文输入捕获正常工作
2. **拼音全拼 + 五笔98 + 五笔86** — 输入方案已配置，默认输出简体中文
3. **4 个 AI 服务商** — MiniMax / OpenAI / 自定义 / 火山方舟
4. **IME 事件前端展示** — RealtimeMonitor 和 ContentBrowser 支持 ime_committed 事件
5. **IMESettings 页面** — Squirrel 状态展示（安装检测、IPC 状态、hook 验证、方案列表）
6. **隐私黑名单** — 密码管理器、银行客户端、VPN 工具 + 密码框窗口标题检测
7. **AI 分析纳入中文输入** — get_recent_events 查询包含 ime_committed
8. **构建自动化** — build-and-install.sh 一键脚本
9. **旧代码清理** — ime-proxy 目录已删除
10. **AI 分析时间戳修复** — 2026-07-12 修复 `get_analysis_data_for_type` 用秒级时间戳查询毫秒级数据的 bug

### AI 服务商配置

当前使用火山方舟（volcengine）：
- api_key: <REDACTED-volcengine-api-key>
- base_url: https://ark.cn-beijing.volces.com/api/coding/v3
- model: deepseek-v4-flash

### 重启后需要验证的事项

1. Squirrel 是否自动启动（应该在系统登录时自动启动）
2. Personal Log AI 主应用启动后 IPC socket 是否建立（`ls -la /tmp/personal-log-ai-ime.sock`）
3. 输入中文后数据库是否有 ime_committed 事件（`sqlite3 ~/Library/Application\ Support/PersonalLogAI/data.db "SELECT COUNT(*) FROM raw_events WHERE event_type='ime_committed';"`）
4. 系统设置输入法列表里鼠须管是否正常显示（应该只有 1 个）
5. 输入法切换快捷键是否正常（Ctrl+Space 切换）

## 已知问题和注意事项

### 输入法签名相关

1. **绝对不能用 codesign --deep 签 Squirrel.app** - 会把官方 dylib 的 Team ID 签名覆盖成 adhoc，导致 IMK 拒绝连接（"unrecognized InputMethodConnectionName"）
2. **Info.plist 权限** - PlistBuddy/defaults 修改 Info.plist 后文件权限会变为 600，导致 codesign 验证失败 + TIS 无法读取 ComponentInputModeDict（子源 Hans/Hant 不注册）。修复：`sudo chmod 644 Info.plist`
3. **xattr 问题** - com.apple.provenance、com.apple.quarantine 会导致签名问题，需 `sudo xattr -cr /Library/Input\ Methods/Squirrel.app`
4. **辅助二进制签名** - rime-install、rime_deployer、rime_dict_manager 需要保留官方签名，否则 bundle 验证失败
5. **添加新文件到 SharedSupport** - 需要 `codesign --force --sign -` 更新 CodeResources，然后重签主二进制
6. **输入法切换快捷键** - `defaults write com.apple.symbolichotkeys` 中的 16（Ctrl+Space）和 17（Ctrl+Opt+Space）被禁用会导致无法切换输入法

### AI 分析相关

7. **timestamp 单位不一致** - 数据库存毫秒级（13位），但 `ai.rs` 的 `get_analysis_data_for_type` 原先用秒级（10位），导致查询不到数据。修复：用 `chrono::Utc::now().timestamp_millis()`
8. **AI 分析缺少日志** - `ai.rs` 原先没有日志输出，调用失败无法定位。已添加 `eprintln!` 调试日志（trigger_analysis called, provider, calling volcengine, volcengine ok/error）
9. **配置缓存** - `config.rs` 使用内存缓存，应用启动时从数据库加载。运行时通过 `set_config` 修改会同时更新内存和数据库

### Squirrel 相关

10. **Squirrel 自动更新** — Sparkle 可能尝试更新并覆盖 IPC hook，需禁用：`sudo defaults write /Library/Input\ Methods/Squirrel.app/Contents/Info.plist SUEnableAutomaticUpdates -bool false`
11. **重新编译后需替换二进制** — 编译后的 Squirrel 二进制需手动替换到 `/Library/Input\ Methods/Squirrel.app/Contents/MacOS/Squirrel` 并用 codesign 签名
12. **输入法重复注册** — 如果 Squirrel.app 出现在多个路径（如编译目录），LaunchServices 会注册多个，需用 `lsregister -u` 反注册多余路径
13. **IMK 修复后客户端应用可能缓存旧状态** — 需注销重新登录才能完全恢复

## 模块结构

```
personal-log-ai/
├── src-tauri/src/
│   ├── main.rs        — Tauri 命令注册
│   ├── db.rs          — SQLite 数据层（timestamp 毫秒级）
│   ├── input.rs       — CGEventTap + IME IPC + 会话超时 + 黑名单
│   ├── focus.rs       — 应用/网站焦点追踪
│   ├── ai.rs          — LLM 分析（4 个服务商，含 volcengine，时间戳用毫秒）
│   ├── config.rs      — 配置管理（内存缓存 + 数据库持久化）
│   ├── ime_ipc.rs     — IME Unix Socket IPC 服务端
│   ├── stats.rs       — 统计计算
│   ├── todo.rs        — TODO 提取
│   ├── scheduler.rs   — 定时分析调度器
│   └── export.rs      — 数据导出
├── src/pages/
│   ├── Dashboard.tsx
│   ├── TimeStats.tsx
│   ├── RealtimeMonitor.tsx — 含 ime_committed 事件标签
│   ├── ContentBrowser.tsx  — 含 ime_committed 筛选选项
│   ├── ReportCenter.tsx    — 5 种分析 + 按日期分组 + 折叠展开
│   ├── TodoList.tsx
│   ├── Settings.tsx        — 4 个 AI 服务商（含火山方舟）
│   ├── IMESettings.tsx     — Squirrel 状态展示页面
│   └── PermissionGuide.tsx
└── src/services/tauri.ts

squirrel-ime/
├── sources/
│   ├── LogIPCClient.swift          — IPC 客户端
│   ├── SquirrelInputController.swift — commit() 含 IPC hook
│   ├── Main.swift                   — appDir 路径
│   └── InputSource.swift            — TIS 注册/启用
├── build-and-install.sh
├── resources/Squirrel.entitlements
└── data/plum/                      — 输入方案（含 wubi98）
```

## 验证方法

```bash
# Rust 编译
cd personal-log-ai/src-tauri && cargo check

# TypeScript 编译
cd personal-log-ai && npx tsc --noEmit

# Squirrel 签名验证
codesign -vv /Library/Input\ Methods/Squirrel.app
codesign -dvv /Library/Input\ Methods/Squirrel.app/Contents/Frameworks/librime.1.dylib | grep TeamIdentifier

# Squirrel IPC hook 验证
strings /Library/Input\ Methods/Squirrel.app/Contents/MacOS/Squirrel | grep personal-log-ai

# IME 事件验证
sqlite3 ~/Library/Application\ Support/PersonalLogAI/data.db "SELECT * FROM raw_events WHERE event_type='ime_committed' ORDER BY rowid DESC LIMIT 5;"

# TIS 注册状态
swift /tmp/check_tis.swift

# IPC socket
ls -la /tmp/personal-log-ai-ime.sock

# 输入法切换快捷键
defaults read com.apple.symbolichotkeys AppleSymbolicHotKeys | grep -A2 "16 =\|17 ="
```

## 今日修复记录（2026-07-12）

### 1. 输入法签名问题（3 个 bug）

**问题 A：Info.plist 权限 600**
- 现象：输入法官网看不到鼠须管
- 原因：PlistBuddy 修改 Info.plist 后权限变 600，codesign 无法读取
- 修复：`sudo chmod 644 Info.plist`

**问题 B：codesign --deep 覆盖官方 dylib 签名**
- 现象：IMK 报 "unrecognized InputMethodConnectionName"
- 原因：--deep 把官方 dylib 的 Team ID 签名覆盖成 adhoc
- 修复：从官方 pkg 恢复整个 Squirrel.app，只替换主二进制

**问题 C：输入法切换快捷键被禁用**
- 现象：切不到其他输入法
- 原因：symbolichotkeys 16/17 被禁用
- 修复：`defaults write com.apple.symbolichotkeys AppleSymbolicHotKeys -dict-add 16 "..."`

### 2. AI 分析时间戳 bug

**问题**：报告中心点击生成分析按钮没有反应
**原因**：`get_analysis_data_for_type` 用秒级时间戳查询毫秒级数据，导致查不到数据
**修复**：`chrono::Utc::now().timestamp()` → `chrono::Utc::now().timestamp_millis()`
**附加**：添加了 eprintln! 调试日志

### 3. AI 报告生成失败（2026-07-12 晚诊断）

**症状**：报告中心点击生成分析按钮 → 后端报错，前端看到错误
**根因**：**火山方舟账号本周配额耗尽**（HTTP 429 AccountQuotaExceeded，2026-07-12 23:59:59 +0800 重置）
**不是代码 bug**：URL/Key/Model 用 curl 单独验证都 OK；账号被火山方舟 rate limit
**临时方案**：
- 等 2026-07-13 00:00 后再点生成分析
- 或去火山方舟后台升级套餐
- 或切到 minimax（DB 里 `UPDATE app_config SET value='minimax' WHERE key='ai_provider';`，把 minimax_api_key 写进去）

### 4. AI 报告相关代码 bug（顺带修了）

**Bug A：demo 模式不写库**
- 文件：src-tauri/src/ai.rs 第 33-51 行
- 现象：`is_configured==false` 返回 Ok 但不写 DB，前端 `loadReports()` 查不到这条，用户感觉"按钮按了没反应"
- 修复：demo 也用 `demo-{uuid}` id 写库，result_text 明确告诉用户哪个 provider 没配

**Bug B：config.rs 默认 volcengine_base_url 错**
- 文件：src-tauri/src/config.rs（init_config + get_config 两处）
- 原值：https://ark.cn-beijing.volces.com/api/v3
- 修正：https://ark.cn-beijing.volces.com/api/coding/v3
- 影响：仅"新建数据库"或"重置配置"时命中；当前 DB 里存的是对的（手动改过）

### 5. 改名为「个人输入统计助理」（2026-07-12 晚）

**范围（保守策略，5 文件 / 6 处）：**

| 文件 | 改动 |
|---|---|
| `personal-log-ai/src-tauri/tauri.conf.json` | `productName` + `identifier` (`com.personallogai.app`→`com.pisa.app`) + window `title` 三处 |
| `personal-log-ai/src-tauri/src/main.rs` 第 602 行 | 托盘 tooltip |
| `personal-log-ai/index.html` | `<title>` |
| `personal-log-ai/src/App.tsx` 第 81 行 | 侧边栏 h1 |
| `personal-log-ai/src/pages/PermissionGuide.tsx` 第 61 行 | h1 |

**验证**：ad-hoc 脚本全过（所有目标字符串命中、cargo check 干净、IPC socket / crate name / db path 排除项未被误改）

**未改清单（按用户决定保留）：**
- Cargo crate name + npm package name → 不重建依赖链
- `/tmp/personal-log-ai-ime.sock` IPC socket 路径（Rust ime_ipc.rs:9 + Swift LogIPCClient.swift:37 双胞胎）→ 中文输入捕获链路
- `~/Library/Application Support/PersonalLogAI/` 数据库路径（db.rs:27,34,41 + export.rs:294,300,306 六个点）→ 历史 9544 条数据
- Squirrel 二进制、`~/Library/Input Methods/LogInputIME.app`、Swift bundle id `im.rime.inputmethod.Squirrel` → 不重新签名、不破坏 IMK 注册

**用户后续动作：**
1. `cd personal-log-ai && npm run tauri build`（或 dev）
2. 新 .app 安装后 Dock 同时显示「个人输入统计助理」+ 老的「Personal Log AI」—— 手删旧的
3. **重新授权 Accessibility + Screen Recording**（bundle id 变了系统视为新 app）
4. 老的 `data.db` 在原路径还在，新 app 直接续用（DB 路径未改）

**完整改名调研报告：** `/Users/yy/.hermes/cache/delegation/subagent-summary-0-20260712_200219_609938.txt`（267 行详尽 per-file table + 风险评估，下次想动剩余 8 个文件时可以查）

### 6. Rime 精简：只留 拼音 + 五笔98（2026-07-12 晚）

**目标：** 用户配置常用 Rime 方案从 9 个减到 2 个

**改动文件：** `~/Library/Rime/default.custom.yaml`

```yaml
patch:
  schema_list:
    - schema: luna_pinyin_simp   # 简体中文拼音
    - schema: wubi98             # 五笔98
  "switcher/hotkeys":
    - Control+grave              # 切下一方案
    - Control+Shift+grave        # 切上一方案
    - F4                         # 打开方案选单
```

**关键步骤：**
1. 编辑 custom.yaml
2. `chmod 644 ~/Library/Rime/default.custom.yaml`（修复 pre-existing 600 权限导致 rime 静默不读的问题——之前精简一直不生效的根因）
3. `killall Squirrel && /Library/Input\ Methods/Squirrel.app/Contents/MacOS/Squirrel --reload`（强制 rime 重新部署 + 清内存缓存）
4. 清理 `~/Library/Rime/build/` 里残留的废弃方案 .bin / .yaml 文件

**验证（ad-hoc 脚本）：**
- `default.custom.yaml` 内容 = 2 个方案 ✓
- 权限 = 644 ✓
- `build/default.yaml` 部署产物 = 2 个方案 ✓
- `build/` 目录无残留方案文件 ✓
- Squirrel 进程在跑（PID 33008）✓

**用户切换方式（无需命令）:**
- 菜单栏松鼠图标 → 方案选单 → 选 拼音 / 五笔98
- 或在任何应用输入框按 `Ctrl+反引号`（反引号 = 键盘最左上角 `~` 同键）循环切换