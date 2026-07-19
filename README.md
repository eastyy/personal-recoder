# Personal Recoder（个人输入统计助理）

> 一个 macOS 桌面工具，持续记录你在电脑上的输入行为（键盘、鼠标、剪贴板、应用切换、IME），通过云端 LLM 进行多维度分析，生成个人化洞察报告。

**状态**：私有个人项目 / Personal use only

---

## ✨ 核心功能

- 🖮 **全量输入采集** - 键盘、鼠标、剪贴板、应用切换、IME 中文输入
- 📊 **多维度统计** - 时段、应用、活跃度、目标追踪
- 🤖 **LLM 分析** - MiniMax / OpenAI / 火山方舟 / 自定义 OpenAI 兼容端点
- 🀄 **中文捕获** - 通过改造版 Squirrel（鼠须管）输入法 + Unix Socket IPC 实现
- 📈 **可视化报告** - React + Tailwind 仪表盘

---

## 🏗 架构

```
┌─────────────────────────────────────────────────────┐
│  用户操作（键盘/鼠标/应用切换/中文输入）                │
└────────┬──────────────────────────┬──────────────────┘
         │                          │
         ▼                          ▼
   ┌──────────┐              ┌──────────────┐
   │ Tauri 后端 │ ◄── IPC ──► │ Squirrel IME │
   │ (Rust)    │  Unix Socket │ (改造版)     │
   │ rusqlite  │   /tmp/...   │ LogIPCClient │
   └─────┬─────┘              └──────────────┘
         │
         ▼
   ┌──────────┐         ┌─────────┐
   │ SQLite   │ ──────► │ LLM API │
   │ data.db  │         │ (云端)  │
   └─────┬────┘         └─────────┘
         │
         ▼
   ┌──────────────────────────────┐
   │ Tauri 前端 (React + Vite)    │
   │ Dashboard / Stats / Goals /  │
   │ Settings / Realtime Monitor  │
   └──────────────────────────────┘
```

### 关键路径
| 路径 | 用途 |
|------|------|
| `personal-log-ai/` | Tauri 2.0 主项目（React + Rust） |
| `squirrel-ime/` | 改造版 Squirrel（鼠须管）输入法 |
| `scripts/` | 自动化脚本（依赖拉取等） |
| `docs/` | 设计文档 |
| `.hermes/` | 项目交接记录 |

---

## 🛠 技术栈

**前端**
- React 18 + TypeScript + TailwindCSS
- Zustand（状态管理）
- Vite 5
- React Router 6
- lucide-react（图标）
- date-fns

**后端**
- Tauri 2.0（Rust）
- rusqlite（SQLite 绑定）
- tokio（异步运行时）
- reqwest（HTTP 客户端）
- objc2 / objc2-foundation / objc2-app-kit（macOS 系统调用）
- chrono / regex / uuid / anyhow / once_cell / lazy_static

**输入法**
- Squirrel（macOS RIME 前端）官方版 + 自编译含 IPC hook 的二进制
- librime（RIME 引擎，C++）
- plum（RIME 配置管理）
- Sparkle（macOS 应用更新框架）

**AI 服务**
- MiniMax（默认）
- OpenAI
- 火山方舟（VolcEngine）
- 自定义 OpenAI 兼容端点

---

## 🚀 快速开始

### 系统要求

- macOS 13+（Apple Silicon 或 Intel）
- Xcode 14.0+
- Homebrew
- Node.js 18+
- Rust 1.70+（`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`）

### 一次性依赖安装

```bash
# 安装系统依赖
brew install cmake boost pkg-config

# 克隆（带子模块）
git clone <your-repo-url>
cd personal-recoder

# 拉取 Squirrel 第三方依赖（librime / plum / Sparkle）
bash scripts/setup-squirrel.sh

# 安装 Tauri 前端依赖
cd personal-log-ai
npm install
cd ..
```

### 开发模式

```bash
cd personal-log-ai
npm run tauri dev
```

### 生产构建

```bash
cd personal-log-ai
npm run tauri build
# 产物：personal-log-ai/src-tauri/target/release/bundle/macos/个人输入统计助理.app
```

