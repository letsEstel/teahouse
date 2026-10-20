#!/bin/sh
# After `tauri android init`: put our activity in place of the generated one
# (edge to edge: pads the page clear of the status bar, navigation bar, camera
# cutout and keyboard; bar colours follow the theme) and keep the activity
# alive across folding/unfolding so the page is not reloaded.
# Used by .github/workflows/app.yml and scripts/build-android.sh; run from app/.
set -eu
GEN=src-tauri/gen/android/app/src/main
cp src-tauri/android/MainActivity.kt "$GEN/java/org/teahouse/app/MainActivity.kt"
python3 - "$GEN/AndroidManifest.xml" <<'PY'
import re, sys
path = sys.argv[1]
src = open(path, encoding='utf-8').read()
need = 'orientation|screenSize|screenLayout|smallestScreenSize|density|keyboard|keyboardHidden|uiMode|locale|layoutDirection|fontScale'.split('|')
def fix(m):
    have = [v for v in m.group(2).split('|') if v]
    return m.group(1) + '|'.join(have + [v for v in need if v not in have]) + '"'
out, n = re.subn(r'(android:configChanges=")([^"]*)"', fix, src, count=1)
if n == 0:
    out, n = re.subn(r'(<activity\b)', r'\1 android:configChanges="' + '|'.join(need) + '"', src, count=1)
assert n == 1, 'no activity in manifest'
open(path, 'w', encoding='utf-8').write(out)
PY
grep -o 'android:configChanges="[^"]*"' "$GEN/AndroidManifest.xml"
