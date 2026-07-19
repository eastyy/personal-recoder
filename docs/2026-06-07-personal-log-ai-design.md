# Personal Log AI — 需求分析与设计文档

> **版本**：v1.0  
> **日期**：2026-06-07  
> **阶段**：需求分析 / 设计  
> **作者**：Personal Log AI 项目组  
> **状态**：待用户审阅

---

## 0. 文档说明

本文档为 **Personal Log AI**（个人电脑输入记录与 AI 分析工具）的完整需求规格与设计方案。  
工具定位：**macOS 平台、个人自用、本地优先采集、云端 AI 分析、Web 仪表盘呈现**的桌面工具。

**核心价值**：让用户通过 AI 看到自己"每天在电脑上都做了什么、想了什么、写了什么"，从而实现自我复盘、效率提升、写作优化、习惯追踪。

**重要前提**：工具记录的是**用户自己的输入**，仅在本机使用；本文档不就合规性问题做企业级背书，读者需自行评估。

---

## 1. 项目概述

### 1.1 项目目标

构建一个 macOS 桌面工具，持续记录用户在电脑上的所有输入行为（键盘、鼠标、剪贴板、应用切换、网页浏览），通过云端 AI 进行多维度深度分析，生成个人化的洞察报告。

### 1.2 核心价值主张

| 价值 | 说明 |
|------|------|
| **自我觉察** | 看到自己每天时间的真实去向 |
| **效率提升** | 识别时间黑洞、碎片化、拖延模式 |
| **写作进化** | 长期追踪表达质量、错别字、口头禅 |
| **知识沉淀** | 自动从碎片输入中提取主题、灵感、TODO |
| **健康守护** | 久坐提醒、深夜告警、专注度建议 |
| **个人复盘** | 日报 / 周报自动生成，省去手动记录 |

### 1.3 范围与边界

**范围内**（In Scope）：
- macOS 12.0+ 系统
- 单机本地部署
- 云端 LLM API（用户自行提供 Key）
- Web 仪表盘（嵌入 Tauri WebView）

**范围外**（Out of Scope）：
- Windows / Linux 平台（v1 暂不支持）
- 移动端 App
- 多用户、多设备同步
- 企业级管理后台
- 屏幕截图 / 摄像头 / 麦克风（v1 暂不实现）
- 本地 LLM 推理（v1 仅云端）

### 1.4 名词约定

| 术语 | 含义 |
|------|------|
| **输入事件** | 一次按键、一次鼠标点击、一次剪贴板变化 |
| **输入会话** | 连续输入聚合的"一段思考"（默认 60s 无输入则分段） |
| **焦点会话** | 当前活跃应用/网站的持续时段（用于时长统计） |
| **分析任务** | 调用 LLM 的一次批处理（如"今日主题提取"） |
| **报告** | 一次分析任务的输出（文本 + 结构化数据） |

---

## 2. 用户场景与使用流程

### 2.1 目标用户画像

- 25-50 岁的知识工作者（程序员、研究者、作家、产品经理、咨询师等）
- macOS 重度用户，每天电脑使用 6h+
- 重视自我提升与时间管理
- 对 AI 工具有基本认知，愿意付费 API
- 对隐私敏感但接受"记录自己的数据"

### 2.2 典型使用场景

#### 场景 1：每日工作复盘
> 用户晚上 9 点下班，打开仪表盘查看"今日概览"：看到自己 VSCode 写了 2h13m，邮件处理了 35min，知乎/掘金刷了 1h12m。AI 生成的"今日洞察"建议：减少刷信息流时间，明早安排 30min 处理邮件。

#### 场景 2：周报自动生成
> 每周日晚，工具自动汇总本周 7 天的日报，生成"本周复盘报告"：项目进展、产出统计、习惯养成情况、下周建议。

#### 场景 3：写作优化
> 用户写了一封重要邮件，工具的"写作分析"提示："这句话过长，可拆为两句"，并给出改写建议。

#### 场景 4：灵感捕获
> 用户在地铁上用手机备忘录快速记下几个灵感（虽然是手机，但打开 Mac 时会同步——v2 考虑），或日常在 Mac 上的短输入被自动高亮为"闪念"。

#### 场景 5：目标追踪
> 用户设定目标"本月写 5 万字"，仪表盘自动追踪并显示："已完成 32,450 / 50,000 字（65%），按当前速度预计 6/24 完成。"

### 2.3 首次使用流程

```
下载 .app → 拖入 Applications → 启动
    ↓
1. 引导向导：欢迎 + 隐私说明（动画展示数据流向）
    ↓
2. 申请权限：辅助功能（必需）→ 用户手动开启
    ↓
3. 基础设置：选择 AI 服务商、填入 API Key、测试连接
    ↓
4. 配置黑名单：建议添加密码管理器、金融 App
    ↓
5. 进入"实时监控"页 → 工具开始工作
    ↓
6. 第二天 03:00 第一次自动日报生成 → 通知用户查看
```

### 2.4 日常使用流程

```
后台常驻 → 实时记录
    ↓
[可选] 通过 ⌘⇧P 调出托盘菜单
    ↓
通过托盘点击"打开仪表盘" → 唤起 WebView
    ↓
浏览 4 个页面：总览 / 内容浏览器 / 报告中心 / 设置
    ↓
[可选] ⌘⇧R 触发"立即生成报告"
    ↓
[可选] 暂停记录 N 分钟（开会/演示时）
```

---

## 3. 功能需求

### 3.1 输入采集模块

#### 3.1.1 采集能力清单

| 类型 | 实现方式 | 默认 | 优先级 |
|------|---------|------|-------|
| **键盘按键** | `CGEventTap` 监听 keydown/keyup | 开启 | P0 |
| **鼠标点击** | `CGEventTap` 监听 mouse click | 开启 | P0 |
| **鼠标活动** | rdev 监听 mouse move（用于判断活跃度） | 开启 | P1 |
| **剪贴板** | `NSPasteboard.general` changeCount 轮询 | 开启（已加强过滤） | P0 |
| **应用切换** | `NSWorkspace.didActivateApplication` | 开启 | P0 |
| **窗口标题** | `NSWorkspace.frontmostApplication` 配合 `CGWindowListCopyWindowInfo` | 开启 | P0 |
| **浏览器 URL** | AppleScript 调 Safari/Chrome | 开启 | P0 |
| **屏幕 OCR** | 周期截图 + Tesseract | 关闭 | P3 |
| **音频** | — | **不实现** | — |

#### 3.1.2 采集字段

```rust
struct RawEvent {
    id: u64,
    timestamp_ms: i64,           // Unix 毫秒
    event_type: EventType,        // KeyDown | KeyUp | MouseClick | MouseMove | Clipboard | AppFocus | WindowChange
    app_bundle_id: Option<String>,  // "com.microsoft.VSCode"
    app_name: Option<String>,       // "Visual Studio Code"
    window_title: Option<String>,   // "main.rs - personal_log"
    content: Option<String>,        // 输入文本 / 剪贴板内容（可空）
    session_id: Option<String>,     // 关联到 InputSession
    is_sensitive: SensitiveLevel,   // Normal | LocalOnly | Discarded
    metadata: serde_json::Value,    // 鼠标坐标、按键码、剪贴板来源 App 等
}

enum SensitiveLevel {
    Normal,      // 正常记录，正常上传
    LocalOnly,   // 仅本地存储，不上传
    Discarded,   // 完全丢弃
}
```

