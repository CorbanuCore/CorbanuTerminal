from __future__ import annotations

import unittest

from textwright.pipeline import render_status_line


class VisibleIntegrationTests(unittest.TestCase):
    def test_literal_pipe_in_status_line_does_not_become_table(self) -> None:
        html = render_status_line("release status: green\\|blue")
        self.assertEqual(html, "<p>release status: green|blue</p>")


if __name__ == "__main__":
    unittest.main()
