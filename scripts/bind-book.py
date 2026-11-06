#!/usr/bin/env python3
"""装订这本书：把代码文件变成阅读版。

书写在代码文件里，装订只是换一种排版，不增加也不减少内容：

* `//|` 开头的行是书的正文（Markdown）；
* 文档注释里 ``` 围起来的例子单独排出来，`cargo test` 每次都会运行它们；
* 其余每一行都是代码，原样放进代码块，一行不少，顺序不变。

章节顺序写在 book/chapters.txt 里。结果放进 book/src/（不进 git），
装好了 mdbook 的话，接着生成 book/out/。

    python3 scripts/bind-book.py           # 装订
    python3 scripts/bind-book.py --check   # 只检查：每章的代码是否一行不少
"""
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BOOK = ROOT / "book"
SRC = BOOK / "src"
REPO = "https://github.com/letsEstel/teahouse/blob/main"

PROSE = re.compile(r"^\s*//\|(?: (.*))?$")
EXAMPLE = re.compile(r"^(\s*)/// ?```")
WIDE = r"[\u2e80-\uffef]"
RAN = '<p class="ran">运行过 · <code>cargo test</code> 每次都执行这段</p>\n'


def join_lines(block):
    """源文件里一段中文要折行；排版时接回去，否则换行处会多出一个空格。"""
    out, fenced = [], False
    for line in block:
        if line.startswith("```"):
            fenced = not fenced
        plain = out and out[-1] and line and not fenced and not line.startswith("```")
        solo = re.match(r"[-#>|*\d]", line)
        if plain and not solo and (re.search(WIDE + "$", out[-1]) or re.match(WIDE, line)):
            out[-1] += line
        else:
            out.append(line)
    return out


def drop_empty_doc(lines):
    """例子前面那一行空的 `///` 只是文档注释的分隔，不算代码。"""
    if lines and lines[-1].strip() == "///":
        lines.pop()


def weave(path):
    """一个代码文件 → 一章 Markdown。返回 (章的文字, 排进代码块的行)。"""
    parts, code, shown = [], [], []

    def flush():
        while code and not code[0].strip():
            code.pop(0)
        while code and not code[-1].strip():
            code.pop()
        if code:
            parts.append("```rust\n" + "\n".join(code) + "\n```\n")
            code.clear()

    lines = (ROOT / path).read_text(encoding="utf8").split("\n")
    i = 0
    while i < len(lines):
        line = lines[i]
        prose = PROSE.match(line)
        if prose:
            flush()
            block = []
            while i < len(lines) and (m := PROSE.match(lines[i])):
                block.append(m.group(1) or "")
                i += 1
            parts.append("\n".join(join_lines(block)) + "\n")
            continue
        fence = EXAMPLE.match(line)
        if fence:
            drop_empty_doc(code)
            drop_empty_doc(shown)
            flush()
            i += 1
            body = []
            while not EXAMPLE.match(lines[i]):
                body.append(re.sub(r"^\s*/// ?", "", lines[i]))
                i += 1
            i += 1
            parts.append(RAN + "\n```rust\n" + "\n".join(body) + "\n```\n")
            continue
        code.append(line)
        shown.append(line)
        i += 1
    flush()
    return "\n".join(parts), shown


def check(path, shown):
    """代码一行不少：去掉书的正文和例子以后，剩下的就是排进书里的那些行。"""
    kept, inside = [], False
    for line in (ROOT / path).read_text(encoding="utf8").split("\n"):
        if EXAMPLE.match(line):
            if not inside:
                drop_empty_doc(kept)
            inside = not inside
        elif not inside and not PROSE.match(line):
            kept.append(line)
    if kept != shown:
        sys.exit(f"{path}: 装订出来的代码和文件不一致")


def chapters():
    for line in (BOOK / "chapters.txt").read_text(encoding="utf8").splitlines():
        line = line.strip()
        if line and not line.startswith("#"):
            yield line


def preface():
    """前言就是 README：链接换成完整地址，图片跟着一起带走。"""
    text = (ROOT / "README.zh.md").read_text(encoding="utf8")
    text = re.sub(r"\]\((?!https?:)([^)#]+\.md)\)", rf"]({REPO}/\1)", text)
    shots = SRC / "docs" / "screenshots"
    shots.mkdir(parents=True, exist_ok=True)
    for f in (ROOT / "docs" / "screenshots").iterdir():
        shutil.copy(f, shots / f.name)
    return text


def bind():
    shutil.rmtree(SRC, ignore_errors=True)
    SRC.mkdir(parents=True)
    (SRC / "preface.md").write_text(preface(), encoding="utf8")
    toc = ["# 目录\n", "[前言](preface.md)\n"]
    n = 0
    for entry in chapters():
        if entry.endswith(".md"):
            text = (ROOT / entry).read_text(encoding="utf8")
        else:
            text, shown = weave(entry)
            check(entry, shown)
            text = f'<p class="source">这一章就是 <code>{entry}</code></p>\n\n' + text
        title = re.search(r"^# (.+)$", text, re.M).group(1)
        n += 1
        name = f"{n:02}.md"
        (SRC / name).write_text(text, encoding="utf8")
        toc.append(f"- [{title}]({name})")
    (SRC / "SUMMARY.md").write_text("\n".join(toc) + "\n", encoding="utf8")
    print(f"装订了 {n} 章 → {SRC.relative_to(ROOT)}")


def main():
    if "--check" in sys.argv:
        for entry in chapters():
            if not entry.endswith(".md"):
                check(entry, weave(entry)[1])
        print("每一章的代码都一行不少")
        return
    bind()
    if shutil.which("mdbook"):
        subprocess.run(["mdbook", "build", str(BOOK)], check=True)


if __name__ == "__main__":
    main()
