#!/usr/bin/env bash
#
# scripts/install-hooks.sh
#
# 一键配置 git hooks 路径到 .githooks/
#
# 用法：
#   bash scripts/install-hooks.sh         # 安装并启用
#   bash scripts/install-hooks.sh --uninstall  # 卸载（恢复默认 .git/hooks）
#
# 原理：
#   设置 core.hooksPath = .githooks（相对于仓库根的路径）
#   这样 .githooks/ 下的 hook 文件会被 git 自动执行
#   且 .githooks/ 是 tracked in git，新 clone 的人运行此脚本即可获得相同配置
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_DIR="$REPO_ROOT/.githooks"

# 颜色
if [ -t 1 ]; then
    C_GREEN='\033[0;32m'
    C_YELLOW='\033[0;33m'
    C_RED='\033[0;31m'
    C_BOLD='\033[1m'
    C_RESET='\033[0m'
else
    C_GREEN=''; C_YELLOW=''; C_RED=''; C_BOLD=''; C_RESET=''
fi

log()  { echo -e "${C_GREEN}[hooks]${C_RESET} $*"; }
warn() { echo -e "${C_YELLOW}[hooks]${C_RESET} $*"; }
err()  { echo -e "${C_RED}[hooks]${C_RESET} $*" >&2; }

# 检查参数
action="install"
for arg in "$@"; do
    case "$arg" in
        install)   action="install" ;;
        --uninstall) action="uninstall" ;;
        --help|-h)
            sed -n '2,15p' "$0"
            exit 0
            ;;
        *)
            err "未知参数：$arg"
            exit 1
            ;;
    esac
done

# 检查 git 仓库
cd "$REPO_ROOT"
if ! git rev-parse --git-dir >/dev/null 2>&1; then
    err "当前目录不是 git 仓库：$REPO_ROOT"
    exit 1
fi

# 卸载模式
if [ "$action" = "uninstall" ]; then
    current=$(git config --local --get core.hooksPath 2>/dev/null || echo "")
    if [ -z "$current" ]; then
        log "未设置 core.hooksPath，无需卸载"
        exit 0
    fi
    git config --local --unset core.hooksPath
    log "已卸载 core.hooksPath（恢复到 .git/hooks/）"
    exit 0
fi

# 安装模式
# 1. 检查 .githooks 目录存在
if [ ! -d "$HOOKS_DIR" ]; then
    err ".githooks/ 目录不存在：$HOOKS_DIR"
    exit 1
fi

# 2. 检查 hook 文件可执行
missing_exec=0
for hook in "$HOOKS_DIR"/*; do
    [ -f "$hook" ] || continue
    if [ ! -x "$hook" ]; then
        warn "$(basename "$hook") 不可执行，自动 chmod +x"
        chmod +x "$hook"
    fi
done

# 3. 设置 core.hooksPath
# 用相对路径（.githooks）相对于 git 仓库根
current=$(git config --local --get core.hooksPath 2>/dev/null || echo "")

if [ "$current" = ".githooks" ]; then
    log "core.hooksPath 已经设置为 .githooks，无需修改"
else
    if [ -n "$current" ]; then
        warn "core.hooksPath 已是：$current，将覆盖为 .githooks"
    fi
    git config --local core.hooksPath .githooks
    log "✅ 已设置 core.hooksPath = .githooks"
fi

# 4. 验证
echo ""
log "当前 hook 配置："
git config --local --get core.hooksPath
echo ""
log ".githooks 内容："
ls -la "$HOOKS_DIR"
echo ""

# 5. 做个烟雾测试（不实际 commit，只跑 hook 看是否报错）
log "烟雾测试（dry-run 跑 hook）..."
HOOK="$HOOKS_DIR/pre-commit"
if [ -x "$HOOK" ]; then
    # 测试 1：纯净状态（无 staged），hook 应快速通过
    if "$HOOK" 2>/dev/null; then
        log "✅ 烟雾测试通过（无 staged 文件时 hook 不报错）"
    else
        warn "烟雾测试未通过，但首次安装通常无害"
        warn "实际触发会在 git commit 时"
    fi
else
    warn "pre-commit hook 不存在或不可执行：$HOOK"
fi

echo ""
log "🎉 安装完成！"
echo ""
echo -e "${C_BOLD}后续使用${C_RESET}："
echo "  • 正常 commit：先更新 CHANGELOG.md，再 git commit"
echo "  • 紧急绕过：git commit --no-verify"
echo "  • 卸载：bash scripts/install-hooks.sh --uninstall"
echo ""
echo -e "${C_BOLD}新机器 / 新 clone 之后${C_RESET}，记得运行："
echo "  ${C_GREEN}bash scripts/install-hooks.sh${C_RESET}"