#!/usr/bin/env python3
"""Independent recomputation from provider-reported usage (PF-60-S05 acceptance; extends the S03 tool).

Sources: stderr of `corbanu exec` run with RUST_LOG=codex_api=trace (the raw SSE events the provider sent, logged before
parsing) and/or JSONL from usage_proxy.py. Wires: Chat Completions (`usage`), Responses (`response.completed`
.response.usage) and Anthropic Messages (message_start.message.usage merged with message_delta.usage). Each response is
keyed by the provider's response id, so a response seen twice counts once.

Prices (USD per 1M tokens) are passed per run: --price noncached,cached,output. Inclusive input: cost =
(input - cached) * noncached + cached * cached_rate + output * output_rate. Reasoning is part of output.
Prints one row per response and exact Decimal totals per tag.
"""
import argparse, json, re, sys
from decimal import Decimal as D

LINE = re.compile(r'^(\S+Z)\s+TRACE\s+(\S+): (?:Chat completions SSE event|SSE event|Anthropic messages SSE event): (\{.*\})\s*$')


def from_trace(path):
    out, anth = [], {}
    for line in open(path, errors="replace"):
        m = LINE.match(line.strip())
        if not m:
            continue
        ts, obj = m.group(1), json.loads(m.group(3))
        t = obj.get("type")
        if isinstance(obj.get("usage"), dict) and "prompt_tokens" in obj["usage"]:
            u = obj["usage"]
            out.append(dict(ts=ts, id=obj.get("id"), wire="chat", input=u["prompt_tokens"],
                            cached=(u.get("prompt_tokens_details") or {}).get("cached_tokens") or 0,
                            output=u["completion_tokens"], stated=u.get("cost")))
        elif t == "response.completed":
            r = obj["response"]; u = r.get("usage") or {}
            out.append(dict(ts=ts, id=r.get("id"), wire="responses", input=u["input_tokens"],
                            cached=(u.get("input_tokens_details") or {}).get("cached_tokens") or 0,
                            output=u["output_tokens"], stated=None))
        elif t == "message_start":
            msg = obj["message"]; u = msg.get("usage") or {}
            anth[msg["id"]] = dict(ts=ts, id=msg["id"], wire="anthropic", input=u.get("input_tokens", 0),
                                   cached=u.get("cache_read_input_tokens") or 0,
                                   write=u.get("cache_creation_input_tokens") or 0, output=u.get("output_tokens", 0),
                                   stated=None)
            last = msg["id"]
        elif t == "message_delta" and anth:
            u = obj.get("usage") or {}
            r = anth[list(anth)[-1]]
            for k, src in (("input", "input_tokens"), ("cached", "cache_read_input_tokens"),
                           ("write", "cache_creation_input_tokens"), ("output", "output_tokens")):
                if u.get(src) is not None:
                    r[k] = u[src]
            r["ts"] = ts
    for r in anth.values():
        # Anthropic input_tokens exclude cache reads and writes: make it inclusive like the other wires.
        r["input"] = r["input"] + r["cached"] + r.get("write", 0)
        out.append(r)
    return out


def from_proxy(path):
    out = []
    for line in open(path):
        rec = json.loads(line)
        for u in rec["usage"]:
            if "prompt_tokens" in u:
                out.append(dict(ts=str(rec["t_end"]), id=(rec["ids"] or [None])[0], wire="chat(proxy)",
                                input=u["prompt_tokens"], cached=(u.get("prompt_tokens_details") or {}).get("cached_tokens") or 0,
                                output=u["completion_tokens"], stated=u.get("cost")))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--tag", action="append", default=[], help="tag=file[,file] (trace stderr)")
    ap.add_argument("--proxy", action="append", default=[], help="tag=file (usage_proxy jsonl)")
    ap.add_argument("--price", help="noncached,cached,output USD per 1M; omit to list tokens only")
    a = ap.parse_args()
    price = [D(x) for x in a.price.split(",")] if a.price else None
    grand = [0, 0, 0, 0, D(0), D(0)]
    for spec, fn in [(s, from_trace) for s in a.tag] + [(s, from_proxy) for s in a.proxy]:
        tag, files = spec.split("=", 1)
        seen, rows = set(), []
        for f in files.split(","):
            for r in fn(f):
                if r["id"] in seen:
                    continue
                seen.add(r["id"]); rows.append(r)
        tot = [0, 0, 0, 0, D(0), D(0)]
        for r in rows:
            usd = ((D(r["input"] - r["cached"]) * price[0] + D(r["cached"]) * price[1] + D(r["output"]) * price[2]) / D(10**6)) if price else None
            st = D(str(r["stated"])) if r.get("stated") is not None else None
            print(f'{tag:<16} {r["ts"][:23]:<23} {str(r["id"])[:38]:<38} {r["wire"]:<10} in={r["input"]:>6} cached={r["cached"]:>6} out={r["output"]:>4}'
                  + (f' usd={usd.normalize()}' if usd is not None else '') + (f' stated={st.normalize()}' if st is not None else ''))
            tot[0] += 1; tot[1] += r["input"]; tot[2] += r["cached"]; tot[3] += r["output"]
            tot[4] += usd or 0; tot[5] += st or 0
        print(f'TOTAL {tag}: requests={tot[0]} input={tot[1]} cached={tot[2]} output={tot[3]} tokens={tot[1] + tot[3]}'
              + (f' usd={tot[4].normalize()}' if price else '') + (f' stated={tot[5].normalize()}' if tot[5] else ''))
        grand = [x + y for x, y in zip(grand, tot)]
    print(f'GRAND: requests={grand[0]} tokens={grand[1] + grand[3]}' + (f' usd={grand[4].normalize()}' if price else ''))


if __name__ == "__main__":
    main()
