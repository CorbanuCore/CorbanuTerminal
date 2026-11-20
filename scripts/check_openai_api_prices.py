#!/usr/bin/env python3
"""Compare the bundled catalogue's OpenAI API-key prices with OpenAI's price page.

Reads the Standard table embedded in https://developers.openai.com/api/docs/pricing
(or a saved copy of that page) and checks every `openai` row in
codex-rs/models-manager/models.json that states an API-key price: input, cached
input, cache writes and output, in USD per million tokens. A row the page lists
but the catalogue leaves unpriced, or doesn't carry at all, is reported too. Exit status 1 on any
mismatch. A catalogued row with an end date (`valid_through_utc`) that is not on
the page gets a WARNING line: a price cell that stopped reading as a number drops
its row from the table, and that row's date is the one to re-check.

Long-context tiers and promotional end dates are stated in prose on each model's
page (developers.openai.com/api/docs/models/<slug>), so this script prints them
for review rather than parsing them.

  python3 scripts/check_openai_api_prices.py [--page saved.html]
"""

from __future__ import annotations

import argparse
import html
import json
import re
import sys
import urllib.request
from decimal import Decimal
from pathlib import Path

PRICING_URL = "https://developers.openai.com/api/docs/pricing"
CATALOGUE = Path(__file__).resolve().parents[1] / "codex-rs/models-manager/models.json"
# A table row: a model name, then price cells that are a number or "-". The
# page also embeds option lists of the same shape (["surface", "lang", ...],
# ["standard", "batch", ...]); their string cells keep them from matching.
ROW = re.compile(r'\[1,\[\[0,"([^"]+)"\]((?:,\[0,(?:"-"|[0-9.]+)\])+)\]\]')
CELL = re.compile(r'\[0,("[^"]*"|[0-9.]+)\]')


def fetch(url: str) -> str:
    request = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(request, timeout=30) as response:
        return response.read().decode("utf-8", "replace")


def standard_table(page: str) -> dict[str, tuple[Decimal | None, ...]]:
    """The first all-models table on the page: Standard processing."""
    text = html.unescape(page)
    rows: dict[str, tuple[Decimal | None, ...]] = {}
    previous_end = None
    for match in ROW.finditer(text):
        # The Standard table is one contiguous run; the Batch table follows
        # after a gap of page markup.
        if previous_end is not None and match.start() - previous_end > 3000:
            break
        previous_end = match.end()
        name = match.group(1).split(" (")[0]
        cells = [cell.strip('"') for cell in CELL.findall(match.group(2))]
        values = tuple(None if cell == "-" else Decimal(cell) for cell in cells)
        # Four columns: input, cached, cache writes, output. Three: no writes.
        if len(values) == 3:
            values = (values[0], values[1], None, values[2])
        rows.setdefault(name, values)
    return rows


def milli(value: int | None) -> Decimal | None:
    return None if value is None else Decimal(value) / 1000


def catalogue_prices() -> tuple[dict[str, tuple[Decimal | None, ...]], set[str]]:
    """The `openai` rows' API-key prices, and the slugs whose prices have an end date."""
    prices = {}
    dated = set()
    models = json.loads(CATALOGUE.read_text())["models"]
    for shard in sorted(CATALOGUE.parent.glob("models-*.json")):
        models.extend(json.loads(shard.read_text())["models"])
    for model in models:
        orchestration = model.get("orchestration") or {}
        billing = orchestration.get("billing")
        if orchestration.get("provider_id") != "openai":
            continue
        if not billing:
            prices[model["slug"]] = None
            continue
        prefix = "api_key_" if billing["kind"] == "auth_dependent" else ""
        if f"{prefix}input_milli_usd_per_million_tokens" not in billing:
            continue
        prices[model["slug"]] = tuple(
            milli(billing.get(f"{prefix}{field}_milli_usd_per_million_tokens"))
            for field in ("input", "cached_input", "cache_write", "output")
        )
        tier = billing.get(f"{prefix}long_context")
        through = billing.get(f"{prefix}valid_through_utc")
        if through:
            dated.add(model["slug"])
        if tier or through:
            print(
                f"review {model['slug']}: long_context={tier} valid_through={through}"
            )
    return prices, dated


def compare(
    published: dict[str, tuple[Decimal | None, ...]],
    catalogue: dict[str, tuple[Decimal | None, ...]],
    dated: set[str],
) -> int:
    """Print one line per row and return the number of mismatches."""
    mismatches = 0
    for slug in sorted(set(published) - set(catalogue)):
        print(f"{slug}: on the page, not in the catalogue")
    for slug, stated in sorted(catalogue.items()):
        sheet = published.get(slug)
        if sheet is None:
            if slug in dated:
                print(
                    f"WARNING {slug}: dated catalogue row not on the page; "
                    f"catalogue states {stated}"
                )
            else:
                print(f"{slug}: not on the page; catalogue states {stated}")
            continue
        if stated != sheet:
            mismatches += 1
            print(f"MISMATCH {slug}: catalogue {stated} page {sheet}")
        else:
            print(f"ok {slug}: {sheet}")
    return mismatches


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--page", type=Path, help="saved copy of the pricing page")
    args = parser.parse_args()
    page = args.page.read_text() if args.page else fetch(PRICING_URL)
    published = standard_table(page)
    if not published:
        print("no Standard table found on the page", file=sys.stderr)
        return 2
    catalogue, dated = catalogue_prices()
    return 1 if compare(published, catalogue, dated) else 0


if __name__ == "__main__":
    sys.exit(main())
