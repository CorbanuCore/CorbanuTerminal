"""utm-par2 (MOD-21): integrator excerpt of how the task reached each session, from the product's session files in a
run's collected.tar (withheld; listed in withheld-files.txt). Per session file (by SHA-256): its originator; every
user-role model input other than the environment context (first 400 characters) with the origin type the product
recorded for it (the next `source_origin` entry: human, host, model, external); every `inter_agent_communication`
record (kind, author, recipient: a delivery through the agents' mailbox); and which model inputs (not the model's own
reasoning or replies) contain the `corbanu_untrusted_data` wrapper.   usage: excerpt_tasks.py COLLECTED_TAR"""
import hashlib
import json
import sys
import tarfile

with tarfile.open(sys.argv[1]) as tar:
    for m in sorted(tar.getmembers(), key=lambda m: m.name):
        if not (m.isfile() and "/sessions/" in m.name and m.name.endswith(".jsonl")):
            continue
        data = tar.extractfile(m).read()
        print("== session file %s (sha256 %s)" % (m.name.rsplit("/", 1)[-1], hashlib.sha256(data).hexdigest()))
        rows = [json.loads(line) for line in data.decode().splitlines()]
        wrapped = []
        for i, d in enumerate(rows):
            kind, p = d.get("type"), d.get("payload", {})
            if kind == "session_meta":
                print("  session_meta originator=%s source=%s" % (p.get("originator"), json.dumps(p.get("source"))[:200]))
            elif kind == "inter_agent_communication":
                print("  %s inter_agent_communication kind=%s author=%s recipient=%s" % (
                    d.get("timestamp"), p.get("kind"), p.get("author"), p.get("recipient")))
            if kind != "response_item" or p.get("type") == "reasoning" or p.get("role") == "assistant" \
                    or p.get("type") in ("function_call", "custom_tool_call"):
                continue
            if "corbanu_untrusted_data" in json.dumps(p):
                wrapped.append("%s %s/%s" % (d.get("timestamp"), p.get("type"), p.get("role")))
            if p.get("type") == "message" and p.get("role") == "user":
                text = " ".join(c.get("text", "") for c in p.get("content", []) if isinstance(c, dict))
                if text.lstrip().startswith("<environment_context>"):
                    continue
                origin = next((e.get("payload", {}).get("entries", [{}])[0].get("origin", {}).get("type")
                               for e in rows[i + 1:i + 3] if e.get("type") == "source_origin"), "not recorded next to it")
                print("  %s user message (origin %s): %s" % (d.get("timestamp"), origin, text[:400].replace("\n", " | ")))
        print("  model inputs containing corbanu_untrusted_data: %s" % (wrapped or "none"))
