#!/usr/bin/env bash
# 静态编译：生成不依赖 GLIBC 的 x86_64 Linux 二进制
# 方式1：本机需已安装 musl-tools (提供 musl-gcc)
# 方式2：使用 cross（需 Docker，无需 musl-tools）：cross build --release --target x86_64-unknown-linux-musl

set -e
TARGET="x86_64-unknown-linux-musl"
export CC_x86_64_unknown_linux_musl="${CC_x86_64_unknown_linux_musl:-musl-gcc}"

if ! command -v musl-gcc &>/dev/null; then
  echo "错误: 未找到 musl-gcc。" >&2
  echo "请任选一种方式：" >&2
  echo "  1) 安装 musl-tools 后重试：" >&2
  echo "     Debian/Ubuntu: sudo apt install musl-tools" >&2
  echo "     Fedora: sudo dnf install musl-gcc" >&2
  echo "  2) 使用 cross（需 Docker，无需本机 musl）：" >&2
  echo "     cargo install cross" >&2
  echo "     cross build --release --target $TARGET" >&2
  exit 1
fi

cargo build --release --target "$TARGET"
echo "Binary: target/$TARGET/release/sms-forwarder"
