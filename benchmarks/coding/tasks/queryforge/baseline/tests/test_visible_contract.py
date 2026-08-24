from __future__ import annotations

import unittest

from queryforge import load_csv


class VisibleContractTests(unittest.TestCase):
    def test_filter_project_and_limit_basic_csv_rows(self) -> None:
        table = load_csv(
            "people",
            "id,name,city\n1,Ada,London\n2,Grace,New York\n3,Lin,London\n",
        )

        result = table.filter("city == 'London'").project("name").limit(1)

        self.assertEqual(result.columns, ["name"])
        self.assertEqual(result.rows, [{"name": "Ada"}])


if __name__ == "__main__":
    unittest.main()
