# AGENTS.md

> 任何 AI agent（Claude Code、Pi、Cursor、Aider 等）接手这个项目时，**先读这个文件**。

---

## 这是什么 / What is this?

一个 **macOS 桌面工具**：持续记录用户在电脑上的输入行为（键盘、鼠标、剪贴板、应用切换、中文 IME），通过云端 LLM 做多维度分析，生成个人化洞察报告。

技术栈：**Tauri 2.0**（Rust + React）+ 改造版 **Squirrel（鼠须管）输入法**。

Tauri productName / window title / 托盘 tooltip：**「个人输入统计助理」**
Rust crate name / npm package / IPC socket 路径 / DB 路径：保持 `personal-log-ai` 不变（避免签名和数据迁移风险）

---

## 🚀 Quick Start（5 分钟上手）

```bash
# 1. 安装依赖（首次必需）
brew install cmake boost pkg-config
cd squirrel-ime && bash scripts/setup-squirrel.sh --shallow && cd ..
cd personal-log-ai && npm install && cd ..

# 2. 开发模式（前端 + 后端热重载）
cd personal-log-ai && npm run tauri dev

# 3. 生产构建
cd personal-log-ai && npm run tauri build
# 产物：personal-log-ai/src-tauri/target/release/bundle/macos/个人输入统计助理.app
```

⚠️ **首次 clone 后**：必须运行 `scripts/setup-squirrel.sh`，否则 librime/plum/Sparkle 缺失
⚠️ **git hooks**：运行 `bash scripts/install-hooks.sh` 启用 CHANGELOG.md 强制更新

---

## 📁 仓库结构 / Repository Layout

```
personal-recoder/
├── AGENTS.md              ← 你正在读的文件（agent 入门）
├── README.md              ← 项目架构 + 用户面向的构建文档
├── CHANGELOG.md           ← 每次 commit 前必须更新
├── .gitignore             ← 全场景忽略规则
│
├── personal-log-ai/       ← 【核心】Tauri 主项目
│   ├── src/               ← React + TypeScript 前端
│   │   ├── pages/         ← Dashboard / Stats / Goals / Settings / RealtimeMonitor
│   │   ├── App.tsx        ← 路由
│   │   └── main.tsx       ← 入口
│   ├── src-tauri/         ← Rust 后端
│   │   ├── src/
│   │   │   ├── main.rs           ← Tauri commands + IPC 服务端
│   │   │   ├── input.rs          ← 键盘/鼠标/剪贴板捕获（1323 行，最大文件）
│   │   │   ├── db.rs             ← SQLite schema
│   │   │   ├── config.rs         ← app_config 表（存 API Key）
│   │   │   ├── ai.rs             ← LLM 调用（MiniMax/OpenAI/Volc）
│   │   │   ├── export.rs         ← 数据导出
│   │   │   ├── stats.rs          ← 统计聚合
│   │   │   ├── todo.rs           ← 目标追踪
│   │   │   └── ime_ipc.rs        ← Unix Socket 接收中文（核心）
│   │   ├── tauri.conf.json       ← Tauri 配置（identifier: com.pisa.app）
│   │   └── Cargo.toml
│   ├── package.json       ← npm scripts: dev / build / tauri
│   ├── vite.config.ts
│   └── index.html
│
├── squirrel-ime/          ← 【核心】改造版 Squirrel（鼠须管）输入法
│   ├── sources/
│   │   ├── LogIPCClient.swift          ← 新增，139 行，通过 Socket 发中文到 Tauri
│   │   ├── SquirrelInputController.swift ← 改造，commit 时触发 IPC 钩子
│   │   └── ... (其他上游 Swift 文件)
│   ├── Squirrel.xcodeproj/
│   ├── build-and-install.sh            ← 一键构建 + 系统安装（需要 sudo）
│   ├── data/
│   │   ├── opencc/                     ← RIME 中文转换数据（保留入库）
│   │   └── plum/                       ← RIME 配置（wubi98.dict.yaml 等用户自定义）
│   ├── package/                        ← pkg 打包脚本
│   ├── .gitignore / .gitmodules / Makefile / INSTALL.md
│
├── scripts/
│   ├── setup-squirrel.sh      ← 拉取 librime/plum/Sparkle
│   ├── install-hooks.sh       ← 配置 git hooks 路径
│   └── scan-secrets.sh        ← 扫描 20+ 种 API Key 格式（被 pre-commit 调用）
│
├── .githooks/
│   └── pre-commit             ← 双重检查：(1) 密钥扫描 (2) CHANGELOG.md 同步更新
│
├── docs/
│   └── 2026-06-07-personal-log-ai-design.md   ← 原始设计文档（1402 行）
│
└── .hermes/                       ← 项目交接记录（用户面向）
    ├── handover.md                ← 完整技术细节 + 签名方案
    └── project.md                 ← 项目上下文
```

