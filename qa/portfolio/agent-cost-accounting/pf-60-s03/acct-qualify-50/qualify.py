# /// script
# requires-python = ">=3.11"
# dependencies = ["pexpect==4.9.0", "pyte==0.8.2"]
# ///
"""Actual-key, code-aware developer evidence; not independent acceptance."""
import argparse
from datetime import datetime, timezone, timedelta
from decimal import Decimal
import hashlib
import gzip
import http.server
import json
import os
from pathlib import Path
import runpy
import sqlite3
import subprocess
import threading
import time
import uuid

import pexpect
import pyte

parser = argparse.ArgumentParser()
parser.add_argument("binary", type=Path)
parser.add_argument("output", type=Path)
parser.add_argument("--mode", choices=["collect", "inspect", "limits", "cap", "recovery"], required=True)
parser.add_argument("--fixture", type=Path)
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=False)
out = args.output.resolve()
repo = Path(__file__).resolve().parents[5]
fixture = (args.fixture or out / "fixture").resolve()
fixture.mkdir(exist_ok=True)
profile = fixture / "profile"
work = fixture / "work"
profile.mkdir(exist_ok=True)
work.mkdir(exist_ok=True)
subprocess.run(["git", "init", "-q", str(work)], check=True)
isolation = runpy.run_path(str(repo / "scripts/isolated_rust_tests.py"))
binary = args.binary.resolve()
receipts = []
keys = []
lock = threading.Lock()
phase = "root"
counts = {}
clock_offset = int(datetime(2026, 8, 10, 12, 15, tzinfo=timezone.utc).timestamp() - time.time())
RATE = {"gpt-5.6-sol": ("5", ".5", "30"), "gpt-5.6-terra": ("2.5", ".25", "15")}
USAGE = {"root": (100, 20, 10, 4), "child": (80, 10, 6, 2), "orphan": (40, 5, 4, 1), "cap": (10, 0, 1, 0)}
def tool_named(body, suffix):
    for item in body.get("tools", []):
        if item.get("name", "").endswith(suffix):
            return None, item["name"]
        for nested in item.get("tools", []):
            if nested.get("name", "").endswith(suffix):
                return item.get("name"), nested["name"]
    raise RuntimeError("Required native tool not exposed: " + suffix)

class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass
    def do_GET(self):
        self.send_error(404)
    def do_POST(self):
        global phase
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        with lock:
            kind = "child" if body.get("model") == "gpt-5.6-terra" else phase
            index = counts.get(kind, 0)
            counts[kind] = index + 1
            inp, cached, output, reasoning = USAGE[kind]
            row = dict(kind=kind, index=index, model=body.get("model"), path=self.path,
                       service_tier=body.get("service_tier"), input=inp, cached=cached,
                       output=output, reasoning=reasoning,
                       real_utc=datetime.now(timezone.utc).isoformat(),
                       fixture_utc=datetime.fromtimestamp(time.time()+clock_offset, timezone.utc).isoformat())
            receipts.append(row)
            if kind != "cap" or (index + 1) % 50 == 0:
                print(f"Sampled {kind} response {index + 1}", flush=True)
        rid = f"qualify-{kind}-{index}"
        events = [{"type": "response.created", "response": {"id": rid}}]
        if kind == "cap" and index < 512:
            namespace, name = tool_named(body, "list_agents")
            item = dict(type="function_call", call_id=f"fixture-list-{index}", name=name, arguments="{}")
            if namespace:
                item["namespace"] = namespace
            events.append({"type": "response.output_item.done", "item": item})
        elif kind == "root" and index == 0:
            namespace, name = tool_named(body, "spawn_agent")
            item = dict(type="function_call", call_id="fixture-spawn", name=name,
                        arguments=json.dumps(dict(task_name="qualify_child", message="Return child fixture completion.",
                                                  model_provider="openai", model="gpt-5.6-terra",
                                                  reasoning_effort="low", fork_turns="none")))
            if namespace:
                item["namespace"] = namespace
            events.append({"type": "response.output_item.done", "item": item})
        else:
            events.append({"type": "response.output_item.done", "item": {
                "type": "message", "role": "assistant", "id": rid+"-message",
                "content": [{"type": "output_text", "text": f"QUALIFY {kind} complete."}]}})
        events.append({"type": "response.completed", "response": {"id": rid, "usage": {
            "input_tokens": inp, "input_tokens_details": {"cached_tokens": cached, "cache_write_tokens": 0},
            "output_tokens": output, "output_tokens_details": {"reasoning_tokens": reasoning},
            "total_tokens": inp+output}}})
        data = "".join(f"event: {e['type']}\ndata: {json.dumps(e)}\n\n" for e in events).encode()
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
threading.Thread(target=server.serve_forever, daemon=True).start()
catalog = json.loads((repo / "codex-rs/models-manager/models.json").read_text())
for model in catalog["models"]:
    model["tool_mode"] = None
    model["use_responses_lite"] = False
    model["multi_agent_version"] = "v2"
