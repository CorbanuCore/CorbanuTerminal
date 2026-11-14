from __future__ import annotations

import argparse
import importlib
import json
import sys
import traceback
from decimal import Decimal
from pathlib import Path
from typing import Any, Callable


TOTAL_TESTS = 12


def as_decimal(value: Any) -> Decimal:
    return Decimal(str(value))


def load_candidate(repo: Path):
    src = repo / "src"
    if not (src / "queryforge").exists():
        raise AssertionError(f"not a queryforge repo: {repo}")
    for name in list(sys.modules):
        if name == "queryforge" or name.startswith("queryforge."):
            del sys.modules[name]
    sys.path.insert(0, str(src))
    try:
        return importlib.import_module("queryforge")
    finally:
        try:
            sys.path.remove(str(src))
        except ValueError:
            pass


def test_01_precedence_and_arithmetic(api) -> None:
    table = api.load_csv(
        "items",
        "\n".join(
            [
                "id,qty,price,kind,flag",
                "1,2,12,a,",
                "2,10,2,b,x",
                "3,5,5,a,x",
            ]
        ),
    )
    result = table.filter("qty * price + 1 > 25 or kind == 'b' and not flag == NULL").project("id")
    assert [row["id"] for row in result.rows] == ["2", "3"]


def test_02_parentheses_override_boolean_precedence(api) -> None:
    table = api.load_csv(
        "items",
        "\n".join(
            [
                "id,qty,kind",
                "1,2,a",
                "2,5,b",
                "3,5,c",
                "4,8,a",
            ]
        ),
    )
    plain = table.filter("kind == 'b' or kind == 'a' and qty < 3")
    grouped = table.filter("(kind == 'b' or kind == 'a') and qty < 3")
    assert [row["id"] for row in plain.rows] == ["1", "2"]
    assert [row["id"] for row in grouped.rows] == ["1"]


def test_03_malformed_expression_has_defined_error(api) -> None:
    table = api.load_csv("x", "id,qty\n1,2\n")
    try:
        table.filter("qty * and 2")
    except api.ExpressionSyntaxError as exc:
        assert "unexpected" in str(exc).lower() or "syntax" in str(exc).lower()
    else:
        raise AssertionError("malformed expression did not raise ExpressionSyntaxError")


def test_04_null_comparisons_never_match(api) -> None:
    table = api.load_csv(
        "tickets",
        "\n".join(
            [
                "id,status,score",
                "1,,5",
                "2,NULL,7",
                "3,open,",
            ]
        ),
    )
    assert table.filter("status == NULL").rows == []
    assert table.filter("status != NULL").rows == []
    assert [row["id"] for row in table.filter("score + 1 > 6").rows] == ["2"]


def test_05_numeric_strings_and_literals_coerce_for_math(api) -> None:
    table = api.load_csv(
        "lines",
        "\n".join(
            [
                "id,qty,price",
                "a,2,10.50",
                "b,5,4",
                "c,10,3",
            ]
        ),
    )
    result = table.filter("qty >= 5 and qty * price >= 20").order_by("qty desc").project("id")
    assert [row["id"] for row in result.rows] == ["c", "b"]


def test_06_join_fanout_and_duplicate_column_names(api) -> None:
    orders = api.load_csv(
        "orders",
        "\n".join(
            [
                "id,customer_id,total",
                "o1,c1,10",
                "o2,c1,15",
                "o3,c2,20",
            ]
        ),
    )
    customers = api.load_csv(
        "customers",
        "\n".join(
            [
                "id,tier",
                "c1,gold",
                "c1,trial",
                "c2,silver",
            ]
        ),
    )
    joined = orders.join(customers, "customer_id", "id")
    assert joined.columns == ["id", "customer_id", "total", "customers.id", "tier"]
    assert len(joined.rows) == 5
    assert [(row["id"], row["customers.id"], row["tier"]) for row in joined.rows[:2]] == [
        ("o1", "c1", "gold"),
        ("o1", "c1", "trial"),
    ]


def test_07_group_by_null_and_aggregates_skip_nulls(api) -> None:
    sales = api.load_csv(
        "sales",
        "\n".join(
            [
                "id,region,amount",
                "1,East,10",
                "2,East,",
                "3,NULL,5",
                "4,,7",
            ]
        ),
    )
    grouped = sales.aggregate(
        group_by=["region"],
        metrics={
            "rows": ("count", "*"),
            "amounts": ("count", "amount"),
            "total": ("sum", "amount"),
            "avg": ("avg", "amount"),
        },
    ).order_by("region asc")
    assert grouped.rows[0]["region"] == "East"
    assert grouped.rows[0]["rows"] == 2
    assert grouped.rows[0]["amounts"] == 1
    assert as_decimal(grouped.rows[0]["total"]) == Decimal("10")
    assert as_decimal(grouped.rows[0]["avg"]) == Decimal("10")
    assert grouped.rows[1]["region"] is None
    assert grouped.rows[1]["rows"] == 2
    assert grouped.rows[1]["amounts"] == 2
    assert as_decimal(grouped.rows[1]["total"]) == Decimal("12")
    assert as_decimal(grouped.rows[1]["avg"]) == Decimal("6")


