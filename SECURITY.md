# Security Policy

> 本项目的安全策略、密钥管理规范、应急响应流程。

**适用范围**：`personal-recoder` 仓库（GitHub: `eastyy/personal-recoder`）
**最后更新**：2026-07-19

---

## 🔑 密钥管理（最重要）

### ❌ 绝不能做的

- 把任何 API Key、Token、Secret、密码**直接写入代码**
- 把任何凭据放进 `.hermes/`、`docs/`、README、AGENTS.md 等**会被 commit 的位置**
- 即使是「临时记录」「以后会删」也不行——一旦 commit 就可能已经泄漏
- 即使仓库是 private 也算泄漏（GitHub 内部访问、第三方备份、fork 后的可见性都可能扩大风险）

### ✅ 应该做的

- API Key 仅存在 **运行时数据库**（`~/Library/Application Support/PersonalLogAI/data.db` 的 `app_config` 表）
- 配置文件用占位符（如 `<YOUR_API_KEY>`、`<REDACTED-volcengine-api-key>`、`xxx`）
- 本地开发用的 Key 放 `.env`（已 gitignore）
- 想分享给别人的项目配置：脱敏后用「请填入你自己的 Key」

### 🔍 自动检查

本项目配置了 **pre-commit hook**，每次 `git commit` 时会自动调用 `scripts/scan-secrets.sh` 扫描 20+ 种常见 API Key 格式：

- OpenAI / Anthropic / MiniMax / DeepSeek (sk-*)
- AWS Access Key (AKIA*)
- Google API Key (AIza*)
- GitHub Token (gh[pousr]_*)
- **Volcengine 火山方舟 (ark-*)** ← 本项目使用
- Stripe / Slack / GitLab / SendGrid / Mailgun
- 通用 Bearer Token / password / secret 字段
- 严格模式下额外检查长 hex / base64 字符串

扫描器会忽略明显占位符（`xxx`、`test_`、`mock_`、`<REDACTED-*>` 等）。

---

## 🚨 发现密钥泄漏怎么办

### 立即执行（按顺序）

#### 1. 从当前文件删除

```bash
# 用 sed 替换为占位符
sed -i '' 's|<actual-key-value>|<REDACTED>|g' <file>
```

#### 2. 重写 git 历史

```bash
# 备份 .git 目录（万一需要恢复）
cp -R .git .git.backup-$(date +%Y%m%d-%H%M%S)

# 用 git filter-branch 重写所有 commit
git filter-branch --force --tree-filter '
sed -i "" "s|<actual-key-value>|<REDACTED>|g" <file>
' -- --all

# 清理残留对象
git stash list | xargs -I {} git stash drop
git for-each-ref --format='%(refname)' refs/original/ | xargs -I {} git update-ref -d {}
git reflog expire --expire=now --all
git gc --prune=now --aggressive
rm -f .git/objects/info/commit-graph
git commit-graph write --reachable
```

#### 3. 强制推送到远端

```bash
git push --force origin main
```

#### 4. ⚠️ **在提供商控制台轮换 Key**（最重要！）

仅删除文件 + 改历史**不能阻止已被泄漏的 Key 被使用**。任何在你 push 之前看过 GitHub 的人 / 任何第三方缓存都可能已经拿到 Key。

| 服务 | 轮换入口 |
|------|---------|
| 火山方舟 | https://console.volcengine.com/ark |
| OpenAI | https://platform.openai.com/api-keys |
| Anthropic | https://console.anthropic.com/settings/keys |
| AWS | https://console.aws.amazon.com/iam/home#/security_credentials |
| Google Cloud | https://console.cloud.google.com/apis/credentials |
| GitHub | https://github.com/settings/tokens |

#### 5. 检查异常活动

轮换前，在提供商控制台查看该 Key 的使用日志，看是否有非预期的调用记录：
- 调用 IP
- 调用时间
- 调用量 / 费用

如果有异常，**可能需要进一步处理账单/封号**。

#### 6. 记录事件

在 `CHANGELOG.md` 的 `[Unreleased] / Security` 段落记录（不记录 Key 值本身，只描述事件）。

---

## 📝 2026-07-19 事件复盘

**问题**：`.hermes/handover.md` 和 `.hermes/project.md` 中包含明文火山方舟 API Key。

**影响**：
- Key 在 2 个 commit 中被推送到 GitHub（私有仓库）
- 仓库是 private，但理论上仍有泄漏风险

**修复**：
1. ✅ 文件中替换为 `<REDACTED-volcengine-api-key>` 占位符
2. ✅ `git filter-branch` 重写所有历史
3. ✅ 强制推送，验证远端无残留
4. ✅ 添加 `scripts/scan-secrets.sh` 自动检测
5. ✅ 更新 pre-commit hook 集成扫描
6. ✅ `.hermes/` 加入 `.gitignore`（不再入库）
7. ✅ 本 SECURITY.md 文档化流程

**教训**：
> **不要把 Key 写进任何「将来要删」的笔记**。即使打算立刻删，commit + push 之间的时间窗口也足够让 Key 泄漏。

**用户待办**：
- ⏳ 在火山方舟控制台轮换作废旧 Key（用户手动）
- ⏳ 检查旧 Key 的使用日志

---

## 🔒 仓库安全设置

### 已配置

- ✅ 仓库设为 **Private**
- ✅ SSH key 认证（无密码登录）
- ✅ `~/.gitconfig` 配置了 fetch 走 ghfast.top、push 走直连的安全分流
- ✅ `core.hooksPath = .githooks`（hook 受版本控制）
- ✅ `.gitignore` 覆盖 `target/` / `node_modules/` / `*.db` / `*.pem` / `*.env` 等

### 建议进一步做

- ⏳ 在 GitHub 启用 2FA（保护账号本身）
- ⏳ 启用 GitHub 的 Dependabot（依赖安全更新）
- ⏳ 定期运行 `scripts/scan-secrets.sh` 审计整个仓库
- ⏳ 如果添加协作者，使用 GitHub 团队的「最小权限」原则

---

## 📞 联系方式

如发现安全问题（即使是怀疑），请联系仓库 owner：
- GitHub: [@eastyy](https://github.com/eastyy)
- Email: eastyy@qq.com

---

**记住：删除文件 + 改历史只是补救，轮换 Key 才是根治。**