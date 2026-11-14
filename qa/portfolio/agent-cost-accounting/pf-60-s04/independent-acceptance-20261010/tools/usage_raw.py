#!/usr/bin/env python3
"""Archive the provider-reported usage objects verbatim (one JSON line per provider event that carries usage), plus a
count of OpenAI responses created vs completed. Input: codex-tui.log files traced with codex_api/tungstenite::protocol=trace.
Usage: usage_raw.py <tag> <file>  -> JSON lines on stdout; the created/completed summary as the last line."""
import json, re, sys
LINE = re.compile(r'^(\S+Z)\s+TRACE\s+(\S+): (?:(?:Chat completions SSE event|SSE event|Anthropic messages SSE event):|Received message) (\{.*\})\s*$')
tag, path = sys.argv[1], sys.argv[2]; created, completed = set(), set()
for line in open(path, errors="replace"):
    m = LINE.match(line.strip())
    if not m: continue
    try: o = json.loads(m.group(3))
    except ValueError: continue
    t = o.get("type"); r = o.get("response") if isinstance(o.get("response"), dict) else None
    if t == "response.created" and r: created.add(r.get("id"))
    if t == "response.completed" and r:
        completed.add(r.get("id"))
        print(json.dumps({"tag": tag, "ts": m.group(1), "event": t, "id": r.get("id"), "model": r.get("model"), "service_tier": r.get("service_tier"), "usage": r.get("usage")}))
    elif isinstance(o.get("usage"), dict) and "prompt_tokens" in o["usage"]:
        print(json.dumps({"tag": tag, "ts": m.group(1), "event": "chat.chunk", "id": o.get("id"), "model": o.get("model"), "usage": o["usage"]}))
    elif t in ("message_start", "message_delta"):
        u = (o.get("message") or {}).get("usage") if t == "message_start" else o.get("usage")
        if u: print(json.dumps({"tag": tag, "ts": m.group(1), "event": t, "id": (o.get("message") or {}).get("id"), "model": (o.get("message") or {}).get("model"), "usage": u}))
print(json.dumps({"tag": tag, "openai_responses_created": len(created), "openai_responses_completed": len(completed), "created_not_completed": sorted(created - completed)}))
