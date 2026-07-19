#!/usr/bin/env bash
#
# scripts/scan-secrets.sh
#
# 通用密钥/API Key 扫描脚本
# 扫描 staged 或工作目录的文件，匹配常见 key 格式
#
# 用法：
#   bash scripts/scan-secrets.sh                    # 扫描工作目录
#   bash scripts/scan-secrets.sh --staged           # 只扫描 staged 文件
#   bash scripts/scan-secrets.sh --strict           # 严格模式（额外检查）
#   bash scripts/scan-secrets.sh --allow "sk-xxx..." # 白名单（已知 placeholder）
#
# 退出码：
#   0 = 未发现密钥
#   1 = 发现可疑密钥
#
# 检测的格式：
#   - OpenAI / Anthropic / DeepSeek 等 sk-* 格式
#   - AWS Access Key (AKIA...)
#   - Google API Key (AIza...)
#   - GitHub Token (gh[pousr]_*)
#   - Volcengine 火山方舟 (ark-*)
#   - Stripe (sk_live_/pk_live_)
#   - Slack (xox[bpars]-*)
#   - GitLab (glpat-*)
#   - SendGrid (SG.*)
#   - Mailgun (key-*)
#   - 通用 Bearer Token
#   - 长十六进制 / base64 字符串
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 颜色
if [ -t 1 ]; then
    C_RED='\033[0;31m'
    C_GREEN='\033[0;32m'
    C_YELLOW='\033[0;33m'
    C_BOLD='\033[1m'
    C_RESET='\033[0m'
else
    C_RED=''; C_GREEN=''; C_YELLOW=''; C_BOLD=''; C_RESET=''
fi

# 参数
mode="working"
strict=0

# 初始化为空数组（避免 set -u 问题）
allow_patterns=("")

for arg in "$@"; do
    case "$arg" in
        --staged)  mode="staged" ;;
        --strict)  strict=1 ;;
        --allow)
            shift
            # 移除初始空元素
            if [ ${#allow_patterns[@]} -eq 1 ] && [ -z "${allow_patterns[0]}" ]; then
                allow_patterns=()
            fi
            allow_patterns+=("$1")
            ;;
        --help|-h)
            sed -n '2,30p' "$0"
            exit 0
            ;;
        *)
            echo "未知参数：$arg" >&2
            exit 1
            ;;
    esac
done

cd "$REPO_ROOT"

# 通用排除（永远不扫描这些位置）
exclude_paths=(
    "*/node_modules/*"
    "*/target/*"
    "*/build/*"
    "*/.git/*"
    "*/dist/*"
    "*/download/*"
    "*/.hermes/*"      # 项目交接笔记，可能含示例
    "*/librime_dist_backup/*"
    "*/librime/*"
    "*/plum/*"
    "*/Sparkle/*"
    "*/Frameworks/*"
    "*/bin/*"
    "*/lib/*"
)

# 构造 grep 的 --exclude 参数
exclude_args=()
for p in "${exclude_paths[@]}"; do
    exclude_args+=(--exclude="$p")
done

# 定义 key 检测模式
# 格式：PATTERN_NAME|REGEX|DESCRIPTION
patterns=(
    "OpenAI/MiniMax/DeepSeek (sk-)|sk-[a-zA-Z0-9_-]{20,}|形如 sk-xxx 的 API key"
    "Anthropic (sk-ant-)|sk-ant-[a-zA-Z0-9_-]{20,}|形如 sk-ant-xxx 的 API key"
    "AWS Access Key|AKIA[0-9A-Z]{16}|AWS 访问密钥"
    "Google API Key|AIza[0-9A-Za-z_-]{35}|Google API 密钥"
    "GitHub Token|gh[pousr]_[a-zA-Z0-9]{36}|GitHub 个人/服务器/用户 Token"
    "Volcengine (ark-)|ark-[a-zA-Z0-9_-]{20,}|火山方舟 API Key"
    "Stripe Live|sk_live_[a-zA-Z0-9]{24,}|Stripe Live 密钥"
    "Stripe Live Public|pk_live_[a-zA-Z0-9]{24,}|Stripe Live 公钥"
    "Slack|xox[bpars]-[a-zA-Z0-9-]+|Slack Token"
    "GitLab|glpat-[a-zA-Z0-9_-]{20,}|GitLab 个人 Token"
    "SendGrid|SG\.[a-zA-Z0-9_-]{22}\.[a-zA-Z0-9_-]{43}|SendGrid API Key"
    "Mailgun|key-[a-zA-Z0-9]{32}|Mailgun API Key"
    "Bearer Token|Bearer\s+[a-zA-Z0-9_.-]{20,}|HTTP Authorization Bearer Token"
    "Generic Password|password\s*[:=]\s*[\"'][a-zA-Z0-9!@#$%^&*()_+=-]{8,}[\"']|密码字段赋值"
    "Generic Secret|secret\s*[:=]\s*[\"'][a-zA-Z0-9_-]{16,}[\"']|secret 字段赋值"
    "Generic API Token|api[-_]?token\s*[:=]\s*[\"'][a-zA-Z0-9_-]{16,}[\"']|api_token 字段赋值"
)