---

## 🔑 关键技术决策（必读）

### 1. IME 中文捕获 —— 整个项目最有意思的部分

**问题**：标准 Tauri 后端只能捕获**键盘按键码**（keyCode），无法识别用户最终输入的**中文字符**。

**解决方案**：改造 Squirrel 输入法，在 RIME 引擎 `commit(string:)` 时，把识别出的汉字通过 **Unix Socket** 发给 Tauri 后端。

**链路**：
```
用户输入拼音 → Squirrel RIME 引擎转汉字
   ↓
SquirrelInputController.swift.commit()
   ↓
LogIPCClient.swift.send(text) → Unix Socket /tmp/personal-log-ai-ime.sock
   ↓
Tauri Rust: ime_ipc.rs 接收
   ↓
存入 SQLite 的 input_event 表
```

**协议**：每行一条 `"{timestamp_ms}|{text}\n"`

**为什么这是关键**：
- 不能删 `LogIPCClient.swift` —— 它是中文捕获的唯一通路
- 不能改 IPC socket 路径 `/tmp/personal-log-ai-ime.sock` —— 改了会让 Squirrel 和 Tauri 失联
- 如果只靠 keyCode 捕获，你只能拿到 `"a b c"` 而不是 `"你好"`（拼音 → 汉字的转换发生在 RIME 引擎）

### 2. 配置和数据库不在仓库里

- **API Key** 存在 `~/Library/Application Support/PersonalLogAI/data.db` 的 `app_config` 表
- **用户输入历史** 也在同一个 db 文件
- **不要 commit `*.db`** —— `.gitignore` 已默认排除
- **改 API Key**：UI 设置面板输入，不要改任何 .ts/.rs 文件
- **个人项目交接笔记**（`.hermes/`）已在 2026-07-19 从 git 移除（参见 [SECURITY.md](SECURITY.md) 的「2026-07-19 事件复盘」）

### 3. Squirrel 系统级安装

- Squirrel 安装到 `/Library/Input Methods/Squirrel.app`（**系统级**，不是用户级）
- 修改主二进制需要 **sudo + 重新签名**
- 详细签名方案：`.hermes/handover.md` 的「签名方案」章节
- 如果不确定，**先问用户**再执行任何 sudo 命令

### 4. macOS 权限依赖

应用首次启动需要这些权限（系统会弹窗）：
- 辅助功能（Accessibility）—— 监听键盘鼠标
- 输入监控（Input Monitoring）—— 全局快捷键
- 屏幕录制（Screen Recording）—— 应用切换检测
- 完全磁盘访问（Full Disk Access）—— 读 `~/Library/Rime/`

---

## 🛠 常见任务 / Common Tasks

### 添加一个新的统计指标

1. **后端**：编辑 `personal-log-ai/src-tauri/src/stats.rs`，加一个 SQL 聚合函数
2. **注册 Tauri command**：编辑 `main.rs` 的 `#[tauri::command]` 列表
3. **前端**：编辑 `personal-log-ai/src/pages/Stats.tsx`，调用新 command
4. **类型同步**：如果用了 TypeScript，确保 `invoke<T>()` 的类型匹配
5. **更新 CHANGELOG.md**：在 `[Unreleased] / Added` 加一行

### 添加一个新的 UI 页面

1. 在 `personal-log-ai/src/pages/` 创建 `NewPage.tsx`
2. 在 `App.tsx` 加 `<Route>`
3. 在导航栏加链接
4. 如果需要新数据，加 Tauri command（同上）

### 修改 IME 捕获逻辑

1. 改 `squirrel-ime/sources/LogIPCClient.swift` 或 `SquirrelInputController.swift`
2. 重新编译：`cd squirrel-ime && bash build-and-install.sh`（需要 sudo）
3. 改完后**重启 Tauri 应用**才能接收新格式的数据
4. **同步检查** `personal-log-ai/src-tauri/src/ime_ipc.rs` 协议是否兼容