(fixture / "catalog.json").write_text(json.dumps(catalog))
config = (
    'model_provider = "openai"\nmodel = "gpt-5.6-sol"\n'
    'model_reasoning_effort = "low"\ncli_auth_credentials_store = "file"\n'
    'check_for_update_on_startup = false\napproval_policy = "never"\n'
    f'model_catalog_json = {json.dumps(str(fixture / "catalog.json"))}\n'
    f'openai_base_url = "http://127.0.0.1:{server.server_port}/v1"\n'
    '[features]\nsqlite = true\nmulti_agent = true\nmulti_agent_v2 = true\n'
    'code_mode_only = false\ncode_mode_host = false\n'
    f'[projects.{json.dumps(str(work))}]\ntrust_level = "trusted"\n'
)
(profile / "config.toml").write_text(config)
(profile / "auth.json").write_text(json.dumps({"OPENAI_API_KEY": "synthetic-loopback-only"}))
env = isolation["test_environment"]({"PATH": os.environ["PATH"], "HOME": str(fixture)}, profile)
env.update(OPENAI_API_KEY="synthetic-loopback-only", TERM="xterm-256color",
           RUST_LOG="trace", NO_COLOR="1")
if args.mode == "collect":
    env.update(DYLD_INSERT_LIBRARIES=str(Path(__file__).with_name("fixture_clock.dylib")),
               ACCT_QUALIFY_CLOCK_OFFSET_SECONDS=str(clock_offset))
# Network enforcement is a developer fixture control, not agent isolation.
sandbox = fixture / "loopback.sb"
sandbox.write_text('(version 1)\n(allow default)\n(deny network*)\n'
                   '(allow network-outbound (remote ip "localhost:*"))\n'
                   '(allow network-inbound (local ip "localhost:*"))\n')
child = None
screen = None
stream = None
raw = None
def visible():
    return "\n".join(line.rstrip() for line in screen.display)
def drain(seconds=.12):
    until = time.monotonic() + seconds
    while time.monotonic() < until:
        try:
            chunk = child.read_nonblocking(65536, timeout=.04)
        except pexpect.TIMEOUT:
            continue
        except pexpect.EOF:
            break
        stream.feed(chunk)
        if "\x1b[6n" in chunk:
            child.send("\x1b[1;1R")
        if "\x1b[c" in chunk:
            child.send("\x1b[?1;2c")
def send(value):
    keys.append(dict(at=time.time(), value=repr(value)))
    child.send(value)
    drain()
def snap(name):
    text = visible()
    (out / (name+".txt")).write_text(text+"\n")
    return text
def wait_for(predicate, name, timeout=90):
    until = time.monotonic()+timeout
    while time.monotonic() < until:
        drain(.3)
        if predicate():
            return
    snap(name+"-timeout")
    raise RuntimeError("Timed out: "+name)
def launch(name, resume=None):
    global child, screen, stream, raw
    screen = pyte.Screen(180, 60)
    stream = pyte.Stream(screen)
    raw = (out / (name+".raw")).open("w")
    cmd = [str(binary), "--no-alt-screen", "-C", str(work), "-c", f'log_dir="{fixture / "logs"}"']
    if resume:
        cmd += ["resume", resume]
    # Launching through sandbox-exec strips DYLD variables, so use env inside it.
    prefix = ["/usr/bin/sandbox-exec", "-f", str(sandbox), "/usr/bin/env"]
    if args.mode == "collect":
        prefix += [f"DYLD_INSERT_LIBRARIES={env['DYLD_INSERT_LIBRARIES']}",
                   f"ACCT_QUALIFY_CLOCK_OFFSET_SECONDS={clock_offset}"]
    child = pexpect.spawn(prefix[0], prefix[1:]+cmd, cwd=work, env=env,
                          dimensions=(60, 180), encoding="utf-8", timeout=1)
    child.logfile_read = raw
    wait_for(lambda: "gpt-5.6-sol" in visible(), name+"-startup")
    snap(name+"-startup")
