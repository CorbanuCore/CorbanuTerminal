# /// script
# requires-python = ">=3.11"
# dependencies = ["pexpect==4.9.0", "pyte==0.8.2"]
# ///
"""Supporting code-aware PTY evidence. Reuses the frozen acct-qualify-50 driver helpers.
Run: uv run --script qualify.py BINARY OUTPUT --mode collect
The profile and exact-clock assets are local synthetic fixtures, not release assets.
"""
from pathlib import Path
import hashlib

prior_path = Path(__file__).resolve().parent.parent / "acct-qualify-50/qualify.py"
prior = prior_path.read_text().split("\ntry:\n    driver_bytes", 1)[0]
# Reuse initialization/navigation, not its old qualification or expectations.
prior = prior.replace('kind = "child" if body.get("model") == "gpt-5.6-terra" else phase',
    'kind = "child" if self.headers.get("x-codex-parent-thread-id") else phase')
prior = prior.replace('inp, cached, output, reasoning = USAGE[kind]',
    'inp, cached, output, reasoning = USAGE[kind]\n            inp += index')
prior = prior.replace('fixture_utc=datetime.fromtimestamp(time.time()+clock_offset, timezone.utc).isoformat()',
    'fixture_utc=sample_utc')
prior = prior.replace('model="gpt-5.6-terra",', 'model="gpt-5.6-sol",')
prior = prior.replace('f"ACCT_QUALIFY_CLOCK_OFFSET_SECONDS={clock_offset}"',
    'f"ACCT_QUALIFY_CLOCK_MS={epoch(sample_utc)}"')
prior = prior.replace('lambda: "gpt-5.6-sol" in visible()', 'lambda: active_model in visible()')
prior = prior.replace('cmd += ["resume", resume]', 'cmd += ["resume", resume, "--model", active_model]')
prior = prior.replace('def drain(seconds=.12):', 'def drain(seconds=.035):')
prior = prior.replace('timeout=.04', 'timeout=.01')
exec(compile(prior, str(prior_path), "exec"))
sample_utc = "2026-08-30T23:00:00Z"
active_model = "gpt-5.6-sol"
USAGE["checkpoint"] = USAGE["root"]
summary = {"buckets": [], "groups": [], "unknown_causes": [], "partial": []}
expected = []
all_attempt_ids = set()
def save(name, value):
    (out / name).write_text(json.dumps(value, indent=2)+"\n")
def epoch(value):
    return int(datetime.fromisoformat(value.replace("Z", "+00:00")).timestamp()*1000)
def set_clock(value):
    global sample_utc
    sample_utc = value
    (fixture / "clock-ms").write_text(str(epoch(value)))
def select_model(model):
    global active_model
    active_model = model
    text = (profile / "config.toml").read_text()
    import re
    text = re.sub(r'^model = "[^"]+"', 'model = "'+model+'"', text, flags=re.MULTILINE)
    (profile / "config.toml").write_text(text)
def sample(name, when, model, resume=None):
    set_clock(when)
    select_model(model)
    before = len(receipts)
    launch(name, resume)
    child.delaybeforesend = 0
    send("Use the fixture response.")
    send("\r")
    if name == "first":
        wait_for(lambda: counts.get("root",0) >= 2 and counts.get("child",0) == 1
                 and "QUALIFY root complete." in visible(), name, 180)
    else:
        wait_for(lambda: len(receipts)>before and "QUALIFY "+phase+" complete." in visible(), name)
    drain(2)
    snap(name+"-complete")
    stop()
    return receipts[before:]
def cost(rows):
    return sum((Decimal(r["exact_usd"]) for r in rows), Decimal(0)).normalize()
def assert_metrics(text, rows):
    require(text, "Known subtotal exact USD: "+format(cost(rows), "f"), "exact independent total")
    for label, val in [
        ("Input", sum(r["input"] for r in rows)),
        ("Noncached input (derived for inclusive input)", sum(r["input"]-r["cached"] for r in rows)),
        ("Cache read", sum(r["cached"] for r in rows)),
        ("Cache write", 0),
        ("Output", sum(r["output"] for r in rows)),
        ("Reasoning (subset, not separately billed)", sum(r["reasoning"] for r in rows)),
        ("Total (not separately billed)", sum(r["input"]+r["output"] for r in rows)),
    ]:
        require(text, f"{label}: {val} known + unknown in 0 attempts", label)
def selected(name):
    return json.loads((out / (name+"-selected.json")).read_text())