# 严格模式额外检查
if [ "$strict" -eq 1 ]; then
    patterns+=(
        "Long Hex (>=32 字符)|[a-f0-9]{32,}|可能是 hex 编码的 key 或 hash"
        "Long Base64 (>=40 字符)|[A-Za-z0-9+/]{40,}={0,2}|可能是 base64 编码的 key"
    )
fi

# 收集要扫描的文件
if [ "$mode" = "staged" ]; then
    files=$(git diff --cached --name-only --diff-filter=ACMR 2>/dev/null | sed '/^$/d')
    if [ -z "$files" ]; then
        echo -e "${C_GREEN}[scan]${C_RESET} 无 staged 文件"
        exit 0
    fi
    file_count=$(echo "$files" | wc -l | tr -d ' ')
    echo -e "${C_BOLD}[scan]${C_RESET} 扫描 $file_count 个 staged 文件..." >&2
else
    # 工作目录扫描（排除 .gitignore 的内容）
    files=$(find . -type f \
        -not -path "*/node_modules/*" \
        -not -path "*/target/*" \
        -not -path "*/build/*" \
        -not -path "*/.git/*" \
        -not -path "*/dist/*" \
        -not -path "*/download/*" \
        -not -path "*/.hermes/*" \
        -not -path "*/librime_dist_backup/*" \
        -not -path "*/librime/*" \
        -not -path "*/plum/*" \
        -not -path "*/Sparkle/*" \
        -not -path "*/Frameworks/*" \
        -not -path "*/bin/*" \
        -not -path "*/lib/*" \
        2>/dev/null | sed 's|^\./||')
    file_count=$(echo "$files" | wc -l | tr -d ' ')
    echo -e "${C_BOLD}[scan]${C_RESET} 扫描工作目录（$file_count 个文件）..." >&2
fi

# 临时存到文件以避免 bash 3.x 的 mapfile 问题
files_tmp=$(mktemp)
trap 'rm -f "$files_tmp"' EXIT
echo "$files" > "$files_tmp"

# 匹配结果
findings=()

# 对每个 pattern 扫描
while IFS='|' read -r name regex desc; do
    [ -z "$name" ] && continue

    # 在所有文件中查找（用 grep -HnE 确保带文件名）
    # -H 强制输出文件名，即使只有一个文件
    while IFS= read -r match_line; do
        [ -z "$match_line" ] && continue

        # 提取文件名和匹配内容
        # grep -Hn 输出格式：filename:linenum:content
        file=$(echo "$match_line" | awk -F: '{print $1}')
        # 提取行号
        lineno=$(echo "$match_line" | awk -F: '{print $2}')
        # 提取内容（去掉前两个字段）
        content=$(echo "$match_line" | cut -d: -f3-)

        # 检查白名单
        skip=0
        for allow in "${allow_patterns[@]}"; do
            [ -z "$allow" ] && continue
            if echo "$content" | grep -qF "$allow"; then
                skip=1
                break
            fi
        done

        # 跳过明显的占位符
        if echo "$content" | grep -qE "(<YOUR|<REDACTED|placeholder|example|TODO|xxx|your-|<API_KEY>|test_|mock_|sample_|dummy_)"; then
            skip=1
        fi

        if [ "$skip" -eq 0 ]; then
            findings+=("$file:$lineno | $name | $content")
        fi
    done < <(xargs grep -HnE "$regex" < "$files_tmp" 2>/dev/null || true)
done < <(printf '%s\n' "${patterns[@]}")

# 报告
if [ ${#findings[@]} -eq 0 ]; then
    echo -e "${C_GREEN}[scan]${C_RESET} ✅ 未发现可疑密钥" >&2
    exit 0
fi

echo "" >&2
echo -e "${C_RED}${C_BOLD}❌ 发现 ${#findings[@]} 个可疑密钥模式${C_RESET}" >&2
echo "" >&2

for finding in "${findings[@]}"; do
    IFS='|' read -r file name content <<< "$finding"
    echo -e "  ${C_RED}⚠${C_RESET}  ${C_BOLD}$name${C_RESET}" >&2
    echo -e "     文件：$file" >&2
    # 截断显示内容
    truncated=$(echo "$content" | head -c 100)
    echo -e "     内容：${C_YELLOW}$truncated${C_RESET}" >&2
    echo "" >&2
done

echo -e "${C_BOLD}怎么办？${C_RESET}" >&2
echo "  1. 如果是误报：用占位符替换（如 <YOUR_API_KEY>、xxx、<REDACTED>）" >&2
echo "  2. 如果是真实密钥：立即在提供商控制台轮换！" >&2
echo "  3. 紧急跳过：git commit --no-verify（不推荐）" >&2

exit 1