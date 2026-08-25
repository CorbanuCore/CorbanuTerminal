from __future__ import annotations

from .parser import parse
from .renderer import render_html
from .transform import transform


def render(source: str) -> str:
    return render_html(transform(parse(source)))


def render_many(sources: list[str]) -> list[str]:
    return [render(source) for source in sources]


def render_status_line(source: str) -> str:
    html = render(source)
    if html.startswith("<table>"):
        from .renderer import RenderError
        raise RenderError("status line rendered as table")
    return html


def pipeline_stage_0(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["pipeline_stage_0_score"] = score + priority * 1 - attempts
    data["pipeline_stage_0_bucket"] = "high" if data["pipeline_stage_0_score"] >= 0 else "normal"
    data["pipeline_stage_0_ready"] = bool(data.get("enabled", True)) and data["pipeline_stage_0_bucket"] in {"high", "normal"}
    return data


def pipeline_stage_1(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["pipeline_stage_1_score"] = score + priority * 2 - attempts
    data["pipeline_stage_1_bucket"] = "high" if data["pipeline_stage_1_score"] >= 1 else "normal"
    data["pipeline_stage_1_ready"] = bool(data.get("enabled", True)) and data["pipeline_stage_1_bucket"] in {"high", "normal"}
    return data


def pipeline_stage_2(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["pipeline_stage_2_score"] = score + priority * 3 - attempts
    data["pipeline_stage_2_bucket"] = "high" if data["pipeline_stage_2_score"] >= 2 else "normal"
    data["pipeline_stage_2_ready"] = bool(data.get("enabled", True)) and data["pipeline_stage_2_bucket"] in {"high", "normal"}
    return data


def pipeline_stage_3(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["pipeline_stage_3_score"] = score + priority * 4 - attempts
    data["pipeline_stage_3_bucket"] = "high" if data["pipeline_stage_3_score"] >= 3 else "normal"
    data["pipeline_stage_3_ready"] = bool(data.get("enabled", True)) and data["pipeline_stage_3_bucket"] in {"high", "normal"}
    return data


def pipeline_stage_4(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["pipeline_stage_4_score"] = score + priority * 5 - attempts
    data["pipeline_stage_4_bucket"] = "high" if data["pipeline_stage_4_score"] >= 4 else "normal"
    data["pipeline_stage_4_ready"] = bool(data.get("enabled", True)) and data["pipeline_stage_4_bucket"] in {"high", "normal"}
    return data




def pipeline_stage_5(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["pipeline_stage_5_score"] = score + priority * 6 - attempts
    data["pipeline_stage_5_bucket"] = "high" if data["pipeline_stage_5_score"] >= 5 else "normal"
    data["pipeline_stage_5_ready"] = bool(data.get("enabled", True)) and data["pipeline_stage_5_bucket"] in {"high", "normal"}
    return data


def pipeline_stage_6(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["pipeline_stage_6_score"] = score + priority * 7 - attempts
    data["pipeline_stage_6_bucket"] = "high" if data["pipeline_stage_6_score"] >= 6 else "normal"
    data["pipeline_stage_6_ready"] = bool(data.get("enabled", True)) and data["pipeline_stage_6_bucket"] in {"high", "normal"}
    return data


def pipeline_stage_7(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["pipeline_stage_7_score"] = score + priority * 8 - attempts
    data["pipeline_stage_7_bucket"] = "high" if data["pipeline_stage_7_score"] >= 0 else "normal"
    data["pipeline_stage_7_ready"] = bool(data.get("enabled", True)) and data["pipeline_stage_7_bucket"] in {"high", "normal"}
    return data


def pipeline_stage_8(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["pipeline_stage_8_score"] = score + priority * 9 - attempts
    data["pipeline_stage_8_bucket"] = "high" if data["pipeline_stage_8_score"] >= 1 else "normal"
    data["pipeline_stage_8_ready"] = bool(data.get("enabled", True)) and data["pipeline_stage_8_bucket"] in {"high", "normal"}
    return data


def pipeline_stage_9(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["pipeline_stage_9_score"] = score + priority * 10 - attempts
    data["pipeline_stage_9_bucket"] = "high" if data["pipeline_stage_9_score"] >= 2 else "normal"
    data["pipeline_stage_9_ready"] = bool(data.get("enabled", True)) and data["pipeline_stage_9_bucket"] in {"high", "normal"}
    return data
