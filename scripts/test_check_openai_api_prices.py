from contextlib import redirect_stdout
from decimal import Decimal
from io import StringIO
from pathlib import Path
import unittest

from scripts.check_openai_api_prices import compare, standard_table

PAGE = Path(__file__).resolve().parent / "fixtures/openai-pricing-20261119-trimmed.html"


def prices(*cells):
    return tuple(None if cell is None else Decimal(cell) for cell in cells)


class StandardTableTests(unittest.TestCase):
    def setUp(self):
        self.rows = standard_table(PAGE.read_text())

    def test_option_lists_before_the_tables_are_not_price_rows(self):
        # The page embeds lists such as ["surface", "lang", ...] and
        # ["standard", "batch", ...] in the same shape as a table row.
        for name in ("surface", "short", "standard"):
            self.assertNotIn(name, self.rows)
        self.assertEqual(next(iter(self.rows)), "gpt-6.2-astra")
        self.assertEqual(len(self.rows), 17)

    def test_reads_the_standard_table_and_stops_before_batch(self):
        self.assertEqual(self.rows["gpt-5.6-sol"], prices("4", "0.4", "5", "20"))
        # Batch lists gpt-6.2-astra at half price; Standard must win.
        self.assertEqual(self.rows["gpt-6.2-astra"], prices("8", "0.4", "10", "40"))

    def test_dash_is_no_price_and_three_columns_have_no_cache_writes(self):
        self.assertEqual(self.rows["gpt-5.5"], prices("5", "0.5", None, "30"))
        self.assertEqual(
            self.rows["gpt-5.4-mini"], prices("0.75", "0.075", None, "4.5")
        )
        self.assertEqual(self.rows["gpt-5.2-pro"], prices("21", None, None, "168"))


class MissingDatedRowTests(unittest.TestCase):
    SOL = "[[0,&quot;gpt-5.6-sol&quot;],[0,4],"

    def setUp(self):
        page = PAGE.read_text()
        self.assertEqual(page.count(self.SOL), 1)
        # The fixture with one Sol price cell turned into a string other than "-".
        broken = "[[0,&quot;gpt-5.6-sol&quot;],[0,&quot;$4.00&quot;],"
        self.rows = standard_table(page.replace(self.SOL, broken))

    def test_a_dated_row_missing_from_the_page_warns(self):
        self.assertNotIn("gpt-5.6-sol", self.rows)
        sol = prices("4", "0.4", "5", "20")
        codex = prices("1.75", "0.175", None, "14")
        out = StringIO()
        with redirect_stdout(out):
            mismatches = compare(
                self.rows,
                {"gpt-5.6-sol": sol, "gpt-5.3-codex": codex},
                {"gpt-5.6-sol"},
            )
        lines = out.getvalue().splitlines()
        self.assertEqual(mismatches, 0)
        self.assertIn(
            f"WARNING gpt-5.6-sol: dated catalogue row not on the page; "
            f"catalogue states {sol}",
            lines,
        )
        # An undated row that is not on the page keeps the plain line.
        self.assertIn(
            f"gpt-5.3-codex: not on the page; catalogue states {codex}", lines
        )


if __name__ == "__main__":
    unittest.main()