#### 3.1.3 敏感信息过滤规则

**A. 应用级黑名单（默认内置 + 用户可扩展）**

| 类别 | 默认黑名单 |
|------|-----------|
| 密码管理器 | 1Password、Bitwarden、KeePassXC、LastPass、Enpass、Strongbox |
| 金融 | 国内主要银行 App、各类证券 App |
| 系统敏感 | System Preferences / System Settings 密码面板 |
| 医疗 | —（用户可自行添加） |

**B. 内容级敏感检测（采集端）**

| 规则 | 检测方式 | 默认行为 |
|------|---------|---------|
| 密码字段 | `isSecureTextField == true` | 直接丢弃 |
| JWT / Token | 正则 `eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+` | 丢弃 |
| 信用卡号 | Luhn 算法 + 16 位数字模式 | 丢弃 |
| 身份证号 | 中国 18 位、港澳台通行证 | LocalOnly |
| 手机号 | 中国 11 位手机号模式 | LocalOnly |
| 邮箱 | 标准邮箱正则 | Normal（用户可改） |
| 银行卡 CVV | 3-4 位数字上下文 | 丢弃 |
| AWS Key | `AKIA[0-9A-Z]{16}` | 丢弃 |
| 私钥 | `-----BEGIN [A-Z]+ PRIVATE KEY-----` | 丢弃 |
| 长度 > 1000 | 超过 1000 字符 | 丢弃（可能误粘大文件） |
| 重复 30s 内 | 同内容短时间重复 | 去重 |

**C. 字段级加密（v1 不实现，但保留扩展点）**

未来可选地对 `content` 字段做加密存储（用户主动开启），v1 暂不加密。

#### 3.1.4 性能预算

| 指标 | 目标 |
|------|------|
| CPU 占用 | 空闲 < 1%，采集活跃时 < 3% |
| 内存占用 | 主进程 < 80MB，WebView 独立计算 |
| 磁盘 IO | 每 5 秒批量写入一次，非高频 |
| 启动时间 | 冷启动 < 2s |

### 3.2 应用/网站时长统计模块

#### 3.2.1 核心指标

| 指标 | 说明 | 展示形式 |
|------|------|---------|
| **总电脑使用时长** | 屏幕解锁 → 锁屏的总活跃时间 | 大数字卡片 |
| **各应用累计时长** | 按 `app_bundle_id` 聚合 | Top 10 条形图 |
| **各网站累计时长** | 按域名聚合（仅 Safari/Chrome） | Top 10 条形图 |
| **应用时长占比** | 各应用占总时长的百分比 | 环形图 |
| **24h 分布** | 每小时的应用使用热力图 | 热力图 |
| **今日 vs 昨日 vs 本周均值** | 对比 | 折线图 / 数值 |
| **应用切换次数** | 单位时间内切换应用的频率 | 折线图 |
| **单次使用时长** | 每次打开某应用的持续时间分布 | 直方图 |
| **锁屏不计** | 锁屏后停止统计 | 自动 |
| **空闲判定** | 60s 无键盘鼠标 → 视为空闲，停止累计 | 自动 |

#### 3.2.2 网站识别实现

**支持的浏览器**（v1）：

| 浏览器 | URL 获取方式 |
|--------|-------------|
| Safari | `osascript -e 'tell application "Safari" to get URL of front document'` |
| Google Chrome | Chrome DevTools Protocol (CDP) — `--remote-debugging-port=9222` |
| 其他浏览器 | 退化为"浏览器名"统一分类（如"Brave - 知乎"可能归为 brave.com） |

**URL 解析**：
- 完整 URL → 提取 `host`（如 `https://www.zhihu.com/question/123` → `zhihu.com`）
- 去掉 `www.` 前缀
- 聚合为"域名"统计

#### 3.2.3 焦点会话算法

```rust
// 每 5 秒执行一次
async fn track_focus() {
    let mut current: Option<FocusSession> = None;
    
    loop {
        sleep(5s).await;
        
        let active = get_active_target().await;  // {app, url}
        let now = now();
        
        match &mut current {
            Some(s) if s.target == active => {
                // 同一目标，累加时长
                s.duration += 5;
                s.last_seen = now;
            }
            Some(s) => {
                // 切换：保存旧会话，开启新会话
                if now - s.last_seen > 60s {
                    // 超过 60s 不算连续（如锁屏）
                } else {
                    save_focus_session(s);
                }
                current = Some(FocusSession::new(active, now));
            }
            None => {
                current = Some(FocusSession::new(active, now));
            }
        }
    }
}
```

#### 3.2.4 时长聚合存储

```sql
-- 焦点会话原始记录
CREATE TABLE focus_sessions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time      INTEGER NOT NULL,
    end_time        INTEGER NOT NULL,
    duration_sec    INTEGER NOT NULL,
    target_type     TEXT NOT NULL,           -- 'app' | 'domain'
    target_id       TEXT NOT NULL,           -- 'com.microsoft.VSCode' | 'github.com'
    target_name     TEXT NOT NULL
);

-- 按日聚合（用于仪表盘快速查询）
CREATE TABLE focus_daily (
    date            TEXT NOT NULL,           -- '2026-06-07'
    target_type     TEXT NOT NULL,
    target_id       TEXT NOT NULL,
    target_name     TEXT NOT NULL,
    duration_sec    INTEGER NOT NULL,
    session_count   INTEGER DEFAULT 0,
    PRIMARY KEY (date, target_type, target_id)
);

-- 按小时聚合（用于热力图）
CREATE TABLE focus_hourly (
    date            TEXT NOT NULL,
    hour            INTEGER NOT NULL,        -- 0-23
    target_type     TEXT NOT NULL,
    target_id       TEXT NOT NULL,
    duration_sec    INTEGER NOT NULL,
    PRIMARY KEY (date, hour, target_type, target_id)
);
```

### 3.3 数据预处理与会话合并

#### 3.3.1 输入会话合并算法

零散按键事件**不直接入库**，先在内存中合并为"输入会话"：

```
状态机：
  IDLE → (键盘事件) → ACTIVE
  ACTIVE → (停顿 > 60s) → IDLE（保存当前会话）
  ACTIVE → (停顿 > 3s) → 记录 pause_count++
  ACTIVE → (字符累积) → buffer.push(char)
  
阈值（可配置）：
  pause_threshold: 3 秒
  session_timeout: 60 秒
  min_session_length: 2 字符
```

**为什么合并**：
- 减少数据库写入（每秒多次 → 每分钟几次）
- 保留思考节奏（停顿次数 = 思考密度指标）
- 减少 AI 分析噪声

#### 3.3.2 应用上下文标记

每个输入会话自动打上"应用类别"标签：

| 应用类型 | 标记 | 说明 |
|---------|------|------|
| 编码类 | `code` | VSCode、Cursor、IntelliJ、Xcode |
| 邮件 | `email` | Mail、Outlook、Spark |
| 聊天 | `chat` | 微信、Slack、Discord、飞书 |
| 文档 | `document` | Pages、Word、Notion、Obsidian |
| 浏览器 | `browse` | Safari、Chrome |
| 终端 | `terminal` | Terminal、iTerm2 |
| 设计 | `design` | Figma、Sketch |
| 其他 | `other` | — |

