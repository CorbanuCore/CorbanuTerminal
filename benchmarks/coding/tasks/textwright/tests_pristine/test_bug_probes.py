from __future__ import annotations

import unittest

from textwright import parse, render
from textwright.transform import transform


class BugProbeTests(unittest.TestCase):
    def test_probe_bug1_escaped_pipe_is_text_not_table_split(self) -> None:
        html = render("value a\\|b")
        self.assertEqual(html, "<p>value a|b</p>")

    def test_probe_bug2_heading_level_matches_hash_count(self) -> None:
        self.assertIn("<h2", render("## Title"))

    def test_probe_bug3_duplicate_heading_slugs_are_disambiguated(self) -> None:
        html = render("# Same\n# Same")
        self.assertIn('id="same"', html)
        self.assertIn('id="same-2"', html)

    def test_probe_bug4_link_href_is_attribute_escaped(self) -> None:
        html = render('[x](https://e.test/?a="b")')
        self.assertIn('&quot;b&quot;', html)

    def test_probe_bug5_inline_code_preserves_stars_without_emphasis(self) -> None:
        html = render("`*literal*`")
        self.assertEqual(html, "<p><code>*literal*</code></p>")


if __name__ == "__main__":
    unittest.main()