### 更新 LLM 服务商

1. 编辑 `personal-log-ai/src-tauri/src/ai.rs`
2. 添加新的 provider enum 值
3. 在 `src/pages/Settings.tsx` 的 provider 选择里加新选项
4. 测试：UI 设置面板输入新 provider 的 API Key，触发分析看是否成功

### 修复一个 bug

1. 复现：先描述清楚触发条件和期望行为
2. 定位：`git log --all --oneline -- <file>` 看相关历史
3. 修复：最小改动 + 清晰注释
4. **更新 CHANGELOG.md** 在 `[Unreleased] / Fixed` 加一行
5. 测试 + 验证

---

## ⛔ 禁忌 / Don'ts

**绝不能做**：
- ❌ **提交任何 API Key、Token、Secret、密码到 git**（包括「临时」放到 `.hermes/` / `docs/` / README 里思考）
  - 即使仓库是 private，即使你觉得“以后会删”
  - 本项目 2026-07 踩过这个坑：`.hermes/handover.md` 里写了火山方舟 key，`git filter-branch` 才清除干净
  - **正确做法**：Key 只存在 `data.db` 的 `app_config` 表（运行时）或本地 `.env`（已 gitignore）
- ❌ 提交 `*.db` 文件（用户输入历史）
- ❌ 提交 `*.pem` / `*.key` / `*.env` / `credentials.json` 等密钥文件
- ❌ 修改 IPC socket 路径 `/tmp/personal-log-ai-ime.sock`（会断链）
- ❌ 修改 DB 路径 `~/Library/Application Support/PersonalLogAI/data.db`（会孤立历史数据）
- ❌ 修改 Tauri identifier `com.pisa.app`（会破坏 macOS 签名和权限关联）
- ❌ 执行 sudo 命令不先问用户（特别是 Squirrel 安装）

**应该避免**：
- ⚠️ 不要改 Rust crate name `personal-log-ai` / npm package name `personal-log-ai`（源码标识，能改但意义不大）
- ⚠️ 不要用 `git add .` 加 `target/`（4.2 GB）—— `.gitignore` 已挡，但如果手动 `git add -f` 会绕过
- ⚠️ 不要在没有更新 CHANGELOG.md 的情况下 commit（hook 会阻止，但可用 `--no-verify` 绕过——**请不要绕过**）

**发现密钥泄漏的应急流程**：
1. 立即删除文件中的明文 key（替换为 placeholder）
2. `git filter-branch` 重写历史（参考本仓库 2026-07 的修复）
3. force push 到远端
4. **在提供商控制台轮换 key**（这是唯一能阻止已泄漏 key 被滥用的步骤）
5. 在 CHANGELOG.md 记录该事件

完整流程见 [SECURITY.md](SECURITY.md)。

---

## 📚 详细文档（按需查阅）

| 文档 | 内容 | 何时读 |
|---|---|---|
| [README.md](README.md) | 用户面向的架构 + 构建步骤 | 用户问"怎么装"时 |
| [CHANGELOG.md](CHANGELOG.md) | 版本变更历史 | 写版本发布说明时 |
| [SECURITY.md](SECURITY.md) | 密钥管理规范 + 应急响应流程 | 发现 / 怀疑泄漏时 |
| [docs/2026-06-07-personal-log-ai-design.md](docs/2026-06-07-personal-log-ai-design.md) | 原始设计文档（1402 行） | 理解早期设计决策时 |
| `.hermes/`（本地） | 个人项目交接笔记（仅本地，2026-07-19 起不入库） | 调试 Squirrel / 签名问题时 |

---

## 🧰 开发环境参考

| 工具 | 版本 / 路径 |
|---|---|
| macOS | 13+ (Apple Silicon / Intel) |
| Xcode | 14.0+ |
| Node.js | 18+ |
| Rust | 1.70+ |
| CMake | 3.x（`brew install cmake`） |
| Boost | 1.84+（`brew install boost`） |

---

## 💡 给 agent 的最后提示

1. **每次回答前先读这个文件**——但不要复读给用户
2. **不确定就问**——特别是 sudo / 删文件 / 改 IPC 协议时
3. **小步快跑**——一次只做一个改动，立刻验证
4. **保持诚实**——不会就说不会，不要编造 API 或代码路径
5. **更新 CHANGELOG.md**——这是这个项目的纪律

Happy hacking 🀄