### 3.4 AI 分析任务

#### 3.4.1 任务调度器

```rust
// tokio-cron-scheduler 调度
struct AnalysisScheduler {
    daily_routine: Cron::Daily(03:00),    // 每日 03:00 跑日报
    weekly_routine: Cron::Weekly(Sun 03:30), // 每周日 03:30 跑周报
    hourly_incremental: Cron::Hourly,     // 每小时跑小型分析（TODO 提取等）
}
```

#### 3.4.2 分析任务清单（v1）

| 任务 | 触发时机 | 输入数据 | 输出 | Prompt 模板 |
|------|---------|---------|------|------------|
| **TODO 提取** ⭐ | 每小时 + 实时（短输入触发） | 最近 1h 输入文本 | TODO 列表 | `TODO_EXTRACT_PROMPT` |
| **每日主题提取** | 每日 03:00 | 昨日所有会话 | 3-5 个核心话题 | `TOPIC_PROMPT` |
| **每日生产力分析** | 每日 03:00 | 昨日 focus_daily | 时间分配评估 + 3 条建议 | `PRODUCTIVITY_PROMPT` |
| **每日写作优化** | 每日 03:00 | 昨日写作类应用会话 | 错别字/语气/可优化点 | `WRITING_PROMPT` |
| **周报综合分析** | 每周日 03:30 | 本周 7 天日报 | 一页纸周报 | `WEEKLY_PROMPT` |
| **闪念捕获** | 实时 | 单个短输入 | 高亮标记 | 本地规则（无 AI） |
| **个人词云** | 每日 03:00 | 昨日全文本 | 高频词统计 | 本地规则（无 AI） |

#### 3.4.3 TODO 自动提取（MVP 重点）

**识别模式**（本地规则先匹配，AI 兜底）：

```rust
fn is_todo_candidate(text: &str) -> Option<TodoItem> {
    // 1. 显式标记
    if has_prefix(text, &["TODO:", "todo:", "待办:", "记得", "记得要", "别忘了", "明天要", "今天要"]) {
        return Some(extract(text));
    }
    
    // 2. 祈使句模式
    if is_imperative_sentence(text) {
        return Some(extract(text));
    }
    
    // 3. 短而完整（< 50 字，无标点结尾）
    if text.len() < 50 && !text.ends_with(['。', '!', '?', '!', '?']) {
        return Some(extract_with_context(text));
    }
    
    None
}
```

**存储与展示**：

```sql
CREATE TABLE todo_items (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    text            TEXT NOT NULL,
    source_session  TEXT,                    -- 关联到 input_session
    extracted_at    INTEGER NOT NULL,
    due_date        TEXT,                    -- 解析出的"明天"/"下周一"等
    status          TEXT DEFAULT 'pending',  -- 'pending' | 'done' | 'cancelled'
    completed_at    INTEGER
);
```

**仪表盘展示**：侧边栏常驻"今日 TODO"列表，可勾选完成。

### 3.5 实时监控

**实时监控页**始终展示当前正在记录的状态，**完全透明**：

```
┌─ 实时记录监控 ─────────────────────┐
│  🟢 正在记录中                    │
│  今日已记录：1,247 条事件          │
│  ⏱ 当前会话：4m 12s              │
│  ⌨ 当前输入：234 字              │
│  📍 当前应用：VS Code            │
│                                    │
│  14:23:15  VS Code   keydown  ✓  │
│  14:23:18  VS Code   keydown  ✓  │
│  14:23:22  1Password keydown  ✗ 黑名单 │
│  14:23:25  VS Code   keydown  ✓  │
│  14:23:31  VS Code   clipboard ✓  │
│                                    │
│  [⏸ 暂停 5m]  [📋 复制最近 100 条] │
└────────────────────────────────────┘
```

### 3.6 报告中心

**报告存储与展示**：

```
┌─ 报告中心 ─────────────────────────────┐
│  📅 2026-06-07  📊 日报               │
│  ├─ TODO 提取     [5 条新]            │
│  ├─ 主题提取      ✓                    │
│  ├─ 生产力分析    ✓                    │
│  └─ 写作优化      ✓                    │
│                                       │
│  📅 2026-06-06  📊 日报              │
│  ...                                  │
│                                       │
│  📅 2026-05-31~06-06  📈 周报         │
│  ...                                  │
│                                       │
│  [+ 手动触发] [📥 导出所有]            │
└───────────────────────────────────────┘
```

**报告详情页**包含：
- 报告全文（Markdown 渲染）
- 关联的原始数据链接（点击跳转到内容浏览器）
- "再分析一次"按钮
- 反馈按钮（"这条建议不错"/"不准确"）—— 用于未来调优

---

## 4. 分析维度全列表

> 你已经确认：**全部 10 大类都加入设计文档**。下表是完整的需求规格。

### 4.1 打字行为分析（基于按键原始事件）

| 维度 | 指标 | 难度 | 优先级 | MVP |
|------|------|------|-------|-----|
| 打字速度 | WPM、CPM、中英文分别 | 低 | P1 | ✓ |
| 击键节奏 | 击键间隔均值/方差 | 低 | P1 | ✓ |
| **退格频率** | 每 100 字退格次数 | 低 | P0 | ✓ |
| 错误模式 | 高频误触键 | 中 | P2 | |
| 打字疲劳度 | 速度/错误率时间曲线 | 中 | P1 | |
| 停顿模式 | 短停顿（思考）vs 长停顿（分心） | 低 | P0 | ✓ |
| 修改密度 | 大段重写 vs 增量修改占比 | 中 | P1 | |
| 速度突变 | 突然加速/减速 → 情绪波动？ | 中 | P2 | |
| 左右手平衡 | 单手过度使用检测 | 中 | P3 | |

### 4.2 时间与生产力分析

| 维度 | 指标 | 难度 | 优先级 | MVP |
|------|------|------|-------|-----|
| **有效输入时间** | 去除停顿后的"净输入时长" | 低 | P0 | ✓ |
| **深度工作时段** | 最长连续专注时段 | 低 | P0 | ✓ |
| **碎片化指数** | 应用切换频率 | 低 | P0 | ✓ |
| 番茄钟自动识别 | 25min 集中 + 5min 切换模式 | 中 | P1 | |
| **时间黑洞检测** | 单次使用某应用超 X 分钟 | 低 | P0 | ✓ |
| 最佳工作时段 | 历史最专注时段 | 中 | P1 | |
| 周/月对比 | 同比环比 | 中 | P1 | ✓ |
| 黄金时间利用率 | "最清醒时段"实际产出 | 中 | P1 | |

### 4.3 思维与认知分析

| 维度 | 指标 | 难度 | 优先级 | MVP |
|------|------|------|-------|-----|
| 思考密度 | 停顿长度分布 → 浅层 vs 深度思考 | 低 | P1 | ✓ |
| 思路连贯性 | 同会话内文本相似度 / 重写频率 | 中 | P1 | |
| 决策痕迹 | 反复对比、列表式输入 | 中 | P2 | |
| 认知负荷日 | 修改密度 + 停顿 + 退格 综合 | 中 | P1 | |
| 信息过载预警 | 短时间大量切换浏览器/搜索 | 中 | P2 | |
| 创造力指标 | 笔记/草稿 vs 复制粘贴为主 | 中 | P2 | |

