#!/usr/bin/env python3
"""usage: recompute.py <provider-usage.jsonl>: price each OpenAI Responses usage record at the Standard rates
OpenAI published on 2026-10-09 (data/openai-pricing-20261009.txt), with Decimal. Input is inclusive: uncached input =
input - cached - cache writes. Above 272K input tokens the long-context rates (2x input/cache, 1.5x output) apply."""
import json, sys
from decimal import Decimal as D
# model: (input, cached input, cache write, output) USD per 1M tokens, Standard, short context.
SHEET = {"gpt-5.6-luna": ("0.20", "0.02", "0.25", "1.20"), "gpt-5.6-terra": ("2.00", "0.20", "2.50", "12.00"),
         "gpt-6-luna": ("0.10", "0.01", "0.125", "0.50")}
M = D(1_000_000)
total = D(0)
for line in open(sys.argv[1]):
    r = json.loads(line); u = r["usage"]; d = u.get("input_tokens_details") or {}
    i, c, w, o = (D(x) for x in SHEET[r["model"]])
    if u["input_tokens"] > 272_000:
        i, c, w, o = i * 2, c * 2, w * 2, o * D("1.5")
    cached, write = d.get("cached_tokens", 0), d.get("cache_write_tokens", 0)
    uncached = u["input_tokens"] - cached - write
    usd = (uncached * i + cached * c + write * w + u["output_tokens"] * o) / M
    total += usd
    print(f'{r["id"][:40]} tier={r["service_tier"]} in={u["input_tokens"]} uncached={uncached} cached={cached} '
          f'write={write} out={u["output_tokens"]} usd={usd.normalize()}')
print(f"TOTAL usd={total.normalize()}")
