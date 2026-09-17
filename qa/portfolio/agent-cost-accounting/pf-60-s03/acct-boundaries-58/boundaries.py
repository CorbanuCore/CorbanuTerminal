# /// script
# requires-python = ">=3.11"
# dependencies = ["pexpect==4.9.0", "pyte==0.8.2"]
# ///
"""Code-aware real-key range evidence; no production rows are fabricated."""
from pathlib import Path
helper_path = Path(__file__).resolve().parent.parent / "acct-qualify-55/qualify.py"
helpers = helper_path.read_text().split('\ntry:\n    (out / "clock.c.gz")', 1)[0]
helpers = helpers.replace("lambda: active_model in visible()", "lambda: active_model.lower() in visible().lower()")
exec(compile(helpers, str(helper_path), "exec"))
from zoneinfo import ZoneInfo
import selectors
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "acct-readers-61"))
from p3_checks import assert_day, assert_compact_hour, assert_timezone
env["DYLD_INSERT_LIBRARIES"] = str(Path(__file__).resolve().parent.parent / "acct-readers-61/tz_clock.dylib")
timezone_controls = []

cases = []
def recalc():
    expected.clear()
    for i, row in enumerate(receipts):
        a, b, c = map(Decimal, RATE[row["model"]])
        amount = ((row["input"]-row["cached"])*a+row["cached"]*b+row["output"]*c)/Decimal(1000000)
        expected.append(dict(**row, emission=i, exact_usd=str(amount), rates_per_million=RATE[row["model"]]))
    save("independent-arithmetic.json", expected)

def members(start, end, deleted=False):
    return [r for r in expected if epoch(start) <= epoch(r["fixture_utc"]) < epoch(end)
            and not (deleted and r["kind"] == "child")]

def record(name, start, end, rows, pages, **extra):
    cases.append(dict(case=name, start=start, end=end, emissions=[r["emission"] for r in rows],
                      exact_usd=str(cost(rows)), rendered={p:selected(p) for p in pages}, **extra))
    save("boundary-results.json", cases)

def day(name, value, deleted=False, unavailable=False):
    start = value+"T00:00:00Z"
    end = (datetime.fromisoformat(start.replace("Z","+00:00"))+timedelta(days=1)).isoformat().replace("+00:00","Z")
    rows = members(start, end, deleted)
    command("/usage requests "+value)
    text = page(name)
    assert_day(text, value)
    if unavailable:
        require(text, "compacted history lost request/provider attribution", name)
        assert "Known subtotal exact USD:" not in text
    else:
        assert_metrics(text, rows)
        if not rows:
            require(text, "No recorded attempts in this day; collection coverage unknown.", name)
    record(name, start, end, rows, [name], expected_state="unavailable-compact" if unavailable else "available", deleted_child=deleted)
    send("\x1b")

def hours(name, start, end, windows):
    command(f"/usage requests {start} {end} hour")
    page(name+"-overview")
    for i, (a,b) in enumerate(windows):
        open_link("Hour ["+a.replace("Z",".000Z")+", "+b.replace("Z",".000Z")+")")
        label = name+"-"+str(i)
        text = page(label)
        rows = members(a,b)
        assert_metrics(text,rows)
        if name in ("spring-America-New_York","fall"):
            detail_members(label,rows)
        record(label,a,b,rows,[label],expected_state="available", process_timezone=env["TZ"])
        send("\x1b")
    send("\x1b")