### 4.4 写作与表达分析

| 维度 | 指标 | 难度 | 优先级 | MVP |
|------|------|------|-------|-----|
| 句长分布 | 平均句长、过长句 | 低 | P1 | ✓ |
| 词汇丰富度 | Type-Token Ratio | 中 | P1 | |
| 中英文混用比 | 输入语言分布 | 低 | P1 | ✓ |
| 标点规范度 | 中英文标点混用 | 中 | P1 | |
| 重复表达检测 | 口头禅分析 | 中 | P2 | |
| 语气分析 | 正式度评分（需 AI） | 高 | P1 | |
| 逻辑连接词 | "因此/但是/首先"频率 | 中 | P2 | |
| **个人词云** | 高频用词可视化 | 中 | P1 | ✓ |
| **错别字高频字** | 持续追踪 | 中 | P1 | ✓ |
| AI 改写建议 | 长难句改写（需 AI） | 高 | P1 | |
| **表达进化** | 3 个月邮件风格变化 | 中 | P2 | |

### 4.5 习惯与模式分析

| 维度 | 指标 | 难度 | 优先级 | MVP |
|------|------|------|-------|-----|
| **作息节律** | 首次/最后输入时间 | 低 | P0 | ✓ |
| **深夜活跃指数** | 23:00 后输入量占比 | 低 | P0 | ✓ |
| 周末 vs 工作日 | 模式差异 | 低 | P1 | ✓ |
| 拖延信号 | 临近截止时间输入激增 | 中 | P2 | |
| 拖延应用识别 | 高频但低产出的应用 | 中 | P1 | |
| **微习惯追踪** | 自定义目标达成度 | 中 | P1 | ✓ |
| 应用过渡路径 | 常见工作流 | 中 | P1 | |

### 4.6 健康与福祉分析

| 维度 | 指标 | 难度 | 优先级 | MVP |
|------|------|------|-------|-----|
| **久坐提醒** | 60min 无窗口切换 | 低 | P0 | ✓ |
| **连续输入提醒** | 每 30min 休息 | 低 | P0 | ✓ |
| **深夜工作告警** | 24:00 后仍活跃 | 低 | P0 | ✓ |
| 周末加班比例 | 周末有工作类应用 | 中 | P1 | |
| 键盘冲击强度 | Shift/Caps 异常 | 中 | P2 | |
| 连续工作极限 | 历史可持续专注时长 | 中 | P2 | |
| 屏幕时间平衡 | 工作/学习/娱乐 比例 | 中 | P1 | ✓ |
| 眼/手腕保护 | 高频连续输入提醒 | 低 | P1 | |

### 4.7 学习与知识管理

| 维度 | 指标 | 难度 | 优先级 | MVP |
|------|------|------|-------|-----|
| 学习时段识别 | Coursera/YouTube/Anki | 低 | P1 | |
| 重复搜索 | 同关键词多次 → 知识盲点 | 中 | P1 | |
| 跨日主题演化 | "上周想 X，本周演化到 Y" | 中 | P1 | |
| 个人知识图谱 | 自动聚类主题 | 高 | P2 | |
| **闪念捕获** | 短而快的输入 → 高亮 | 低 | P1 | ✓ |
| **TODO 自动提取** | 识别"明天要..."类 | 中 | P1 | ✓ |
| **日记自动生成** | 汇总每日 → 第一人称 | 中 | P1 | |
| 信息源画像 | 最常查资料的网站 | 中 | P2 | |

### 4.8 协作与社交

| 维度 | 指标 | 难度 | 优先级 | MVP |
|------|------|------|-------|-----|
| 响应速度 | 消息 → 回复的间隔 | 中 | P1 | |
| 协作对象频次 | 高频联系人 | 中 | P1 | |
| 会议 vs 异步 | 短消息集中爆发 | 中 | P2 | |
| 沟通占比 | 沟通类应用占比 | 中 | P1 | |
| 回复质量趋势 | 邮件长度变化 | 中 | P2 | |

### 4.9 个人目标追踪（需用户主动设定）

| 维度 | 指标 | 难度 | 优先级 | MVP |
|------|------|------|-------|-----|
| **目标进度** | "本月写 5 万字" | 中 | P1 | ✓ |
| 习惯打卡 | 自动检测目标相关应用 | 中 | P2 | |
| 里程碑识别 | "恭喜！本周深度工作 25h" | 中 | P1 | |
| **周/月复盘报告** | 一页纸可读复盘 | 高 | P0 | ✓ |

### 4.10 创意 / 进阶分析

| 维度 | 指标 | 难度 | 优先级 | MVP |
|------|------|------|-------|-----|
| AI 自动日记 | 每日输入 → 第一人称 | 中 | P2 | |
| 个人传记素材 | 累积 → 半年轨迹 | 高 | P3 | |
| 决策日志 | 识别"我决定..." | 中 | P3 | |
| 情绪曲线 | 一天中积极/消极词 | 高 | P2 | |
| AI 提问助手 | 主动提问"需要帮你理清吗？" | 高 | P2 | |

---

## 5. 非功能需求

| 类别 | 需求 |
|------|------|
| **性能** | 后台 CPU < 3%、内存 < 80MB、启动 < 2s、报告生成不阻塞 UI |
| **可用性** | 启动 / 关闭 / 暂停 / 恢复 路径全部可发现；权限丢失时自动降级（仅应用/网站时长） |
| **可靠性** | 崩溃后数据不丢失（事务写入）、断电保护、SQLite 定期备份 |
| **可维护性** | Rust 模块化、TypeScript 组件化、AI Prompt 模板独立文件 |
| **可扩展性** | 浏览器支持、LLM 服务商、分析维度 都设计为可插拔 |
| **本地化** | UI 中文（默认）、可切换英文 |
| **兼容性** | macOS 12.0+（Monterey 起），适配 Intel / Apple Silicon |

---

## 6. 技术架构

### 6.1 整体技术栈

| 层 | 技术 | 用途 |
|---|------|------|
| **应用框架** | Tauri 2.0 | 单二进制集成 Rust 后端 + WebView |
| **核心语言** | Rust 2021 | 后台采集、AI 调度、IPC |
| **异步运行时** | tokio | I/O、定时任务、API 调用 |
| **输入采集** | rdev + core-graphics-rs | 键盘/鼠标监听 + macOS 系统 API |
| **数据存储** | rusqlite | SQLite（v1 不加密） |
| **AI 通信** | reqwest + serde_json | 调用云端 LLM |
| **定时任务** | tokio-cron-scheduler | 每日/每周报告调度 |
| **WebView 前端** | React 18 + Vite | 仪表盘 UI |
| **状态管理** | Zustand | 前端轻量状态 |
| **图表** | ECharts | 仪表盘图表 |
| **样式** | TailwindCSS | 样式系统 |
| **路由** | React Router v6 | 前端路由 |

### 6.2 系统架构图

