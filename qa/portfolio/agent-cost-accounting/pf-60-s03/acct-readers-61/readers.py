# /// script
# requires-python = ">=3.11"
# dependencies = ["pexpect==4.9.0", "pyte==0.8.2"]
# ///
"""Code-aware real-key reader evidence using only synthetic local HTTP providers."""
from pathlib import Path
helper_path = Path(__file__).resolve().parent.parent / "acct-qualify-55/qualify.py"
helpers = helper_path.read_text().split('\ntry:\n    (out / "clock.c.gz")', 1)[0]
helpers = helpers.replace("lambda: active_model in visible()", "lambda: active_model.lower() in visible().lower()")
exec(compile(helpers, str(helper_path), "exec"))
from p3_checks import assert_day
env["DYLD_INSERT_LIBRARIES"] = str(Path(__file__).with_name("tz_clock.dylib"))
env["ANTHROPIC_API_KEY"] = "synthetic-loopback-only"
usage_present = True
provider = "openai"
reader_cases = []
backend_backup_dirty = False

def restore_backend_fixture():
    global backend_backup_dirty
    if backend_backup_dirty:
        with sqlite3.connect(fixture/"backend-pristine.sqlite") as source, sqlite3.connect(database()) as target:
            source.backup(target)
        backend_backup_dirty = False
        save("backend-restored.json",dict(method="exact synthetic SQLite backup restored after stopping TUI",
            tables=query("SELECT name FROM sqlite_master WHERE name IN (\'draft_accounting_estimates\',\'qa_held_estimates\')")))

def handle(self):
    body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
    anthropic = "/messages" in self.path
    emitted = dict(provider=provider, model=body["model"],
                   fixture_utc=sample_utc, usage_present=usage_present, path=self.path,
                   input=100, cached=20, cache_write=0, output=10, reasoning=None if anthropic else 4)
    receipts.append(emitted)
    index = len(receipts)
    rid = f"reader-{index}"
    message = "QUALIFY "+phase+" complete."
    if anthropic:
        events = [
            {"type":"message_start","message":{"id":rid,"model":body["model"],
                "usage":{"input_tokens":100,"cache_read_input_tokens":20,"cache_creation_input_tokens":0}}},
            {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}},
            {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":message}},
            {"type":"content_block_stop","index":0},
            {"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":10}},
            {"type":"message_stop"}]
    else:
        response = {"id":rid}
        if usage_present:
            response["usage"] = {"input_tokens":100,"input_tokens_details":{"cached_tokens":20,"cache_write_tokens":0},
                "output_tokens":10,"output_tokens_details":{"reasoning_tokens":4},"total_tokens":110}
        events = [{"type":"response.created","response":{"id":rid}},
            {"type":"response.output_item.done","item":{"type":"message","role":"assistant","id":rid+"-message",
                "content":[{"type":"output_text","text":message}]}},
            {"type":"response.completed","response":response}]
    data = "".join(f"event: {e['type']}\ndata: {json.dumps(e)}\n\n" for e in events).encode()
    self.send_response(200)
    self.send_header("Content-Type","text/event-stream")
    self.send_header("Content-Length",str(len(data)))
    self.end_headers()
    self.wfile.write(data)
    print("EMIT", index, emitted["provider"], emitted["model"], "usage", usage_present, flush=True)
Handler.do_POST = handle

def configure(model, selected_provider="openai", sqlite=True):
    global active_model, provider
    import re
    active_model, provider = model, selected_provider
    text = config.replace('model = "gpt-5.6-sol"', f'model = "{model}"')
    text = text.replace('model_provider = "openai"', f'model_provider = "{provider}"')
    if not sqlite:
        text = text.replace('sqlite = true', 'sqlite = false')
    text += (f'\n[model_providers.anthropic]\nname = "Anthropic fixture"\n'
             f'base_url = "http://127.0.0.1:{server.server_port}/v1"\n'
             'wire_api = "anthropic"\nenv_key = "ANTHROPIC_API_KEY"\n'
             'request_max_retries = 0\nstream_max_retries = 0\n')
    text += (f'\n[model_providers.reader_local]\nname = "Reader local fixture"\n'
             f'base_url = "http://127.0.0.1:{server.server_port}/v1"\n'
             'wire_api = "responses"\nrequest_max_retries = 0\nstream_max_retries = 0\n')
    (profile/"config.toml").write_text(text)

