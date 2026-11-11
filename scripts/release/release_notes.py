#!/usr/bin/env python3
"""Unreleased release-note fragments (D431).

Each change adds its own file under `qa/release/unreleased/notes/` instead of
editing one shared file, so PRs no longer conflict over release notes. A
fragment is a `## Section` heading followed by its entries. Fragments that
share a heading are merged under it at release time.

    release_notes.py check                  # validate fragments (CI)
    release_notes.py preview                # print the assembled notes
    release_notes.py assemble --version X   # write qa/release/X/RELEASE_NOTES.md
                                            # and delete the fragments it used
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
UNRELEASED = Path("qa/release/unreleased")
NOTES = UNRELEASED / "notes"
LEGACY_FILE = UNRELEASED / "RELEASE_NOTES.md"
NAME_RE = re.compile(r"^[a-z0-9][a-z0-9._-]*\.md$")
HOW_TO = (
    "Add a fragment instead: qa/release/unreleased/notes/<pr-or-issue>-<slug>.md "
    "starting with a '## Section' heading (reuse an existing heading to join that "
    "section). See qa/release/unreleased/README.md."
)


@dataclass
class Fragment:
    path: Path
    heading: str
    body: str


def parse(path: Path, root: Path) -> tuple[Fragment | None, list[str]]:
    shown = path.relative_to(root)
    errors: list[str] = []
    if not NAME_RE.match(path.name):
        errors.append(
            f"{shown}: name must be lowercase letters, digits, '.', '_' or '-', ending in .md"
        )
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return None, errors + [f"{shown}: not UTF-8"]
    raw = text.splitlines()
    first = next((i for i, line in enumerate(raw) if line.strip()), len(raw))
    lines = raw[first:]
    if not lines or not lines[0].startswith("## ") or not lines[0][3:].strip():
        return None, errors + [f"{shown}: must start with a '## Section' heading"]
    body = "\n".join(lines[1:]).strip("\n")
    if not body.strip():
        errors.append(f"{shown}: has a heading but no entries")
    fence = ""
    for number, line in enumerate(raw, start=1):
        if line != line.rstrip():
            errors.append(f"{shown}:{number}: trailing whitespace")
        marker = re.match(r"^\s*(`{3,}|~{3,})", line)
        if marker and (not fence or marker.group(1).startswith(fence)):
            fence = "" if fence else marker.group(1)
            continue
        if fence:
            continue
        if line.startswith("# "):
            errors.append(f"{shown}:{number}: '# ' titles are added at release time")
        if number > first + 1 and line.startswith("## "):
            errors.append(f"{shown}:{number}: one '## ' section per fragment")
    if fence:
        errors.append(f"{shown}: a {fence} code block is never closed")
    if errors:
        return None, errors
    return Fragment(path, lines[0][3:].strip(), body), []


def load(root: Path) -> tuple[list[Fragment], list[str]]:
    fragments: list[Fragment] = []
    errors: list[str] = []
    if (root / LEGACY_FILE).exists():
        errors.append(f"{LEGACY_FILE} must not exist (D431). {HOW_TO}")
    notes = root / NOTES
    if notes.is_dir():
        for path in sorted(notes.iterdir()):
            if path.name == ".gitkeep":
                continue
            if not path.is_file():
                errors.append(f"{path.relative_to(root)}: not a file")
                continue
            fragment, problems = parse(path, root)
            errors.extend(problems)
            if fragment:
                fragments.append(fragment)
    return fragments, errors


def _ends_in_list(text: str) -> bool:
    last_block = text.rstrip("\n").split("\n\n")[-1]
    return last_block.lstrip().startswith("- ")


def assemble(fragments: list[Fragment]) -> str:
    """Sections in order of first use (fragments sorted by name); entries in name order."""
    sections: dict[str, list[str]] = {}
    for fragment in fragments:
        sections.setdefault(fragment.heading, []).append(fragment.body)
    out: list[str] = []
    for heading, bodies in sections.items():
        merged = bodies[0]
        for body in bodies[1:]:
            joiner = "\n" if _ends_in_list(merged) and body.startswith("- ") else "\n\n"
            merged += joiner + body
        out.append(f"## {heading}\n\n{merged}\n")
    return "\n".join(out)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--root", type=Path, default=REPO_ROOT, help=argparse.SUPPRESS)
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("check")
    sub.add_parser("preview")
    assemble_cmd = sub.add_parser("assemble")
    assemble_cmd.add_argument("--version", required=True)
    args = parser.parse_args(argv)

    fragments, errors = load(args.root)
    if errors:
        for error in errors:
            print(f"error: {error}", file=sys.stderr)
        return 1
    if args.command == "check":
        print(f"{len(fragments)} release-note fragment(s) OK")
        return 0
    notes = assemble(fragments)
    if args.command == "preview":
        sys.stdout.write(notes)
        return 0
    if not fragments:
        print("error: no fragments to assemble", file=sys.stderr)
        return 1
    # The release workflow's pattern (corbanu-terminal-release.yml).
    if not re.fullmatch(r"\d+\.\d+\.\d+(-(alpha|beta)(\.\d+)?)?", args.version):
        print(f"error: bad version {args.version!r}", file=sys.stderr)
        return 1
    target = args.root / "qa/release" / args.version / "RELEASE_NOTES.md"
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists():
        existing = target.read_text(encoding="utf-8").rstrip("\n")
        # Appending a section the file already has would list it twice (and a
        # repeated run would duplicate entries): merge those by hand.
        already = sorted(
            {f.heading for f in fragments}
            & set(re.findall(r"(?m)^## (.+?)\s*$", existing))
        )
        if already:
            print(
                f"error: {target.relative_to(args.root)} already has: "
                + "; ".join(already)
                + ". Merge by hand or rename the fragment headings.",
                file=sys.stderr,
            )
            return 1
        target.write_text(f"{existing}\n\n{notes}", encoding="utf-8")
    else:
        target.write_text(
            f"# Corbanu Terminal {args.version}\n\n{notes}", encoding="utf-8"
        )
    for fragment in fragments:
        fragment.path.unlink()
    print(
        f"wrote {target.relative_to(args.root)} from {len(fragments)} fragment(s); fragments removed"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