```
┌──────────────────────────────────────────────────────────┐
│                  Tauri 2.0 主进程 (Rust)                  │
│                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │ 采集模块     │  │ 焦点会话     │  │ AI 调度模块  │   │
│  │ input_       │→ │ focus_       │→ │ ai_          │   │
│  │ capture      │  │ tracker      │  │ orchestrator │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
│         ↓                  ↓                ↓            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │ 预处理       │  │ 存储模块     │  │ TODO 提取    │   │
│  │ preprocessor │→ │ storage_     │  │ todo_        │   │
│  │ (会话合并)   │  │ manager      │  │ extractor    │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
│                            ↓                             │
│  ┌──────────────────────────────────────────────────┐   │
│  │           IPC Command Bridge (Tauri)             │   │
│  └──────────────────────────────────────────────────┘   │
└────────────────────────┬─────────────────────────────────┘
                         │ IPC (invoke / emit)
┌────────────────────────┴─────────────────────────────────┐
│                  WebView 前端 (React)                    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│  │ 总览仪表盘 │  │ 内容浏览器│  │ 报告中心  │  │ 设置     │ │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘ │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │ 时长统计 │  │ 实时监控 │  │ TODO 列表│              │
│  └──────────┘  └──────────┘  └──────────┘              │
└──────────────────────────────────────────────────────────┘
                              ↕ HTTPS
┌──────────────────────────────────────────────────────────┐
│              云端 AI 服务（用户配置）                      │
│   OpenAI GPT-4o / Claude 3.5 / DeepSeek / 自定义        │
└──────────────────────────────────────────────────────────┘
```

### 6.3 进程模型

| 进程 | 职责 |
|------|------|
| **Tauri 主进程（Rust）** | 输入采集、焦点会话、数据预处理、AI 调度、IPC 桥接 |
| **WebView 子进程** | 仅渲染仪表盘，独立沙箱 |
| **LaunchAgent**（可选） | macOS 启动时自动加载 |

### 6.4 部署形态

- 单个 `.app` 文件
- 数据存储：`~/Library/Application Support/PersonalLogAI/`
  - `data.db`：SQLite 主数据库
  - `cache/`：临时文件
  - `logs/`：运行日志
  - `config.toml`：本地配置
- API Key 存储：macOS **Keychain**（系统级）
- LaunchAgent plist：`~/Library/LaunchAgents/com.personallogai.agent.plist`（可选）

---

## 7. 数据模型

### 7.1 表结构总览

```sql
-- ============ 原始事件层 ============

CREATE TABLE raw_events (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp       INTEGER NOT NULL,
    event_type      TEXT NOT NULL,              -- keydown|mouse_click|mouse_move|clipboard|app_focus|window_change
    app_bundle_id   TEXT,
    app_name        TEXT,
    window_title    TEXT,
    content         TEXT,                       -- 输入文本 / 剪贴板内容
    is_sensitive    INTEGER DEFAULT 0,          -- 0=Normal 1=LocalOnly 2=Discarded
    session_id      TEXT,
    metadata        TEXT                        -- JSON
);

CREATE INDEX idx_raw_events_ts ON raw_events(timestamp);
CREATE INDEX idx_raw_events_app_ts ON raw_events(app_bundle_id, timestamp);
CREATE INDEX idx_raw_events_session ON raw_events(session_id);

-- ============ 输入会话层 ============

CREATE TABLE input_sessions (
    id              TEXT PRIMARY KEY,
    app_bundle_id   TEXT NOT NULL,
    app_name        TEXT,
    start_time      INTEGER NOT NULL,
    end_time        INTEGER,
    char_count      INTEGER DEFAULT 0,
    text_preview    TEXT,                       -- 前 200 字符
    pause_count     INTEGER DEFAULT 0,
    context_tag     TEXT,                       -- code/email/chat/document/browse/terminal/design
    ai_analyzed     INTEGER DEFAULT 0
);

-- ============ 焦点会话层（时长统计） ============

CREATE TABLE focus_sessions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time      INTEGER NOT NULL,
    end_time        INTEGER NOT NULL,
    duration_sec    INTEGER NOT NULL,
    target_type     TEXT NOT NULL,              -- 'app' | 'domain'
    target_id       TEXT NOT NULL,
    target_name     TEXT NOT NULL
);

CREATE TABLE focus_daily (
    date            TEXT NOT NULL,
    target_type     TEXT NOT NULL,
    target_id       TEXT NOT NULL,
    target_name     TEXT NOT NULL,
    duration_sec    INTEGER NOT NULL,
    session_count   INTEGER DEFAULT 0,
    PRIMARY KEY (date, target_type, target_id)
);

CREATE TABLE focus_hourly (
    date            TEXT NOT NULL,
    hour            INTEGER NOT NULL,
    target_type     TEXT NOT NULL,
    target_id       TEXT NOT NULL,
    duration_sec    INTEGER NOT NULL,
    PRIMARY KEY (date, hour, target_type, target_id)
);

-- ============ AI 分析结果层 ============

CREATE TABLE analysis_results (
    id                TEXT PRIMARY KEY,
    analysis_type     TEXT NOT NULL,            -- 'productivity'|'topic'|'writing'|'weekly'|'todo'
    time_range_start  INTEGER,
    time_range_end    INTEGER,
    prompt_tokens     INTEGER,
    completion_tokens INTEGER,
    result_text       TEXT,
    result_json       TEXT,
    created_at        INTEGER NOT NULL
);

-- ============ TODO 项 ============

CREATE TABLE todo_items (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    text            TEXT NOT NULL,
    source_session  TEXT,
    extracted_at    INTEGER NOT NULL,
    due_date        TEXT,
    status          TEXT DEFAULT 'pending',
    completed_at    INTEGER
);

-- ============ 闪念 ============

CREATE TABLE flash_ideas (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    text            TEXT NOT NULL,
    source_session  TEXT,
    captured_at     INTEGER NOT NULL
);

-- ============ 配置 & 规则 ============

CREATE TABLE app_rules (
    bundle_id       TEXT PRIMARY KEY,
    rule_type       TEXT NOT NULL,              -- 'blacklist'|'whitelist'|'sensitive'
    enabled         INTEGER DEFAULT 1,
    note            TEXT
);

CREATE TABLE user_goals (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    title           TEXT NOT NULL,
    metric_type     TEXT NOT NULL,              -- 'word_count'|'app_duration'|'focus_hours'
    target_value    INTEGER NOT NULL,
    current_value   INTEGER DEFAULT 0,
    period          TEXT NOT NULL,              -- 'daily'|'weekly'|'monthly'
    start_date      TEXT,
    end_date        TEXT,
    created_at      INTEGER NOT NULL
);

CREATE TABLE app_config (
    key             TEXT PRIMARY KEY,
    value           TEXT,
    updated_at      INTEGER
);
```

### 7.2 索引设计原则

- 高频查询路径（按时间范围 + 应用）必须走索引
- 每日聚合数据与原始数据分离，仪表盘查询走聚合表
- 定时任务按 `created_at` 索引清理过期数据

---

## 8. AI Prompt 模板

### 8.1 TODO 提取 Prompt