def one(name, when, model, selected_provider="openai", resume=None, with_usage=True):
    global usage_present
    set_clock(when)
    configure(model, selected_provider)
    usage_present = with_usage
    before = len(receipts)
    before_threads = {r[0] for r in query("SELECT id FROM threads")} if list(profile.glob("*state_*.sqlite")) else set()
    launch(name, resume)
    child.delaybeforesend = 0
    send("Use the fixture response.")
    send("\r")
    wait_for(lambda: len(receipts)>before and "QUALIFY "+phase+" complete." in visible(), name, 90)
    drain(1)
    snap(name+"-complete")
    stop()
    save("emissions.json",receipts)
    if resume:
        return resume
    new_threads = {r[0] for r in query("SELECT id FROM threads")} - before_threads
    assert len(new_threads) == 1, new_threads
    return new_threads.pop()

def capture(name, day_value, thread, model="gpt-5.6-sol", selected_provider="openai", width=180, sqlite=True, backend_fault=False):
    configure(model, selected_provider, sqlite)
    launch(name+"-inspect", thread)
    child.delaybeforesend = 0
    if width != 180:
        child.setwinsize(24,width)
        screen.resize(24,width)
        drain(1)
    global backend_backup_dirty
    if backend_fault:
        with sqlite3.connect(database()) as source, sqlite3.connect(fixture/"backend-pristine.sqlite") as target:
            source.backup(target)
        backend_backup_dirty = True
        # Hide one backend table while the UI is running, without altering rows.
        # This simulates a backend read failure, not an absent database.
        with sqlite3.connect(database()) as db:
            db.execute("ALTER TABLE draft_accounting_estimates RENAME TO qa_held_estimates")
        save("backend-fault.json",dict(fault="temporarily rename draft_accounting_estimates",
            pid=child.pid,tables=query("SELECT name FROM sqlite_master WHERE name IN ('draft_accounting_estimates','qa_held_estimates')")))
    command("/usage requests "+day_value)
    text = page(name)
    assert_day(text,day_value)
    if backend_fault:
        require(text,"Unavailable — accounting evidence is corrupt, incompatible or could not be read. Refresh to retry; no repair performed.","backend read failure")
        assert "Known subtotal exact USD:" not in text
    reader_cases.append(dict(case=name,day=day_value,thread=thread,pid=child.pid,
                             columns=width,rows=24 if width!=180 else 60,
                             rendered=selected(name)))
    save("reader-cases.json",reader_cases)
    return text

def close():
    send("\x1b")
    stop()

def inspect_all_attempts(name):
    # Observe details even when missing price means no expected exact dollar total.
    for req in [s for s in selected(name) if s.startswith("Request ") and s[8:].isdigit()]:
        open_link(req)
        req_name=name+"-"+req.replace(" ","-")
        page(req_name)
        for attempt in [s for s in selected(req_name) if s.startswith("Attempt ") and s[8:].isdigit()]:
            open_link(attempt)
            page(req_name+"-"+attempt.replace(" ","-"))
            send("\x1b")
        send("\x1b")