def stop():
    if child is not None and child.isalive():
        for _ in range(2):
            if not child.isalive():
                break
            try:
                send("\x03")
            except OSError:
                break
        drain(.5)
        if child.isalive():
            child.terminate(force=True)
    if raw:
        raw.close()
def database():
    return next(profile.glob("*state_*.sqlite"))
def query(sql, params=()):
    with sqlite3.connect(database()) as db:
        return db.execute(sql, params).fetchall()
def page(name):
    # Home and End are actual keys. Preserve each unique viewport while walking.
    send("\x1b[H")
    views = []
    selected = []
    for _ in range(180):
        text = visible()
        if text not in views:
            views.append(text)
        selections = [line.strip()[1:].strip() for line in text.splitlines()
                      if line.lstrip().startswith("›")]
        current = selections[-1] if selections else ""
        selected.append(current)
        if current == "Close":
            break
        send("\x1b[B")
    else:
        raise RuntimeError("Page traversal never reached Close: "+name)
    (out / (name+".txt")).write_text("\n\n--- NEXT VIEWPORT ---\n\n".join(views)+"\n")
    (out / (name+"-selected.json")).write_text(json.dumps(selected, indent=2)+"\n")
    print("Captured "+name, flush=True)
    return "\n".join(views)
def open_link(label):
    send("\x1b[H")
    for _ in range(180):
        selections = [line.strip()[1:].strip() for line in visible().splitlines()
                      if line.lstrip().startswith("›")]
        current = selections[-1] if selections else ""
        if current == label or (label.endswith("*") and current.startswith(label[:-1])):
            send("\r")
            drain(.3)
            return
        send("\x1b[B")
    raise RuntimeError("Link not found: "+label)
def command(value):
    send(value)
    send("\r")
    wait_for(lambda: ("↑↓ scroll" in visible() and "Loading" not in visible()), "inspector")
def require(text, expected, label):
    if expected not in text:
        raise RuntimeError(f"DEFECT or missing rendered evidence {label}: expected {expected!r}")