def detail_members(name, rows):
    # Traverse every rendered request/attempt. Expected token/time/model tuples
    # come exclusively from server emissions and chosen clock inputs.
    found = []
    for request_label in [s for s in selected(name) if s.startswith("Request ") and s[8:].isdigit()]:
        open_link(request_label)
        req_name = name+"-"+request_label.replace(" ", "-")
        page(req_name)
        for attempt_label in [s for s in selected(req_name) if s.startswith("Attempt ") and s[8:].isdigit()]:
            open_link(attempt_label)
            detail_name = req_name+"-"+attempt_label.replace(" ", "-")
            text = page(detail_name)
            lines = selected(detail_name)
            def field(prefix):
                return next(s[len(prefix):] for s in lines if s.startswith(prefix))
            item = dict(attempt=field("Attempt: "), model=field("Model: "),
                        admitted_ms=int(field("Admission time: ").split()[0]),
                        input=int(field("Input: ")), exact_usd=field("Known subtotal exact USD: "))
            matches = [r for r in rows if r["model"]==item["model"] and r["input"]==item["input"]
                       and epoch(r["fixture_utc"])==item["admitted_ms"]]
            assert len(matches)==1, (name, item, matches)
            require(text, "Known subtotal exact USD: "+matches[0]["exact_usd"], detail_name)
            item["emission"] = matches[0]["emission"]
            found.append(item)
            send("\x1b")
        send("\x1b")
    assert sorted(r["emission"] for r in found)==sorted(r["emission"] for r in rows), (name, found, rows)
    assert len({r["attempt"] for r in found})==len(found), name
    return found
def bucket(group, start, end, windows, label):
    command(f"/usage requests {start} {end} {group}")
    page(label+"-overview")
    for index, (a,b) in enumerate(windows):
        link = f"{group.title()} [{a.replace('Z','.000Z')}, {b.replace('Z','.000Z')})"
        open_link(link)
        name = label+"-"+str(index)
        text = page(name)
        rows = [r for r in expected if r["kind"]!="orphan" and epoch(a)<=epoch(r["fixture_utc"])<epoch(b)]
        assert_metrics(text, rows)
        members = detail_members(name, rows)
        summary["buckets"].append(dict(page=name, group=group, start=a, end=b, exact_usd=str(cost(rows)),
                                       members=members, excluded_emissions=[
                                           r["emission"] for r in expected if r["kind"]!="orphan" and r not in rows]))
        send("\x1b")
    send("\x1b")
    save("results.json", summary)
