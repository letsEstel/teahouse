#!/usr/bin/env bash
# WSL 里 Playwright 的无头浏览器缺 libnss3 / libnspr4，又没有 sudo 时用这个：
# 只下载 .deb 并解压到 ~/.cache/teahouse/chrome-libs，不装进系统。shoot.mjs 会自动用这个目录。
set -euo pipefail
dest="$HOME/.cache/teahouse/chrome-libs"
tmp="$(mktemp -d)"
cd "$tmp"
apt-get download libnss3 libnspr4
for d in *.deb; do dpkg -x "$d" x; done
mkdir -p "$dest"
cp -a x/usr/lib/x86_64-linux-gnu/. "$dest/"
rm -rf "$tmp"
echo "已解压到 $dest"
