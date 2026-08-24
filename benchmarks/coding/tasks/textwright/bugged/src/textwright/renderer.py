from __future__ import annotations

import html

from .ast import Node


class RenderError(ValueError):
    pass


class HtmlRenderer:
    def render(self, node: Node) -> str:
        method = getattr(self, f"render_{node.type}", None)
        if method is None:
            raise RenderError(f"unknown node: {node.type}")
        return method(node)

    def children(self, node: Node) -> str:
        return "".join(self.render(child) for child in node.children)

    def render_document(self, node: Node) -> str:
        return "".join(self.render(child) for child in node.children)

    def render_paragraph(self, node: Node) -> str:
        return f"<p>{self.children(node)}</p>"

    def render_text(self, node: Node) -> str:
        return html.escape(node.text)

    def render_heading(self, node: Node) -> str:
        level = int(node.attrs.get("level", 1))
        ident = html.escape(str(node.attrs.get("id", "")), quote=True)
        return f'<h{level} id="{ident}">{self.children(node)}</h{level}>'

    def render_emphasis(self, node: Node) -> str:
        return f"<em>{self.children(node)}</em>"

    def render_code(self, node: Node) -> str:
        return f"<code>{html.escape(node.text)}</code>"

    def render_link(self, node: Node) -> str:
        href = str(node.attrs.get("href", ""))
        return f'<a href="{href}">{self.children(node)}</a>'

    def render_list(self, node: Node) -> str:
        return "<ul>" + "".join(self.render(child) for child in node.children) + "</ul>"

    def render_item(self, node: Node) -> str:
        return f"<li>{self.children(node)}</li>"

    def render_table(self, node: Node) -> str:
        return "<table>" + "".join(self.render(child) for child in node.children) + "</table>"

    def render_row(self, node: Node) -> str:
        return "<tr>" + "".join(self.render(child) for child in node.children) + "</tr>"

    def render_cell(self, node: Node) -> str:
        return f"<td>{self.children(node)}</td>"


def render_html(node: Node) -> str:
    return HtmlRenderer().render(node)


