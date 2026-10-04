"""Verify disjoint saved store rows, without inferring empty-page session roots."""
import hashlib
import json
from pathlib import Path
import sys

here = Path(__file__).resolve().parent
source61 = here.parent / "acct-readers-61/reader-run-05"
source62 = here.parent / "acct-scope-62/scope-run-01"


def compare(bindings, identities, before, during):
    left = [json.loads(attempt) for attempt, _ in bindings]
    right = []
    for row in before["draft_accounting_attempts"]:
        payload = json.loads(row["payload"])
        assert (row["attempt_id"], row["request_id"]) == (
            payload["attempt_id"], payload["request_id"]), "row/payload identity mismatch"
        right.append(payload)
    assert len(left) == len(right) == 4, "expected four rows per read-back"
    assert before == during, "round-62 store changed during fresh reader"
    assert set(identities) == {"historical_and_mixed", "missing_price", "no_usage"}
    assert {a["thread_id"] for a in left} == set(identities.values()), "round-61 thread mapping mismatch"
    populations = {}
    for key, count in (("attempt_id", 4), ("request_id", 4), ("thread_id", 3)):
        a, b = ({row[key] for row in rows} for rows in (left, right))
        assert len(a) == len(b) == count, ("population cardinality mismatch", key)
        assert a.isdisjoint(b), ("saved populations overlap", key)
        populations[key] = dict(round61=sorted(a), round62=sorted(b), intersection=[])
    return populations


def main():
    output = Path(sys.argv[1])
    output.mkdir(parents=True, exist_ok=False)
    paths = [source61 / "native-bindings.json", source61 / "identities.json",
             source62 / "store-before-fresh-reader.json", source62 / "store-during-fresh-reader.json"]
    inputs = [json.loads(p.read_text()) for p in paths]
    populations = compare(*inputs)
    controls = []
    # Mutate the actual fields used by each comparison; require the named reason.
    for key in ("attempt_id", "request_id", "thread_id"):
        copied = json.loads(json.dumps(inputs))
        first = json.loads(copied[0][0][0])
        row = copied[2]["draft_accounting_attempts"][0]
        payload = json.loads(row["payload"])
        if key == "thread_id":
            old = payload[key]
            for record in copied[2]["draft_accounting_attempts"]:
                item = json.loads(record["payload"])
                if item[key] == old:
                    item[key] = first[key]
                    record["payload"] = json.dumps(item)
        else:
            row[key] = first[key]
            payload[key] = first[key]
            row["payload"] = json.dumps(payload)
        copied[3] = json.loads(json.dumps(copied[2]))
        try:
            compare(*copied)
        except AssertionError as error:
            assert error.args == (("saved populations overlap", key),), error
            controls.append(dict(mutation="overlap-" + key, error=str(error)))
        else:
            raise AssertionError("overlap accepted: " + key)
    copied = json.loads(json.dumps(inputs))
    copied[3]["draft_accounting_attempts"].pop()
    try:
        compare(*copied)
    except AssertionError as error:
        assert str(error) == "round-62 store changed during fresh reader", error
        controls.append(dict(mutation="changed-during-reader", error=str(error)))
    else:
        raise AssertionError("changed store accepted")
    report = dict(passed=True, rows_per_dataset=4, populations=populations,
                  round62_before_during_equal=True, controls=controls,
                  sources=[dict(path=str(p.relative_to(here.parent)),
                                sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in paths],
                  limitation="Disjoint saved store attempt/request/thread identifiers only. "
                  "This does not bind the root or attempt population resolved by either "
                  "byte-identical empty page, prove capture-time store attachment, or "
                  "independently authenticate the saved read-backs.")
    (output / "results.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(dict(rows_per_dataset=4, disjoint_identity_fields=3,
                          unchanged_round62=True, negative_controls=len(controls))))


if __name__ == "__main__":
    main()
