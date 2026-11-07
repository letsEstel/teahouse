#!/usr/bin/env python3
"""装订这本书。书有两种小节，目录在 book/chapters.txt：

* `.md`：文字和代码写在一起。`cargo test --manifest-path book/Cargo.toml` 运行里面的代码。
* `.rs`：写在真实代码文件里。`//|` 开头的行是书的文字，文档注释里的例子单独排出来，
  其余每一行都是代码，原样放进代码块，一行不少，顺序不变。

结果放进 book/src/（不进 git）。装好了 mdbook 的话，接着生成 book/out/，
其中 reader.html 是给人读的一个文件的阅读版（模板是 book/reader.html）。

    python3 scripts/bind-book.py           # 装订
    python3 scripts/bind-book.py --check   # 检查：书上印的结果和真实运行的结果一样，.rs 小节的代码一行不少
"""
import base64
import json
import re
import shutil
import subprocess
import sys
import tempfile
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


def contents():
    """目录里的每一行：(部分的标题, None) 或 (None, 文件)。"""
    for line in (BOOK / "chapters.txt").read_text(encoding="utf8").splitlines():
        line = line.strip()
        if line.startswith("## "):
            yield line[3:].split(" | ")[0], None
        elif line and not line.startswith("#"):
            yield None, line


def chapters():
    return [entry for _, entry in contents() if entry]


FENCE = re.compile(r"^```(\S*)\n(.*?)\n```$", re.S | re.M)


def blocks(text):
    """每段 Rust 代码，配上它后面最近的结果：(标记, 代码, 结果的种类, 结果)。"""
    pairs = []
    for info, body in FENCE.findall(text):
        if info.startswith("rust"):
            pairs.append([info, body, "", ""])
        elif info in ("text,output", "text,error") and pairs and not pairs[-1][2]:
            pairs[-1][2:] = [info.split(",")[1], body]
    return pairs


def clean(text):
    text = re.sub(r"finished in [\d.]+s", "finished in 0.00s", text)
    text = re.sub(r"thread '([^']*)' \(\d+\)", r"thread '\1'", text)
    return [line.rstrip() for line in text.split("\n") if line.strip()]


def run(tags, code):
    """像读者那样编译并运行一段代码，返回 (是否编译成功, 输出)。"""
    with tempfile.TemporaryDirectory() as d:
        (Path(d) / "src").mkdir()
        (Path(d) / "src/main.rs").write_text(code, encoding="utf8")
        args = ["rustc", "--edition=2021", "-Awarnings", "src/main.rs", "-o", "prog"]
        if "test" in tags:
            args.insert(1, "--test")
        done = subprocess.run(args, cwd=d, capture_output=True, text=True)
        if done.returncode:
            return False, done.stderr
        # 测试默认并行，一个一个跑，输出的顺序才固定
        command = [str(Path(d) / "prog")] + (["--test-threads=1"] if "test" in tags else [])
        ran = subprocess.run(command, cwd=d, capture_output=True, text=True)
        # should_panic 和 fails：程序就是要失败，报错也算结果
        failing = "should_panic" in tags or "fails" in tags
        if bool(ran.returncode) != failing:
            sys.exit(f"这段代码运行的结果不对（退出状态 {ran.returncode}）：\n{code}\n{ran.stderr}")
        return True, ran.stdout + (ran.stderr if failing else "")


def check_quote(entry, code):
    """标着出处的代码必须是真实代码的原文；`// …` 表示中间省略。"""
    source, *lines = code.split("\n")
    real = (ROOT / source.removeprefix("// ")).read_text(encoding="utf8")
    piece = []
    for line in lines + ["// …"]:
        if line.strip() == "// …":
            if "\n".join(piece) + "\n" not in real:
                sys.exit(f"{entry}: 这段和 {source} 对不上：\n" + "\n".join(piece))
            piece = []
        else:
            piece.append(line)


def check_outputs(entry):
    """书上印的结果必须和真实运行的结果一样；印出来的报错必须是真实报错里的原话。
    没有印结果的代码也要能编译（compile_fail 的要编译不过）。"""
    text = (ROOT / entry).read_text(encoding="utf8")
    n = 0
    for tags, code, kind, printed in blocks(text):
        if "ignore" in tags:
            if code.startswith("// "):
                check_quote(entry, code)
                n += 1
            continue
        compiled, actual = run(tags, code)
        if "compile_fail" in tags:
            if compiled:
                sys.exit(f"{entry}: 这段代码应该编译不过，却编译过了：\n{code}")
        elif not compiled:
            sys.exit(f"{entry}: 这段代码编译不过：\n{code}\n{actual}")
        if kind == "output" and clean(printed) != clean(actual):
            sys.exit(f"{entry}: 书上印的结果不对。\n书上：\n{printed}\n实际：\n{actual}")
        if kind == "error" and not all(line in clean(actual) for line in clean(printed)):
            sys.exit(f"{entry}: 书上印的报错不对。\n书上：\n{printed}\n实际：\n{actual}")
        n += 1
    return n


