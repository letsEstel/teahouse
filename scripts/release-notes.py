#!/usr/bin/env python3
"""What a build brings, for the release bot (打包机) to post in the teahouse.

    scripts/release-notes.py <previous commit> <this commit>

Every pull request lands on main as one squashed commit whose subject says
what it does ("问题里能插图了：… (#67)"), so the subjects between the last
announced build and this one are the release notes. They are sorted by
what they are (fixes, removals, docs and CI, the rest) and shortened to
their first clause; the PR number stays so it can be looked up.
"""
import re
import subprocess
import sys

KINDS = [
    ("修复", re.compile(r"^(修复|修好|fix)|修复：|bug", re.I)),
    ("删减", re.compile(r"^(删掉|删除|去掉|移除|拿掉|remove)", re.I)),
    ("其他（文档、CI、测试）", re.compile(r"^(docs|ci|test|readme|文档|测试)|^\S+\.md[：:]", re.I)),
]
OTHER = "新增和改进"
LIMIT = 12


def short(subject: str) -> str:
    """'问题里能插图了：提出问题… (#67)' -> '问题里能插图了 (#67)'."""
    m = re.search(r"\s*\((#\d+)\)\s*$", subject)
    pr = f" ({m.group(1)})" if m else ""
    body = subject[: m.start()] if m else subject
    parts = [p.strip() for p in re.split(r"：|；|;|——", body) if p.strip()]
    # "界面：…", "CI 打包机：…", "DEVELOP.md：…": the part before the colon is a label or a file
    if len(parts) > 1 and (len(parts[0]) <= 4 or re.search(r"\.md$|^(CI|docs?)\b", parts[0], re.I)):
        parts = parts[1:]
    text = parts[0] if parts else body
    # drop a trailing aside in brackets, if what stays still says something
    cut = re.split(r"[（(]", text, maxsplit=1)[0].strip()
    text = cut if len(cut) >= 6 else text
    if len(text) > 40:
        text = text[:39] + "…"
    return text + pr


def main() -> int:
    prev, this = sys.argv[1], sys.argv[2]
    log = subprocess.run(
        ["git", "log", "--first-parent", "--format=%s", f"{prev}..{this}"],
        capture_output=True, text=True, check=True,
    ).stdout.splitlines()
    groups: dict[str, list[str]] = {}
    for s in log:
        kind = next((k for k, rx in KINDS if rx.search(s)), OTHER)
        groups.setdefault(kind, []).append(short(s))
    if not groups:
        print("和上一次打包相比没有新的改动。")
        return 0
    out = [f"这一版有 {len(log)} 处改动（上一次打包是 {prev[:7]}）："]
    for kind in [OTHER, "修复", "删减", "其他（文档、CI、测试）"]:
        items = groups.get(kind, [])
        if not items:
            continue
        out.append(f"{kind}：")
        out += [f"· {i}" for i in items[:LIMIT]]
        if len(items) > LIMIT:
            out.append(f"· 还有 {len(items) - LIMIT} 处")
    print("\n".join(out))
    return 0


if __name__ == "__main__":
    sys.exit(main())
