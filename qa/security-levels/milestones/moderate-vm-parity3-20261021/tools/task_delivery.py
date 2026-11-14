"""utm-par3 (#1175, D95/D140): how MOD-21's task reached the product's models in one T17 run.

Part 1, the guest mediator's flags (written for every product request; no text is logged): for each request in the
tester's window that carries the task in some message, the message's role and whether the task sits inside a
`corbanu_untrusted_data` or `inter_agent_message` element. Part 2, the product's session files (withheld; from
collected.tar): utm-par2's excerpt_tasks.py output (user messages with their recorded origin, mailbox deliveries).
usage: task_delivery.py RUN_DIR > task-delivery.txt"""
import datetime
import json
import pathlib
import subprocess
import sys

run = pathlib.Path(sys.argv[1])


def ns(stamp):
    return datetime.datetime.strptime(stamp.strip(), "%Y-%m-%dT%H:%M:%SZ").replace(
        tzinfo=datetime.timezone.utc).timestamp() * 1e9


t0, t1 = ns((run / "executor-start.txt").read_text()) - 120e9, ns((run / "executor-end.txt").read_text()) + 5e9
rows = []
for line in (run / "mediator-at-end.jsonl").read_text().splitlines():
    try:
        e = json.loads(line)
    except ValueError:
        continue
    if e.get("event") == "forwarded" and e.get("role") == "product" and t0 <= e.get("time_ns", 0) <= t1:
        rows.append(e)
carrying = [e for e in rows if e.get("task_messages")]
print("== part 1: guest mediator flags (%s)" % run.name)
print("product requests in the window: %d; carrying the task: %d" % (len(rows), len(carrying)))
wrapped = [e for e in carrying if any(m["in_untrusted"] or m["in_inter_agent"] for m in e["task_messages"])]
print("requests carrying the task inside corbanu_untrusted_data or inter_agent_message: %d" % len(wrapped))
for e in carrying:
    msgs = "; ".join("#%d %s%s%s" % (m["index"], m["role"], " IN-UNTRUSTED" if m["in_untrusted"] else "",
                                     " IN-INTER-AGENT" if m["in_inter_agent"] else "") for m in e["task_messages"])
    print("  %s model=%s n_messages=%s status=%s task in: %s" % (e["ts"], e.get("model"), e.get("n_messages"),
                                                              e.get("status"), msgs))
print()
print("== part 2: product session files (integrator excerpt; files withheld)")
sys.stdout.flush()
subprocess.run([sys.executable, str(pathlib.Path(__file__).with_name("excerpt_tasks.py")), str(run / "collected.tar")],
               check=True)
