#!/usr/bin/env python3
"""PF-84-S05: reconcile per-account usage. Provider-reported usage comes from each thread's
rollout (the last cumulative `token_count` the provider reported, and its account from the
turn context); the ledger side from the developer-accounting state DB. Prints names and counts
only: no credential value is read."""
import collections, glob, json, os, sqlite3, sys

home = sys.argv[1]
provider_reported = collections.defaultdict(lambda: [0, 0, 0])  # threads, requests, total tokens
for path in glob.glob(os.path.join(home, "sessions", "**", "rollout-*.jsonl"), recursive=True):
    provider = account = None
    total = 0
    responses = 0
    for line in open(path):
        item = json.loads(line)
        payload = item.get("payload", {})
        if item.get("type") == "turn_context":
            provider = payload.get("model_provider") or provider
            account = payload.get("provider_account") or account
        if item.get("type") == "event_msg" and payload.get("type") == "token_count" and payload.get("info"):
            info = payload["info"]
            total = info["total_token_usage"]["total_tokens"]
            responses += 1
    key = (provider or "?", account or "default")
    row = provider_reported[key]
    row[0] += 1
    row[2] += total
print("Provider-reported usage by (provider, account) from rollouts: threads, final cumulative total tokens")
for key, (threads, _, tokens) in sorted(provider_reported.items()):
    print(f"  {key[0]} · account {key[1]}: {threads} thread(s), {tokens} tokens")

db = sqlite3.connect(os.path.join(home, "pfterminal_state_5.sqlite"))
print("Ledger format (applied accounting migrations):",
      [v for (v,) in db.execute("SELECT version FROM _accounting_migrations ORDER BY version")])
ledger = collections.defaultdict(lambda: [0, 0])
for (payload,) in db.execute(
        "SELECT e.payload FROM draft_accounting_contributions c "
        "JOIN draft_accounting_estimates e ON e.attempt_id = c.attempt_id AND e.evidence = c.evidence"):
    quote = json.loads(payload)
    attempt = quote["attempt"]
    key = (attempt["provider"], (attempt.get("account") or {}).get("name", "default"))
    usage = quote.get("usage", {})
    total = usage.get("total")
    if total is None and usage.get("input") is not None and usage.get("output") is not None:
        total = usage["input"] + usage["output"]
    ledger[key][0] += 1
    ledger[key][1] += total or 0
print("Ledger by (provider, account): the current estimate of each attempt; attempts and total tokens")
for key, (attempts, tokens) in sorted(ledger.items()):
    print(f"  {key[0]} · account {key[1]}: {attempts} attempt(s), {tokens} tokens")
fps = sorted({json.dumps(json.loads(p).get("account")) for (p,) in db.execute("SELECT payload FROM draft_accounting_attempts")})
print("Distinct recorded account objects:", fps)
print("Throttle state (provider, model, key):")
for row in db.execute("SELECT provider_id, model, key_fingerprint, last_status FROM provider_request_state ORDER BY 1, 3"):
    print("  ", row)
