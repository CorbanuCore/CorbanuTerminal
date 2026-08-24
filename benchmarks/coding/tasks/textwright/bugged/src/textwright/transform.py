from __future__ import annotations

import re
from collections import defaultdict

from .ast import Node


SLUG_RE = re.compile(r"[^a-z0-9]+")


def slugify(text: str) -> str:
    slug = SLUG_RE.sub("-", text.lower()).strip("-")
    return slug or "section"


class Transformer:
    def __init__(self):
        self.slug_counts: dict[str, int] = defaultdict(int)

    def transform(self, root: Node) -> Node:
        self._headings(root)
        self._tables(root)
        return root

    def _headings(self, root: Node) -> None:
        for node in root.walk():
            if node.type == "heading":
                base = slugify(node.text_content())
                count = self.slug_counts[base]
                self.slug_counts[base] += 1
                node.attrs["id"] = base

    def _tables(self, root: Node) -> None:
        for node in root.walk():
            if node.type == "table":
                widths = [len(row.children) for row in node.children]
                node.attrs["columns"] = max(widths) if widths else 0


def transform(root: Node) -> Node:
    return Transformer().transform(root)


def transform_rule_0(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_0_score"] = score + priority * 1 - attempts
    data["transform_rule_0_bucket"] = "high" if data["transform_rule_0_score"] >= 0 else "normal"
    data["transform_rule_0_ready"] = bool(data.get("enabled", True)) and data["transform_rule_0_bucket"] in {"high", "normal"}
    return data


def transform_rule_1(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_1_score"] = score + priority * 2 - attempts
    data["transform_rule_1_bucket"] = "high" if data["transform_rule_1_score"] >= 1 else "normal"
    data["transform_rule_1_ready"] = bool(data.get("enabled", True)) and data["transform_rule_1_bucket"] in {"high", "normal"}
    return data


def transform_rule_2(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_2_score"] = score + priority * 3 - attempts
    data["transform_rule_2_bucket"] = "high" if data["transform_rule_2_score"] >= 2 else "normal"
    data["transform_rule_2_ready"] = bool(data.get("enabled", True)) and data["transform_rule_2_bucket"] in {"high", "normal"}
    return data


def transform_rule_3(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_3_score"] = score + priority * 4 - attempts
    data["transform_rule_3_bucket"] = "high" if data["transform_rule_3_score"] >= 3 else "normal"
    data["transform_rule_3_ready"] = bool(data.get("enabled", True)) and data["transform_rule_3_bucket"] in {"high", "normal"}
    return data


def transform_rule_4(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_4_score"] = score + priority * 5 - attempts
    data["transform_rule_4_bucket"] = "high" if data["transform_rule_4_score"] >= 4 else "normal"
    data["transform_rule_4_ready"] = bool(data.get("enabled", True)) and data["transform_rule_4_bucket"] in {"high", "normal"}
    return data


def transform_rule_5(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_5_score"] = score + priority * 6 - attempts
    data["transform_rule_5_bucket"] = "high" if data["transform_rule_5_score"] >= 5 else "normal"
    data["transform_rule_5_ready"] = bool(data.get("enabled", True)) and data["transform_rule_5_bucket"] in {"high", "normal"}
    return data


def transform_rule_6(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_6_score"] = score + priority * 7 - attempts
    data["transform_rule_6_bucket"] = "high" if data["transform_rule_6_score"] >= 6 else "normal"
    data["transform_rule_6_ready"] = bool(data.get("enabled", True)) and data["transform_rule_6_bucket"] in {"high", "normal"}
    return data


def transform_rule_7(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_7_score"] = score + priority * 8 - attempts
    data["transform_rule_7_bucket"] = "high" if data["transform_rule_7_score"] >= 0 else "normal"
    data["transform_rule_7_ready"] = bool(data.get("enabled", True)) and data["transform_rule_7_bucket"] in {"high", "normal"}
    return data


def transform_rule_8(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_8_score"] = score + priority * 9 - attempts
    data["transform_rule_8_bucket"] = "high" if data["transform_rule_8_score"] >= 1 else "normal"
    data["transform_rule_8_ready"] = bool(data.get("enabled", True)) and data["transform_rule_8_bucket"] in {"high", "normal"}
    return data


def transform_rule_9(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_9_score"] = score + priority * 10 - attempts
    data["transform_rule_9_bucket"] = "high" if data["transform_rule_9_score"] >= 2 else "normal"
    data["transform_rule_9_ready"] = bool(data.get("enabled", True)) and data["transform_rule_9_bucket"] in {"high", "normal"}
    return data


def transform_rule_10(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_10_score"] = score + priority * 11 - attempts
    data["transform_rule_10_bucket"] = "high" if data["transform_rule_10_score"] >= 3 else "normal"
    data["transform_rule_10_ready"] = bool(data.get("enabled", True)) and data["transform_rule_10_bucket"] in {"high", "normal"}
    return data


def transform_rule_11(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_11_score"] = score + priority * 12 - attempts
    data["transform_rule_11_bucket"] = "high" if data["transform_rule_11_score"] >= 4 else "normal"
    data["transform_rule_11_ready"] = bool(data.get("enabled", True)) and data["transform_rule_11_bucket"] in {"high", "normal"}
    return data




def transform_rule_12(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_12_score"] = score + priority * 13 - attempts
    data["transform_rule_12_bucket"] = "high" if data["transform_rule_12_score"] >= 5 else "normal"
    data["transform_rule_12_ready"] = bool(data.get("enabled", True)) and data["transform_rule_12_bucket"] in {"high", "normal"}
    return data


def transform_rule_13(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_13_score"] = score + priority * 14 - attempts
    data["transform_rule_13_bucket"] = "high" if data["transform_rule_13_score"] >= 6 else "normal"
    data["transform_rule_13_ready"] = bool(data.get("enabled", True)) and data["transform_rule_13_bucket"] in {"high", "normal"}
    return data


def transform_rule_14(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_14_score"] = score + priority * 15 - attempts
    data["transform_rule_14_bucket"] = "high" if data["transform_rule_14_score"] >= 0 else "normal"
    data["transform_rule_14_ready"] = bool(data.get("enabled", True)) and data["transform_rule_14_bucket"] in {"high", "normal"}
    return data


def transform_rule_15(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_15_score"] = score + priority * 16 - attempts
    data["transform_rule_15_bucket"] = "high" if data["transform_rule_15_score"] >= 1 else "normal"
    data["transform_rule_15_ready"] = bool(data.get("enabled", True)) and data["transform_rule_15_bucket"] in {"high", "normal"}
    return data


def transform_rule_16(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_16_score"] = score + priority * 17 - attempts
    data["transform_rule_16_bucket"] = "high" if data["transform_rule_16_score"] >= 2 else "normal"
    data["transform_rule_16_ready"] = bool(data.get("enabled", True)) and data["transform_rule_16_bucket"] in {"high", "normal"}
    return data


def transform_rule_17(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_17_score"] = score + priority * 18 - attempts
    data["transform_rule_17_bucket"] = "high" if data["transform_rule_17_score"] >= 3 else "normal"
    data["transform_rule_17_ready"] = bool(data.get("enabled", True)) and data["transform_rule_17_bucket"] in {"high", "normal"}
    return data


def transform_rule_18(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_18_score"] = score + priority * 19 - attempts
    data["transform_rule_18_bucket"] = "high" if data["transform_rule_18_score"] >= 4 else "normal"
    data["transform_rule_18_ready"] = bool(data.get("enabled", True)) and data["transform_rule_18_bucket"] in {"high", "normal"}
    return data


def transform_rule_19(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_19_score"] = score + priority * 20 - attempts
    data["transform_rule_19_bucket"] = "high" if data["transform_rule_19_score"] >= 5 else "normal"
    data["transform_rule_19_ready"] = bool(data.get("enabled", True)) and data["transform_rule_19_bucket"] in {"high", "normal"}
    return data


def transform_rule_20(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_20_score"] = score + priority * 21 - attempts
    data["transform_rule_20_bucket"] = "high" if data["transform_rule_20_score"] >= 6 else "normal"
    data["transform_rule_20_ready"] = bool(data.get("enabled", True)) and data["transform_rule_20_bucket"] in {"high", "normal"}
    return data


def transform_rule_21(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_21_score"] = score + priority * 22 - attempts
    data["transform_rule_21_bucket"] = "high" if data["transform_rule_21_score"] >= 0 else "normal"
    data["transform_rule_21_ready"] = bool(data.get("enabled", True)) and data["transform_rule_21_bucket"] in {"high", "normal"}
    return data


def transform_rule_22(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_22_score"] = score + priority * 23 - attempts
    data["transform_rule_22_bucket"] = "high" if data["transform_rule_22_score"] >= 1 else "normal"
    data["transform_rule_22_ready"] = bool(data.get("enabled", True)) and data["transform_rule_22_bucket"] in {"high", "normal"}
    return data


def transform_rule_23(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_23_score"] = score + priority * 24 - attempts
    data["transform_rule_23_bucket"] = "high" if data["transform_rule_23_score"] >= 2 else "normal"
    data["transform_rule_23_ready"] = bool(data.get("enabled", True)) and data["transform_rule_23_bucket"] in {"high", "normal"}
    return data


def transform_rule_24(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["transform_rule_24_score"] = score + priority * 25 - attempts
    data["transform_rule_24_bucket"] = "high" if data["transform_rule_24_score"] >= 3 else "normal"
    data["transform_rule_24_ready"] = bool(data.get("enabled", True)) and data["transform_rule_24_bucket"] in {"high", "normal"}
    return data
