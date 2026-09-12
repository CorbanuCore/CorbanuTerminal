"""Read-only operations notices. Links resolve only inside the published corpus."""
import hashlib
import html
import re
from collections import Counter

SPRINT = re.compile(r"\bPF-\d{2}-S\d{2}\b")
HISTORY = "docs/plans/delivery-history-reconciliation.md"
LEGACY = "PF-76-S01 history held for source reconciliation; not assigned to a main initiative."


def document_url(path):
    return "doc-" + hashlib.sha256(path.encode()).hexdigest()[:20] + ".html"


def issue(problem, count=1):
    """Explanations, not inferred decisions or changes to the underlying warning."""
    item = dict(summary=problem, owner="Corbanu manager", kind="Manager investigation",
                background="The source validator reported this condition. Detailed cause has not yet been recorded.",
                impact="The affected source or report cannot be treated as reconciled evidence; the manager must determine the scope.",
                next_step="Inspect the source validation/report evidence and record the cause, affected work and proposed remedy.",
                question=None, legacy=False, count=count)
    if problem == LEGACY:
        item.update(summary="PF-76-S01 historical ID conflict — reports held for reconciliation",
                    kind="Manager bookkeeping", legacy=True,
                    background="The recovery source used PF-76-S01 for delivery control. Main uses that same ID for unrelated provider-profile persistence. The receiving delivery-control sprint is PF-80-S01.",
                    impact="Historical reports are excluded from current initiative joins to avoid crediting the wrong sprint. This notice does not block current PF-80-S01 implementation or ask you to approve it. Historical queued events stay held; they are not rewritten or replayed.",
                    next_step="Inventory historical reports and queued events against their original source and task mappings; record each disposition before any explicit migration. Preserve original IDs and receipts. Keep live posting OFF.")
    elif problem.startswith("A worker report was rejected"):
        item.update(background="A report failed validation or did not identify a known sprint. Its contents are deliberately omitted here.",
                    impact="That report is not included in current progress. This is missing evidence, not proof that the worker stopped.",
                    next_step="Inspect the rejected report locally, identify its source and fix the producer or mapping; do not invent a progress state.")
    elif problem.startswith("Writeback status is unreadable"):
        item.update(background="The saved writeback observation could not be read or validated.",
                    impact="Delivery status is unknown. Dashboard rendering does not prove a Task Node post succeeded.",
                    next_step="Inspect the writeback status producer and restore a valid observation; do not retry an uncertain external post automatically.")
    return item


def render(items, sprints, documents):
    if not items:
        return ""
    destinations = {}
    for sprint in sprints:
        path = sprint["path"]
        if path in documents:
            destinations.setdefault(sprint["sprint_id"], set()).add(path)
    unavailable = set()

    def linked(text, legacy=False):
        chunks, start = [], 0
        for match in SPRINT.finditer(text):
            sid = match.group()
            paths = {HISTORY} if legacy and sid == "PF-76-S01" and HISTORY in documents else destinations.get(sid, set())
            # A missing legacy explanation must never fall through to modern PF76.
            if legacy and sid == "PF-76-S01" and HISTORY not in documents:
                paths = set()
            if len(paths) == 1:
                href = document_url(next(iter(paths)))
            else:
                unavailable.add(sid)
                href = "#attention-context-" + sid
            chunks += [html.escape(text[start:match.start()]), f'<a href="{href}">{sid}</a>']
            start = match.end()
        return ''.join(chunks) + html.escape(text[start:])

    body = '<aside class="notice attention" id="attention"><h2>Operations notices</h2><p class="muted">Manager-owned follow-up is separate from decisions that need you. Expand a summary for context.</p>'
    for item in items:
        key = hashlib.sha256(item["summary"].encode()).hexdigest()[:16]
        text = lambda value: linked(value, item.get("legacy", False))
        category = "Needs your decision" if item.get("question") else item["kind"]
        body += f'<details class="attention-item" id="notice-{key}"><summary>{text(item["summary"])} <span class="badge">{html.escape(category)}</span></summary><div class="attention-body">'
        for label, field in (("Owner", "owner"), ("What happened", "background"), ("Impact", "impact"), ("Next step", "next_step")):
            body += f'<p><strong>{label}</strong> {text(item[field])}</p>'
        if item.get("question"):
            body += '<p class="decision-question"><strong>Question for you</strong> ' + text(item["question"]) + '</p><p>This view is read-only. Answer in the manager task; Slack reply routing is not connected yet.</p>'
        else:
            body += '<p><strong>Your input</strong> No decision needed from you now. The manager owns the next action and will ask a specific question if a product decision is needed.</p>'
        if item.get("count", 1) > 1:
            body += f'<small>{int(item["count"])} source warnings grouped here; no historical records changed.</small>'
        body += '</div></details>'
    for sid in sorted(unavailable):
        body += f'<div class="attention-context" id="attention-context-{sid}" tabindex="-1"><h3>Unresolved sprint reference: <a href="#attention-context-{sid}">{sid}</a></h3><p>No unique published context is available for this reference. The manager must reconcile it; no substitute sprint has been selected.</p><a href="#attention">Back to notices</a></div>'
    return body + '</aside>'


def notices(data):
    counts = Counter(data["problems"])
    return render([issue(problem, count) for problem, count in counts.items()],
                  data["sprints"]["sprints"], data.get("documents", {}))