```markdown
你是一个 TODO 提取助手。请从以下用户输入中识别待办事项。

【用户输入】
{input_text}

【输出要求】
- 仅输出明确的待办事项
- 提取动作、对象、时间约束
- 格式：每行一个 TODO，格式为 "时间 | 动作 | 对象"
- 不要输出任何解释
- 如果没有 TODO，输出 "无"

【示例】
明天要给张总发邮件确认项目排期
记得下午 3 点开会
周五前完成需求文档

【输出】
明天 | 发邮件确认项目排期 | 张总
2026-06-07 15:00 | 开会 | —
2026-06-12 前 | 完成 | 需求文档
```

### 8.2 主题提取 Prompt

```markdown
你是一个知识管理专家。请分析以下用户过去 24 小时的输入内容，识别核心话题。

【应用分类摘要】
{app_grouped_summary}

【要求】
- 提取 3-5 个核心话题
- 为每个话题列出关联的具体内容片段（不超过 5 个）
- 识别用户在思考的"主线"和"分支"
- 推荐 1-2 个相关学习资源方向
- 不超过 500 字
- 客观、不臆测
- 语言：中文
```

### 8.3 生产力分析 Prompt

```markdown
你是一个时间管理分析师。请分析以下用户过去 24 小时在 macOS 上的输入数据。

【应用使用时长】
{app_usage_data}

【网站访问时长】
{website_usage_data}

【输入会话统计】
{session_statistics}

【要求】
- 列出前 5 个最高频使用的应用
- 计算每个应用的有效输入时间（去除停顿时间）
- 评估专注度（基于会话长度和停顿模式）
- 识别"黄金时间"（产出最高的时间段）
- 给出 3 条具体的时间管理建议
- 客观、基于数据、避免主观评价
- 语言：中文
- 不超过 400 字
```

### 8.4 写作优化 Prompt

```markdown
你是一个写作教练。请分析以下用户在邮件/聊天/文档应用中的输入。

【写作类应用会话】
{writing_sessions}

【要求】
- 找出 3 个表达不够清晰或礼貌的句子
- 找出 2 个常见错别字或语法错误
- 评估整体语气（专业/随意/情绪化）
- 给出 3 条具体的改进建议
- 建设性、不批评人
- 语言：中文
- 不超过 400 字
```

### 8.5 周报综合 Prompt

```markdown
你是一个个人成长教练。请基于用户本周 7 天的日报数据，生成一份结构化周报。

【本周日报摘要】
{weekly_daily_summaries}

【本周时间统计】
{weekly_time_statistics}

【本周完成 TODO】
{completed_todos}

【要求】
- 三大板块：📊 数据回顾 / 🎯 关键产出 / 💡 改进建议
- 数据回顾：3-5 个关键数据点
- 关键产出：3-5 条本周主要做的事
- 改进建议：3 条具体可执行的建议
- 一页纸可读完
- 客观、有建设性
- 语言：中文
```

### 8.6 错误处理：AI 调用失败

```rust
async fn call_llm_with_retry(prompt: &str, max_retries: u32) -> Result<String> {
    let mut backoff_ms = 1000;
    for attempt in 0..max_retries {
        match call_llm_once(prompt).await {
            Ok(resp) => return Ok(resp),
            Err(e) if is_retryable(&e) && attempt < max_retries - 1 => {
                warn!("AI 调用失败，{}ms 后重试: {}", backoff_ms, e);
                sleep(backoff_ms).await;
                backoff_ms *= 2;
            }
            Err(e) => return Err(e),
        }
    }
    Err(anyhow!("AI 调用失败，已达最大重试次数"))
}
```

**降级策略**：
- LLM 不可用时，仅展示本地统计（不退化）
- TODO 提取降级为本地规则匹配
- 报告生成失败时记录到 `analysis_results` 表（status='failed'），用户可手动重试

---

## 9. 仪表盘 UI 设计

### 9.1 页面架构

| 路径 | 名称 | 主要内容 |
|------|------|---------|
| `/` | 总览 | 今日概览 + AI 报告摘要 + 活动热力图 |
| `/time` | 时长统计 | 应用/网站时长、Top 10、对比 |
| `/content` | 内容浏览器 | 按时间线查看原始输入段 |
| `/reports` | 报告中心 | 历史日报/周报列表 |
| `/todos` | TODO 列表 | 全部待办，可勾选完成 |
| `/flash` | 闪念库 | 捕获的灵感 |
| `/realtime` | 实时监控 | 当前正在记录的状态 |
| `/settings` | 设置 | 黑名单、AI、保留期、API Key |

### 9.2 总览页布局

