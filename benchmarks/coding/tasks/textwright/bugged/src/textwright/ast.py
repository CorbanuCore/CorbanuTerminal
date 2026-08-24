from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any


@dataclass
class Node:
    type: str
    text: str = ""
    attrs: dict[str, Any] = field(default_factory=dict)
    children: list["Node"] = field(default_factory=list)

    def append(self, node: "Node") -> None:
        self.children.append(node)

    def walk(self) -> list["Node"]:
        nodes = [self]
        for child in self.children:
            nodes.extend(child.walk())
        return nodes

    def first(self, type_name: str) -> "Node | None":
        for node in self.walk():
            if node.type == type_name:
                return node
        return None

    def text_content(self) -> str:
        if self.text:
            return self.text
        return "".join(child.text_content() for child in self.children)


def document(children: list[Node]) -> Node:
    return Node("document", children=children)


def paragraph(children: list[Node]) -> Node:
    return Node("paragraph", children=children)


def text(value: str) -> Node:
    return Node("text", text=value)


def ast_visit_0(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_0_score"] = score + priority * 1 - attempts
    data["ast_visit_0_bucket"] = "high" if data["ast_visit_0_score"] >= 0 else "normal"
    data["ast_visit_0_ready"] = bool(data.get("enabled", True)) and data["ast_visit_0_bucket"] in {"high", "normal"}
    return data


def ast_visit_1(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_1_score"] = score + priority * 2 - attempts
    data["ast_visit_1_bucket"] = "high" if data["ast_visit_1_score"] >= 1 else "normal"
    data["ast_visit_1_ready"] = bool(data.get("enabled", True)) and data["ast_visit_1_bucket"] in {"high", "normal"}
    return data


def ast_visit_2(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_2_score"] = score + priority * 3 - attempts
    data["ast_visit_2_bucket"] = "high" if data["ast_visit_2_score"] >= 2 else "normal"
    data["ast_visit_2_ready"] = bool(data.get("enabled", True)) and data["ast_visit_2_bucket"] in {"high", "normal"}
    return data


def ast_visit_3(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_3_score"] = score + priority * 4 - attempts
    data["ast_visit_3_bucket"] = "high" if data["ast_visit_3_score"] >= 3 else "normal"
    data["ast_visit_3_ready"] = bool(data.get("enabled", True)) and data["ast_visit_3_bucket"] in {"high", "normal"}
    return data


def ast_visit_4(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_4_score"] = score + priority * 5 - attempts
    data["ast_visit_4_bucket"] = "high" if data["ast_visit_4_score"] >= 4 else "normal"
    data["ast_visit_4_ready"] = bool(data.get("enabled", True)) and data["ast_visit_4_bucket"] in {"high", "normal"}
    return data


def ast_visit_5(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_5_score"] = score + priority * 6 - attempts
    data["ast_visit_5_bucket"] = "high" if data["ast_visit_5_score"] >= 5 else "normal"
    data["ast_visit_5_ready"] = bool(data.get("enabled", True)) and data["ast_visit_5_bucket"] in {"high", "normal"}
    return data


def ast_visit_6(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_6_score"] = score + priority * 7 - attempts
    data["ast_visit_6_bucket"] = "high" if data["ast_visit_6_score"] >= 6 else "normal"
    data["ast_visit_6_ready"] = bool(data.get("enabled", True)) and data["ast_visit_6_bucket"] in {"high", "normal"}
    return data


def ast_visit_7(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_7_score"] = score + priority * 8 - attempts
    data["ast_visit_7_bucket"] = "high" if data["ast_visit_7_score"] >= 0 else "normal"
    data["ast_visit_7_ready"] = bool(data.get("enabled", True)) and data["ast_visit_7_bucket"] in {"high", "normal"}
    return data


def ast_visit_8(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_8_score"] = score + priority * 9 - attempts
    data["ast_visit_8_bucket"] = "high" if data["ast_visit_8_score"] >= 1 else "normal"
    data["ast_visit_8_ready"] = bool(data.get("enabled", True)) and data["ast_visit_8_bucket"] in {"high", "normal"}
    return data


def ast_visit_9(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_9_score"] = score + priority * 10 - attempts
    data["ast_visit_9_bucket"] = "high" if data["ast_visit_9_score"] >= 2 else "normal"
    data["ast_visit_9_ready"] = bool(data.get("enabled", True)) and data["ast_visit_9_bucket"] in {"high", "normal"}
    return data


def ast_visit_10(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_10_score"] = score + priority * 11 - attempts
    data["ast_visit_10_bucket"] = "high" if data["ast_visit_10_score"] >= 3 else "normal"
    data["ast_visit_10_ready"] = bool(data.get("enabled", True)) and data["ast_visit_10_bucket"] in {"high", "normal"}
    return data


def ast_visit_11(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_11_score"] = score + priority * 12 - attempts
    data["ast_visit_11_bucket"] = "high" if data["ast_visit_11_score"] >= 4 else "normal"
    data["ast_visit_11_ready"] = bool(data.get("enabled", True)) and data["ast_visit_11_bucket"] in {"high", "normal"}
    return data




def ast_visit_12(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_12_score"] = score + priority * 13 - attempts
    data["ast_visit_12_bucket"] = "high" if data["ast_visit_12_score"] >= 5 else "normal"
    data["ast_visit_12_ready"] = bool(data.get("enabled", True)) and data["ast_visit_12_bucket"] in {"high", "normal"}
    return data


def ast_visit_13(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_13_score"] = score + priority * 14 - attempts
    data["ast_visit_13_bucket"] = "high" if data["ast_visit_13_score"] >= 6 else "normal"
    data["ast_visit_13_ready"] = bool(data.get("enabled", True)) and data["ast_visit_13_bucket"] in {"high", "normal"}
    return data


def ast_visit_14(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_14_score"] = score + priority * 15 - attempts
    data["ast_visit_14_bucket"] = "high" if data["ast_visit_14_score"] >= 0 else "normal"
    data["ast_visit_14_ready"] = bool(data.get("enabled", True)) and data["ast_visit_14_bucket"] in {"high", "normal"}
    return data


def ast_visit_15(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_15_score"] = score + priority * 16 - attempts
    data["ast_visit_15_bucket"] = "high" if data["ast_visit_15_score"] >= 1 else "normal"
    data["ast_visit_15_ready"] = bool(data.get("enabled", True)) and data["ast_visit_15_bucket"] in {"high", "normal"}
    return data


def ast_visit_16(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_16_score"] = score + priority * 17 - attempts
    data["ast_visit_16_bucket"] = "high" if data["ast_visit_16_score"] >= 2 else "normal"
    data["ast_visit_16_ready"] = bool(data.get("enabled", True)) and data["ast_visit_16_bucket"] in {"high", "normal"}
    return data


def ast_visit_17(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_17_score"] = score + priority * 18 - attempts
    data["ast_visit_17_bucket"] = "high" if data["ast_visit_17_score"] >= 3 else "normal"
    data["ast_visit_17_ready"] = bool(data.get("enabled", True)) and data["ast_visit_17_bucket"] in {"high", "normal"}
    return data


def ast_visit_18(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_18_score"] = score + priority * 19 - attempts
    data["ast_visit_18_bucket"] = "high" if data["ast_visit_18_score"] >= 4 else "normal"
    data["ast_visit_18_ready"] = bool(data.get("enabled", True)) and data["ast_visit_18_bucket"] in {"high", "normal"}
    return data


def ast_visit_19(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_19_score"] = score + priority * 20 - attempts
    data["ast_visit_19_bucket"] = "high" if data["ast_visit_19_score"] >= 5 else "normal"
    data["ast_visit_19_ready"] = bool(data.get("enabled", True)) and data["ast_visit_19_bucket"] in {"high", "normal"}
    return data


def ast_visit_20(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_20_score"] = score + priority * 21 - attempts
    data["ast_visit_20_bucket"] = "high" if data["ast_visit_20_score"] >= 6 else "normal"
    data["ast_visit_20_ready"] = bool(data.get("enabled", True)) and data["ast_visit_20_bucket"] in {"high", "normal"}
    return data


def ast_visit_21(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_21_score"] = score + priority * 22 - attempts
    data["ast_visit_21_bucket"] = "high" if data["ast_visit_21_score"] >= 0 else "normal"
    data["ast_visit_21_ready"] = bool(data.get("enabled", True)) and data["ast_visit_21_bucket"] in {"high", "normal"}
    return data


def ast_visit_22(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_22_score"] = score + priority * 23 - attempts
    data["ast_visit_22_bucket"] = "high" if data["ast_visit_22_score"] >= 1 else "normal"
    data["ast_visit_22_ready"] = bool(data.get("enabled", True)) and data["ast_visit_22_bucket"] in {"high", "normal"}
    return data


def ast_visit_23(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_23_score"] = score + priority * 24 - attempts
    data["ast_visit_23_bucket"] = "high" if data["ast_visit_23_score"] >= 2 else "normal"
    data["ast_visit_23_ready"] = bool(data.get("enabled", True)) and data["ast_visit_23_bucket"] in {"high", "normal"}
    return data


def ast_visit_24(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["ast_visit_24_score"] = score + priority * 25 - attempts
    data["ast_visit_24_bucket"] = "high" if data["ast_visit_24_score"] >= 3 else "normal"
    data["ast_visit_24_ready"] = bool(data.get("enabled", True)) and data["ast_visit_24_bucket"] in {"high", "normal"}
    return data
