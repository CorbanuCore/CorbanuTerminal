You are implementing the QueryForge gnarly benchmark task.

Work in this repository only. Do not remove tests. Do not bypass the verifier.

Goal:
Implement `queryforge`, a small in-memory query engine over CSV-loaded tables.
It must parse expressions, filter rows, project columns, join tables, aggregate,
sort, and slice results with stable, deterministic semantics.

Core API:

```python
from queryforge import ExpressionSyntaxError, QueryForgeError, Table, load_csv

people = load_csv(
    "people",
    "id,name,age,city\n1,Ada,37,London\n2,Grace,31,New York\n",
)

result = (
    people
    .filter("age >= 30 and city != 'Boston'")
    .project("name", "age")
    .order_by("age desc", "name asc")
    .limit(10, offset=0)
)

assert result.rows == [
    {"name": "Ada", "age": "37"},
    {"name": "Grace", "age": "31"},
]
```

The public surface must include:

```python
load_csv(name: str, csv_text: str) -> Table
Table.from_csv(name: str, csv_text: str) -> Table
Table(name: str, columns: list[str], rows: list[dict])
Table.rows
Table.columns
Table.filter(expression: str) -> Table
Table.project(*columns: str) -> Table
Table.join(other: Table, left_key: str, right_key: str) -> Table
Table.aggregate(group_by: list[str] | None = None, metrics: dict | None = None) -> Table
Table.order_by(*specs: str) -> Table
Table.limit(count: int | None, offset: int = 0) -> Table
```

Required expression semantics:

1. Expressions support comparison operators `==`, `!=`, `<`, `<=`, `>`, `>=`.
2. Boolean operators are `and`, `or`, and unary `not`.
3. Arithmetic operators are `+`, `-`, `*`, and `/`.
4. Precedence must be standard: parentheses, unary operators, multiplication
   and division, addition and subtraction, comparisons, `not`, `and`, `or`.
5. String literals may be single-quoted or double-quoted.
6. Numeric literals support integers and decimals.
7. Column references are bare identifiers and may contain letters, digits,
   underscores, and dots.
8. `NULL` is a literal null value.
9. Malformed expressions must raise `ExpressionSyntaxError` with a useful
   message. Do not silently treat bad syntax as a no-match filter.

Required table operations:

1. CSV loading uses the first row as headers and returns all non-empty cells as
   strings. Blank cells and cells equal to `NULL` case-insensitively become
   Python `None`.
2. `filter(expression)` returns rows where the expression evaluates truthy.
3. Any comparison involving `NULL` is false, including `col == NULL`.
4. Arithmetic involving `NULL` returns `NULL`; comparisons then no-match.
5. Numeric-looking strings and numeric literals must compare and calculate as
   numbers. `"10" > 2` is true and `qty * price > 20` works for CSV strings.
6. `project(*columns)` returns only the requested columns in order.
7. `join(other, left_key, right_key)` is an inner join using explicit keys.
8. Join fan-out is required: duplicate matching keys produce every matching
   pair.
9. For duplicate column names during joins, keep left column names unchanged and
   prefix conflicting right columns with `<right_table_name>.`. If that still
   conflicts, use `<right_table_name>_<column>`, then add numeric suffixes as
   needed.
10. `aggregate(group_by, metrics)` supports metric specs shaped as
    `{"alias": ("count"|"sum"|"avg"|"min"|"max", column)}`. For `count`, column
    may be `"*"`.
11. Aggregates skip `NULL` values. `count("*")` counts rows. `sum` over no
    values is `0`; `avg`, `min`, and `max` over no values are `None`.
12. Group-by treats `NULL` as an ordinary key value.
13. Aggregating an empty table with no group-by returns one row containing the
    aggregate identities. Aggregating an empty table with group-by returns no
    rows.
14. `order_by(*specs)` accepts specs like `"name asc"` and `"age desc"`.
15. Ordering supports multiple keys, is stable, and places `NULL` values last
    for both ascending and descending sorts.
16. `limit(count, offset=0)` returns a sliced table. `count=None` means no
    upper bound. Negative counts or offsets must raise `QueryForgeError`.
17. Empty tables and empty results must preserve column metadata and remain
    usable by later operations.
18. No network dependencies or third-party packages.

Definition of done:

- `python3 -m unittest discover -s tests` passes.
- The benchmark harness's external verifier passes.
- Keep code readable and scoped.