### 构建自定义 Squirrel（可选，含中文 IPC 捕获）

```bash
cd squirrel-ime
bash build-and-install.sh
```

⚠️ Squirrel 是 **系统级输入法**（`/Library/Input Methods/Squirrel.app`），安装会修改系统状态，需要 sudo 权限。详见 `squirrel-ime/INSTALL.md`。

---

## 📂 数据库

应用数据存在用户目录：

```
~/Library/Application Support/PersonalLogAI/data.db
```

**不要把数据库提交到 Git** —— 包含你的个人输入历史、API Key 配置等隐私数据。`.gitignore` 已默认排除 `*.db`。

如需重置数据，删除该文件后重启应用即可（表结构会自动重建）。

---

## ⚙️ 配置

应用首次启动后，UI 设置面板里需要填写：

- **AI 服务商选择**（MiniMax / OpenAI / 火山方舟 / 自定义）
- **API Key** + **Group ID** / **Base URL** / **Model**

配置存在 `data.db` 的 `app_config` 表里（运行时），不会进 Git。

---

## 🧩 IME 中文捕获原理

> 这是本项目最有意思的部分

1. 用户用鼠须管输入中文
2. Squirrel 的 RIME 引擎做拼音/五笔 → 汉字转换
3. 在 `commit(string:)` 方法提交汉字到目标应用的同时
4. **改造版** `LogIPCClient.swift` 通过 Unix Socket 把同样的汉字发给 Tauri 后端
5. Rust 端 `ime_ipc.rs` 接收并存入 SQLite

**关键文件**
- `squirrel-ime/sources/LogIPCClient.swift`（新增，139 行）
- `squirrel-ime/sources/SquirrelInputController.swift`（改造，约 620 行）
- `personal-log-ai/src-tauri/src/main.rs`（IPC 服务端，~556 行）

**IPC 协议**
- Socket 路径：`/tmp/personal-log-ai-ime.sock`
- 数据格式：`{timestamp_ms}|{text}\n`（每行一条记录）

---

## 🔐 安全注意事项

### 不要提交
- `*.pem` / `*.key` / `*.p12` - 签名证书 / 私钥
- `.env*` - API Key
- `*.db` - 用户数据库
- `target/` / `node_modules/` / `build/` - 编译产物

`.gitignore` 已默认覆盖上述全部。

### macOS 权限
应用需要以下权限（首次启动会弹窗）：
- 辅助功能（Accessibility） - 监听键盘鼠标
- 屏幕录制（Screen Recording） - 应用切换检测
- 输入监控（Input Monitoring） - 全局快捷键
- 完全磁盘访问（Full Disk Access） - 读取 `~/Library/Rime/`

---

## 📜 第三方依赖

| 组件 | 来源 | 许可证 |
|------|------|--------|
| Tauri | https://github.com/tauri-apps/tauri | MIT / Apache-2.0 |
| React | https://github.com/facebook/react | MIT |
| Squirrel | https://github.com/rime/squirrel | GPL-3.0 |
| librime | https://github.com/rime/librime | GPL-3.0 |
| plum | https://github.com/rime/plum | GPL-3.0 |
| Sparkle | https://github.com/sparkle-project/Sparkle | MIT |
| OpenCC | https://github.com/BYVoid/OpenCC | Apache-2.0 |
| TailwindCSS | https://github.com/tailwindlabs/tailwindcss | MIT |

详细许可见各子目录 `LICENSE*` 文件。

---

## 📝 项目交接

项目交接记录见 `.hermes/handover.md`，包含：
- 完整技术栈说明
- 构建命令
- Squirrel 签名方案
- 已知问题和注意事项

历史设计文档见 `docs/2026-06-07-personal-log-ai-design.md`。

---

## 📅 更新日志

### 2026-07-12
- 项目改名「个人输入统计助理」
- RIME 配置精简到拼音/五笔98
- 报告生成 bug 修复
- 输入法签名修复

### 2026-06-07
- 初始设计文档（`docs/2026-06-07-personal-log-ai-design.md`）

---

**Made with 🀄 in Shanghai**