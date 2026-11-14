#!/usr/bin/env python3
"""merge_models_json.py BASE UPSTREAM MAIN OUT: three-way merge of codex-rs/models-manager/models.json at JSON level.

Corbanu's catalog (main) owns the model list: every main row is kept, rows
upstream added or removed are reported, not applied. Within a row both sides
share, a field (recursively for objects) takes upstream's value only when main
left it as at the merge base; any field main changed keeps main's value.
"""

import json, subprocess, sys

BASE, UP, MAIN = sys.argv[1:4]
OUT = sys.argv[4]
ABSENT = object()


def load(ref):
    return json.loads(
        subprocess.check_output(
            ["git", "show", f"{ref}:codex-rs/models-manager/models.json"]
        )
    )


def merge(b, u, m, path, log):
    if m == b:
        if u is not m and u != m:
            log.append(("upstream", path))
        return u
    if u == b or u == m:
        return m
    if isinstance(b, dict) and isinstance(u, dict) and isinstance(m, dict):
        out = {}
        keys = list(m.keys()) + [k for k in u.keys() if k not in m and k not in b]
        for k in keys:
            v = merge(
                b.get(k, ABSENT), u.get(k, ABSENT), m.get(k, ABSENT), f"{path}.{k}", log
            )
            if v is not ABSENT:
                out[k] = v
        return out
    log.append(("corbanu-kept", path))
    return m


b_all, u_all, m_all = load(BASE), load(UP), load(MAIN)
bs = {x["slug"]: x for x in b_all["models"]}
us = {x["slug"]: x for x in u_all["models"]}
log = []
models = []
for row in m_all["models"]:
    slug = row["slug"]
    if slug in us:
        merged = (
            merge(bs.get(slug, ABSENT), us[slug], row, slug, log) if slug in bs else row
        )
        if slug not in bs:
            log.append(("corbanu-row (upstream added independently)", slug))
        models.append(merged)
    else:
        if slug in bs:
            log.append(("corbanu-kept-row (upstream removed)", slug))
        models.append(row)
for slug in us:
    if slug not in bs and not any(r["slug"] == slug for r in m_all["models"]):
        log.append(("upstream-new-row-not-added", slug))
out = dict(m_all)
out["models"] = models
open(OUT, "w").write(json.dumps(out, indent=2, ensure_ascii=False) + "\n")
for kind, path in log:
    print(kind, path)

# Upstream's Bedrock and Vertex catalogs read gpt-6.1-terra from this file, so
# the row is required. It is built from Corbanu's gpt-6.1-sol row (Corbanu
# prompt and shape) with Terra's identity and OpenAI's published API prices
# (Oct 26 2026: $1 / $0.05 cached / $5, cache write $1.25; above 272K input
# $2 / $0.10 / $7.50, cache write $2.50).
import copy

rows = out["models"]
# Upstream's instructions_template is literal text now (personality variables
# were removed). Corbanu rows still holding the old placeholder get its
# default value, as main rendered them without a personality.
for row in rows:
    messages = row.get("model_messages") or {}
    template = messages.get("instructions_template")
    if template and "{{ personality }}" in template:
        default = (messages.get("instructions_variables") or {}).get(
            "personality_default"
        ) or ""
        messages["instructions_template"] = template.replace(
            "{{ personality }}", default
        )
        print("corbanu-row personality placeholder rendered", row["slug"])
if not any(r["slug"] == "gpt-6.1-terra" for r in rows):
    sol_index = next(i for i, r in enumerate(rows) if r["slug"] == "gpt-6.1-sol")
    terra = copy.deepcopy(rows[sol_index])
    terra["slug"] = "gpt-6.1-terra"
    terra["display_name"] = "GPT-6.1 Terra"
    terra["description"] = (
        "Mid-tier GPT-6.1 at half the Sol price. Requires eligible OpenAI "
        "account access; ChatGPT sign-in may reject it."
    )
    terra["default_reasoning_level"] = "medium"
    terra["priority"] = 3
    terra["base_instructions"] = terra["base_instructions"].replace(
        "based on GPT-6.1 Sol", "based on GPT-6.1 Terra"
    )
    terra["orchestration"] = {
        "status": "disabled",
        "provider_id": "openai",
        "capability": "balanced",
        "reason": "manual selection only; automatic allocation economics are not configured",
        "billing": {
            "kind": "metered",
            "input_milli_usd_per_million_tokens": 1000,
            "output_milli_usd_per_million_tokens": 5000,
            "cached_input_milli_usd_per_million_tokens": 50,
            "cache_write_milli_usd_per_million_tokens": 1250,
            "long_context": {
                "above_input_tokens": 272000,
                "input_milli_usd_per_million_tokens": 2000,
                "output_milli_usd_per_million_tokens": 7500,
                "cached_input_milli_usd_per_million_tokens": 100,
                "cache_write_milli_usd_per_million_tokens": 2500,
            },
        },
    }
    rows.insert(sol_index + 1, terra)
    print("corbanu-row added (required by Bedrock/Vertex catalogs) gpt-6.1-terra")
open(OUT, "w").write(json.dumps(out, indent=2, ensure_ascii=False) + "\n")