def delete_child(thread):
    # Supported app-server deletion routes through native accounting cleanup.
    prefix = ["/usr/bin/sandbox-exec", "-f", str(sandbox), "/usr/bin/env",
              "DYLD_INSERT_LIBRARIES="+env["DYLD_INSERT_LIBRARIES"],
              "ACCT_QUALIFY_CLOCK_MS="+str(epoch(sample_utc))]
    process = subprocess.Popen(prefix+[str(binary), "app-server"], cwd=work, env=env,
                               stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=(out/"delete-server.log").open("w"), text=True, bufsize=1)
    transcript = []
    sel = selectors.DefaultSelector()
    sel.register(process.stdout, selectors.EVENT_READ)
    def rpc(ident, method, params):
        request = dict(id=ident, method=method, params=params)
        process.stdin.write(json.dumps(request)+"\n")
        process.stdin.flush()
        transcript.append(dict(sent=request))
        until = time.monotonic()+60
        while time.monotonic() < until:
            if not sel.select(timeout=1):
                continue
            line = process.stdout.readline()
            if not line:
                raise RuntimeError("app-server exited")
            response = json.loads(line)
            # Initialization/deletion only; never read auth or request bodies.
            if response.get("id") == ident:
                transcript.append(dict(received=response))
                assert "error" not in response, response
                return
        raise RuntimeError("app-server RPC timeout "+method)
    try:
        rpc(1, "initialize", dict(clientInfo=dict(name="acct_boundaries_fixture", version="1"),
                                   capabilities=dict(experimentalApi=True)))
        process.stdin.write(json.dumps(dict(method="initialized", params={}))+"\n")
        process.stdin.flush()
        rpc(2, "thread/delete", dict(threadId=thread))
    finally:
        process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
        sel.close()
        save("delete-rpc.json", transcript)

