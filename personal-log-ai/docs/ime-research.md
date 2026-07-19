# macOS 开源输入法项目调研 -- 中文 IME 文本捕获集成方案

> 调研日期：2026-06-10
> 目标：寻找可用于 Personal Log AI 项目的开源 macOS 输入法项目，以捕获中文 IME 提交文本（committed text）

## 背景

当前项目使用 `CGEventTap` 监听键盘事件，但 CGEventTap 无法捕获 IME（输入法）的最终提交文本。中文输入法在用户输入拼音后，通过 IME 引擎转换并提交汉字，这个提交过程不经过 CGEventTap 的键盘事件回调。因此需要寻找替代方案来捕获 IME 提交的文本。

## macOS 输入法架构概述

macOS 的第三方输入法基于 **InputMethodKit** 框架构建，核心类是 `IMKInputController`。输入法通过以下方式与系统交互：

- `inputText:selection:replacement:` -- 向客户端应用提交文本
- `commitComposition:` -- 确认组合文本
- `setMarkedText:selection:replacement:` -- 设置组合中的候选文本

**关键发现**：InputMethodKit 的文本提交是输入法到目标应用的单向通道，外部应用无法直接拦截这些提交事件。要捕获 IME 文本，有以下几种可行路径：

1. **开发自定义输入法** -- 基于 InputMethodKit 创建一个代理输入法，在提交文本时同步通知我们的应用
2. **使用 Accessibility API** -- 通过 AX API 读取文本框内容变化
3. **Hook/注入** -- 技术风险高，不符合 App Store 审核
4. **剪贴板 + 键盘事件组合推断** -- 间接方案

## 开源项目调研

### 1. Squirrel（鼠须管）-- Rime for macOS

| 属性 | 详情 |
|------|------|
| GitHub URL | https://github.com/rime/squirrel |
| Stars | 6,062 |
| 最后更新 | 2026-06-02 |
| 语言 | Swift 87.5% |
| 协议 | GPL-3.0 |
| 使用 InputMethodKit | 是 |

**简介**：Squirrel 是 Rime 输入法引擎的 macOS 前端，是 macOS 上最流行的开源中文输入法之一。它通过 `librime` C 库进行拼音转换，Swift 层负责 UI 和 InputMethodKit 集成。

**集成可行性分析**：
- **优势**：代码结构清晰，Swift 编写，InputMethodKit 使用规范，社区活跃
- **劣势**：GPL-3.0 协议有传染性；项目体量大，提取 IME 拦截逻辑需要深度改造
- **可行性评级**：中等。可以作为 InputMethodKit 集成的参考项目，学习其如何处理 `commitComposition` 和 `inputText:selection:replacement:` 回调。但不建议直接 fork 作为文本捕获机制

### 2. McBopomofo（小麦注音）

| 属性 | 详情 |
|------|------|
| GitHub URL | https://github.com/openvanilla/McBopomofo |
| Stars | 792 |
| 最后更新 | 2026-06-04 |
| 语言 | Swift |
| 协议 | MIT |
| 使用 InputMethodKit | 是（imkit 标签） |

**简介**：macOS 上的开源注音输入法，由 OpenVanilla 组织维护。支持注音、拼音等多种输入方式。

**集成可行性分析**：
- **优势**：MIT 协议友好；代码结构清晰，`InputMethodHandler` 类封装了 IMKInputController 逻辑；活跃维护中
- **劣势**：主要面向繁体中文/注音，非拼音输入
- **可行性评级**：高。MIT 协议允许自由使用和修改。其 `InputMethodHandler` 和 `McBopomofoInputController` 是学习 InputMethodKit 文本提交流程的优秀参考。可以参考其架构设计自定义的文本捕获输入法

### 3. hallelujahIM（哈利路亚英文输入法）

| 属性 | 详情 |
|------|------|
| GitHub URL | https://github.com/dongyuwei/hallelujahIM |
| Stars | 2,512 |
| 最后更新 | 2026-05-24 |
| 语言 | Objective-C++ |
| 协议 | GPL-3.0 |
| 使用 InputMethodKit | 是（inputmethodkit 标签） |

**简介**：macOS 上的智能英文输入法，支持自动补全、拼写检查等功能。同时也支持拼音输入。

**集成可行性分析**：
- **优势**：同时支持英文和拼音，与我们的中文文本捕获需求高度相关；代码成熟稳定
- **劣势**：GPL-3.0 协议；Objective-C++ 编写，与现代 Swift 生态集成较难
- **可行性评级**：中等。其拼音处理逻辑有参考价值，但协议和语言技术栈不太适合直接集成

