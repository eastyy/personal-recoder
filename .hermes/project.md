# Personal Log AI - 项目上下文

| 更新时间：2026-07-12（输入法签名修复 + AI分析时间戳bug修复，交接记录见 .hermes/handover.md）

## 项目概述
Personal Log AI 是一个 macOS 桌面工具（Tauri 2.0 + React + Rust），持续记录用户在电脑上的输入行为（键盘、鼠标、剪贴板、应用切换、IME），通过云端 LLM 进行多维度分析，生成个人化洞察报告。

## 技术栈
- 前端：React 18 + TypeScript + TailwindCSS + Zustand + Vite（3,016行）
- 后端：Rust (Tauri 2.0) + SQLite (rusqlite)（4,382行）
- IME：Squirrel（鼠须管）Fork，植入 IPC hook（路径 squirrel-ime/）
- AI：MiniMax / OpenAI 兼容 API

## 模块结构
```
src-tauri/src/
├── main.rs (501)    - Tauri 命令注册、应用入口、托盘
├── db.rs (513)      - SQLite 数据层（动态查询、聚合）
├── input.rs (1310)  - CGEventTap 输入采集 + IME IPC 集成 + 会话超时
├── focus.rs (212)   - 应用/网站焦点追踪
├── ai.rs (396)      - LLM 分析（MiniMax + OpenAI 兼容 + 5种结构化提示词）
├── config.rs (90)   - 配置管理（多AI提供商）
├── ime_ipc.rs (130) - IME Unix Socket IPC
├── stats.rs (301)   - 统计计算（WPM、退格率、专注度、打字节奏、应用切换）
├── todo.rs (465)    - TODO 提取（7种正则模式 + 批量提取 + 截止日期解析）
├── scheduler.rs (156) - 定时分析调度器（每日/每周/每小时 + 数据清理）
└── export.rs (308)  - 数据导出（JSON/CSV）+ 清理 + DB 统计

src/
├── App.tsx (200)               - 主应用 + 路由 + 侧边栏 + 8个页面
├── stores/appStore.ts (57)     - Zustand 全局状态
├── services/tauri.ts (51)      - Tauri API 封装（25个命令）
└── pages/
    ├── Dashboard.tsx (332)     - 总览（统计卡片 + 24h热力图 + Top应用 + AI洞察）
    ├── TimeStats.tsx (597)     - 时长统计（日期选择 + 饼图 + 切换统计 + 对比）
    ├── RealtimeMonitor.tsx (278) - 实时监控（5种监控类型 + 事件流）
    ├── ContentBrowser.tsx (235) - 内容浏览器（日期/类型/搜索筛选 + 分页）
    ├── ReportCenter.tsx (178)  - 报告中心（5种分析 + 按日期分组 + 折叠展开）
    ├── TodoList.tsx (221)      - TODO 列表（逾期检测 + 截止日期 + 添加）
    ├── Settings.tsx (400)      - 设置（3种API提供商 + 采集配置 + 数据管理）
    ├── IMESettings.tsx (262)   - 输入法设置
    └── PermissionGuide.tsx (192) - 权限引导
```

## 数据库表
raw_events, input_sessions, focus_sessions, focus_daily, focus_hourly, analysis_results, todo_items, flash_ideas, app_rules, user_goals, app_config

## 构建
- 前端：`cd personal-log-ai && npm run build`（tsc + vite build）
- Rust：`cd personal-log-ai/src-tauri && cargo check`
- 完整：`cd personal-log-ai && npm run tauri dev`

### 2026-07-12 签名修复

1. **问题：codesign --deep 重签 bundle 导致 IMK 拒绝连接**
   - 原因：--deep 把官方 dylib 的 Team ID 签名覆盖成 adhoc，IMK 不信任 adhoc 签名的 bundle
   - 修复：从 brew 缓存的官方 pkg 恢复整个 Squirrel.app，只替换主二进制
   - 教训：绝对不能对 Squirrel.app 用 codesign --deep

2. **问题：Info.plist 权限 600 导致 TIS 子源不注册**
   - 原因：PlistBuddy 修改 plist 后权限变为 600，codesign 无法读取
   - 修复：chmod 644

3. **问题：输入法切换快捷键被禁用**
   - 修复：defaults write com.apple.symbolichotkeys 16/17 enabled=1

4. **签名方案（最终版）**：
   - 官方 pkg 安装（Team ID 28HU5A7B46 签名）
   - 只替换主二进制为含 IPC hook 的版本
   - `sudo codesign --force --sign - --entitlements Squirrel.entitlements Squirrel`（只签主二进制）
   - 不签 bundle，不签 dylib，不用 --deep
   - 如果添加了新文件到 SharedSupport，需要 `sudo codesign --force --sign - Squirrel.app` 更新 CodeResources，然后重签主二进制

### 2026-07-12 AI 分析修复

5. **问题：报告中心点击生成分析按钮没有反应**
   - 原因：`get_analysis_data_for_type` 用秒级时间戳查询毫秒级数据，导致查不到数据
   - 修复：`chrono::Utc::now().timestamp()` → `chrono::Utc::now().timestamp_millis()`
   - 附加：添加 eprintln! 调试日志（trigger_analysis called, provider, calling volcengine, ok/error）

6. **AI 服务商配置**（火山方舟）：
   - api_key: <REDACTED-volcengine-api-key>
   - base_url: https://ark.cn-beijing.volces.com/api/coding/v3
   - model: deepseek-v4-flash

### 第一轮（基础功能补全）
1. 新增 scheduler.rs - 定时分析（每日03:00、每周日03:30、每小时）
2. 新增 export.rs - JSON/CSV导出 + 数据清理 + DB统计
3. 增强 stats.rs - 打字统计、24h节奏、生产力评分、应用切换统计
4. 增强 todo.rs - 更多提取模式 + 批量提取 + 截止日期解析
5. 重写 ai.rs - 支持 MiniMax + OpenAI 兼容 API + 5种结构化提示词
6. 实现 session_timeout 逻辑 - 会话空闲超时自动保存
7. 前端新增 ContentBrowser 页面（事件浏览 + 筛选 + 分页）
8. 前端新增 24h 热力图组件
9. 前端增强 Settings - API提供商选择 + 数据导出 + 数据清理
10. 前端增强 Dashboard - WPM/退格率/CPM 统计卡片
11. 前端增强 ReportCenter - 周报 + 按日期分组 + 折叠展开
12. 修复所有编译器警告
13. 增强 db.rs query_events - 支持日期/搜索/分页筛选
14. 增强 TodoList - 逾期检测 + 截止日期显示

### 第二轮（UI 增强）
15. 增强 TimeStats - 日期选择器 + 饼图 + 应用切换统计 + 今日vs昨日对比

## 多 Agent 工作模式
- Hermes 主控：协调任务、处理前端、集成后端
- 子 Agent A：创建 scheduler.rs + export.rs
- 子 Agent B：增强 stats.rs + todo.rs
- 子 Agent C：增强 TimeStats.tsx