try:
    save("manifest.json", dict(commit=subprocess.check_output(["git","rev-parse","HEAD"],cwd=repo,text=True).strip(),
        binary=str(binary), binary_sha256=hashlib.file_digest(binary.open("rb"),"sha256").hexdigest(),
        driver_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        helper_sha256=hashlib.sha256(helper_path.read_bytes()).hexdigest(),
        classification="Routine QA only; code-aware supporting evidence, not independent acceptance",
        clock="CLOCK_REALTIME/time frozen; monotonic and gettimeofday remain real",
        isolation="synthetic HOME/profile, debug keyring denial, loopback-only OS network"))
    for name, data in [("driver.py.gz",Path(__file__).read_bytes()),
                       ("helpers.py.gz",helpers.encode()),("expanded-helpers.py.gz",prior.encode()),
                       ("clock.c.gz",Path(__file__).with_name("fixture_clock.c").read_bytes())]:
        (out/name).write_bytes(gzip.compress(data,mtime=0))
    env["TZ"] = "America/New_York"
    sample("first","2026-03-08T06:59:59Z","gpt-5.6-sol")
    root, descendant = query("SELECT parent_thread_id,child_thread_id FROM thread_spawn_edges")[0]
    sample("spring-after","2026-03-08T07:00:00Z","gpt-5.6-sol",root)
    phase = "checkpoint"
    sample("spring-checkpoint","2026-03-10T12:00:00Z","gpt-5.6-sol",root)
    recalc()
    for zone in ["America/New_York","UTC","Asia/Kolkata","America/Phoenix"]:
        env["TZ"] = zone
        probe_path = out / ("timezone-"+zone.replace("/","-")+".json")
        env["ACCT_TZ_PROBE"] = str(probe_path)
        launch("spring-inspect-"+zone.replace("/","-"),root)
        timezone_controls.append(assert_timezone(probe_path, child.pid, zone, sample_utc))
        save("timezone-controls.json", timezone_controls)
        child.delaybeforesend = 0
        label = "spring-"+zone.replace("/","-")
        hours(label,"2026-03-08T06:00:00Z","2026-03-08T08:00:00Z",[
            ("2026-03-08T06:00:00Z","2026-03-08T07:00:00Z"),
            ("2026-03-08T07:00:00Z","2026-03-08T08:00:00Z")])
        day(label+"-day","2026-03-08")
        stop()
    env.pop("ACCT_TZ_PROBE")
    launch("before-deletion-inspect",root)
    child.delaybeforesend = 0
    day("before-deletion","2026-03-08")
    stop()
    delete_child(descendant)
    save("deleted-store-readback.json",dict(thread=descendant,
        native_rows=query("SELECT id FROM threads WHERE id=?",(descendant,)),
        compact_rows=query("SELECT thread_id,utc_day FROM draft_accounting_compact_days WHERE thread_id=?",(descendant,)),
        raw_rows=query("SELECT attempt_id FROM draft_accounting_attempts WHERE json_extract(payload,'$.thread_id')=?",(descendant,))))
    assert not query("SELECT id FROM threads WHERE id=?",(descendant,))
    assert not query("SELECT attempt_id FROM draft_accounting_attempts WHERE json_extract(payload,'$.thread_id')=?",(descendant,))
    launch("after-deletion-inspect",root)
    child.delaybeforesend = 0
    day("after-deletion","2026-03-08",deleted=True)
    stop()
    phase = "root"
    env["TZ"] = "America/New_York"
    sample("partial-leading","2026-10-31T23:00:00Z","gpt-5.6-sol",root)
    sample("fall-before","2026-11-01T05:59:59Z","gpt-5.6-sol",root)
    sample("fall-after","2026-11-01T06:00:00Z","gpt-5.6-terra",root)
    sample("partial-trailing","2026-11-02T00:00:00Z","gpt-5.6-terra",root)
    phase = "checkpoint"
    sample("fall-checkpoint","2026-11-04T12:00:00Z","gpt-5.6-sol",root)
    recalc()
    launch("fall-inspect",root)
    child.delaybeforesend = 0
    hours("fall","2026-11-01T05:00:00Z","2026-11-01T07:00:00Z",[
        ("2026-11-01T05:00:00Z","2026-11-01T06:00:00Z"),
        ("2026-11-01T06:00:00Z","2026-11-01T07:00:00Z")])
    day("fall-day","2026-11-01")
    day("empty-day","2026-11-03")
    command("/usage requests 2026-10-31T12:00:00Z 2026-11-02T12:00:00Z day")
    text = page("partial-days-overview")
    require(text,"Range total unavailable","partial days")
    for label in ["2026-10-31","2026-11-02"]:
        a = label+"T00:00:00Z"
        b = (datetime.fromisoformat(a.replace("Z","+00:00"))+timedelta(days=1)).isoformat().replace("+00:00","Z")
        open_link("Day ["+a.replace("Z",".000Z")+", "+b.replace("Z",".000Z")+")")
        name = "partial-day-"+label
        text = page(name)
        require(text,"Partial bucket — excluded from totals",name)
        assert "Known subtotal exact USD:" not in text
        record(name,a,b,members(a,b),[name],expected_state="unavailable-partial",
               requested_start="2026-10-31T12:00:00Z",requested_end="2026-11-02T12:00:00Z")
        send("\x1b")
    open_link("Day [2026-11-01T00:00:00.000Z, 2026-11-02T00:00:00.000Z)")
    text = page("partial-days-full-middle")
    assert_metrics(text,members("2026-11-01T00:00:00Z","2026-11-02T00:00:00Z"))
    record("partial-days-full-middle","2026-11-01T00:00:00Z","2026-11-02T00:00:00Z",
           members("2026-11-01T00:00:00Z","2026-11-02T00:00:00Z"),
           ["partial-days-overview","partial-days-full-middle"],expected_state="available-bucket-unavailable-range")
    send("\x1b")
    send("\x1b")
    # Discover and preserve the reader-facing rule for explicit offsets.
    send("/usage requests 2026-11-01T01:00:00-04:00 2026-11-01T02:00:00-05:00 hour")
    send("\r")
    drain(2)
    text = snap("offset-input")
    if "↑↓ scroll" in text:
        page("offset-input")
        send("\x1b")
    save("offset-input-result.json",dict(rendered=text))
    stop()
    # 90 days after 05:59:59: first fall attempt expires, 06:00 remains raw.
    phase = "checkpoint"
    sample("mixed-checkpoint","2027-01-30T05:59:59Z","gpt-5.6-sol",root)
    recalc()
    state = dict(raw=query("SELECT attempt_id,payload FROM draft_accounting_attempts"),
                 compact=query("SELECT thread_id,utc_day,payload FROM draft_accounting_compact_days"),
                 checkpoint=query("SELECT * FROM draft_accounting_retention_checkpoint"))
    save("mixed-store-readback.json",state)
    fall_raw = [json.loads(r[1]) for r in state["raw"] if
                epoch("2026-11-01T00:00:00Z") <= json.loads(r[1])["dispatched_at_ms"] < epoch("2026-11-02T00:00:00Z")]
    assert len(fall_raw)==1 and fall_raw[0]["dispatched_at_ms"]==epoch("2026-11-01T06:00:00Z"), fall_raw
    assert any(r[1]==epoch("2026-11-01T00:00:00Z")//86400000 for r in state["compact"])
    launch("mixed-inspect",root)
    child.delaybeforesend = 0
    day("mixed-day","2026-11-01",unavailable=True)
    command("/usage requests 2026-11-01T05:00:00Z 2026-11-01T07:00:00Z hour")
    page("mixed-hours-overview")
    open_link("Hour [2026-11-01T05:00:00.000Z, 2026-11-01T06:00:00.000Z)")
    text = page("mixed-compact-hour")
    assert_compact_hour(text)
    assert "Known subtotal exact USD:" not in text
    record("mixed-compact-hour","2026-11-01T05:00:00Z","2026-11-01T06:00:00Z",
           members("2026-11-01T05:00:00Z","2026-11-01T06:00:00Z"),
           ["mixed-hours-overview","mixed-compact-hour"],expected_state="unavailable-precision")
    send("\x1b")
    open_link("Hour [2026-11-01T06:00:00.000Z, 2026-11-01T07:00:00.000Z)")
    text = page("mixed-raw-hour")
    rows = members("2026-11-01T06:00:00Z","2026-11-01T07:00:00Z")
    assert_metrics(text,rows)
    detail_members("mixed-raw-hour",rows)
    record("mixed-raw-hour","2026-11-01T06:00:00Z","2026-11-01T07:00:00Z",rows,["mixed-raw-hour"],expected_state="available")
    send("\x1b")
    send("\x1b")
    command("/usage requests 2025-01-01 2025-01-02 day")
    text = page("expired-range")
    assert "Known subtotal exact USD:" not in text
    require(text,"Range total unavailable","expired range")
    record("expired-range","2025-01-01T00:00:00Z","2025-01-02T00:00:00Z",[],["expired-range"],expected_state="unavailable-expired")
    send("\x1b")
    day("deleted-history-after-retention","2026-03-08",deleted=True,unavailable=True)
    assert not query("SELECT id FROM threads WHERE id=?",(descendant,))
    assert not query("SELECT thread_id FROM draft_accounting_compact_days WHERE thread_id=?",(descendant,))
    save("dst-conversions.json",[
        dict(utc=s,local=datetime.fromisoformat(s.replace("Z","+00:00")).astimezone(ZoneInfo("America/New_York")).isoformat())
        for s in ["2026-03-08T06:59:59Z","2026-03-08T07:00:00Z","2026-11-01T05:59:59Z","2026-11-01T06:00:00Z"]])
    save("completion.json",dict(passed=True,cases=len(cases),checkpoint_role="selected root, outside compared ranges"))
finally:
    stop()
    save("requests.json",receipts)
    save("keys.json",keys)
    for path in out.glob("*"):
        if path.suffix in (".raw",".txt",".log"):
            path.with_suffix(path.suffix+".gz").write_bytes(gzip.compress(path.read_bytes(),mtime=0))
    server.shutdown()