try:
    (out / "clock.c.gz").write_bytes(gzip.compress(Path(__file__).with_name("fixture_clock.c").read_bytes(),mtime=0))
    (out / "driver.py.gz").write_bytes(gzip.compress(Path(__file__).read_bytes(),mtime=0))
    (out / "expanded-helpers.py.gz").write_bytes(gzip.compress(prior.encode(),mtime=0))
    save("manifest.json", dict(commit=subprocess.check_output(["git","rev-parse","HEAD"],cwd=repo,text=True).strip(),
        binary=str(binary), binary_sha256=hashlib.file_digest(binary.open("rb"),"sha256").hexdigest(),
        driver_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        helper_sha256=hashlib.sha256(prior_path.read_bytes()).hexdigest(),
        clock_source_sha256=hashlib.sha256(Path(__file__).with_name("fixture_clock.c").read_bytes()).hexdigest(),
        clock_binary_sha256=hashlib.sha256(Path(__file__).with_name("fixture_clock.dylib").read_bytes()).hexdigest(),
        qualification="code-aware developer evidence, not independent acceptance",
        clock="fixed exact wall-clock per stopped-process launch; monotonic timers remain real",
        isolation="synthetic HOME and all profile aliases; debug keyring denial; loopback-only OS network rule"))
    sample("first", "2026-08-30T23:00:00Z", "gpt-5.6-sol")
    root, descendant = query("SELECT parent_thread_id,child_thread_id FROM thread_spawn_edges")[0]
    sample("week-boundary", "2026-08-31T00:00:00Z", "gpt-5.6-sol", root)
    sample("third-hour", "2026-08-31T23:00:00Z", "gpt-5.6-terra", root)
    sample("month-boundary", "2026-09-01T00:00:00Z", "gpt-5.6-terra", root)
    phase = "orphan"
    sample("unrelated", "2026-09-01T00:00:00Z", "gpt-5.6-sol")
    orphan = next(r[0] for r in query("SELECT DISTINCT json_extract(payload,'$.thread_id') FROM draft_accounting_attempts")
                  if r[0] not in [root,descendant])
    for i,row in enumerate(receipts):
        rate = RATE[row["model"]]
        noncached, cache, output = map(Decimal,rate)
        amount = ((row["input"]-row["cached"])*noncached+row["cached"]*cache+row["output"]*output)/Decimal(1000000)
        expected.append(dict(**row, emission=i, rates_per_million=rate, exact_usd=str(amount)))
    save("independent-arithmetic.json", expected)
    save("identities.json",dict(root=root,child=descendant,unrelated=orphan))
    bindings = query("SELECT a.payload,s.payload FROM draft_accounting_attempts a JOIN draft_accounting_price_bindings b USING(attempt_id) LEFT JOIN draft_accounting_price_snapshots s USING(snapshot_id)")
    save("bound-prices-and-attempts.json", bindings)
    for a,s in bindings:
        a,s = json.loads(a),json.loads(s)
        assert tuple(Decimal(s["rates"][k]) for k in ["noncached","read","output"])==tuple(map(Decimal,RATE[a["model"]]))
    # Selected-root checkpoint at Oct 2 lies outside every compared range.
    phase = "checkpoint"
    sample("checkpoint", "2026-10-02T12:00:00Z", "gpt-5.6-sol", root)
    launch("inspect", root)
    child.delaybeforesend = 0
    bucket("hour","2026-08-30T23:00:00Z","2026-08-31T01:00:00Z",[
        ("2026-08-30T23:00:00Z","2026-08-31T00:00:00Z"),
        ("2026-08-31T00:00:00Z","2026-08-31T01:00:00Z")],"week-edge-hours")
    bucket("hour","2026-08-31T23:00:00Z","2026-09-01T01:00:00Z",[
        ("2026-08-31T23:00:00Z","2026-09-01T00:00:00Z"),
        ("2026-09-01T00:00:00Z","2026-09-01T01:00:00Z")],"month-edge-hours")
    bucket("week","2026-08-24","2026-09-07",[
        ("2026-08-24T00:00:00Z","2026-08-31T00:00:00Z"),
        ("2026-08-31T00:00:00Z","2026-09-07T00:00:00Z")],"weeks")
    bucket("month","2026-08-01","2026-10-01",[
        ("2026-08-01T00:00:00Z","2026-09-01T00:00:00Z"),
        ("2026-09-01T00:00:00Z","2026-10-01T00:00:00Z")],"months")
    # Same root, day containing Sol own + Sol descendant, compared with the
    # whole August month which also has Terra own attempts.
    command("/usage requests 2026-08-01 2026-09-01 month")
    page("breakdown-overview")
    open_link("Month *")
    page("breakdown-bucket")
    august = [r for r in expected if r["kind"]!="orphan" and epoch(r["fixture_utc"])<epoch("2026-09-01T00:00:00Z")]
    for label,rows in [
        ("Root's own attempts",[r for r in august if r["kind"]=="root"]),
        ("Descendant attempts",[r for r in august if r["kind"]=="child"]),
        ("Provider: openai; Model: gpt-5.6-sol",[r for r in august if r["model"]=="gpt-5.6-sol"]),
        ("Provider: openai; Model: gpt-5.6-terra",[r for r in august if r["model"]=="gpt-5.6-terra"]),
    ]:
        open_link(label)
        name="group-"+str(len(summary["groups"]))
        text=page(name)
        require(text,"Known estimate exact USD: "+format(cost(rows),"f"),label)
        require(text,"Recorded attempts: "+str(len(rows)),label)
        summary["groups"].append(dict(label=label,emissions=[r["emission"] for r in rows],exact_usd=str(cost(rows))))
        send("\x1b")
    send("\x1b")
    send("\x1b")
    # Edges inside the first hour: no partial monetary total; following full
    # bucket still exposes only its own boundary attempt.
    command("/usage requests 2026-08-30T23:30:00Z 2026-08-31T01:00:00Z hour")
    overview=page("partial-overview")
    require(overview,"Range total unavailable","partial range")
    open_link("Hour [2026-08-30T23:00:00.000Z, 2026-08-31T00:00:00.000Z)")
    text=page("partial-bucket")
    require(text,"Partial bucket — excluded from totals","partial bucket")
    assert "Known subtotal exact USD:" not in text and "Estimated token cost for recorded attempts:" not in text
    summary["partial"].append(dict(request_start="2026-08-30T23:30:00Z",bucket_start="2026-08-30T23:00:00Z",
        total_withheld=True,excluded_emissions=[r["emission"] for r in expected if r["fixture_utc"]=="2026-08-30T23:00:00Z"]))
    send("\x1b")
    open_link("Hour [2026-08-31T00:00:00.000Z, 2026-08-31T01:00:00.000Z)")
    text=page("partial-full-neighbour")
    rows=[r for r in expected if r["fixture_utc"]=="2026-08-31T00:00:00Z"]
    assert_metrics(text,rows)
    summary["partial"][0]["full_neighbour_members"]=detail_members("partial-full-neighbour",rows)
    send("\x1b")
    send("\x1b")
    # All faults alter only native ancestry metadata of a truly sampled
    # unrelated request. Never modify accounting attempts/usage/prices.
    original_source=query("SELECT source FROM threads WHERE id=?",(orphan,))[0][0]
    edge=query("SELECT * FROM thread_spawn_edges LIMIT 1")[0]
    with sqlite3.connect(database()) as db:
        columns=[r[1] for r in db.execute("PRAGMA table_info(thread_spawn_edges)")]
    def source(parent):
        return json.dumps({"subagent":{"thread_spawn":{"parent_thread_id":parent,"depth":1}}})
    missing=str(uuid.uuid4())
    faults=[
        ("malformed-source","malformed",root),
        ("absent-ancestor",source(missing),missing),
        ("conflicting-source-edge",source(descendant),root),
        ("cycle",source(orphan),orphan),
        ("source-without-edge",source(root),None),
    ]
    for label,src,parent in faults:
        stop()
        with sqlite3.connect(database()) as db:
            db.execute("DELETE FROM thread_spawn_edges WHERE child_thread_id=?",(orphan,))
            db.execute("UPDATE threads SET source=? WHERE id=?",(src,orphan))
            if parent:
                values=dict(zip(columns,edge))
                values.update(parent_thread_id=parent,child_thread_id=orphan)
                db.execute("INSERT INTO thread_spawn_edges ("+",".join(columns)+") VALUES ("+",".join("?" for _ in columns)+")",list(values.values()))
        name="unknown-"+label
        launch(name+"-fresh-inspect", root)
        child.delaybeforesend = 0
        # Read the actual committed cause immediately before issuing the UI command.
        with sqlite3.connect(database()) as db:
            actual_source = db.execute("SELECT source FROM threads WHERE id=?", (orphan,)).fetchone()[0]
            actual_edges = db.execute("SELECT parent_thread_id FROM thread_spawn_edges WHERE child_thread_id=?", (orphan,)).fetchall()
        assert actual_source == src and actual_edges == ([(parent,)] if parent else []), label
        readback = dict(cause=label, thread=orphan, source=actual_source,
                        edge_parents=[r[0] for r in actual_edges], inspect_pid=child.pid,
                        read_at_monotonic_ns=time.monotonic_ns())
        save(name+"-store-readback.json", readback)
        command("/usage requests 2026-09-01")
        text=page(name)
        rows=[r for r in expected if r["kind"]=="root" and r["fixture_utc"]=="2026-09-01T00:00:00Z"]
        assert_metrics(text,rows)
        require(text,"Unknown parent population: 1 attempts, excluded from root total",label)
        root_members=detail_members(name,rows)
        open_link("Unknown parent population")
        page(name+"-population")
        open_link("Request *")
        detail=page(name+"-attempt")
        unknown=[r for r in expected if r["kind"]=="orphan"]
        require(detail,"Thread: "+orphan,label)
        require(detail,"Known subtotal exact USD: "+format(cost(unknown),"f"),label)
        summary["unknown_causes"].append(dict(cause=label,source=src,edge_parent=parent,readback=readback,
            start="2026-09-01T00:00:00Z",end="2026-09-02T00:00:00Z",
            unknown_emissions=[r["emission"] for r in unknown],unknown_exact_usd=str(cost(unknown)),
            root_members=root_members,root_exact_usd=str(cost(rows))))
        send("\x1b")
        send("\x1b")
        send("\x1b")
        save("results.json",summary)
    with sqlite3.connect(database()) as db:
        db.execute("DELETE FROM thread_spawn_edges WHERE child_thread_id=?",(orphan,))
        db.execute("UPDATE threads SET source=? WHERE id=?",(original_source,orphan))
    summary["passed"]=True
    save("results.json",summary)
finally:
    stop()
    save("requests.json",receipts)
    save("keys.json",keys)
    for path in out.glob("*"):
        if path.suffix in (".raw",".txt"):
            path.with_suffix(path.suffix+".gz").write_bytes(gzip.compress(path.read_bytes(),mtime=0))
    server.shutdown()