try:
    driver_bytes = Path(__file__).read_bytes()
    (out / "driver.py.gz").write_bytes(gzip.compress(driver_bytes, mtime=0))
    manifest = dict(binary=str(binary), driver_sha256=hashlib.sha256(driver_bytes).hexdigest(),
        sha256=hashlib.file_digest(binary.open("rb"), "sha256").hexdigest(),
        commit=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip(),
        mode=args.mode, fixture=str(fixture), clock_offset_seconds=clock_offset if args.mode=="collect" else 0,
        isolation="synthetic profile and minimal HOME/env; debug keyring denial; OS loopback-only network; code-aware developer execution",
        catalog_override="Only tool_mode, use_responses_lite and multi_agent_version overridden; price authority remains bundled")
    (out / "manifest.json").write_text(json.dumps(manifest, indent=2)+"\n")
    if args.mode == "collect":
        launch("root")
        send("Use the fixture response.")
        send("\r")
        wait_for(lambda: counts.get("child", 0)>=1 and counts.get("root", 0)>=2 and
                 "QUALIFY root complete." in visible(), "root-and-child", 180)
        snap("root-and-child-complete")
        drain(3)
        root = query("SELECT parent_thread_id FROM thread_spawn_edges")[0][0]
        child_id = query("SELECT child_thread_id FROM thread_spawn_edges")[0][0]
        stop()
        phase = "orphan"
        launch("orphan")
        send("Use the fixture response.")
        send("\r")
        wait_for(lambda: counts.get("orphan", 0)>=1 and "QUALIFY orphan complete." in visible(), "orphan")
        snap("orphan-complete")
        stop()
        owners = query("SELECT DISTINCT json_extract(payload,'$.thread_id') FROM draft_accounting_attempts")
        orphan = next(row[0] for row in owners if row[0] not in [root, child_id])
        # Deliberate metadata fault injection only: the request and price/usage rows remain sampled.
        with sqlite3.connect(database()) as db:
            old_source = db.execute("SELECT source FROM threads WHERE id=?", (orphan,)).fetchone()[0]
            db.execute("UPDATE threads SET source='unknown' WHERE id=?", (orphan,))
        (fixture / "identities.json").write_text(json.dumps(dict(root=root, child=child_id, orphan=orphan,
                                                        original_orphan_source=old_source), indent=2)+"\n")
        amounts = []
        for row in receipts:
            inp, cache, output = map(Decimal, RATE[row["model"]])
            cost = ((row["input"]-row["cached"])*inp+row["cached"]*cache+row["output"]*output)/Decimal(1000000)
            amounts.append(dict(**row, rates_per_million=RATE[row["model"]], exact_usd=str(cost)))
        (fixture / "traffic-expectations.json").write_text(json.dumps(amounts, indent=2)+"\n")
        bindings = query("SELECT a.payload,s.payload FROM draft_accounting_attempts a JOIN draft_accounting_price_bindings b USING(attempt_id) LEFT JOIN draft_accounting_price_snapshots s USING(snapshot_id)")
        (out / "bound-prices-and-attempts.json").write_text(json.dumps(bindings, indent=2)+"\n")
        for attempt_json, snapshot_json in bindings:
            attempt = json.loads(attempt_json)
            snapshot = json.loads(snapshot_json)
            bound = snapshot["rates"]
            wanted = RATE[attempt["model"]]
            if tuple(Decimal(bound[k]) for k in ["noncached", "read", "output"]) != tuple(map(Decimal, wanted)):
                raise RuntimeError("Bound rates disagree with the frozen independent catalogue constants")
        actual_times = [json.loads(a)["dispatched_at_ms"] for a, _ in bindings]
        if not all(datetime.fromtimestamp(int(t)/1000, timezone.utc).date().isoformat()=="2026-08-10" for t in actual_times):
            raise RuntimeError("Historical clock fixture failed; no range qualification claimed")
    elif args.mode == "inspect":
        identities = json.loads((fixture / "identities.json").read_text())
        # One current-time observation advances the normal checkpoint; it is outside historical queries.
        phase = "orphan"
        launch("inspect", identities["root"])
        send("Advance the fixture checkpoint.")
        send("\r")
        wait_for(lambda: counts.get("orphan",0)>=1 and "QUALIFY orphan complete." in visible(), "checkpoint")
        command("/usage requests 2026-08-10")
        text = page("day")
        expected = json.loads((fixture / "traffic-expectations.json").read_text())
        total = sum(Decimal(r["exact_usd"]) for r in expected if r["kind"]!="orphan")
        require(text, "Known subtotal exact USD: "+format(total,"f"), "standalone day total")
        root_rows = [r for r in expected if r["kind"] != "orphan"]
        metric_expectations = {
            "Input": sum(r["input"] for r in root_rows),
            "Noncached input (derived for inclusive input)": sum(r["input"]-r["cached"] for r in root_rows),
            "Cache read": sum(r["cached"] for r in root_rows),
            "Cache write": 0,
            "Output": sum(r["output"] for r in root_rows),
            "Reasoning (subset, not separately billed)": sum(r["reasoning"] for r in root_rows),
            "Total (not separately billed)": sum(r["input"]+r["output"] for r in root_rows),
        }
        for label, n in metric_expectations.items():
            require(text, f"{label}: {n} known + unknown in 0 attempts", label)
        require(text, "Unknown parent population: 1 attempts, excluded from root total", "unknown split")
        for label in ["Root's own attempts","Descendant attempts","Provider: openai; Model: gpt-5.6-sol",
                      "Provider: openai; Model: gpt-5.6-terra","Unknown parent population"]:
            open_link(label)
            group = page("group-"+label.replace("/","-").replace(":","").replace(";",""))
            if label != "Unknown parent population":
                kind = "child" if ("Descendant" in label or "terra" in label) else "root"
                group_rows = [r for r in expected if r["kind"] == kind]
                amount = sum(Decimal(r["exact_usd"]) for r in group_rows)
                require(group, "Known estimate exact USD: "+format(amount,"f"), label)
                require(group, "Recorded attempts: "+str(len(group_rows)), label)
            else:
                require(group, "Recorded attempts with unresolved ancestry: 1", label)
                open_link("Request *")
                orphan_attempt = page("unknown-parent-original-price")
                amount = sum(Decimal(r["exact_usd"]) for r in expected if r["kind"] == "orphan")
                require(orphan_attempt, "Known subtotal exact USD: "+format(amount,"f"), label+" exact attempt")
                send("\x1b")
            send("\x1b")
        request_links = [s for s in json.loads((out / "day-selected.json").read_text())
                         if s.startswith("Request ") and s[8:].isdigit()]
        for request_label in request_links:
            request_number = request_label[8:]
            open_link(request_label)
            page(f"logical-request-{request_number}")
            attempt_links = [s for s in json.loads((out / f"logical-request-{request_number}-selected.json").read_text())
                             if s.startswith("Attempt ") and s[8:].isdigit()]
            for attempt_label in attempt_links:
                open_link(attempt_label)
                page(f"original-price-request-{request_number}-attempt-{attempt_label[8:]}")
                send("\x1b")
            send("\x1b")
        send("\x1b")
        ranges = [
            ("hour","2026-08-10T12:00:00Z","2026-08-10T13:00:00Z","Hour "),
            ("week","2026-08-10","2026-08-17","Week "),
            ("month","2026-08-01","2026-09-01","Month "),
        ]
        for label, start, end, link in ranges:
            command(f"/usage requests {start} {end} {label}")
            page(label+"-overview")
            open_link(link+"*")
            bucket = page(label+"-bucket")
            require(bucket, "Known subtotal exact USD: "+format(total,"f"), label+" day equality")
            for metric, n in metric_expectations.items():
                require(bucket, f"{metric}: {n} known + unknown in 0 attempts", label+" "+metric)
            send("\x1b")
            send("\x1b")
        (out / "comparisons.json").write_text(json.dumps(dict(expected_exact_usd=str(total), range_day_equality=["hour","week","month"]), indent=2)+"\n")
    elif args.mode == "recovery":
        identities = json.loads((fixture / "identities.json").read_text())
        launch("recovery", identities["root"])
        command("/usage requests 2026-08-10")
        recovered = page("restored-day")
        require(recovered, "Known subtotal exact USD: 0.0016875", "recovery after padding removal")
        send("\x1b")
        cases = [
            ("hour", "2026-08-10T12:15:00Z", "2026-08-10T13:00:00Z", "Hour [2026-08-10T12:00:00.000Z, 2026-08-10T13:00:00.000Z)"),
            ("week", "2026-08-10T12:00:00Z", "2026-08-17", "Week [2026-08-10T00:00:00.000Z, 2026-08-17T00:00:00.000Z)"),
            ("month", "2026-08-10", "2026-09-01", "Month [2026-08-01T00:00:00.000Z, 2026-09-01T00:00:00.000Z)"),
        ]
        for group, start, end, expected_link in cases:
            command(f"/usage requests {start} {end} {group}")
            overview = page("partial-"+group+"-overview")
            require(overview, expected_link, group+" UTC alignment from unaligned input")
            require(overview, "Range total unavailable", group+" partial range withholding")
            open_link(expected_link)
            partial = page("partial-"+group+"-bucket")
            require(partial, "Partial bucket — excluded from totals", group+" partial bucket")
            if "Known subtotal exact USD:" in partial or "Estimated token cost for recorded attempts:" in partial:
                raise RuntimeError("DEFECT: partial bucket displayed a partial monetary total")
            send("\x1b")
            send("\x1b")
        (out / "recovery-result.json").write_text(json.dumps(dict(restored_exact_usd="0.0016875",
             unaligned_utc_buckets=["hour", "week", "month"], partial_totals_withheld=True),indent=2)+"\n")
    elif args.mode == "cap":
        phase = "cap"
        launch("cap")
        send("Run the fixture sequence.")
        send("\r")
        wait_for(lambda: counts.get("cap", 0) == 513 and "QUALIFY cap complete." in visible(),
                 "513-sampled-attempts", 1800)
        snap("cap-sampling-complete")
        sampled_count = query("SELECT count(*) FROM draft_accounting_attempts")[0][0]
        if sampled_count != 513:
            raise RuntimeError(f"Cap setup has {sampled_count} stored attempts, expected 513")
        command("/usage requests")
        rendered = page("cap-refusal")
        require(rendered, "Range too large for this inspector. No total shown.", "513 sampled attempts")
        if "Known subtotal exact USD:" in rendered or "Estimated token cost for recorded attempts:" in rendered:
            raise RuntimeError("DEFECT: cap refusal includes a partial numeric total")
        (out / "cap-result.json").write_text(json.dumps(dict(sent_attempts=513,stored_attempts=sampled_count,
                                                            refused_without_total=True),indent=2)+"\n")
    else:
        identities = json.loads((fixture / "identities.json").read_text())
        launch("limits", identities["root"])
        command("/usage requests 2026-08-10")
        page("baseline")
        send("\x1b")
        # Work-budget padding on another day never contributes to the selected totals.
        seed = json.loads(query("SELECT payload FROM draft_accounting_attempts LIMIT 1")[0][0])
        filler_owner = str(uuid.uuid4())
        filler_day = int(datetime(2026, 8, 9, tzinfo=timezone.utc).timestamp()*1000)
        measurements = []
        initial = query("SELECT count(*) FROM draft_accounting_attempts")[0][0]
        def populate(total):
            with sqlite3.connect(database()) as db:
                db.execute("DELETE FROM draft_accounting_attempts WHERE request_id='00000000-0000-0000-0000-ffffffffffff'")
                data = []
                for i in range(total-initial):
                    aid = str(uuid.UUID(int=i+1))
                    p = dict(seed, attempt_id=aid, request_id="00000000-0000-0000-0000-ffffffffffff",
                             thread_id=filler_owner, dispatched_at_ms=filler_day)
                    data.append((aid,"00000000-0000-0000-0000-ffffffffffff",json.dumps(p,separators=(",",":"))))
                    if len(data)==10000:
                        db.executemany("INSERT INTO draft_accounting_attempts VALUES (?,?,?)",data)
                        data=[]
                db.executemany("INSERT INTO draft_accounting_attempts VALUES (?,?,?)",data)
        # Empty historical day isolates the global row denominator and complete TUI path.
        for total in [571427,571428,571429,666666,666667]:
            populate(total)
            for selected_day in ["2026-08-08", datetime.now(timezone.utc).date().isoformat()]:
                start = time.monotonic()
                command("/usage requests "+selected_day)
                rendered = page(f"rows-{total}-day-{selected_day}")
                refused = "Range too large for this inspector. No total shown." in rendered
                if refused:
                    if "Known subtotal exact USD:" in rendered or "Estimated token cost for recorded attempts:" in rendered:
                        raise RuntimeError("DEFECT: work-budget refusal leaked a partial total")
                else:
                    require(rendered, "Known subtotal exact USD:", "work-budget admitted complete view")
                selected_count = query("SELECT count(*) FROM draft_accounting_attempts WHERE json_extract(payload,'$.thread_id')=? AND date(json_extract(payload,'$.dispatched_at_ms')/1000,'unixepoch')=?",
                                       (identities["root"], selected_day))[0][0]
                measurements.append(dict(attempt_rows=total,compact_day_rows=query("SELECT count(*) FROM draft_accounting_compact_days")[0][0],
                                         day=selected_day, selected_attempts=selected_count,refused=refused,elapsed=time.monotonic()-start))
                (out / "measurements.json").write_text(json.dumps(measurements,indent=2)+"\n")
                send("\x1b")
        with sqlite3.connect(database()) as db:
            db.execute("DELETE FROM draft_accounting_attempts WHERE request_id='00000000-0000-0000-0000-ffffffffffff'")
        (out / "measurements.json").write_text(json.dumps(measurements,indent=2)+"\n")
finally:
    stop()
    (out / "requests.json").write_text(json.dumps(receipts, indent=2)+"\n")
    (out / "keys.json").write_text(json.dumps(keys, indent=2)+"\n")
    server.shutdown()
