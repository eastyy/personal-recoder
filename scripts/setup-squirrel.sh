#!/usr/bin/env bash
#
# setup-squirrel.sh
#
# 一键拉取 Squirrel 输入法构建所需的第三方依赖：
#   - librime   (https://github.com/rime/librime)        RIME C++ 引擎
#   - plum      (https://github.com/rime/plum)           RIME 配置管理工具
#   - Sparkle   (https://github.com/sparkle-project/Sparkle) macOS 应用更新框架
#
# 这些目录在 .gitignore 中被排除，所以 clone 后需要运行本脚本补齐。
#
# 用法：
#   bash scripts/setup-squirrel.sh                # 默认：源码克隆 + 可选下载预编译 librime
#   bash scripts/setup-squirrel.sh --no-download  # 只 clone 源码（自己编译）
#   bash scripts/setup-squirrel.sh --shallow      # shallow clone（节省时间和带宽）
#
# 注意：
#   - librime 体积较大（~500MB 含 git 历史），--shallow 推荐
#   - plum 仅用来下载 RIME 配方（default.yaml 等），可浅克隆
#   - Sparkle 作为子项目嵌入 Squirrel，可浅克隆

set -euo pipefail

# 路径定位：脚本在 scripts/setup-squirrel.sh
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SQUIRREL_DIR="$REPO_ROOT/squirrel-ime"

# librime / Sparkle 版本（与原 squirrel-ime/action-install.sh 一致）
RIME_VERSION="1.17.0"
RIME_GIT_HASH="33e7814"
SPARKLE_VERSION="2.6.2"

# 选项解析
SHALLOW=0
NO_DOWNLOAD=0
for arg in "$@"; do
    case "$arg" in
        --shallow)     SHALLOW=1 ;;
        --no-download) NO_DOWNLOAD=1 ;;
        --help|-h)
            sed -n '2,20p' "$0"
            exit 0
            ;;
        *)
            echo "未知参数：$arg" >&2
            echo "运行 '$0 --help' 查看帮助" >&2
            exit 1
            ;;
    esac
done

# 颜色输出（终端支持时）
if [ -t 1 ]; then
    C_GREEN='\033[0;32m'
    C_YELLOW='\033[0;33m'
    C_RED='\033[0;31m'
    C_RESET='\033[0m'
else
    C_GREEN=''; C_YELLOW=''; C_RED=''; C_RESET=''
fi

log()  { echo -e "${C_GREEN}[setup]${C_RESET} $*"; }
warn() { echo -e "${C_YELLOW}[setup]${C_RESET} $*"; }
err()  { echo -e "${C_RED}[setup]${C_RESET} $*" >&2; }

# 检查目录存在
if [ ! -d "$SQUIRREL_DIR" ]; then
    err "找不到 squirrel-ime/ 目录：$SQUIRREL_DIR"
    err "请在仓库根目录运行本脚本"
    exit 1
fi

# 检查 git
if ! command -v git >/dev/null 2>&1; then
    err "需要 git，请先安装 Xcode Command Line Tools：xcode-select --install"
    exit 1
fi

cd "$SQUIRREL_DIR"

# 构造 git clone 参数
clone_args=()
if [ "$SHALLOW" -eq 1 ]; then
    clone_args+=(--depth 1)
fi

# 1) librime
if [ -d "librime" ] && [ -d "librime/.git" ]; then
    log "librime/ 已存在，跳过"
elif [ "$NO_DOWNLOAD" -eq 0 ] && [ -d "download/dist" ]; then
    log "检测到 download/ 预编译产物，复用之（跳过 librime clone）"
else
    log "克隆 librime（RIME C++ 引擎）..."
    if [ "$SHALLOW" -eq 1 ]; then
        # 浅克隆特定 commit
        git clone "${clone_args[@]}" https://github.com/rime/librime.git librime
        (cd librime && git fetch --depth 1 origin "$RIME_GIT_HASH" && git checkout "$RIME_GIT_HASH")
    else
        git clone "${clone_args[@]}" https://github.com/rime/librime.git librime
    fi
fi

# 2) plum
if [ -d "plum" ] && [ -d "plum/.git" ]; then
    log "plum/ 已存在，跳过"
else
    log "克隆 plum（RIME 配置管理）..."
    git clone "${clone_args[@]}" https://github.com/rime/plum.git plum
fi

# 3) Sparkle
if [ -d "Sparkle" ] && [ -d "Sparkle/.git" ]; then
    log "Sparkle/ 已存在，跳过"
else
    log "克隆 Sparkle（macOS 更新框架）..."
    git clone "${clone_args[@]}" https://github.com/sparkle-project/Sparkle.git Sparkle
fi

# 4) 可选：下载 librime 预编译包（节省编译时间）
if [ "$NO_DOWNLOAD" -eq 0 ]; then
    mkdir -p download
    cd download

    rime_archive="rime-${RIME_GIT_HASH}-macOS-universal.tar.bz2"
    rime_url="https://github.com/rime/librime/releases/download/${RIME_VERSION}/${rime_archive}"

    if [ -f "$rime_archive" ]; then
        log "librime 预编译包已存在，跳过下载"
    else
        log "下载 librime 预编译包（避免编译 Boost）..."
        if curl -fL -O "$rime_url" 2>/dev/null; then
            tar --bzip2 -xf "$rime_archive"
            log "已解压 $rime_archive"
        else
            warn "预编译包下载失败（可能版本/网络问题），跳过。"
            warn "需要的话可以手动执行 squirrel-ime/action-install.sh 编译 librime"
        fi
    fi
    cd "$SQUIRREL_DIR"
fi

# 5) 检查最终状态
echo ""
log "依赖状态："
for dir in librime plum Sparkle; do
    if [ -d "$dir" ] && [ -d "$dir/.git" ]; then
        echo -e "  ${C_GREEN}✓${C_RESET} $dir/  ($(du -sh "$dir" | cut -f1))"
    else
        echo -e "  ${C_RED}✗${C_RESET} $dir/  缺失"
    fi
done
echo ""

log "下一步："
echo "  cd squirrel-ime"
echo "  bash build-and-install.sh    # 构建并安装 Squirrel（需要 sudo）"
echo ""
echo "或者仅开发 Tauri 主项目，跳过 Squirrel："
echo "  cd personal-log-ai && npm install && npm run tauri dev"