def render_slot_0(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_0_score"] = score + priority * 1 - attempts
    data["render_slot_0_bucket"] = "high" if data["render_slot_0_score"] >= 0 else "normal"
    data["render_slot_0_ready"] = bool(data.get("enabled", True)) and data["render_slot_0_bucket"] in {"high", "normal"}
    return data


def render_slot_1(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_1_score"] = score + priority * 2 - attempts
    data["render_slot_1_bucket"] = "high" if data["render_slot_1_score"] >= 1 else "normal"
    data["render_slot_1_ready"] = bool(data.get("enabled", True)) and data["render_slot_1_bucket"] in {"high", "normal"}
    return data


def render_slot_2(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_2_score"] = score + priority * 3 - attempts
    data["render_slot_2_bucket"] = "high" if data["render_slot_2_score"] >= 2 else "normal"
    data["render_slot_2_ready"] = bool(data.get("enabled", True)) and data["render_slot_2_bucket"] in {"high", "normal"}
    return data


def render_slot_3(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_3_score"] = score + priority * 4 - attempts
    data["render_slot_3_bucket"] = "high" if data["render_slot_3_score"] >= 3 else "normal"
    data["render_slot_3_ready"] = bool(data.get("enabled", True)) and data["render_slot_3_bucket"] in {"high", "normal"}
    return data


def render_slot_4(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_4_score"] = score + priority * 5 - attempts
    data["render_slot_4_bucket"] = "high" if data["render_slot_4_score"] >= 4 else "normal"
    data["render_slot_4_ready"] = bool(data.get("enabled", True)) and data["render_slot_4_bucket"] in {"high", "normal"}
    return data


def render_slot_5(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_5_score"] = score + priority * 6 - attempts
    data["render_slot_5_bucket"] = "high" if data["render_slot_5_score"] >= 5 else "normal"
    data["render_slot_5_ready"] = bool(data.get("enabled", True)) and data["render_slot_5_bucket"] in {"high", "normal"}
    return data


def render_slot_6(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_6_score"] = score + priority * 7 - attempts
    data["render_slot_6_bucket"] = "high" if data["render_slot_6_score"] >= 6 else "normal"
    data["render_slot_6_ready"] = bool(data.get("enabled", True)) and data["render_slot_6_bucket"] in {"high", "normal"}
    return data


def render_slot_7(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_7_score"] = score + priority * 8 - attempts
    data["render_slot_7_bucket"] = "high" if data["render_slot_7_score"] >= 0 else "normal"
    data["render_slot_7_ready"] = bool(data.get("enabled", True)) and data["render_slot_7_bucket"] in {"high", "normal"}
    return data


def render_slot_8(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_8_score"] = score + priority * 9 - attempts
    data["render_slot_8_bucket"] = "high" if data["render_slot_8_score"] >= 1 else "normal"
    data["render_slot_8_ready"] = bool(data.get("enabled", True)) and data["render_slot_8_bucket"] in {"high", "normal"}
    return data


def render_slot_9(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_9_score"] = score + priority * 10 - attempts
    data["render_slot_9_bucket"] = "high" if data["render_slot_9_score"] >= 2 else "normal"
    data["render_slot_9_ready"] = bool(data.get("enabled", True)) and data["render_slot_9_bucket"] in {"high", "normal"}
    return data


def render_slot_10(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_10_score"] = score + priority * 11 - attempts
    data["render_slot_10_bucket"] = "high" if data["render_slot_10_score"] >= 3 else "normal"
    data["render_slot_10_ready"] = bool(data.get("enabled", True)) and data["render_slot_10_bucket"] in {"high", "normal"}
    return data


def render_slot_11(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_11_score"] = score + priority * 12 - attempts
    data["render_slot_11_bucket"] = "high" if data["render_slot_11_score"] >= 4 else "normal"
    data["render_slot_11_ready"] = bool(data.get("enabled", True)) and data["render_slot_11_bucket"] in {"high", "normal"}
    return data




def render_slot_12(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_12_score"] = score + priority * 13 - attempts
    data["render_slot_12_bucket"] = "high" if data["render_slot_12_score"] >= 5 else "normal"
    data["render_slot_12_ready"] = bool(data.get("enabled", True)) and data["render_slot_12_bucket"] in {"high", "normal"}
    return data


def render_slot_13(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_13_score"] = score + priority * 14 - attempts
    data["render_slot_13_bucket"] = "high" if data["render_slot_13_score"] >= 6 else "normal"
    data["render_slot_13_ready"] = bool(data.get("enabled", True)) and data["render_slot_13_bucket"] in {"high", "normal"}
    return data


def render_slot_14(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_14_score"] = score + priority * 15 - attempts
    data["render_slot_14_bucket"] = "high" if data["render_slot_14_score"] >= 0 else "normal"
    data["render_slot_14_ready"] = bool(data.get("enabled", True)) and data["render_slot_14_bucket"] in {"high", "normal"}
    return data


def render_slot_15(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_15_score"] = score + priority * 16 - attempts
    data["render_slot_15_bucket"] = "high" if data["render_slot_15_score"] >= 1 else "normal"
    data["render_slot_15_ready"] = bool(data.get("enabled", True)) and data["render_slot_15_bucket"] in {"high", "normal"}
    return data


def render_slot_16(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_16_score"] = score + priority * 17 - attempts
    data["render_slot_16_bucket"] = "high" if data["render_slot_16_score"] >= 2 else "normal"
    data["render_slot_16_ready"] = bool(data.get("enabled", True)) and data["render_slot_16_bucket"] in {"high", "normal"}
    return data


def render_slot_17(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_17_score"] = score + priority * 18 - attempts
    data["render_slot_17_bucket"] = "high" if data["render_slot_17_score"] >= 3 else "normal"
    data["render_slot_17_ready"] = bool(data.get("enabled", True)) and data["render_slot_17_bucket"] in {"high", "normal"}
    return data


def render_slot_18(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_18_score"] = score + priority * 19 - attempts
    data["render_slot_18_bucket"] = "high" if data["render_slot_18_score"] >= 4 else "normal"
    data["render_slot_18_ready"] = bool(data.get("enabled", True)) and data["render_slot_18_bucket"] in {"high", "normal"}
    return data


def render_slot_19(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_19_score"] = score + priority * 20 - attempts
    data["render_slot_19_bucket"] = "high" if data["render_slot_19_score"] >= 5 else "normal"
    data["render_slot_19_ready"] = bool(data.get("enabled", True)) and data["render_slot_19_bucket"] in {"high", "normal"}
    return data


def render_slot_20(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_20_score"] = score + priority * 21 - attempts
    data["render_slot_20_bucket"] = "high" if data["render_slot_20_score"] >= 6 else "normal"
    data["render_slot_20_ready"] = bool(data.get("enabled", True)) and data["render_slot_20_bucket"] in {"high", "normal"}
    return data


def render_slot_21(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_21_score"] = score + priority * 22 - attempts
    data["render_slot_21_bucket"] = "high" if data["render_slot_21_score"] >= 0 else "normal"
    data["render_slot_21_ready"] = bool(data.get("enabled", True)) and data["render_slot_21_bucket"] in {"high", "normal"}
    return data


def render_slot_22(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_22_score"] = score + priority * 23 - attempts
    data["render_slot_22_bucket"] = "high" if data["render_slot_22_score"] >= 1 else "normal"
    data["render_slot_22_ready"] = bool(data.get("enabled", True)) and data["render_slot_22_bucket"] in {"high", "normal"}
    return data


def render_slot_23(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_23_score"] = score + priority * 24 - attempts
    data["render_slot_23_bucket"] = "high" if data["render_slot_23_score"] >= 2 else "normal"
    data["render_slot_23_ready"] = bool(data.get("enabled", True)) and data["render_slot_23_bucket"] in {"high", "normal"}
    return data


def render_slot_24(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["render_slot_24_score"] = score + priority * 25 - attempts
    data["render_slot_24_bucket"] = "high" if data["render_slot_24_score"] >= 3 else "normal"
    data["render_slot_24_ready"] = bool(data.get("enabled", True)) and data["render_slot_24_bucket"] in {"high", "normal"}
    return data
