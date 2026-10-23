# /// script
# requires-python = ">=3.11"
# dependencies = ["pexpect==4.9.0", "pyte==0.8.2"]
# ///
"""Four actually collected priced requests across three roots, then a fresh reader."""
from pathlib import Path
import sys

reader_path = Path(__file__).resolve().parent.parent / "acct-readers-61/readers.py"
sys.path.insert(0, str(reader_path.parent))
reader_helpers = reader_path.read_text().split("\ntry:\n    save(\"manifest.json\"", 1)[0]
exec(compile(reader_helpers, str(reader_path), "exec"))
env["DYLD_INSERT_LIBRARIES"] = str(reader_path.with_name("tz_clock.dylib"))

def read_store():
    with sqlite3.connect(database()) as db:
        tables = [r[0] for r in db.execute(
            "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'draft_accounting_%' ORDER BY name")]
        result = {}
        for table in tables:
            cursor = db.execute('SELECT * FROM "' + table + '"')
            columns = [d[0] for d in cursor.description]
            result[table] = [dict(zip(columns, row)) for row in cursor.fetchall()]
        return result

try:
    save("manifest.json", dict(
        commit=subprocess.check_output(["git","rev-parse","HEAD"],cwd=repo,text=True).strip(),
        binary=str(binary),binary_sha256=hashlib.file_digest(binary.open("rb"),"sha256").hexdigest(),
        driver_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        reader_helpers_sha256=hashlib.sha256(reader_helpers.encode()).hexdigest(),
        qualification="Code-aware developer reproduction, not independent acceptance",
        isolation="Synthetic HOME/profile, debug native-keyring denial, OS loopback-only network"))
    (out/"driver.py.gz").write_bytes(gzip.compress(Path(__file__).read_bytes(),mtime=0))
    (out/"reader-helpers.py.gz").write_bytes(gzip.compress(reader_helpers.encode(),mtime=0))
    first = one("root-a-first", "2026-08-10T12:00:00Z", "gpt-5.6-sol")
    one("root-a-second", "2026-08-10T12:00:00Z", "gpt-5.6-sol", resume=first)
    second = one("root-b", "2026-08-10T12:00:00Z", "gpt-5.6-sol")
    third = one("root-c", "2026-08-10T12:00:00Z", "gpt-5.6-sol")
    before = read_store()
    save("store-before-fresh-reader.json", before)
    bindings = query("SELECT a.payload,s.payload FROM draft_accounting_attempts a "
                     "JOIN draft_accounting_price_bindings b USING(attempt_id) "
                     "LEFT JOIN draft_accounting_price_snapshots s USING(snapshot_id)")
    save("native-bindings.json", bindings)
    attempts = [json.loads(a) for a, _ in bindings]
    assert len(attempts) == 4 and len({a["thread_id"] for a in attempts}) == 3
    assert all(a["dispatched_at_ms"] == epoch("2026-08-10T12:00:00Z") for a in attempts)
    rates = {"noncached": Decimal(5), "read": Decimal(".5"), "output": Decimal(30)}
    assert all(s and {k:Decimal(json.loads(s)["rates"][k]) for k in rates} == rates for _,s in bindings)
    assert len(receipts) == 4 and all(e["usage_present"] for e in receipts)
    for e in receipts:
        assert (e["input"],e["cached"],e["output"]) == (100,20,10)
    amount = ((Decimal(100)-20)*5 + Decimal(20)*Decimal(".5") + Decimal(10)*30)/1000000
    assert amount == Decimal(".00071")
    configure("gpt-5.6-sol")
    launch("fresh-scope-reader")
    fresh_pid = child.pid
    command("/usage requests 2026-08-10")
    text = page("fresh-scope-zero")
    assert_day(text, "2026-08-10")
    lines = selected("fresh-scope-zero")
    for phrase in ("No recorded attempts in this day; collection coverage unknown.",
                   "Known subtotal exact USD: 0", "oldest recorded day: None"):
        require(text, phrase, "scope-zero reproduction")
    assert not any(s.startswith("Request ") for s in lines)
    after = read_store()
    save("store-during-fresh-reader.json", after)
    assert before == after, "Inspection changed accounting evidence"
    close()
    controls = []
    for label, root, count in (("root-a-control",first,2),("root-b-control",second,1),("root-c-control",third,1)):
        text = capture(label, "2026-08-10", root)
        total = format(amount * count, "f")
        require(text, "Known subtotal exact USD: "+total, label)
        inspect_all_attempts(label)
        controls.append(dict(page=label,thread=root,attempt_count=count,expected_exact_usd=total))
        close()
    save("scope-zero-result.json", dict(
        reproduced=True,day="2026-08-10",fresh_reader_pid=fresh_pid,rendered=lines,
        collected_attempts=attempts,attempt_count=4,priced_attempt_count=4,thread_count=3,
        per_attempt_expected_usd=str(amount),store_known_day_usd=str(amount*4),
        original_store_preserved=True,root_controls=controls,
        verdict="P2 scope misread: a fresh root shows zero while the same store holds 4 priced attempts that day. This is not a day-wide total.",
        proposed_text="No recorded attempts for this session and its resolved descendants on 2026-08-10. Other sessions are excluded. This is not your total spend for the day. Known subtotal for this session: USD 0; oldest recorded day in this session: none."))
finally:
    stop()
    save("emissions.json",receipts)
    save("keys.json",keys)
    for path in out.glob("*"):
        if path.suffix in (".raw",".txt",".log"):
            path.with_suffix(path.suffix+".gz").write_bytes(gzip.compress(path.read_bytes(),mtime=0))
    server.shutdown()
