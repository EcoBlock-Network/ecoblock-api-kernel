#!/usr/bin/env python3
"""
Strip Rust comments (// and /* */) from source files while preserving string and char literals.
Usage: scripts/strip_comments.py file1.rs file2.rs ...
This script overwrites files in-place and keeps a '.bak' copy.
"""
import sys
import os

def strip_comments(text: str) -> str:
    i = 0
    n = len(text)
    out = []
    NORMAL, LINE, BLOCK, SQUOTE, DQUOTE = 0,1,2,3,4
    state = NORMAL
    while i < n:
        ch = text[i]
        nxt = text[i+1] if i+1 < n else ''
        if state == NORMAL:
            if ch == '/' and nxt == '/':
                state = LINE
                i += 2
                continue
            elif ch == '/' and nxt == '*':
                state = BLOCK
                i += 2
                continue
            elif ch == '\\' and nxt in ('"', "'"):
                # stray backslash outside string, keep both
                out.append(ch)
                out.append(nxt)
                i += 2
                continue
            elif ch == '"':
                out.append(ch)
                state = DQUOTE
                i += 1
                continue
            elif ch == "'":
                out.append(ch)
                state = SQUOTE
                i += 1
                continue
            else:
                out.append(ch)
                i += 1
                continue
        elif state == LINE:
            # skip until newline
            if ch == '\n':
                out.append(ch)
                state = NORMAL
            i += 1
            continue
        elif state == BLOCK:
            # skip until */
            if ch == '*' and nxt == '/':
                i += 2
                state = NORMAL
            else:
                i += 1
            continue
        elif state == DQUOTE:
            # handle escapes
            if ch == '\\':
                out.append(ch)
                if i+1 < n:
                    out.append(text[i+1])
                    i += 2
                else:
                    i += 1
                continue
            elif ch == '"':
                out.append(ch)
                state = NORMAL
                i += 1
                continue
            else:
                out.append(ch)
                i += 1
                continue
        elif state == SQUOTE:
            if ch == '\\':
                out.append(ch)
                if i+1 < n:
                    out.append(text[i+1])
                    i += 2
                else:
                    i += 1
                continue
            elif ch == "'":
                out.append(ch)
                state = NORMAL
                i += 1
                continue
            else:
                out.append(ch)
                i += 1
                continue
    return ''.join(out)

def process_file(path: str):
    with open(path, 'r', encoding='utf-8') as f:
        src = f.read()
    stripped = strip_comments(src)
    # write backup
    bak = path + '.bak'
    with open(bak, 'w', encoding='utf-8') as f:
        f.write(src)
    with open(path, 'w', encoding='utf-8') as f:
        f.write(stripped)
    print(f"Stripped comments: {path} (backup: {bak})")

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: strip_comments.py <files...>")
        sys.exit(1)
    for p in sys.argv[1:]:
        if os.path.isdir(p):
            # walk directory for .rs
            for root, dirs, files in os.walk(p):
                for fn in files:
                    if fn.endswith('.rs'):
                        process_file(os.path.join(root, fn))
        else:
            process_file(p)
