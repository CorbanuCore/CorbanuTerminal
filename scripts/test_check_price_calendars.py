import datetime as dt
import json
import unittest

from scripts.check_price_calendars import CATALOGUE
from scripts.check_price_calendars import check


def schedule(slug, dates):
    return {
        "slug": slug,
        "orchestration": {
            "billing": {"kind": "metered_schedule", "off_peak_dates_utc": dates}
        },
    }


def promotion(slug, through):
    return {
        "slug": slug,
        "orchestration": {
            "billing": {"kind": "auth_dependent", "api_key_valid_through_utc": through}
        },
    }


class PriceCalendarTests(unittest.TestCase):
    MODELS = [
        schedule("deepseek", ["2026-09-25", "2026-10-07"]),
        promotion("sol", "2026-11-21T23:59:59.999Z"),
    ]

    def run_check(self, today, warn_only=False):
        return check(
            self.MODELS, dt.date.fromisoformat(today), days=30, warn_only=warn_only
        )

    def test_holiday_list_covers_its_last_year_and_fails_thirty_days_before(self):
        status, lines = self.run_check("2026-11-30")
        self.assertEqual(status, 0)
        self.assertIn(
            "ok deepseek: off-peak holiday dates cover through 2026-12-31", lines
        )
        status, lines = self.run_check("2026-12-01")
        self.assertEqual(status, 1)
        self.assertTrue(any(line.startswith("::error::deepseek:") for line in lines))

    def test_warn_only_reports_without_failing(self):
        status, lines = self.run_check("2026-12-01", warn_only=True)
        self.assertEqual(status, 0)
        self.assertTrue(any(line.startswith("::warning::deepseek:") for line in lines))

    def test_promotion_end_warns_but_never_fails(self):
        for today, prefix in [
            ("2026-10-21", "ok sol"),
            ("2026-10-22", "::warning::sol"),
            ("2026-11-21", "::warning::sol"),
            ("2026-11-22", "sol: rates ended"),
        ]:
            _, lines = self.run_check(today)
            self.assertTrue(
                any(line.startswith(prefix) for line in lines), (today, lines)
            )
        status, _ = check(
            self.MODELS[1:], dt.date(2026, 11, 1), days=30, warn_only=False
        )
        self.assertEqual(status, 0)

    def test_bundled_catalogue_parses(self):
        models = json.loads(CATALOGUE.read_text())["models"]
        status, lines = check(models, dt.date(2026, 10, 14), days=30, warn_only=False)
        self.assertEqual(status, 0, lines)
        self.assertTrue(
            any(line.startswith("ok deepseek-v4-pro:") for line in lines), lines
        )


if __name__ == "__main__":
    unittest.main()
