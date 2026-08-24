from __future__ import annotations

import csv
import io
from dataclasses import dataclass
from typing import Any


class QueryForgeError(Exception):
    pass


class ExpressionSyntaxError(QueryForgeError):
    pass


def _cell(value: str) -> Any:
    value = value.strip()
    if value == "" or value.upper() == "NULL":
        return None
    return value


def load_csv(name: str, csv_text: str) -> "Table":
    return Table.from_csv(name, csv_text)


@dataclass
class Table:
    name: str
    columns: list[str]
    rows: list[dict[str, Any]]

    @classmethod
    def from_csv(cls, name: str, csv_text: str) -> "Table":
        reader = csv.DictReader(io.StringIO(csv_text))
        columns = list(reader.fieldnames or [])
        rows: list[dict[str, Any]] = []
        for row in reader:
            rows.append({column: _cell(row.get(column, "")) for column in columns})
        return cls(name, columns, rows)

    def filter(self, expression: str) -> "Table":
        expression = expression.strip()
        if "==" not in expression:
            raise ExpressionSyntaxError("baseline only handles equality")
        left, right = [part.strip() for part in expression.split("==", 1)]
        if right.startswith(("'", '"')) and right.endswith(("'", '"')):
            right = right[1:-1]
        kept = [row for row in self.rows if str(row.get(left)) == right]
        return Table(self.name, list(self.columns), kept)

    def project(self, *columns: str) -> "Table":
        projected = [{column: row.get(column) for column in columns} for row in self.rows]
        return Table(self.name, list(columns), projected)

    def join(self, other: "Table", left_key: str, right_key: str) -> "Table":
        raise NotImplementedError("join is not implemented in the baseline")

    def aggregate(self, group_by=None, metrics=None) -> "Table":
        raise NotImplementedError("aggregate is not implemented in the baseline")

    def order_by(self, *specs: str) -> "Table":
        return Table(self.name, list(self.columns), list(self.rows))

    def limit(self, count: int | None, offset: int = 0) -> "Table":
        end = None if count is None else offset + count
        return Table(self.name, list(self.columns), self.rows[offset:end])
