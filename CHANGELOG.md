# Changelog

本项目的所有重要变更都会记录在此文件。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循 [Semantic Versioning](https://semver.org/lang/zh-CN/)。

---

## [Unreleased]

### Added
- **`CHANGELOG.md`**：Keep a Changelog 格式的版本变更记录，规定每次 commit 前必更新
- **`AGENTS.md`**：面向任何 AI agent（Claude Code、Pi、Cursor、Aider 等）的精简交接文档
  - 覆盖项目概述、快速开始、仓库结构、关键技术决策（IME IPC 机制）、常见任务、禁忌
- **`SECURITY.md`**：安全策略文档（密钥管理规范、应急流程、事件复盘）
- **Git 工作流纪律**
  - `.githooks/pre-commit`：双重检查 (1) 密钥扫描 (2) CHANGELOG.md 同步更新，否则拒绝 commit
  - `scripts/scan-secrets.sh`：扫描 20+ 种 API Key 格式（OpenAI / Anthropic / AWS / Google / GitHub / Volcengine / Stripe / Slack / GitLab / SendGrid / Mailgun / 通用 Bearer Token 等）
  - `scripts/install-hooks.sh`：一键启用 git hooks（设置 `core.hooksPath = .githooks`）
  - 紧急绕过：`git commit --no-verify`

### Changed
- **`README.md`**：新增「文档索引」表格（README / AGENTS / CHANGELOG / SECURITY / 设计文档）
- **`README.md`**：在「快速开始」中增加 `bash scripts/install-hooks.sh` 步骤
- **`README.md`**：新增「Git 工作流」章节，说明 CHANGELOG.md 纪律和 hooks 用法
- **`AGENTS.md`**：加入 SECURITY.md 引用，文档索引表中标明 `.hermes/` 为本地笔记
- `.gitignore`：增加 `.hermes/` 排除规则（个人项目交接笔记，仅本地保留）

### Removed
- **`.hermes/` 从 git 索引移除**：`handover.md` 和 `project.md` 不再入库（本地保留）
  - 这些文件可能含签名命令、个人工作模式等隐私信息
  - 2026-07-19 起改为本地笔记

### Security
- **🚨 火山方舟 API Key 泄漏修复（在 .hermes/ 中）**：发现并删除明文 API Key，并用 `git filter-branch` 重写所有 git 历史
- **🚨 第二次泄漏（仓库误设为 public）**：filter-branch + force-push 不能清除 GitHub 内部存储的旧 commit 对象
  - 旧 commit `b865233` 仍可通过直链访问，包含完整 key 值
  - 应对：仓库将「删除重建」（这是唯一可靠方案）
- **⚠️ 用户必作手动动作**：
  - 在火山方舟控制台（https://console.volcengine.com/ark）轮换作废旧 Key
  - 检查旧 Key 的使用日志看是否有异常调用
- **新增自动化防护**：
  - `scripts/scan-secrets.sh` 在每次 commit 前自动扫描
  - `.githooks/pre-commit` 集成扫描 + CHANGELOG 检查
  - `.hermes/` 不再入库（防御性）

---

## [v0.1.0] - 2026-07-12

### Added
- **初始发布**：项目首次提交到 GitHub 私有仓库 `eastyy/personal-recoder`
- **Tauri 主项目** `personal-log-ai/`：React 18 + TypeScript 前端，Rust 后端
  - 全量输入采集（键盘、鼠标、剪贴板、应用切换、IME）
  - SQLite 本地存储（`~/Library/Application Support/PersonalLogAI/data.db`）
  - 多 LLM 服务商支持（MiniMax / OpenAI / 火山方舟 / 自定义 OpenAI 兼容端点）
  - 数据可视化仪表盘（Dashboard / Stats / Goals / Settings / RealtimeMonitor）
- **改造版 Squirrel 输入法** `squirrel-ime/`
  - 新增 `sources/LogIPCClient.swift`（139 行，通过 Unix Socket 发送中文到 Tauri）
  - 修改 `sources/SquirrelInputController.swift`（在 commit 时触发 IPC 钩子）
  - RIME 配方精简到拼音 / 五笔98
- **项目文档**
  - `README.md`：架构、构建步骤、安全说明
  - `docs/2026-06-07-personal-log-ai-design.md`：原始设计文档（1402 行）
  - `.hermes/handover.md` + `.hermes/project.md`：项目交接记录
  - `AGENTS.md`：精简版 agent 交接文档（**新增**）
- **自动化脚本**
  - `scripts/setup-squirrel.sh`：一键拉取 librime / plum / Sparkle 第三方依赖
- **Git 工作流**
  - `.githooks/pre-commit`：强制要求 CHANGELOG.md 同步更新（**新增**）
  - `scripts/install-hooks.sh`：安装 git hooks（**新增**）

### Changed
- 项目名从「Personal Log AI」改为「个人输入统计助理」（保持 Rust crate name / IPC socket 路径 / DB 路径不变以避免签名和数据迁移问题）

### Fixed
- 输入法签名修复（2026-07-12）
- AI 报告生成 bug 修复（2026-07-12）

### Security
- 所有 API Key 仅存于 SQLite `app_config` 表（运行时），不进入代码仓库
- `.gitignore` 覆盖 `*.pem` `*.key` `*.env` `*.db` 等敏感文件
- `.gitignore` 排除 `target/` (4.2 GB) / `node_modules/` (186 MB) / `build/` / `download/` 等构建产物
- 仓库设为 Private（`eastyy/personal-recoder`）

---

## 如何使用本文件

### 每次 commit 前必须更新

本项目有 **pre-commit hook**，如果你改了任何代码文件但没有更新 `CHANGELOG.md`，hook 会拒绝 commit。

### 正确做法

1. 写代码改动
2. **同时更新** `CHANGELOG.md` 的 `[Unreleased]` 段落
3. `git add . && git commit -m "..."`

### CHANGELOG 条目格式

在 `[Unreleased]` 段落下选一个分类（Added / Changed / Fixed / Removed / Security），加一行：

```markdown
### Added
- 新功能描述（动词开头，简洁具体）
- 另一个新功能

### Fixed
- 修复了 XXX 问题（说明原因和影响范围）
```

### 跳过 hook（紧急情况）

```bash
git commit --no-verify -m "emergency hotfix"
```

⚠️ **跳过前请确认**：真的紧急吗？通常更新 CHANGELOG.md 只要 30 秒。

### 发布新版本时

1. 把 `[Unreleased]` 内容移到新的版本段落：
   ```markdown
   ## [v0.2.0] - 2026-XX-XX

   ### Added
   - ...
   ```
2. 清空 `[Unreleased]`
3. 提交：`git commit -am "chore(release): v0.2.0"`
4. 打 tag：`git tag -a v0.2.0 -m "v0.2.0"`
5. 推送：`git push && git push --tags`

---

[Unreleased]: https://github.com/eastyy/personal-recoder/compare/v0.1.0...HEAD
[v0.1.0]: https://github.com/eastyy/personal-recoder/releases/tag/v0.1.0