def test_08_aggregate_empty_table_identities(api) -> None:
    sales = api.load_csv("sales", "id,region,amount\n1,East,10\n")
    empty = sales.filter("amount > 100")
    summary = empty.aggregate(
        metrics={
            "rows": ("count", "*"),
            "non_null": ("count", "amount"),
            "total": ("sum", "amount"),
            "avg": ("avg", "amount"),
            "lo": ("min", "amount"),
            "hi": ("max", "amount"),
        }
    )
    assert summary.rows == [{"rows": 0, "non_null": 0, "total": 0, "avg": None, "lo": None, "hi": None}]
    assert empty.aggregate(group_by=["region"], metrics={"rows": ("count", "*")}).rows == []


def test_09_stable_multikey_order_with_nulls_last(api) -> None:
    table = api.load_csv(
        "scores",
        "\n".join(
            [
                "id,grp,score",
                "a,A,2",
                "b,A,",
                "c,A,2",
                "d,B,3",
                "e,B,1",
                "f,,9",
            ]
        ),
    )
    ordered = table.order_by("grp asc", "score desc")
    assert [row["id"] for row in ordered.rows] == ["a", "c", "b", "d", "e", "f"]


def test_10_limit_offset_edges(api) -> None:
    table = api.load_csv("items", "id\n1\n2\n3\n4\n")
    assert [row["id"] for row in table.limit(2, offset=1).rows] == ["2", "3"]
    assert [row["id"] for row in table.limit(None, offset=2).rows] == ["3", "4"]
    for args in [(-1, 0), (1, -1)]:
        try:
            table.limit(args[0], offset=args[1])
        except api.QueryForgeError:
            pass
        else:
            raise AssertionError(f"limit{args!r} did not raise QueryForgeError")


def test_11_min_max_use_numeric_order_when_values_are_strings(api) -> None:
    table = api.load_csv("numbers", "id,value\n1,10\n2,2\n3,30\n")
    result = table.aggregate(metrics={"lo": ("min", "value"), "hi": ("max", "value")})
    assert result.rows[0]["lo"] == "2"
    assert result.rows[0]["hi"] == "30"


def test_12_manual_table_empty_pipeline_preserves_columns(api) -> None:
    table = api.Table("manual", ["id", "kind", "value"], [{"id": "1", "kind": "x", "value": "5"}])
    empty = table.filter("kind == 'missing'")
    assert empty.columns == ["id", "kind", "value"]
    projected = empty.project("id", "value").order_by("value desc").limit(5)
    assert projected.columns == ["id", "value"]
    assert projected.rows == []


TESTS: list[tuple[str, Callable[[Any], None]]] = [
    ("precedence_and_arithmetic", test_01_precedence_and_arithmetic),
    ("parentheses_override_boolean_precedence", test_02_parentheses_override_boolean_precedence),
    ("malformed_expression_has_defined_error", test_03_malformed_expression_has_defined_error),
    ("null_comparisons_never_match", test_04_null_comparisons_never_match),
    ("numeric_strings_and_literals_coerce_for_math", test_05_numeric_strings_and_literals_coerce_for_math),
    ("join_fanout_and_duplicate_column_names", test_06_join_fanout_and_duplicate_column_names),
    ("group_by_null_and_aggregates_skip_nulls", test_07_group_by_null_and_aggregates_skip_nulls),
    ("aggregate_empty_table_identities", test_08_aggregate_empty_table_identities),
    ("stable_multikey_order_with_nulls_last", test_09_stable_multikey_order_with_nulls_last),
    ("limit_offset_edges", test_10_limit_offset_edges),
    ("min_max_use_numeric_order_when_values_are_strings", test_11_min_max_use_numeric_order_when_values_are_strings),
    ("manual_table_empty_pipeline_preserves_columns", test_12_manual_table_empty_pipeline_preserves_columns),
]


def run(repo: Path) -> tuple[list[str], dict[str, Any]]:
    lines: list[str] = []
    failures: list[dict[str, str]] = []
    try:
        api = load_candidate(repo)
    except Exception as exc:  # noqa: BLE001 - verifier should report import failures.
        failures.append({"test": "import", "error": "".join(traceback.format_exception_only(type(exc), exc)).strip()})
        return lines, {"passed": 0, "total": TOTAL_TESTS, "ok": False, "failures": failures}
    passed = 0
    for name, func in TESTS:
        try:
            func(api)
        except Exception as exc:  # noqa: BLE001 - record partial credit for any failure type.
            lines.append(f"FAIL {name}: {exc}")
            failures.append({"test": name, "error": traceback.format_exc(limit=8)})
        else:
            passed += 1
            lines.append(f"PASS {name}")
    return lines, {"passed": passed, "total": len(TESTS), "ok": passed == len(TESTS), "failures": failures}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("repo", help="Benchmark repo copy to verify")
    args = parser.parse_args()
    repo = Path(args.repo).resolve()
    lines, summary = run(repo)
    for line in lines:
        print(line)
    public_summary = {"passed": summary["passed"], "total": summary["total"], "ok": summary["ok"]}
    print("QUERYFORGE_VERIFIER_SUMMARY", json.dumps(public_summary, sort_keys=True))
    if summary["failures"]:
        print("QUERYFORGE_VERIFIER_FAILURES", json.dumps(summary["failures"], sort_keys=True))
    return 0 if summary["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