```
┌────────────────────────────────────────────────────────────────┐
│  Personal Log AI                          ⚙ 设置 🔔 报告 ⏸ 暂停 │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  ┌─ 今日概览 ─────────────────────────────────────────────┐  │
│  │  ⌨️ 总输入  3,847 字   📱 主要应用 VS Code (42%)        │  │
│  │  ⏱  有效工作 4h 23m    🎯 最长专注 47m                  │  │
│  │  💡 今日 AI 洞察 已生成 →  [查看]                       │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                                │
│  ┌─ 24h 输入活动热力图 ─────┐  ┌─ 应用使用时长 Top 5 ─────┐  │
│  │ ░░▓▓▓██▓▓░░▓▓▓▓▓▓░░    │  │ VSCode  ████████  2h13m │  │
│  │                          │  │ Chrome  ██████    1h48m │  │
│  │                          │  │ 微信    ███       42m   │  │
│  │                          │  │ 邮件    ██        35m   │  │
│  └──────────────────────────┘  │ 终端    █         28m   │  │
│                                 └──────────────────────────┘  │
│                                                                │
│  ┌─ 今日 AI 报告 ────────────────────────────────────────┐  │
│  │  📊 生产力分析：                                       │  │
│  │  你今天的深度工作时间集中在上午 10-12 点...            │  │
│  │                                                        │  │
│  │  🧠 主题提取：                                         │  │
│  │  你今天主要在思考：(1) Rust 异步编程 (2) 项目架构...   │  │
│  │                                                        │  │
│  │  ✍️ 写作建议：                                         │  │
│  │  在邮件中发现 2 处可以更简洁的表达...                  │  │
│  │                                       [查看完整报告]   │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                                │
│  ┌─ 今日 TODO（5 条）──────────┐  ┌─ 闪念（3 条）────────┐  │
│  │ ☐ 给张总发排期确认邮件       │  │ 💡 "先做 MVP..."     │  │
│  │ ☐ 完成需求文档 v2            │  │ 💡 "Tauri 2.0 真香"  │  │
│  │ ☐ 下午 3 点开会              │  │ 💡 "周末读《人月》"   │  │
│  │ ☐ ...                        │  │                       │  │
│  └──────────────────────────────┘  └──────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

### 9.3 时长统计页

```
┌─ 今日使用时长 ─────────────────────────────────────────────┐
│                                                              │
│  💻 总使用时长                                               │
│  ┌────────────────────────┐                                  │
│  │   6h 47m / 12h 在线    │  ← 活跃/解锁时间                 │
│  └────────────────────────┘                                  │
│                                                              │
│  🏆 Top 10 应用                       Top 10 网站            │
│  ▓▓▓▓▓▓▓▓▓▓▓▓▓ VSCode   2h 13m │ ▓▓▓▓▓▓▓▓ github  1h 32m│
│  ▓▓▓▓▓▓▓▓▓▓  Chrome    1h 48m │ ▓▓▓▓▓▓▓▓ zhihu    43m   │
│  ▓▓▓▓▓▓▓      Safari    58m   │ ▓▓▓▓▓▓    jianshu  31m   │
│  ...                              ...                        │
│                                                              │
│  📊 24h 应用分布                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  00 02 04 06 08 10 12 14 16 18 20 22  (小时)        │   │
│  │  ░░░░░░░░░░▓▓▓▓▓▓▓▓▓▓██▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░         │   │
│  │            ↑   ↑   ↑                                  │   │
│  │       早高峰  午休  下午工作                            │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  📈 今日 vs 昨日 vs 本周均值                                  │
│  ─────────────────────────────────                          │
│  VSCode   2h13m  1h47m  2h05m   ▲ +24% vs 昨日              │
│  Chrome   1h48m  2h03m  1h52m   ▼ -12%                      │
└──────────────────────────────────────────────────────────────┘
```

### 9.4 实时监控页

```
┌─ 实时记录监控 ─────────────────────────────────────────────┐
│  🟢 正在记录中                    [⏸ 暂停 5m]  [📋 复制]  │
│  今日已记录：1,247 条事件                                   │
│  ⏱ 当前会话：4m 12s              ⌨ 当前输入：234 字         │
│  📍 当前应用：VS Code                                        │
│                                                              │
│  时间            应用         事件        状态               │
│  ──────────────────────────────────────────────────         │
│  14:23:15.234   VS Code      keydown    ✓ 已记录           │
│  14:23:18.102   VS Code      keydown    ✓ 已记录           │
│  14:23:22.456   1Password    keydown    ✗ 黑名单           │
│  14:23:25.789   VS Code      keydown    ✓ 已记录           │
│  14:23:31.001   VS Code      clipboard  ✓ 已记录           │
│  ...                                                         │
└──────────────────────────────────────────────────────────────┘
```

### 9.5 设置页

```
┌─ 设置 ─────────────────────────────────────────────────────┐
│  📡 采集设置                                                │
│  ├─ [✓] 键盘输入                  [✓] 鼠标活动              │
│  ├─ [✓] 剪贴板监听（已加强过滤）  [ ] 屏幕 OCR              │
│  └─ [✓] 应用切换记录              [✓] 浏览器 URL            │
│                                                             │
│  🚫 黑名单应用（当前 3 个）                  [+ 添加]       │
│  ├─ 1Password          [编辑] [删除]                        │
│  ├─ 工商银行            [编辑] [删除]                        │
│  └─ KeePassXC          [编辑] [删除]                        │
│                                                             │
│  🤖 AI 配置                                                  │
│  ├─ 服务商： [OpenAI ▾]                                     │
│  ├─ 模型：   [gpt-4o ▾]                                     │
│  ├─ API Key： [sk-...********] [测试连接]                   │
│  └─ [✓] 启用 AI 分析   [ ] 启用云端备份                    │
│                                                             │
│  🎯 我的目标（当前 2 个）                     [+ 添加]      │
│  ├─ 本月写 5 万字  32,450 / 50,000 (65%)                   │
│  └─ 每天深度工作 4h  3h45m / 4h (94%)                       │
│                                                             │
│  🗑 数据管理                                                │
│  ├─ 原始事件保留：[90 天 ▾]                                │
│  ├─ [立即清理过期数据]                                      │
│  ├─ [导出全部数据]  [导入备份]                              │
│  └─ ⚠️ [清空所有数据]                                       │
└─────────────────────────────────────────────────────────────┘
```

### 9.6 前端技术实现

```typescript
// src/router/index.tsx
const router = createBrowserRouter([
  { path: "/", element: <Dashboard /> },
  { path: "/time", element: <TimeStats /> },
  { path: "/content", element: <ContentBrowser /> },
  { path: "/content/:sessionId", element: <SessionDetail /> },
  { path: "/reports", element: <ReportCenter /> },
  { path: "/reports/:id", element: <ReportDetail /> },
  { path: "/todos", element: <TodoList /> },
  { path: "/flash", element: <FlashIdeas /> },
  { path: "/realtime", element: <RealtimeMonitor /> },
  { path: "/settings", element: <Settings /> },
]);

// src/services/tauri.ts - 与 Rust 后端通信
import { invoke } from "@tauri-apps/api/tauri";

export const api = {
  getDailyStats: (date: string) => invoke("get_daily_stats", { date }),
  getAppUsage: (start: number, end: number) => invoke("get_app_usage", { start, end }),
  queryEvents: (params: EventQuery) => invoke("query_events", { params }),
  triggerAnalysis: (type: AnalysisType) => invoke("trigger_analysis", { type }),
  getTodos: (status?: string) => invoke("get_todos", { status }),
  toggleTodo: (id: number) => invoke("toggle_todo", { id }),
  // ...
};

// 实时事件订阅
import { listen } from "@tauri-apps/api/event";

listen<RawEvent>("raw_event_captured", (event) => {
  // 实时更新"此刻正在记录"面板
  realtimeStore.pushEvent(event.payload);
});

listen<AnalysisResult>("analysis_completed", (result) => {
  toast.success(`${result.payload.analysis_type} 报告已生成`);
  reportStore.addReport(result.payload);
});
```

---

## 10. macOS 平台专项

### 10.1 系统权限

| 权限 | 用途 | 是否必需 | 申请方式 |
|------|------|---------|---------|
| **辅助功能（Accessibility）** | 监听键盘/鼠标事件 | 必需 | `AXIsProcessTrusted()` + 系统设置引导 |
| **自动化（Automation）** | AppleScript 控制浏览器 | 必需 | 用户首次触发时系统弹窗 |
| **完全磁盘访问** | 读取某些应用数据 | 一般不需要 | 可选 |
| **通知** | 报告生成提醒 | 推荐 | `NSUserNotificationCenter` |
| **Keychain** | 存储 API Key | 必需（间接） | 系统自动 |

### 10.2 关键 macOS API

```rust
// 1. 键盘/鼠标事件
unsafe extern "C" fn event_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    // 处理 keydown, keyup, mouse click 等
}

// 2. 活跃应用
let app = NSWorkspace::sharedWorkspace().frontmostApplication();

// 3. 窗口标题
let windows = CGWindowListCopyWindowInfo(
    kCGWindowListOptionOnScreenOnly,
    kCGNullWindowID,
);

// 4. 剪贴板
let pb = NSPasteboard::generalPasteboard();
let content = pb.stringForType(NSPasteboardTypeString);

// 5. Safari URL
let script = r#"tell application "Safari" to get URL of front document"#;
let output = run_osascript(script).await?;

