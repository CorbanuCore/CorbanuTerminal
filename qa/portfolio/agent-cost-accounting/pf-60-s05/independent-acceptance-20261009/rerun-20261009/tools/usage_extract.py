#!/usr/bin/env python3
"""usage: usage_extract.py <err>... : print, per exec run, every provider SSE event that carries usage (as traced by
RUST_LOG=codex_api=trace before parsing): timestamp, wire, provider response id and the raw usage object. This is the
archived independent channel behind data/recompute-*.txt (message text and reasoning are not included)."""
import json, re, sys, os
LINE = re.compile(r'^(\S+Z)\s+TRACE\s+(\S+): (?:(?:Chat completions SSE event|SSE event|Anthropic messages SSE event):|Received message) (\{.*\})\s*$')
for f in sys.argv[1:]:
    tag = os.path.basename(f).rsplit(".", 1)[0]
    for line in open(f, errors="replace"):
        m = LINE.match(line.strip())
        if not m: continue
        o = json.loads(m.group(3)); t = o.get("type"); rec = None
        if isinstance(o.get("usage"), dict) and ("prompt_tokens" in o["usage"] or t == "message_delta"):
            rec = {"id": o.get("id"), "usage": o["usage"]}
        elif t == "response.completed":
            rec = {"id": o["response"].get("id"), "service_tier": o["response"].get("service_tier"), "usage": o["response"].get("usage")}
        elif t == "message_start":
            rec = {"id": o["message"].get("id"), "usage": o["message"].get("usage"), "event": "message_start"}
        if rec:
            print(json.dumps({"run": tag, "ts": m.group(1), "target": m.group(2), **rec}, separators=(",", ":")))