try:
    save("manifest.json",dict(commit=subprocess.check_output(["git","rev-parse","HEAD"],cwd=repo,text=True).strip(),
        binary=str(binary),binary_sha256=hashlib.file_digest(binary.open("rb"),"sha256").hexdigest(),
        driver_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        helper_sha256=hashlib.sha256(helper_path.read_bytes()).hexdigest(),
        qualification="Code-aware developer evidence, not independent acceptance",
        isolation="Synthetic HOME/profile, debug native-keyring denial, OS loopback-only network",
        dimensions="180x60 and 64x24",clock="Fixed wall time; monotonic timers remain real"))
    for name,data in [("driver.py.gz",Path(__file__).read_bytes()),("helpers.py.gz",helpers.encode()),("expanded-helpers.py.gz",prior.encode())]:
        (out/name).write_bytes(gzip.compress(data,mtime=0))
    root = one("historical-sample","2026-08-08T12:00:00Z","gpt-5.6-sol")
    # A later real request advances the native checkpoint; reopen the historical session.
    one("later-sample","2026-08-10T12:00:00Z","gpt-5.6-sol",resume=root)
    text = capture("historical","2026-08-08",root)
    require(text,"Known subtotal exact USD: 0.00071","independent historical amount")
    inspect_all_attempts("historical")
    close()
    text = capture("narrow","2026-08-08",root,width=64)
    inspect_all_attempts("narrow")
    close()
    # Same native session, two actually sampled providers, not renamed stored rows.
    one("custom-provider-sample","2026-08-10T12:00:00Z","reader-local-model","reader_local",resume=root)
    text = capture("mixed-provider","2026-08-10",root,"reader-local-model","reader_local")
    inspect_all_attempts("mixed-provider")
    for label in [s for s in selected("mixed-provider") if s.startswith("Provider: ")]:
        open_link(label)
        page("mixed-provider-"+label.split(";")[0].replace(": ","-"))
        send("\x1b")
    close()
    # Known model catalogue entry with no accounting price snapshot.
    missing = one("missing-price-sample","2026-08-10T12:00:00Z","gpt-6-astra")
    text = capture("missing-price","2026-08-10",missing,"gpt-6-astra")
    require(text,"Estimated token cost: unknown","missing price")
    inspect_all_attempts("missing-price")
    close()
    empty = one("no-usage-sample","2026-08-10T12:00:00Z","gpt-5.6-sol",with_usage=False)
    text = capture("no-usage","2026-08-10",empty)
    require(text,"Estimated token cost: unknown","no usage")
    inspect_all_attempts("no-usage")
    close()
    # Also distinguish zero requests from one request with no usage observation.
    text = capture("zero-attempt-day","2026-08-09",empty)
    require(text,"No recorded attempts in this day; collection coverage unknown.","empty day")
    close()
    configure("gpt-5.6-sol")
    launch("never-prompted")
    command("/usage requests 2026-08-10")
    page("never-prompted")
    close()
    # Explicit backend read failure; sqlite=false did not close an existing DB.
    text = capture("unavailable-backend","2026-08-08",root,backend_fault=True)
    open_link("Refresh")
    refreshed = page("backend-refresh")
    save("backend-refresh-result.json",dict(recovered="Known subtotal exact USD: 0.00071" in refreshed,rendered=selected("backend-refresh")))
    close()
    restore_backend_fixture()
    restored = capture("backend-reopened","2026-08-08",root)
    require(restored,"Known subtotal exact USD: 0.00071","exact backend fixture restore")
    close()
    # Deliberate, disclosed stale-contribution fault. Keep original values and
    # persisted amounts; never fabricate a price, observation or monetary amount.
    with sqlite3.connect(database()) as db:
        before = db.execute("SELECT attempt_id,evidence FROM draft_accounting_contributions").fetchall()
        save("stale-before.json",before)
        db.execute("UPDATE draft_accounting_contributions SET evidence='[]' WHERE attempt_id IN "
                   "(SELECT attempt_id FROM draft_accounting_attempts WHERE json_extract(payload,'$.thread_id')=?)",(root,))
        after = db.execute("SELECT attempt_id,evidence FROM draft_accounting_contributions").fetchall()
        save("stale-injected.json",after)
    text = capture("stale-estimate","2026-08-08",root)
    require(text,"Recorded totals unavailable — stored contributions need refresh.","stale state")
    assert "Known subtotal exact USD:" not in text
    open_link("Refresh")
    page("stale-refresh")
    save("stale-after-refresh.json",query("SELECT attempt_id,evidence FROM draft_accounting_contributions"))
    assert query("SELECT attempt_id,evidence FROM draft_accounting_contributions")==after
    close()
    with sqlite3.connect(database()) as db:
        db.executemany("UPDATE draft_accounting_contributions SET evidence=? WHERE attempt_id=?",
                       [(evidence,attempt) for attempt,evidence in before])
    capture("stale-restored-control","2026-08-08",root)
    close()
    save("native-bindings.json",query("SELECT a.payload,s.payload FROM draft_accounting_attempts a "
         "JOIN draft_accounting_price_bindings b USING(attempt_id) "
         "LEFT JOIN draft_accounting_price_snapshots s USING(snapshot_id)"))
    save("identities.json",dict(historical_and_mixed=root,missing_price=missing,no_usage=empty))
    save("completion.json",dict(passed=True,cases=len(reader_cases)))
finally:
    stop()
    restore_backend_fixture()
    save("emissions.json",receipts)
    save("keys.json",keys)
    for path in out.glob("*"):
        if path.suffix in (".raw",".txt",".log"):
            path.with_suffix(path.suffix+".gz").write_bytes(gzip.compress(path.read_bytes(),mtime=0))
    server.shutdown()
