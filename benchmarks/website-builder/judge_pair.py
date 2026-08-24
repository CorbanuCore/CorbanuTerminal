#!/usr/bin/env python3
"""Run balanced blind visual judging for one website benchmark wave."""

from __future__ import annotations

import argparse
import base64
import json
import secrets
from pathlib import Path
from typing import Any


CAPTURE_IDS = [
    "desktop_hero",
    "desktop_full",
    "desktop_install",
    "desktop_orchestration",
    "mobile_hero",
    "mobile_menu",
]
RUBRIC_MAX = {
    "visual_design_and_art_direction": 25,
    "product_communication_and_credibility": 15,
    "information_architecture_and_content": 15,
    "interaction_clarity_and_usability": 15,
    "responsive_design": 15,
    "generated_image_quality_and_integration": 10,
    "accessibility_and_readability": 5,
}

JUDGE_PROMPT = """You are the independent visual-quality judge for a blind coding-agent
benchmark. Two implementations received the same product brief. You do not
know which tool made either site.

Judge only the supplied screenshots and the stated product brief. Do not infer
the author from style. Do not reward or punish a site for technology choice,
token use, cost, implementation complexity, or image-generation prompt. Do
not assume a labeled interaction worked unless the corresponding screenshot
visibly proves the intended state.

Product facts
- Corbanu Terminal is a terminal-native agentic trading and coding environment.
- It orchestrates multiple frontier models in one auditable workflow.
- It emphasizes explicit routing, resumable sessions, local workspace control,
  and timing/token/cost visibility.
- Audience: senior developers, technical founders, traders, and engineering leads.
- Primary action: view installation.
- Secondary action: explore orchestration.
- Tagline: "One terminal. Many models. One auditable workflow."

For each site, score:
- visual_design_and_art_direction: 0-25
- product_communication_and_credibility: 0-15
- information_architecture_and_content: 0-15
- interaction_clarity_and_usability: 0-15
- responsive_design: 0-15
- generated_image_quality_and_integration: 0-10
- accessibility_and_readability: 0-5

Apply explicit penalties, up to 30 total, for visible clipping or overlap,
broken or unchanged interaction states, horizontal overflow, illegible text,
obvious generic-template treatment, deceptive unsupported claims, missing
content, inconsistent responsive behavior, or generated imagery that is
hidden, irrelevant, low quality, or poorly integrated.

Base every score and penalty on observable evidence. Identify the screenshot
ID for important observations. Use the full-page capture for composition and
coverage, and viewport captures for fold quality, legibility, and state
details.

Return strict JSON only. Select winner "a", "b", or "tie". A difference of
three points or less should normally be a tie unless there is a clear,
evidence-backed functional or responsive distinction. Do not guess which
coding tool produced either site."""


def evidence_item_schema(points: bool = False) -> dict[str, Any]:
    properties: dict[str, Any] = {
        "screenshot_ids": {
            "type": "array",
            "items": {"type": "string", "enum": CAPTURE_IDS},
            "minItems": 1,
        }
    }
    required = ["screenshot_ids"]
    if points:
        properties["points"] = {"type": "integer", "minimum": 1, "maximum": 30}
        properties["reason"] = {"type": "string"}
        required = ["points", "reason", "screenshot_ids"]
    else:
        properties["observation"] = {"type": "string"}
        required = ["observation", "screenshot_ids"]
    return {
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": False,
    }


def site_schema() -> dict[str, Any]:
    return {
        "type": "object",
        "properties": {
            "rubric": {
                "type": "object",
                "properties": {
                    key: {"type": "integer", "minimum": 0, "maximum": maximum}
                    for key, maximum in RUBRIC_MAX.items()
                },
                "required": list(RUBRIC_MAX),
                "additionalProperties": False,
            },
            "subtotal": {"type": "integer", "minimum": 0, "maximum": 100},
            "penalties": {
                "type": "array",
                "items": evidence_item_schema(points=True),
                "maxItems": 10,
            },
            "total": {"type": "integer", "minimum": 0, "maximum": 100},
            "strengths": {
                "type": "array",
                "items": evidence_item_schema(),
                "minItems": 1,
                "maxItems": 8,
            },
            "weaknesses": {
                "type": "array",
                "items": evidence_item_schema(),
                "minItems": 1,
                "maxItems": 8,
            },
        },
        "required": ["rubric", "subtotal", "penalties", "total", "strengths", "weaknesses"],
        "additionalProperties": False,
    }


SCHEMA = {
    "type": "object",
    "properties": {
        "site_a": site_schema(),
        "site_b": site_schema(),
        "winner": {"type": "string", "enum": ["a", "b", "tie"]},
        "margin": {"type": "integer", "minimum": 0, "maximum": 100},
        "confidence": {"type": "number", "minimum": 0, "maximum": 1},
        "decisive_evidence": {
            "type": "array",
            "items": evidence_item_schema(),
            "minItems": 1,
            "maxItems": 8,
        },
    },
    "required": [
        "site_a",
        "site_b",
        "winner",
        "margin",
        "confidence",
        "decisive_evidence",
    ],
    "additionalProperties": False,
}


def image_data_url(path: Path) -> str:
    encoded = base64.b64encode(path.read_bytes()).decode("ascii")
    return f"data:image/png;base64,{encoded}"


