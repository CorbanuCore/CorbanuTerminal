#!/usr/bin/env python3
"""Independent recompute for the PF-60-S04 independent acceptance (extends the S03/S05/S04-lane recompute tools).

Input: codex-tui.log files written with RUST_LOG=codex_api=trace,tungstenite::protocol=trace — the raw provider events
(SSE data and WebSocket frames) logged before the product parses them. Wires: Responses (`response.completed`
.response.usage, WebSocket or SSE), Chat Completions (`usage` on a chunk) and Anthropic Messages (message_start usage
merged with message_delta usage). Each response is keyed by its provider id, so a response logged twice counts once.

Prices are the providers' published list prices (USD per 1M tokens), keyed by the served model and, for OpenAI, the
served service_tier. Inclusive input: cost = (input - cached - write) * in + cached * cached_rate + write * write_rate + output * out
(write only where the price lists a cache-write rate).
Usage: recompute_ia.py [--since ISO] [--until ISO] file...   Prints one row per response and Decimal totals per model.
"""
import argparse, json, re
from decimal import Decimal as D

LINE = re.compile(r'^(\S+Z)\s+TRACE\s+(\S+): (?:(?:Chat completions SSE event|SSE event|Anthropic messages SSE event):|Received message) (\{.*\})\s*$')
# Published prices, fetched 2026-10-10 (data/published-prices-20261010.txt).
PRICES = {
    ("gpt-5.4", "default"): (D("2.50"), D("0.25"), D("15.00")),
    ("gpt-5.4", "priority"): (D("5.00"), D("0.50"), D("30.00")),  # Fast (formerly Priority)
    ("glm-5.3-flash", None): (D("0.15"), D("0.03"), D("0.50")),
    # 4th value: cache-write rate (OpenAI GPT-5.6 tiers publish one; cache writes are part of input_tokens).
    ("gpt-5.6-luna", "default"): (D("0.20"), D("0.02"), D("1.20"), D("0.25")),
    ("gpt-5.6-terra", "default"): (D("2.00"), D("0.20"), D("12.00"), D("2.50")),
}


def price_for(model, tier):
    m = (model or "").split("/")[-1]
    for (pm, pt), p in PRICES.items():
        # "auto" (reported on the WebSocket prewarm) is served on the default tier.
        served = "default" if tier in (None, "auto") else tier
        if m.startswith(pm) and (pt is None or pt == served):
            return p
    return None


def rows_of(path):
    out, anth = [], {}
    for line in open(path, errors="replace"):
        m = LINE.match(line.strip())
        if not m:
            continue
        ts = m.group(1)
        try:
            obj = json.loads(m.group(3))
        except ValueError:
            continue
        t = obj.get("type")
        if isinstance(obj.get("usage"), dict) and "prompt_tokens" in obj["usage"]:
            u = obj["usage"]
            out.append(dict(ts=ts, id=obj.get("id"), wire="chat", model=obj.get("model"), tier=None,
                            input=u["prompt_tokens"],
                            cached=(u.get("prompt_tokens_details") or {}).get("cached_tokens") or 0,
                            write=0, output=u["completion_tokens"]))
        elif t == "response.completed":
            r = obj["response"]; u = r.get("usage") or {}
            d = u.get("input_tokens_details") or {}
            out.append(dict(ts=ts, id=r.get("id"), wire="responses", model=r.get("model"), tier=r.get("service_tier"),
                            input=u["input_tokens"], cached=d.get("cached_tokens") or 0,
                            write=d.get("cache_write_tokens") or 0, output=u["output_tokens"]))
        elif t == "message_start":
            msg = obj["message"]; u = msg.get("usage") or {}
            anth[msg["id"]] = dict(ts=ts, id=msg["id"], wire="anthropic", model=msg.get("model"), tier=None,
                                   input=u.get("input_tokens", 0), cached=u.get("cache_read_input_tokens") or 0,
                                   write=u.get("cache_creation_input_tokens") or 0, output=u.get("output_tokens", 0))
        elif t == "message_delta" and anth:
            u = obj.get("usage") or {}
            r = anth[list(anth)[-1]]
            for k, src in (("input", "input_tokens"), ("cached", "cache_read_input_tokens"),
                           ("write", "cache_creation_input_tokens"), ("output", "output_tokens")):
                if u.get(src) is not None:
                    r[k] = u[src]
    for r in anth.values():
        r["input"] = r["input"] + r["cached"] + r["write"]  # Anthropic input excludes cache reads/writes
        out.append(r)
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--since"); ap.add_argument("--until")
    ap.add_argument("files", nargs="+")
    a = ap.parse_args()
    seen, rows = set(), []
    for f in a.files:
        for r in rows_of(f):
            if r["id"] in seen or (a.since and r["ts"] < a.since) or (a.until and r["ts"] >= a.until):
                continue
            seen.add(r["id"]); rows.append(r)
    rows.sort(key=lambda r: r["ts"])
    tot = {}
    for r in rows:
        p = price_for(r["model"], r["tier"])
        w = r["write"] if p and len(p) > 3 else 0
        usd = (D(r["input"] - r["cached"] - w) * p[0] + D(r["cached"]) * p[1] + D(w) * (p[3] if w else 0)
               + D(r["output"]) * p[2]) / D(10**6) if p else None
        print(f'{r["ts"][:23]} {str(r["id"])[:40]:<40} {r["wire"]:<9} {str(r["model"])[:26]:<26} tier={r["tier"]} '
              f'in={r["input"]} cached={r["cached"]} write={r["write"]} out={r["output"]} tokens={r["input"] + r["output"]}'
              + (f' usd={usd.normalize()}' if usd is not None else ' usd=n/a'))
        k = (str(r["model"]), r["tier"])
        t = tot.setdefault(k, [0, 0, D(0), usd is not None])
        t[0] += 1; t[1] += r["input"] + r["output"]; t[2] += usd or 0
    paid = D(0)
    for (m, tier), t in sorted(tot.items()):
        print(f'TOTAL model={m} tier={tier}: requests={t[0]} tokens={t[1]}' + (f' usd={t[2].normalize()}' if t[3] else ' usd=n/a'))
        paid += t[2]
    print(f'PAY-PER-USE (priced rows) usd={paid.normalize()}')


if __name__ == "__main__":
    main()
