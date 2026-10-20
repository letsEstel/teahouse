#!/bin/sh
# Build the Android APK on this machine (the same steps as the android job in
# .github/workflows/app.yml), for when CI is unavailable and for the emulator.
#
#   sh scripts/build-android.sh [aarch64|x86_64]     (default aarch64: phones)
#
# Needs JAVA_HOME, ANDROID_HOME and NDK_HOME (see app/README.md). Unlike CI it
# signs with a fixed key kept outside the repository
# (~/.config/teahouse/android.jks, created on first use), so a newer build
# installs over the old one without uninstalling.
# Output: app/teahouse-<target>.apk
set -eu
cd "$(dirname "$0")/.."
TARGET=${1:-aarch64}
case $TARGET in aarch64) RT=aarch64-linux-android ;; x86_64) RT=x86_64-linux-android ;; *) echo "target: aarch64 | x86_64" >&2; exit 2 ;; esac
: "${JAVA_HOME:?}" "${ANDROID_HOME:?}" "${NDK_HOME:?}"
export ANDROID_NDK_ROOT="$NDK_HOME" ANDROID_NDK_HOME="$NDK_HOME"
export PATH="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH"
rustup target add "$RT" >/dev/null

[ -d node_modules ] || npm ci
[ -d src-tauri/gen/android ] || npx tauri android init --ci
npx tauri icon src-tauri/icons/source.png >/dev/null
sh scripts/android-patch.sh
npx tauri android build --apk --target "$TARGET" --ci

BT=$(ls -d "$ANDROID_HOME"/build-tools/* | sort -V | tail -n1)
KS=${TEAHOUSE_ANDROID_KEYSTORE:-$HOME/.config/teahouse/android.jks}
if [ ! -f "$KS" ]; then
  mkdir -p "$(dirname "$KS")"
  keytool -genkeypair -keystore "$KS" -storepass teahouse -keypass teahouse -alias teahouse \
    -keyalg RSA -keysize 2048 -validity 10000 -dname "CN=Teahouse" -noprompt
fi
APK=$(ls -t src-tauri/gen/android/app/build/outputs/apk/*/release/*-unsigned.apk | head -n1)
"$BT/zipalign" -f -p 4 "$APK" aligned.apk
"$BT/apksigner" sign --ks "$KS" --ks-pass pass:teahouse --key-pass pass:teahouse --out "teahouse-$TARGET.apk" aligned.apk
rm -f aligned.apk
"$BT/apksigner" verify "teahouse-$TARGET.apk"
echo "built app/teahouse-$TARGET.apk"