### 4. azooKey-Desktop

| 属性 | 详情 |
|------|------|
| GitHub URL | https://github.com/azooKey/azooKey-Desktop |
| Stars | 931 |
| 最后更新 | 2026-05-25 |
| 语言 | Swift |
| 协议 | MIT |
| 使用 InputMethodKit | 是（inputmethodkit 标签） |

**简介**：开源日语输入法，使用 Zenzai 神经网络进行假名到汉字的转换，支持实时转换和 LLM 辅助转换。

**集成可行性分析**：
- **优势**：MIT 协议；纯 Swift 编写，代码现代；InputMethodKit 集成方式清晰；活跃开发中
- **劣势**：面向日语输入，非中文
- **可行性评级**：高。虽然是日语输入法，但其 InputMethodKit 集成架构和文本提交流程是最佳的现代 Swift 参考实现。其 `commitText` 处理逻辑可以直接参考用于构建中文文本捕获方案

### 5. NavilIMEforMac

| 属性 | 详情 |
|------|------|
| GitHub URL | https://github.com/navilera/NavilIMEforMac |
| Stars | 121 |
| 最后更新 | 2025-03-28 |
| 语言 | Swift |
| 协议 | GPL-3.0 |
| 使用 InputMethodKit | 是（imkit 标签） |

**简介**：macOS 韩文输入法，代码结构相对简单。

**集成可行性分析**：
- **优势**：代码量小，适合快速理解 InputMethodKit 的最小实现
- **劣势**：GPL-3.0；韩文输入，非中文；维护不太活跃
- **可行性评级**：低。仅适合作为学习 InputMethodKit 基础概念的入门参考

### 6. PurrType

| 属性 | 详情 |
|------|------|
| GitHub URL | https://github.com/355070xx/PurrType |
| Stars | 2 |
| 最后更新 | 2026-06-08 |
| 语言 | Objective-C |
| 协议 | MIT |
| 使用 InputMethodKit | 是（inputmethodkit 标签） |

**简介**：Local-first 的繁体中文输入法，支持粤语和繁体中文。

**集成可行性分析**：
- **优势**：MIT 协议；直接面向中文输入；项目非常新
- **劣势**：Stars 极少，成熟度低；Objective-C 编写
- **可行性评级**：中等。MIT 协议友好且直接面向中文输入，但项目成熟度不足

## 推荐方案

### 方案 A：开发轻量级代理输入法（推荐）

基于 InputMethodKit 开发一个极简的"透传"输入法，核心功能：

1. 注册为 macOS 输入法
2. 用户切换到该输入法时，不改变原有输入体验
3. 在 `inputText:selection:replacement:` 回调中，将提交的文本通过 IPC（XPC/Socket）发送给 Personal Log AI 主应用
4. 主应用接收后记录到数据库

**参考项目**：McBopomofo（MIT，架构清晰）+ azooKey-Desktop（MIT，现代 Swift）

**优点**：
- 能精确捕获所有 IME 提交文本
- 不依赖 Accessibility 权限
- 可区分直接键盘输入和 IME 输入

**缺点**：
- 需要用户安装并启用自定义输入法
- 开发成本较高
- 需要处理输入法切换逻辑

### 方案 B：Accessibility API 轮询（备选）

通过 Accessibility API 定期读取当前焦点文本框的内容，对比差异来推断输入的文本。

**优点**：
- 不需要开发输入法
- 可以捕获所有文本变化（包括 IME）

**缺点**：
- 依赖 Accessibility 权限
- 轮询有延迟，不够实时
- 无法准确区分来源（键盘 vs IME vs 粘贴）
- 某些应用可能不支持 Accessibility 文本读取

### 方案 C：混合方案（短期可行）

保持现有 CGEventTap 监听直接键盘输入，同时：
1. 监听 `NSTextInputContext` 的通知（如果可行）
2. 结合剪贴板监听
3. 在应用层面做文本差异对比

**优点**：不需要额外开发输入法
**缺点**：无法可靠捕获所有 IME 文本

## 下一步建议

1. **短期**：采用方案 C，在现有 CGEventTap 基础上增加启发式 IME 文本检测
2. **中期**：参考 McBopomofo 和 azooKey-Desktop 的代码，基于 InputMethodKit 开发轻量级文本捕获输入法原型
3. **验证**：先实现一个最小可行产品（MVP），仅捕获 `commitComposition` 中的文本并通过本地 socket 发送给主应用