def render_decisions(raw, now, sprints, documents):
    """Pure, unconnected fixture fragment; documents is the approved safe corpus."""
    from decisions import FRESH_SECONDS, project, stamp

    view = project(raw, now)
    body = '<section id="decisions"><h2>Needs your decision</h2><p>Offline manager assessment. Slack not connected. Answer in the manager task. No operational approval or agent activity is established here.</p>'
    if view["state"] == "unknown":
        return body + '<p>Unknown: decision input unavailable; open count unknown.</p></section>'
    feed = view["feed"]
    esc = html.escape
    stale = view["state"] == "stale"
    body += f'<p>Source {esc(feed["feed_id"])} / revision {feed["revision"]}; assessed {esc(feed["assessed_at"])}. {"Stale: current decisions unknown; showing last-known records." if stale else "Fresh manager assessment."}</p>'
    opened = [d for d in feed["decisions"] if d["id"] in view["open"]]
    body += f'<p>{"Last-known open" if stale else "Open decisions"}: {len(opened)}.'
    if opened:
        raised = min(d["revisions"][0]["raised_at"] for d in opened)
        age = int((stamp(now) - stamp(raised)).total_seconds() // 60)
        body += f' <span id="oldest-open-decision-age" data-raised-at="{esc(raised)}">Oldest raised {age} minutes ago.</span>'
    elif not stale:
        body += ' No open decisions found in this fresh assessment.'
    body += '</p>'
    unavailable = {}

    def link(label, path, available):
        if available:
            href = document_url(path)
        else:
            key = hashlib.sha256((label + str(path)).encode()).hexdigest()[:20]
            href = '#decision-context-' + key
            unavailable[key] = label
        return f'<a href="{href}">{esc(label)}</a>'

    def content(record, summary_only=False):
        refs = {}
        for ref in record["sprints"]:
            sid, path = ref["sprint_id"], ref["path"]
            available = path in documents and (path == HISTORY if ref["historical"] else any(s["sprint_id"] == sid and s["path"] == path for s in sprints))
            refs[sid] = link(sid + (' (historical identity)' if ref["historical"] else ''), path, available)

        def linked(value):
            value = value if value is not None else 'Unknown: not assessed'
            return ''.join(refs.get(part) or (link(part, None, False) if SPRINT.fullmatch(part) else esc(part))
                           for part in re.split(r'(PF-\d{2}-S\d{2})', value))

        if summary_only:
            return linked(record["summary"])
        old = stale or (stamp(now) - stamp(record["updated_at"])).total_seconds() > FRESH_SECONDS
        result = f'<p>Revision {record["revision"]}: {esc(record["status"])}; raised {esc(record["raised_at"])}; context updated {esc(record["updated_at"])}. {"Stale context: stopped/continuing work and other details are last-known." if old else "Context within freshness window."}</p>'
        result += '<p>Sprints: ' + ', '.join(refs.values()) + '</p>'
        for label, field in (("Initiative", "initiative"), ("Summary", "summary"), ("Owner", "owner"), ("Background", "background"), ("Impact / tradeoffs", "impact"), ("Stopped work", "stopped"), ("Work that can continue", "continuing"), ("Recommendation (not an answer)", "recommendation")):
            result += f'<p><strong>{label}</strong> {linked(record[field])}</p>'
        result += '<p><strong>Options</strong></p><ul>' + ''.join('<li>' + linked(option) + '</li>' for option in record["options"]) + '</ul>'
        result += '<p><strong>Question</strong> ' + linked(record["question"]) + '</p><p><strong>Evidence / context / test links</strong></p>'
        if not record["evidence"]:
            result += '<p>Unknown: no evidence supplied.</p>'
        for item in record["evidence"]:
            assessed = item["assessed_at"]
            freshness = 'Unknown assessment time' if assessed is None else ('Stale evidence' if (stamp(now) - stamp(assessed)).total_seconds() > FRESH_SECONDS else 'Evidence within freshness window')
            result += '<p>' + link(item["label"], item["path"], item["path"] in documents) + f' — {freshness}; assessed {esc(assessed or "unknown")}</p>'
        if record["status"] == "acknowledged":
            result += '<p>Acknowledged; unresolved. Acknowledgement is not approval.</p>'
        if record["resolution"]:
            answer = record["resolution"]
            result += f'<p>Recorded {esc(record["status"])} against question revision {answer["answered_revision"]}; {esc(answer["recorded_at"])}.</p>'
            for field in ("actor", "answer", "scope"):
                result += f'<p><strong>{field.title()}</strong> {linked(answer[field])}</p>'
        return result

    for title, records in (("Open questions", opened), ("Decision history", [d for d in feed["decisions"] if d["id"] not in view["open"]])):
        body += f'<div><h3>{title}</h3>'
        for decision in records:
            record = decision["revisions"][-1]
            anchor = 'decision-' + decision["id"]
            body += f'<details id="{anchor}"><summary>{content(record, True)} — {esc(record["owner"] or "Unknown owner")} — {esc(record["status"])}</summary><a href="#{anchor}">Permanent decision link</a>'
            body += content(record)
            for earlier in decision["revisions"][:-1]:
                body += f'<details><summary>Retained revision {earlier["revision"]} (historical)</summary>' + content(earlier) + '</details>'
            body += '</details>'
        body += '</div>'
    for key, label in unavailable.items():
        body += f'<div id="decision-context-{key}" tabindex="-1"><h3>{esc(label)}: unavailable context</h3><p>No exact approved published context is available; no substitute selected.</p><a href="#decisions">Back to decisions</a></div>'
    return body + '</section>'
