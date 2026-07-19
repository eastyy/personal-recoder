#!/bin/bash
# ============================================================
# Squirrel IME 一键构建 + 安装脚本
# 用法: ./build-and-install.sh
# 功能: 编译含 IPC hook 的 Squirrel，替换官方版二进制
# ============================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="${SCRIPT_DIR:-/Users/yy/Documents/trae_projects/recoder/squirrel-ime}"
BUILD_DIR="$PROJECT_DIR/build/Build/Products/Release/Squirrel.app"
INSTALL_DIR="/Library/Input Methods"
ENT_FILE="$PROJECT_DIR/resources/Squirrel.entitlements"

echo "========================================"
echo "  Squirrel IME 构建 + 安装"
echo "========================================"

# 1. 编译
echo "[1/5] 编译 universal binary..."
cd "$PROJECT_DIR"
xcodebuild -project Squirrel.xcodeproj -scheme Squirrel -configuration Release \
    -derivedDataPath build -arch arm64 -arch x86_64 ONLY_ACTIVE_ARCH=NO \
    clean build 2>&1 | tail -3

if [ ! -f "$BUILD_DIR/Contents/MacOS/Squirrel" ]; then
    echo "❌ 编译失败"
    exit 1
fi
echo "  ✅ 编译成功"

# 2. 验证 IPC hook
echo "[2/5] 验证 IPC hook..."
if ! strings "$BUILD_DIR/Contents/MacOS/Squirrel" | grep -q "personal-log-ai-ime.sock"; then
    echo "❌ IPC hook 不在二进制中"
    exit 1
fi
echo "  ✅ IPC hook 验证通过"

# 3. 拷贝五笔方案到 SharedSupport
echo "[3/5] 拷贝输入方案..."
for f in wubi98 wubi86; do
    cp "$PROJECT_DIR/data/plum/${f}"*.yaml "$BUILD_DIR/Contents/SharedSupport/" 2>/dev/null || true
done
echo "  ✅ 方案已拷贝"

# 4. 替换官方版二进制（需要 sudo）
echo "[4/5] 替换官方版二进制..."
echo "  需要 sudo 权限来替换 /Library/Input Methods/Squirrel.app"
sudo cp "$BUILD_DIR/Contents/MacOS/Squirrel" "$INSTALL_DIR/Squirrel.app/Contents/MacOS/Squirrel"

# 只签名主二进制（保留官方 dylib 签名）
codesign --force --sign - --entitlements "$ENT_FILE" \
    "$INSTALL_DIR/Squirrel.app/Contents/MacOS/Squirrel" 2>&1
echo "  ✅ 二进制已替换并签名"

# 5. 重启 Squirrel
echo "[5/5] 重启 Squirrel..."
killall Squirrel 2>/dev/null || true
sleep 1
"$INSTALL_DIR/Squirrel.app/Contents/MacOS/Squirrel" --register-input-source 2>&1
"$INSTALL_DIR/Squirrel.app/Contents/MacOS/Squirrel" --enable-input-source 2>&1
"$INSTALL_DIR/Squirrel.app/Contents/MacOS/Squirrel" --reload 2>&1
echo "  ✅ Squirrel 已重启"

echo ""
echo "========================================"
echo "  安装完成！"
echo "========================================"
echo ""
echo "验证："
echo "  strings $INSTALL_DIR/Squirrel.app/Contents/MacOS/Squirrel | grep personal-log-ai"
echo "  sqlite3 ~/Library/Application\\ Support/PersonalLogAI/data.db \"SELECT COUNT(*) FROM raw_events WHERE event_type='ime_committed';\""
