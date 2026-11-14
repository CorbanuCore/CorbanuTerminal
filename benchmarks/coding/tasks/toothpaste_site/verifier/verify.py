from __future__ import annotations

import argparse
import html.parser
import json
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

try:
    from PIL import Image
except Exception:  # pragma: no cover
    Image = None


SUMMARY_PREFIX = "TOOTHPASTE_SITE_VERIFIER_SUMMARY"


class ImgParser(html.parser.HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.images: list[str] = []
        self.text: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        if tag.lower() == "img":
            attr = dict(attrs)
            src = attr.get("src")
            if src:
                self.images.append(src)

    def handle_data(self, data: str) -> None:
        if data.strip():
            self.text.append(data.strip())


def load_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace") if path.exists() else ""


def find_index(root: Path) -> Path | None:
    candidates = [
        root / "index.html",
        root / "dist" / "index.html",
        root / "public" / "index.html",
        root / "build" / "index.html",
    ]
    for path in candidates:
        if path.exists():
            return path
    matches = list(root.rglob("index.html"))
    return matches[0] if matches else None


def resolve_asset(root: Path, manifest_path: Path, filename: str) -> Path:
    raw = Path(filename)
    candidates = []
    if raw.is_absolute():
        candidates.append(raw)
    else:
        candidates.extend([root / raw, manifest_path.parent / raw, root / "public" / raw])
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return candidates[0]


def image_size(path: Path) -> tuple[int, int] | None:
    if Image is None:
        return None
    try:
        with Image.open(path) as img:
            return int(img.width), int(img.height)
    except Exception:
        return None


def chrome_screenshot(url: str, out: Path, width: int, height: int) -> dict[str, Any]:
    chrome = os.environ.get("CHROME_BIN") or "/usr/bin/google-chrome"
    cmd = [
        chrome,
        "--headless=new",
        "--disable-gpu",
        "--no-sandbox",
        "--hide-scrollbars",
        f"--window-size={width},{height}",
        f"--screenshot={out}",
        url,
    ]
    proc = subprocess.run(cmd, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=60)
    return {
        "cmd": cmd,
        "returncode": proc.returncode,
        "output_tail": proc.stdout[-1200:],
        "path": str(out),
        "exists": out.exists(),
        "size_bytes": out.stat().st_size if out.exists() else 0,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("workspace", type=Path)
    parser.add_argument("--out", type=Path, default=None)
    args = parser.parse_args()

    root = args.workspace.resolve()
    out_dir = (args.out or root / "verification_artifacts").resolve()
    out_dir.mkdir(parents=True, exist_ok=True)
    errors: list[str] = []
    warnings: list[str] = []

    index = find_index(root)
    if not index:
        errors.append("missing index.html")
        index = root / "index.html"
    html = load_text(index)
    parser_obj = ImgParser()
    parser_obj.feed(html)
    all_text = " ".join(parser_obj.text)
    searchable = " ".join([html, *[load_text(p) for p in root.glob("*.js")], *[load_text(p) for p in root.glob("*.css")]])

    manifest_path = root / "image_manifest.json"
    if not manifest_path.exists():
        matches = list(root.rglob("image_manifest.json"))
        manifest_path = matches[0] if matches else manifest_path
    manifest: Any = None
    if manifest_path.exists():
        try:
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            errors.append(f"image_manifest.json invalid JSON: {exc}")
    else:
        errors.append("missing image_manifest.json")

    entries = manifest if isinstance(manifest, list) else manifest.get("images") if isinstance(manifest, dict) else []
    if not isinstance(entries, list):
        entries = []
        errors.append("image_manifest.json must be a list or contain images list")
    if len(entries) != 3:
        errors.append(f"expected exactly 3 manifest image entries, found {len(entries)}")

    image_rows = []
    generated_refs = 0
    for i, entry in enumerate(entries):
        if not isinstance(entry, dict):
            errors.append(f"manifest entry {i} is not an object")
            continue
        model = entry.get("model")
        filename = str(entry.get("filename") or entry.get("path") or "")
        prompt = str(entry.get("prompt") or "")
        if model != "gpt-image-2":
            errors.append(f"manifest entry {i} model is {model!r}, expected gpt-image-2")
        if len(prompt) < 30:
            errors.append(f"manifest entry {i} prompt is missing or too short")
        if not filename:
            errors.append(f"manifest entry {i} missing filename")
            continue
        path = resolve_asset(root, manifest_path, filename)
        if not path.exists():
            errors.append(f"manifest image missing: {filename}")
            continue
        size = image_size(path)
        if not size:
            errors.append(f"image did not decode: {path}")
            continue
        if min(size) < 900:
            errors.append(f"image too small: {path} {size[0]}x{size[1]}")
        rel_names = {filename, str(path.relative_to(root)) if path.is_relative_to(root) else path.name, path.name}
        if any(ref in html for ref in rel_names):
            generated_refs += 1
        image_rows.append({"filename": filename, "path": str(path), "width": size[0], "height": size[1], "bytes": path.stat().st_size})

    if generated_refs < 2:
        errors.append(f"rendered HTML references only {generated_refs} generated assets; expected at least 2")

    lower_text = all_text.lower()
    lower_search = searchable.lower()
    required_terms = ["lumadent", "toothpaste", "mint", "checkout"]
    for term in required_terms:
        if term not in lower_search:
            errors.append(f"missing required site term: {term}")
    if "$" not in searchable and "price" not in lower_search:
        errors.append("missing obvious price display")
    checkout_terms = ["cart", "quantity", "checkout", "email", "shipping", "confirm"]
    missing_checkout = [term for term in checkout_terms if term not in lower_search]
    if len(missing_checkout) > 1:
        errors.append(f"checkout affordances incomplete, missing: {missing_checkout}")
    if re.search(r"<img[^>]+src=[\"']https?://", html, flags=re.I):
        errors.append("external image URL found in img src")
    if re.search(r"sk-[A-Za-z0-9_-]{12,}", searchable):
        errors.append("possible OpenAI/API key string found in site files")

    screenshots = {}
    if index.exists():
        url = index.resolve().as_uri()
        shots = {
            "desktop_home": (url, 1440, 1000),
            "mobile_home": (url, 390, 844),
            "desktop_checkout": (url + "#checkout", 1440, 1000),
        }
        for name, (shot_url, width, height) in shots.items():
            rec = chrome_screenshot(shot_url, out_dir / f"{name}.png", width, height)
            screenshots[name] = rec
            if rec["returncode"] != 0 or not rec["exists"] or rec["size_bytes"] < 5000:
                errors.append(f"screenshot failed: {name}")

    summary = {
        "ok": not errors,
        "errors": errors,
        "warnings": warnings,
        "index": str(index),
        "image_manifest": str(manifest_path),
        "image_manifest_count": len(entries),
        "generated_assets_referenced": generated_refs,
        "images": image_rows,
        "screenshots": screenshots,
        "checkout_static_check": len(missing_checkout) <= 1,
        "console_error_count": None,
        "tests_passed": 1 if not errors else 0,
        "tests_total": 1,
    }
    (out_dir / "toothpaste_verify.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"{SUMMARY_PREFIX} {json.dumps(summary, sort_keys=True)}")
    return 0 if summary["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