// 6. Chrome URL（CDP）
let response = reqwest::get("http://localhost:9222/json").await?;
```

### 10.3 LaunchAgent（可选自动启动）

```xml
<!-- ~/Library/LaunchAgents/com.personallogai.agent.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.personallogai.agent</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Applications/PersonalLogAI.app/Contents/MacOS/personal-log-ai</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
</dict>
</plist>
```

### 10.4 系统托盘

- 菜单栏图标（图标状态：运行中/暂停/空闲）
- 右键菜单：打开仪表盘 / 暂停记录 / 设置 / 退出
- 全局快捷键：⌘⇧P 暂停、⌘⇧R 立即生成报告、⌘⇧D 打开仪表盘

### 10.5 打包与签名

- **开发期**：`cargo tauri dev`
- **打包**：`cargo tauri build` 生成 `.app` 和 `.dmg`
- **签名**：v1 仅个人自用，可不签名（首次启动需右键"打开"绕过 Gatekeeper）
- **公证**：v1 可选，未来如要分发需 Apple Developer 账号

---

## 11. 隐私与安全

### 11.1 数据流（关键路径）

```
[键盘/鼠标/剪贴板]
    ↓ 实时事件流
[Rust 采集器]  ← 所有过滤在此发生
    ↓ 写入（明文，v1 不加密）
[本地 SQLite] ~/Library/Application Support/PersonalLogAI/data.db
    ↓ 定时任务（每日 03:00）或手动触发
[AI 调度器]  ← 读取"待分析文本段"，不发送完整数据库
    ↓ 通过 HTTPS
[云端 LLM]   ← 只发"待分析文本 + 上下文元数据"
    ↓ 返回结果
[本地 SQLite]
```

### 11.2 关键隐私保护措施

1. **采集端过滤**：密码字段、信用卡、Token 自动丢弃
2. **应用黑名单**：密码管理器、金融 App 完全静默
3. **暂停机制**：用户随时 ⌘⇧P 暂停记录（开会/演示时）
4. **透明监控**：实时显示"现在在记什么"
5. **可删除**：用户可删除任意记录或全部清空
6. **本地优先**：原始数据永不上传，仅分析时按需上传片段
7. **可审计**：用户可查看"今天上传给 AI 的内容"
8. **API Key 隔离**：仅存 Keychain，数据库无 Key

### 11.3 数据生命周期

| 数据类型 | 默认保留期 | 可配置 |
|---------|----------|-------|
| 原始事件 | 90 天 | 30/180/365/永久 |
| 输入会话 | 跟随原始事件 | 同上 |
| 焦点会话 | 90 天聚合 + 永久统计 | 否 |
| AI 报告 | 永久 | 否（数据量小） |
| TODO | 完成后保留 30 天 | 是 |

### 11.4 数据导出与备份

- **导出格式**：JSON（完整）、CSV（时长统计）、Markdown（报告）
- **导出位置**：用户选择
- **导入**：仅支持"清空后导入备份"
- **自动备份**：每周一次打包到 `~/Library/Application Support/PersonalLogAI/backups/`

### 11.5 已知安全风险

| 风险 | 缓解措施 |
|------|---------|
| 数据库未加密（v1） | 文件权限 700，目录权限 700 |
| API Key 泄露 | 存 Keychain，不进数据库 |
| 第三方依赖漏洞 | 定期 `cargo audit` / `npm audit` |
| 系统权限被滥用 | 仅在用户授权时使用，不上传原始数据库 |

---

## 12. 风险与挑战

| 风险 | 影响 | 缓解策略 |
|------|------|---------|
| **macOS 权限申请失败** | 高 | 完善引导、详细文档；权限丢失时降级到仅时长统计 |
| **macOS 系统升级导致 API 变更** | 中 | 跟随 Tauri 与 rdev 更新，CI 验证 |
| **AI API 限流 / 失败** | 中 | 多服务商支持 + 本地降级 + 重试机制 |
| **数据量膨胀** | 中 | 90 天默认清理、聚合表分离 |
| **电池消耗** | 中 | 空闲时降频、批量写入 |
| **误记录敏感信息** | 高 | 严格过滤规则、实时监控、用户可一键清除 |
| **误判工作状态** | 低 | 算法可调、用户反馈机制 |

---

## 13. 里程碑与开发计划

### 13.1 MVP（v0.1）— 2 周

**目标**：可用的核心采集 + 基础展示

- [ ] Tauri 项目初始化（Rust + React + Vite）
- [ ] macOS 权限申请引导
- [ ] 键盘 + 鼠标 + 应用切换 + 窗口标题 采集
- [ ] SQLite 存储 + 数据模型 v1
- [ ] 焦点会话 + 应用/网站时长统计
- [ ] Web 仪表盘：总览页 + 时长统计页 + 实时监控页
- [ ] 敏感信息过滤（基础规则）
- [ ] 黑名单应用
- [ ] 系统托盘 + 暂停/恢复

### 13.2 V1.0 — 1 个月

**目标**：完整可用 + AI 分析

- [ ] 剪贴板采集 + 加强过滤
- [ ] 浏览器 URL 采集（Safari + Chrome）
- [ ] AI 调度器 + OpenAI 集成
- [ ] **TODO 自动提取**（本地规则 + AI）
- [ ] 每日 03:00 定时分析（生产力、主题、写作）
- [ ] **打字速度 / 退格率 / 停顿**分析
- [ ] 报告中心 + 内容浏览器
- [ ] 个人目标追踪
- [ ] 闪念捕获
- [ ] 数据导出

### 13.3 V1.x — 2 个月

- [ ] Claude / DeepSeek 等多 LLM 支持
- [ ] 番茄钟自动识别
- [ ] 词云 / 错别字追踪
- [ ] 中英文混用分析
- [ ] 跨日主题演化
- [ ] 周报自动生成
- [ ] 自定义分析维度

### 13.4 V2.0 — 远期

- [ ] 数据库加密（可选）
- [ ] 本地 LLM 支持（Ollama 集成）
- [ ] 个人知识图谱
- [ ] AI 主动提问助手
- [ ] 个人传记 / 决策日志
- [ ] Windows / Linux 适配评估
- [ ] 移动端配套（仅捕获，不展示）

---

## 14. 待确认事项

以下事项在开发前需用户最终确认：

1. **AI 服务商**：默认 OpenAI GPT-4o，是否确认？还是想默认 DeepSeek（中文友好 + 便宜）？
2. **数据保留期**：默认 90 天，是否调整？
3. **打包格式**：`.app` + `.dmg` 是否满足？
4. **签名 / 公证**：v1 不签名可以吗？
5. **MVP 范围**：上面 MVP 列表 2 周内可完成，节奏是否合适？
6. **个性化目标**：你有什么具体的"想追踪的个人目标"想加进 MVP？

---

## 15. 附录

### 15.1 相关文档

- `tech-stack-decision.md`（待写）：技术选型详细对比
- `api-reference.md`（待写）：Tauri IPC 命令清单
- `deployment.md`（待写）：macOS 打包与发布流程

### 15.2 参考资源

- Tauri 2.0 官方文档：https://tauri.app/
- rdev（Rust 跨平台输入监听）：https://github.com/Narsil/rdev
- macOS Accessibility API：https://developer.apple.com/documentation/accessibility
- AppleScript Safari 文档：https://developer.apple.com/library/archive/documentation/AppleScript/Conceptual/AppleScriptLangGuide/

### 15.3 变更记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-06-07 | v1.0 | 初始设计文档 |

---

**文档结束。** 请审阅后告知修改意见或确认通过，确认后进入实施阶段（生成实施计划 → 编码）。