def content_for(run_root: Path, order: list[str], wave: int) -> list[dict[str, Any]]:
    content: list[dict[str, Any]] = [{"type": "input_text", "text": JUDGE_PROMPT}]
    for site_label, lane in zip(("A", "B"), order, strict=True):
        content.append(
            {
                "type": "input_text",
                "text": f"Begin Site {site_label}. The next six images are this site only.",
            }
        )
        capture_dir = run_root / "results" / lane / f"wave-{wave:03d}" / "captures"
        for capture_id in CAPTURE_IDS:
            path = capture_dir / f"{capture_id}.png"
            if not path.is_file():
                raise RuntimeError(f"missing capture: {path}")
            content.append(
                {
                    "type": "input_text",
                    "text": f"Site {site_label} — screenshot ID: {capture_id}",
                }
            )
            content.append(
                {
                    "type": "input_image",
                    "image_url": image_data_url(path),
                    "detail": "original",
                }
            )
    return content


def normalize_arithmetic(payload: dict[str, Any]) -> dict[str, Any]:
    mismatches = []
    for site_key in ("site_a", "site_b"):
        site = payload[site_key]
        subtotal = sum(int(site["rubric"][key]) for key in RUBRIC_MAX)
        penalty = min(30, sum(int(item["points"]) for item in site["penalties"]))
        total = max(0, subtotal - penalty)
        if site.get("subtotal") != subtotal or site.get("total") != total:
            mismatches.append(
                {
                    "site": site_key,
                    "model_subtotal": site.get("subtotal"),
                    "computed_subtotal": subtotal,
                    "model_total": site.get("total"),
                    "computed_total": total,
                }
            )
        site["subtotal"] = subtotal
        site["total"] = total
    payload["margin"] = abs(payload["site_a"]["total"] - payload["site_b"]["total"])
    payload["_arithmetic_mismatches"] = mismatches
    return payload


def underlying_winner(payload: dict[str, Any], order: list[str]) -> str:
    winner = payload["winner"]
    if winner == "tie":
        return "tie"
    return order[0] if winner == "a" else order[1]


def run_pass(
    client: Any,
    run_root: Path,
    judge_model: str,
    pass_name: str,
    order: list[str],
    wave: int,
    output_dir: Path,
) -> dict[str, Any]:
    response = client.responses.create(
        model=judge_model,
        input=[{"role": "user", "content": content_for(run_root, order, wave)}],
        reasoning={"effort": "high"},
        text={
            "format": {
                "type": "json_schema",
                "name": "visual_bakeoff_judgment",
                "strict": True,
                "schema": SCHEMA,
            },
            "verbosity": "medium",
        },
        store=False,
        max_output_tokens=12000,
    )
    (output_dir / f"{pass_name}.response.json").write_text(
        response.model_dump_json(indent=2),
        encoding="utf-8",
    )
    payload = normalize_arithmetic(json.loads(response.output_text))
    payload["_pass"] = pass_name
    payload["_order"] = {"a": order[0], "b": order[1]}
    payload["_underlying_winner"] = underlying_winner(payload, order)
    payload["_response_id"] = response.id
    payload["_model"] = response.model
    payload["_usage"] = response.usage.model_dump() if response.usage else None
    (output_dir / f"{pass_name}.judgment.json").write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return payload


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-root", type=Path, required=True)
    parser.add_argument("--openai-key-file", type=Path, required=True)
    parser.add_argument("--wave", type=int, required=True)
    parser.add_argument("--judge-model", default="gpt-5.6-sol")
    parser.add_argument("--confirm-paid-run", action="store_true")
    args = parser.parse_args()
    if not args.confirm_paid_run:
        raise SystemExit("live visual judging requires --confirm-paid-run")
    if args.wave < 1:
        raise SystemExit("wave must be positive")

    try:
        from openai import OpenAI
    except ImportError as error:
        raise SystemExit("install the website benchmark requirements first") from error

    run_root = args.run_root.resolve()
    blind_dir = run_root / "blind" / f"wave-{args.wave:03d}"
    output_dir = blind_dir / "judgments"
    output_dir.mkdir(parents=True, exist_ok=True)
    first_order = (
        ["corbanu", "claude-code"]
        if secrets.randbelow(2) == 0
        else ["claude-code", "corbanu"]
    )
    mapping = {
        "normal": {"a": first_order[0], "b": first_order[1]},
        "swapped": {"a": first_order[1], "b": first_order[0]},
    }
    (blind_dir / "mapping.json").write_text(
        json.dumps(mapping, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (blind_dir / "judge_prompt.md").write_text(JUDGE_PROMPT + "\n", encoding="utf-8")
    (blind_dir / "judge_schema.json").write_text(
        json.dumps(SCHEMA, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    key = args.openai_key_file.read_text(encoding="utf-8").strip()
    if not key:
        raise SystemExit(f"empty key file: {args.openai_key_file}")
    client = OpenAI(api_key=key)
    normal = run_pass(
        client,
        run_root,
        args.judge_model,
        "normal",
        first_order,
        args.wave,
        output_dir,
    )
    swapped_order = list(reversed(first_order))
    swapped = run_pass(
        client,
        run_root,
        args.judge_model,
        "swapped",
        swapped_order,
        args.wave,
        output_dir,
    )
    passes = [normal, swapped]
    winners = [item["_underlying_winner"] for item in passes]
    if winners[0] == winners[1]:
        verdict = winners[0]
        reason = "both balanced A/B orders selected the same underlying site"
    else:
        verdict = "tie_inconclusive_order_sensitive"
        reason = "the balanced A/B orders did not select the same underlying site"

    summary = {
        "model": args.judge_model,
        "wave": args.wave,
        "verdict": verdict,
        "verdict_reason": reason,
        "passes": [
            {
                "pass": item["_pass"],
                "order": item["_order"],
                "underlying_winner": item["_underlying_winner"],
                "site_a_total": item["site_a"]["total"],
                "site_b_total": item["site_b"]["total"],
                "confidence": item["confidence"],
                "response_id": item["_response_id"],
                "usage": item["_usage"],
            }
            for item in passes
        ],
    }
    (output_dir / "summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(json.dumps(summary, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