def present(text):
    """给运行结果和真实代码加上小标题；docs/ 里的文章开头有切换语言的链接，书里用不着。"""
    text = re.sub(r"^\[English\].*\n\n", "", text, flags=re.M)
    text = re.sub(
        r"```rust,ignore\n// (\S+)\n",
        lambda m: f'<p class="from">摘自 <code>{m.group(1)}</code></p>\n\n```rust\n',
        text,
    )
    return text.replace("```text,output", '<p class="out">运行结果</p>\n\n```text').replace(
        "```text,error", '<p class="out">报错</p>\n\n```text'
    )


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
    for part, entry in contents():
        if part:
            toc.append(f"\n# {part}\n")
            continue
        if entry.endswith(".md"):
            text = present((ROOT / entry).read_text(encoding="utf8"))
        else:
            text, shown = weave(entry)
            check(entry, shown)
            text = f'<p class="source">这一节就是 <code>{entry}</code></p>\n\n' + text
        title = re.search(r"^# (.+)$", text, re.M).group(1)
        n += 1
        name = f"{n:02}.md"
        (SRC / name).write_text(text, encoding="utf8")
        toc.append(f"- [{title}]({name})")
    (SRC / "SUMMARY.md").write_text("\n".join(toc) + "\n", encoding="utf8")
    print(f"装订了 {n} 节 → {SRC.relative_to(ROOT)}")


def reader():
    """阅读版：一个文件的书，有目录、翻页、搜索和进度。文字由 mdbook 排好，这里只取正文。"""
    def body(name):
        html = (BOOK / "out" / name).read_text(encoding="utf8")
        html = re.search(r"<main>(.*)</main>", html, re.S).group(1)
        html = re.sub(r'<a class="header"[^>]*>(.*?)</a>', r"\1", html, flags=re.S)
        html = re.sub(r"<a [^>]*class=\"[^\"]*mobile-nav[^>]*>.*?</a>", "", html, flags=re.S)
        for shot in re.findall(r'src="[^"]*?docs/screenshots/([^"]+)"', html):
            data = base64.b64encode((ROOT / "docs/screenshots" / shot).read_bytes()).decode()
            html = re.sub(rf'src="[^"]*?docs/screenshots/{re.escape(shot)}"', f'src="data:image/jpeg;base64,{data}"', html)
        return html

    sections = [{"id": "preface", "num": "", "part": "", "title": "前言", "html": body("preface.html")}]
    parts, n, p, j = [], 0, 0, 0
    lines = (BOOK / "chapters.txt").read_text(encoding="utf8").splitlines()
    for line in (x.strip() for x in lines):
        if line.startswith("## "):
            title, who = line[3:].split(" | ")
            p, j = p + 1, 0
            parts.append({"title": title, "who": who})
        elif line and not line.startswith("#"):
            n, j = n + 1, j + 1
            text = (BOOK / "src" / f"{n:02}.md").read_text(encoding="utf8")
            sections.append({
                "id": f"s{p}-{j}" if p else re.sub(r"^\d+-", "", Path(line).stem).split(".")[0].lower(),
                "num": f"{p}.{j}" if p else "",
                "part": p,
                "title": re.search(r"^# (.+)$", text, re.M).group(1),
                "html": body(f"{n:02}.html"),
            })
    data = json.dumps({"parts": parts, "sections": sections}, ensure_ascii=False).replace("</", "<\\/")
    page = (BOOK / "reader.html").read_text(encoding="utf8").replace("__BOOK__", data)
    (BOOK / "out" / "reader.html").write_text(page, encoding="utf8")
    print(f"阅读版 → book/out/reader.html（{len(page) // 1024} KB）")


def main():
    if "--check" in sys.argv:
        n = 0
        for entry in chapters():
            if entry.endswith(".md"):
                n += check_outputs(entry)
            else:
                check(entry, weave(entry)[1])
        print(f"书上的 {n} 段代码都和真实运行一致")
        return
    bind()
    if shutil.which("mdbook"):
        subprocess.run(["mdbook", "build", str(BOOK)], check=True)
        reader()


if __name__ == "__main__":
    main()
