#!/usr/bin/env python3
"""Lightweight checks for GitHub PR description documents."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


EN_SECTIONS = [
    "Summary",
    "Change Type",
    "Related Context",
    "Key Changes",
    "Scope Boundaries and Risks",
    "Testing and Verification",
    "Documentation Impact",
    "Screenshots or Recordings",
    "Pre-Merge Checklist",
]

ZH_SECTIONS = [
    "摘要",
    "变更类型",
    "关联事项",
    "主要变更",
    "能力边界与风险",
    "测试与验证",
    "文档影响",
    "截图或录屏",
    "提交前检查",
]

FORBIDDEN_PATTERNS = [
    (re.compile(r"token\s*=", re.I), "possible token assignment"),
    (re.compile(r"(password|passwd|secret|api[_-]?key)\s*[:=]", re.I), "possible credential"),
    (re.compile(r"https?://(?:10\.|192\.168\.|172\.(?:1[6-9]|2\d|3[0-1])\.)"), "private network URL"),
    (re.compile(r"(?im)^##?\s*(Risks?|风险)\s*\n\s*(None|无)\s*\.?\s*$"), "empty risk section"),
]


def headings(text: str) -> set[str]:
    return {match.group(1).strip() for match in re.finditer(r"^##\s+(.+?)\s*$", text, re.M)}


def main() -> int:
    parser = argparse.ArgumentParser(description="Check GitHub PR document structure and obvious leaks.")
    parser.add_argument("path", type=Path)
    args = parser.parse_args()

    text = args.path.read_text(encoding="utf-8")
    seen = headings(text)
    missing_en = [section for section in EN_SECTIONS if section not in seen]
    missing_zh = [section for section in ZH_SECTIONS if section not in seen]

    errors: list[str] = []
    if missing_en and missing_zh:
        errors.append(
            "missing required section set; expected English or Chinese GitHub PR template headings"
        )

    for pattern, label in FORBIDDEN_PATTERNS:
        if pattern.search(text):
            errors.append(f"forbidden pattern detected: {label}")

    if not re.search(r"- \[[ xX]\]", text):
        errors.append("no GitHub checklist items found")

    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1

    print(f"OK: {args.path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
