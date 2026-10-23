#!/usr/bin/env python3
"""Warn before a dated price-catalogue list runs out.

Two kinds of catalogue data in codex-rs/models-manager/models.json stop being
true on a known date:

- A `metered_schedule` row's `off_peak_dates_utc` (DeepSeek: Chinese public
  holidays are off-peak all day). The dates come from the State Council's
  yearly holiday notice, which covers one calendar year, so a list whose last
  date falls in year Y covers through Y-12-31. After that, a holiday is priced
  at the peak rate. Fails (exit 1) from `--days` days before that end.
- A `valid_through_utc` / `api_key_valid_through_utc` end (OpenAI's GPT-5.6 Sol
  promotion). After it the row states no price, never a stale one, so this only
  prints a warning from `--days` days before the end: re-read the page.

  python3 scripts/check_price_calendars.py [--today YYYY-MM-DD] [--days 30] [--warn-only]

`--warn-only` reports a calendar about to run out as a warning and exits 0
(pull-request CI); the scheduled nightly run fails instead.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import sys
from pathlib import Path

CATALOGUE = Path(__file__).resolve().parents[1] / "codex-rs/models-manager/models.json"


def billing_of(model: dict) -> dict | None:
    return (model.get("orchestration") or {}).get("billing")


def holiday_list_ends(models: list[dict]) -> dict[str, dt.date]:
    """Each scheduled row's coverage end: 31 December of its last listed year."""
    ends = {}
    for model in models:
        billing = billing_of(model)
        if not billing or billing.get("kind") != "metered_schedule":
            continue
        dates = billing.get("off_peak_dates_utc") or []
        if not dates:
            continue
        last = max(dt.date.fromisoformat(date) for date in dates)
        ends[model["slug"]] = dt.date(last.year, 12, 31)
    return ends


def validity_ends(models: list[dict]) -> dict[str, dt.datetime]:
    ends = {}
    for model in models:
        billing = billing_of(model) or {}
        through = billing.get("valid_through_utc") or billing.get(
            "api_key_valid_through_utc"
        )
        if through:
            ends[model["slug"]] = dt.datetime.fromisoformat(
                through.replace("Z", "+00:00")
            )
    return ends


def check(
    models: list[dict], today: dt.date, days: int, warn_only: bool
) -> tuple[int, list[str]]:
    lines = []
    failed = False
    horizon = dt.timedelta(days=days)
    for slug, end in sorted(holiday_list_ends(models).items()):
        if today + horizon < end:
            lines.append(f"ok {slug}: off-peak holiday dates cover through {end}")
            continue
        level = "warning" if warn_only else "error"
        failed |= not warn_only
        lines.append(
            f"::{level}::{slug}: off-peak holiday dates cover only through {end}; add the next "
            "year's Chinese public holidays from the State Council notice (DeepSeek bills them "
            "off-peak) before then"
        )
    for slug, end in sorted(validity_ends(models).items()):
        if end.date() < today:
            lines.append(
                f"{slug}: rates ended {end.isoformat()}; the row states no price since"
            )
        elif today + horizon >= end.date():
            lines.append(
                f"::warning::{slug}: published rates hold through {end.isoformat()}; re-read the "
                "provider's price page (scripts/check_openai_api_prices.py for OpenAI)"
            )
        else:
            lines.append(f"ok {slug}: rates hold through {end.isoformat()}")
    return (1 if failed else 0), lines


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--today",
        type=dt.date.fromisoformat,
        default=dt.datetime.now(dt.timezone.utc).date(),
    )
    parser.add_argument("--days", type=int, default=30)
    parser.add_argument("--warn-only", action="store_true")
    parser.add_argument("--catalogue", type=Path, default=CATALOGUE)
    args = parser.parse_args()
    models = json.loads(args.catalogue.read_text())["models"]
    status, lines = check(models, args.today, args.days, args.warn_only)
    print("\n".join(lines))
    return status


if __name__ == "__main__":
    sys.exit(main())
