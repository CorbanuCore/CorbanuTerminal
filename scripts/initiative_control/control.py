#!/usr/bin/env python3
"""Private, deterministic delivery dashboard. No agent dispatch or approval authority."""

import argparse
import contextlib
import datetime as dt
import fcntl
import hashlib
import html
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import tempfile
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import unquote, urlsplit
import uuid

from markdown_it import MarkdownIt
from activity import latest_reports, presentation
from attention import notices
from facilities import facilities
import decision_feed

HERE = Path(__file__).resolve().parent
MAX_FILE = 1024 * 1024
STALE_SECONDS = 45 * 60
RUN_STATUSES = {"working", "blocked", "awaiting_review", "finished", "failed", "cancelled"}
# Recovery delivery control and main provider persistence share this raw ID.
# Hold ambiguous reports in place; never join them to main's provider plan.
LEGACY_DELIVERY_SPRINT = "PF-76-S01"
SECRET = re.compile(r"(?i)(?:bearer\s+\S+|(?:password|api[_ -]?key|access[_ -]?token|refresh[_ -]?token|secret)\s*[:=]\s*\S+|-----BEGIN .*PRIVATE KEY-----|\b(?:sk-|ghp_|github_pat_)[A-Za-z0-9_-]{12,})")


def now():
    return dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")


def timestamp(value):
    parsed = dt.datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        raise ValueError("timestamps must include timezone")
    return parsed


def safe_text(value, limit=2000):
    if not isinstance(value, str) or len(value) > limit or SECRET.search(value):
        raise ValueError("invalid or potentially sensitive publication text")
    if any(ord(c) < 32 and c not in "\n\t" for c in value):
        raise ValueError("control characters are not publishable")
    return value


def read_file(path, root):
    path, root = Path(path), Path(root).resolve()
    resolved = path.resolve(strict=True)
    if not resolved.is_relative_to(root) or path.is_symlink() or not resolved.is_file():
        raise ValueError("source is not a regular file within the allowed root")
    if resolved.stat().st_size > MAX_FILE:
        raise ValueError("source exceeds publication size limit")
    return resolved.read_text(encoding="utf-8")


def read_json(path, root):
    return json.loads(read_file(path, root))


def atomic_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    fd, name = tempfile.mkstemp(prefix=".pending-", dir=path.parent)
    try:
        with os.fdopen(fd, "w") as stream:
            json.dump(value, stream, indent=2, sort_keys=True)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(name, path)
    finally:
        if os.path.exists(name):
            os.unlink(name)


@contextlib.contextmanager
def locked(path):
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    with path.open("a") as stream:
        fcntl.flock(stream, fcntl.LOCK_EX)
        yield


def load_checker(repo, name):
    spec = importlib.util.spec_from_file_location(name, repo / "docs" / name / "check.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def route(path):
    return "doc-" + hashlib.sha256(path.encode()).hexdigest()[:20] + ".html"


def reference_documents(config):
    """Explicit manager-selected Markdown only; never follow document links."""
    paths = config.get("reference_documents", [])
    if not isinstance(paths, list) or len(paths) > 100:
        raise ValueError("invalid reference document inventory")
    for value in paths:
        if not isinstance(value, str) or not re.fullmatch(r"(?:qa|docs/research)/[A-Za-z0-9_/-]+\.md", value):
            raise ValueError("reference document must be an explicit research/qa Markdown path")
        if PurePosixPath(value).as_posix() != value:
            raise ValueError("reference document path must be canonical")
    return sorted(set(paths))


def collect_references(repo, config, hashes):
    result = {}
    for relative in reference_documents(config):
        if relative not in hashes:
            raise ValueError("uncollected reference document; resync required")
        result[relative] = safe_text(read_file(repo / relative, repo), MAX_FILE)
    return result


def checked_run(value):
    required = {"run_id", "sprint_id", "machine", "role", "agent", "session_id", "status", "summary", "updated_at", "commit", "branch", "worktree"}
    if not isinstance(value, dict) or set(value) != required:
        raise ValueError("run report must have exactly the documented fields")
    for field, text in value.items():
        safe_text(text, 2000 if field == "summary" else 300)
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_-]{0,79}", value["run_id"]):
        raise ValueError("invalid run identifier")
    if not re.fullmatch(r"PF-\d{2}-S\d{2}", value["sprint_id"]):
        raise ValueError("invalid sprint identifier")
    if value["status"] not in RUN_STATUSES:
        raise ValueError("invalid run status")
    if not re.fullmatch(r"[a-f0-9]{40}", value["commit"]):
        raise ValueError("run requires exact source commit")
    if timestamp(value["updated_at"]) > timestamp(now()) + dt.timedelta(minutes=5):
        raise ValueError("run timestamp is in the future")
    return value


def report(state, value):
    """One immutable file per report; identical retries have the same event id."""
    checked_run(value)
    digest = hashlib.sha256(json.dumps(value, sort_keys=True).encode()).hexdigest()
    with locked(state / ".report.lock"):
        destination = state / "events" / (digest + ".json")
        if not destination.exists():
            atomic_json(destination, value)
    return digest


def safe_markdown(text, path, documents):
    markdown = MarkdownIt("commonmark", {"html": False, "linkify": False}).enable("table")
    tokens = markdown.parse(text)
    for token in tokens:
        for child in token.children or []:
            if child.type == "image":
                child.type, child.tag = "text", ""
                child.content = "[image omitted]"
            if child.type == "link_open":
                original = child.attrGet("href") or ""
                parsed = urlsplit(original)
                if parsed.scheme == "https" and not parsed.username and not parsed.password and not parsed.query and not SECRET.search(original):
                    child.attrSet("rel", "noreferrer noopener")
                    continue
                destination = os.path.normpath(str(PurePosixPath(path).parent / unquote(parsed.path)))
                if not parsed.scheme and not parsed.netloc and destination in documents:
                    child.attrSet("href", route(destination))
                elif original.startswith("#"):
                    child.attrSet("href", "#")
                else:
                    child.attrSet("href", "#unpublished")
                    child.attrSet("title", "Not in this private publication; consult the source record")
    return markdown.renderer.render(tokens, markdown.options, {})


def e(value):
    return html.escape(str(value), quote=True)


def link(path, title):
    return f'<a href="{route(path)}">{e(title)}</a>'


def badge(status):
    return f'<span class="badge {e(status)}">{e(status.replace("_", " "))}</span>'


def activity_presentation(reports):
    label, tone, seen = presentation(reports)
    attrs = f' data-activity-seen="{e(seen)}" data-activity-label="{e(label)}"' if seen else ""
    if seen and (timestamp(now()) - timestamp(seen)).total_seconds() > STALE_SECONDS:
        label, tone = "Stale report — last: " + label, "stale"
    return e(label), tone, attrs


def sprint_status(sprint, data):
    """Status belongs to the manager's sprint; worker claims never override it."""
    status, sid = sprint["status"], sprint["sprint_id"]
    if status not in {"blocked", "in_progress"}:
        return badge(status), ""
    target = "status-" + sid
    reports = latest_reports(r for r in data["runs"] if r["sprint_id"] == sid)
    if status == "blocked":
        reasons = [t["reason"] for t in data["config"].get("human_tests", [])
                   if t["sprint_id"] == sid and t.get("status") == "blocked"]
        reasons += [r["summary"] for r in reports if r["status"] == "blocked"]
        text = ''.join(f'<li>{e(reason)}</li>' for reason in dict.fromkeys(reasons))
        details = ('<ul>' + text + '</ul>') if text else '<p>No condensed blocker recorded. Manager must supply the reason; consult the sprint record below.</p>'
        label = f'<a class="badge blocked" href="#{e(target)}" aria-label="{e(sid)} blocked: view reasons">blocked</a>'
        title = "Block reasons"
    else:
        latest = reports[0] if reports else None
        summary = ' '.join(latest["summary"].split()) if latest else "No progress report connected; implementation status is manager-recorded, not a live worker signal."
        if len(reports) != len({r["run_id"] for r in reports}):
            summary = "Conflicting same-time reports; no reliable latest note can be selected. See run details below."
        short = summary if len(summary) <= 260 else summary[:257].rstrip() + "…"
        observed = ("Last report " + latest["updated_at"]) if latest else "No report timestamp"
        stale = latest and (timestamp(now()) - timestamp(latest["updated_at"])).total_seconds() > STALE_SECONDS
        note = ("STALE · " if stale else "") + short + " · " + observed
        text, tone, attrs = activity_presentation(reports)
        label = f'<span class="status-hint"><a class="badge {tone}" href="#{e(target)}" aria-describedby="hint-{e(sid)}"{attrs}>{text}</a><span class="status-tooltip" role="tooltip" id="hint-{e(sid)}">{e(note)}</span></span>'
        details = f'<p>{e(summary)}</p><small>{e(observed)}{" · STALE report" if stale else ""}</small>'
        if len(reports) > 1:
            details += '<ul>' + ''.join(f'<li>{e(r["run_id"])} · {e(presentation([r])[0])} · {e(r["updated_at"])}<p>{e(r["summary"])}</p></li>' for r in reports) + '</ul>'
        title = "Latest progress"
    label = '<span class="activity-heading">Current activity</span>' + label + f'<small class="lifecycle">Sprint lifecycle: {e(status.replace("_", " "))}</small>'
    details += '<p>' + link(sprint["path"], sid + " — full sprint and remaining gates") + '</p>'
    return label, f'<article class="test status-detail" id="{e(target)}" tabindex="-1"><h3>{e(sid)} · {title}</h3>{details}<a href="#initiatives">Back to workstreams</a></article>'


def page(title, body, collected, generation=""):
    return f'''<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="referrer" content="no-referrer"><title>{e(title)} · Corbanu Control</title>
<link rel="stylesheet" href="style.css"><link rel="stylesheet" href="facilities.css"><script src="status.js" defer></script><script src="facilities.js" defer></script></head>
<body data-collected="{e(collected)}" data-generation="{e(generation)}"><a class="skip" href="#main">Skip to content</a>
<header><a class="brand" href="index.html">CORBANU <span>CONTROL</span></a>
<nav aria-label="Sections"><a href="index.html#initiatives">Workstreams</a><a href="facilities.html">Facilities</a><a href="index.html#human">Human tests</a><a href="index.html#runs">Runs & machines</a><a href="index.html#tasknode">Task Node</a></nav></header>
<div class="freshness" id="freshness" role="status">Source collected {e(collected)} · checking publisher health</div>
<main id="main">{body}</main><footer>Private operations view · worker reports are claims, not acceptance · no dispatch or approval controls</footer></body></html>'''


def collect(repo, state):
    plans_check, sprints_check = load_checker(repo, "plans"), load_checker(repo, "sprints")
    plans = plans_check.check_plan_root(repo / "docs/plans")
    sprints = sprints_check.check_sprints(repo / "docs/sprints", repo)
    config = read_json(state / "control.json", state)
    source = read_json(state / "source.json", state)
    decision_snapshot = decision_feed.read_snapshot(repo, source, now())
    timestamp(source["collected_at"])
    if not re.fullmatch(r"[a-f0-9]{40}", source["commit"]):
        raise ValueError("invalid source commit")
    # Revalidate at read time as well as upload time: a partial activation must
    # never lend a new timestamp/commit to a different on-disk document tree.
    hashes = source["files"]
    if not isinstance(hashes, dict) or not hashes or len(hashes) > 2000:
        raise ValueError("invalid source file manifest")
    if hashlib.sha256(json.dumps(hashes, sort_keys=True).encode()).hexdigest() != source["tree_digest"]:
        raise ValueError("invalid source tree digest")
    for path, expected in hashes.items():
        if Path(path).is_absolute() or ".." in Path(path).parts:
            raise ValueError("invalid source manifest path")
        if hashlib.sha256(read_file(repo / path, repo).encode()).hexdigest() != expected:
            raise ValueError("source differs from collected file manifest; resync required")
    # These are explicit publication roots, never a recursive scan of logs/homes.
    documents = {}
    for directory in (repo / "docs/plans", repo / "docs/sprints"):
        for path in sorted(directory.rglob("*.md")):
            relative = path.relative_to(repo).as_posix()
            if relative not in hashes:
                raise ValueError("uncollected document; resync required")
            text = read_file(path, repo)
            text = SECRET.sub("[credential-like text withheld]", text)
            safe_text(text, MAX_FILE)
            documents[relative] = re.sub(r"\A---\n.*?\n---\n", "", text, count=1, flags=re.S)
    for test in config.get("human_tests", []):
        path = repo / test["path"]
        if not str(test["path"]).startswith("qa/") or ".." in Path(test["path"]).parts:
            raise ValueError("human plan must be an explicit qa path")
        documents[test["path"]] = safe_text(read_file(path, repo), MAX_FILE)
    documents.update(collect_references(repo, config, hashes))
    if len(documents) > 1500 or sum(len(v) for v in documents.values()) > 25 * MAX_FILE:
        raise ValueError("publication exceeds document budget")
    events, problems = [], []
    paths = sorted((state / "events").glob("*.json"))
    if len(paths) > 10000:
        raise ValueError("event retention requires manager maintenance")
    known = {s["sprint_id"] for s in sprints["sprints"]}
    for path in paths:
        try:
            value = checked_run(read_json(path, state))
            if value["sprint_id"] == LEGACY_DELIVERY_SPRINT:
                problems.append("PF-76-S01 history held for source reconciliation; not assigned to a main initiative.")
                continue
            if value["sprint_id"] not in known:
                raise ValueError("unknown sprint")
            events.append(value)
        except (ValueError, KeyError, OSError, TypeError):
            problems.append("A worker report was rejected; manager inspection required.")
    events.sort(key=lambda r: (timestamp(r["updated_at"]), r["run_id"]))
    latest = {r["run_id"]: r for r in events}
    writeback = None
    status_path = state / "writeback-status.json"
    if status_path.exists():
        try:
            writeback = read_json(status_path, state)
            timestamp(writeback["checked_at"])
            safe_text(json.dumps(writeback))
        except (OSError, ValueError, KeyError, TypeError):
            writeback = None
            problems.append("Writeback status is unreadable; manager inspection required.")
    return dict(plans=plans, sprints=sprints, config=config, source=source, writeback=writeback,
                decision_snapshot=decision_snapshot,
                documents=documents, events=events[-100:], runs=list(latest.values()),
                display_runs=latest_reports(events),
                problems=problems + plans["errors"] + sprints["errors"])


def overview(data):
    # Conflicting observations stay visible without changing writeback selection.
    data = {**data, "runs": data.get("display_runs", data["runs"])}
    plans, sprints, config, source = data["plans"], data["sprints"], data["config"], data["source"]
    active = sorted([p for p in plans["plans"] if p["status"] == "active"], key=lambda p: (p["priority"] or "P9", p["title"] or ""))
    reserved = [s for s in sprints["sprints"] if s["status"] in {"in_progress", "blocked"}]
    body = '<section class="heading"><div><p class="eyebrow">DELIVERY / OPERATIONS</p><h1>Initiative map</h1></div>'
    body += f'<div class="counters"><strong>{len(active)} / {plans["active_limit"]}<small>active initiatives</small></strong><strong>{len(reserved)} / 3<small>reserved sprints</small></strong><strong>1<small>sprint per initiative</small></strong></div></section>'
    body += f'<aside class="notice"><strong>Source boundary</strong> {e(source["label"])} · {e(source["branch"])} · {e(source["commit"][:12])}. {e(source.get("note", ""))}<small>Checkout: {e(source.get("checkout", "not recorded"))} · content {e(source.get("tree_digest", "unknown")[:12])}</small><a href="manifest.json">Exact publication manifest</a></aside>'
    body += decision_feed.render(data.get("decision_snapshot", {"feed": None}), now(), sprints["sprints"], data.get("documents", {}))
    body += notices(data)
    body += '<p class="muted">Current activity describes worker reports. Sprint lifecycle tracks overall completion; an open sprint does not mean an agent is running.</p>'
    body += '<section id="initiatives" aria-label="Active workstreams" class="lanes">'
    status_details = []
    for plan in active:
        path = "docs/plans/" + plan["path"]
        children = sorted([s for s in sprints["sprints"] if s["plan_file"] == path], key=lambda s: int(s["execution_order"] or 0))
        current = [s for s in children if s["lifecycle"] == "current"]
        completed = sum(s["status"] == "completed" for s in children)
        working = [s for s in current if s["status"] in {"in_progress", "blocked"}]
        body += f'<article class="lane"><div class="lane-top">{badge(plan["priority"] or "unknown")} {badge("initiative_active")}</div><h2>{link(path, plan["title"])}</h2><p class="muted">{e(plan["owner"])}</p>'
        body += f'<p>{completed} / {len(children)} sprints archived complete</p><progress value="{completed}" max="{max(1,len(children))}" aria-label="Archived sprint completion"></progress>'
        body += '<ol class="sprint-map">'
        ordered = working + [s for s in current if s not in working]
        for sprint in ordered[:4]:
            label, detail = sprint_status(sprint, data)
            if detail:
                status_details.append(detail)
            body += f'<li>{label} <b>{link(sprint["path"], sprint["sprint_id"])}</b><div>{e(sprint["title"])}</div>'
            deps = sprint.get("depends_on", "none")
            if deps != "none":
                body += f'<small>Requires {e(deps)}</small>'
            body += '</li>'
        if not current:
            body += '<li>Implementation sprints archived. Release / human qualification remains separate.</li>'
        body += '</ol>'
        run_ids = {s["sprint_id"] for s in children}
        workers = [r for r in data["runs"] if r["sprint_id"] in run_ids]
        body += '<div class="lane-foot">' + (', '.join(e(r["machine"] + ' · ' + r["role"]) for r in workers) or 'No live machine report connected') + '</div>'
        body += f'<details><summary>All {len(children)} sprints & dependencies</summary><ul>'
        for sprint in children:
            body += f'<li>{link(sprint["path"], sprint["sprint_id"])} · {e(sprint["status"])} · {e(sprint["title"])}<small>← {e(sprint["depends_on"])}</small></li>'
        body += '</ul></details></article>'
    body += '</section><section id="status-details"><h2>Block reasons & latest progress</h2><div class="test-grid">' + ''.join(status_details) + '</div></section>'
    body += '<section id="human"><div class="section-title"><h2>Human test queue</h2><span>Review budget: ~1 hour/day</span></div><div class="test-grid">'
    for index, test in enumerate(config.get("human_tests", [])):
        status = {"blocked": "blocked", "preparing": "manager preparation"}.get(test.get("status"), "pending human acceptance")
        target = f"human-reason-{index}"
        label = f'<a class="badge blocked" href="#{target}">blocked</a>' if status == "blocked" else badge(status)
        body += f'<article class="test"><p class="eyebrow">{e(test["sprint_id"])}</p><h3>{link(test["path"],test["title"])}</h3>{label}<p id="{target}">{e(test["reason"])}</p></article>'
    if not config.get("human_tests"):
        body += '<p>No human test plans mapped. This is missing coverage, not a pass.</p>'
    body += '</div></section><section id="runs"><h2>Runs & machines</h2><p class="muted">Explicit, redacted worker reports. “Finished” does not complete a sprint.</p><div class="table-wrap"><table><thead><tr><th>Sprint / run</th><th>Machine / role</th><th>Agent / session</th><th>Reported state</th><th>Last report</th></tr></thead><tbody>'
    for run in data["runs"]:
        text, tone, attrs = activity_presentation([run])
        conflict = '<small>Conflicting same-time reports — manager check</small>' if sum(r["run_id"] == run["run_id"] for r in data["runs"]) > 1 else ""
        body += f'<tr><td>{e(run["sprint_id"])}<small>{e(run["run_id"])}</small></td><td>{e(run["machine"])}<small>{e(run["role"])}</small></td><td>{e(run["agent"])}<small>{e(run["session_id"])}</small></td><td><span class="badge {tone}"{attrs}>{text}</span>{conflict}</td><td><time data-seen="{e(run["updated_at"])}">{e(run["updated_at"])}</time></td></tr>'
    body += '</tbody></table></div><details><summary>Progress log · last 100 reports</summary><ol class="log">'
    for run in reversed(data["events"]):
        body += f'<li><time>{e(run["updated_at"])}</time><b>{e(run["sprint_id"])} / {e(run["run_id"])}</b><p>{e(run["summary"])}</p><small>{e(run["branch"])} @ {e(run["commit"])} · {e(run["worktree"])}</small></li>'
    body += '</ol></details></section><section id="tasknode"><h2>Task Node writeback</h2>'
    configured = config.get("tasknode", {})
    body += f'<aside class="notice"><strong>Live delivery {"enabled" if configured.get("enabled") is True else "disabled"}</strong><p>{e(configured.get("detail", "No verified task mapping or delegated authorization. No live writes."))}</p>'
    writeback = data.get("writeback")
    if writeback:
        age = (timestamp(now()) - timestamp(writeback["checked_at"])).total_seconds()
        body += f'<p>{badge("stale" if age > STALE_SECONDS else "observed")} Last delivery check: <time data-seen="{e(writeback["checked_at"])}">{e(writeback["checked_at"])}</time> · enrollment {"verified" if writeback.get("enrollment_verified") else "not verified"}</p>'
        counts = ', '.join(f'{e(k)} {e(v)}' for k, v in sorted(writeback.get("outbox", {}).items())) or 'empty'
        body += '<p>Outbox: ' + counts
        body += f' · delivered this check {e(writeback.get("delivered_this_run", 0))} · unmapped runs {e(writeback.get("unmapped_runs", 0))} · rejected runs {e(writeback.get("rejected_runs", 0))}</p>'
        if writeback.get("error"):
            body += f'<p class="notice danger">{e(writeback["error"])}</p>'
    else:
        body += '<p>Delivery state unknown: no successful delivery check recorded.</p>'
    body += '<a href="https://tasknode.postfiat.org" rel="noreferrer">Open Post Fiat Task Node</a></aside></section>'
    body += '<section><h2>Feature-delivery gates</h2><p>Merge ≠ enable ≠ release. Use existing default-OFF feature flags; test OFF, ON, disable-after-use and restart recovery before promotion.</p>'
    body += link("docs/plans/feature-delivery.md", "Flag contract and merge / enable checklist") + '</section>'
    proposed = [p for p in plans["plans"] if p["status"] == "draft"]
    body += f'<section><details><summary>Proposed backlog · {len(proposed)} initiatives · not activated</summary><ul>'
    for plan in proposed:
        body += f'<li>{link("docs/plans/"+plan["path"],plan["title"] or plan["path"])}</li>'
    return body + '</ul></details></section>'


def publish(repo, state, output):
    output.mkdir(parents=True, exist_ok=True, mode=0o700)
    with locked(output / ".publish.lock"):
        generation = None
        try:
            data = collect(repo, state)
            failure = state / "sync-failure.json"
            if failure.exists() and timestamp(read_json(failure, state)["attempted_at"]) >= timestamp(data["source"]["collected_at"]):
                raise ValueError("source synchronization failed; require a newer successful collection")
            releases = output / "releases"
            releases.mkdir(exist_ok=True, mode=0o700)
            generation = Path(tempfile.mkdtemp(prefix="build-", dir=releases))
            for name in ("style.css", "facilities.css", "status.js", "facilities.js"):
                shutil.copyfile(HERE / name, generation / name)
            for path, text in data["documents"].items():
                body = '<p><a href="index.html">← Initiative map</a></p><article class="document">' + safe_markdown(text, path, data["documents"]) + '</article><p id="unpublished" class="muted">Unpublished source links require the repository. They are not copied automatically.</p>'
                (generation / route(path)).write_text(page(path, body, data["source"]["collected_at"], generation.name), encoding="utf-8")
            (generation / "index.html").write_text(page("Initiative map", overview(data), data["source"]["collected_at"], generation.name), encoding="utf-8")
            (generation / "facilities.html").write_text(page("Facilities", facilities(), data["source"]["collected_at"], generation.name), encoding="utf-8")
            feed_health = decision_feed.health(data.get("decision_snapshot", {"feed": None, "status": "unrecorded"}), data["source"], now())
            atomic_json(generation / "manifest.json", {"published_at": now(), "source": data["source"], "decision_feed": feed_health, "documents": sorted(data["documents"]), "run_count": len(data["runs"])})
            pending = output / (".current-" + uuid.uuid4().hex)
            pending.symlink_to(generation.relative_to(output))
            os.replace(pending, output / "current")
            atomic_json(output / "health.json", {"ok": True, "generation": generation.name, "published_at": now(), "collected_at": data["source"]["collected_at"], "decision_feed": feed_health, "warning_count": len(data["problems"])})
            # Only this publisher's generated directories; keep three complete generations.
            candidates = [generation] + sorted((p for p in releases.iterdir()
                                               if p != generation and re.fullmatch(r"build-[a-z0-9_]{8}", p.name)
                                               and p.is_dir() and not p.is_symlink()),
                                              key=lambda p: p.stat().st_mtime, reverse=True)
            for old in candidates[3:]:
                if old.resolve() != (output / "current").resolve() and ((old / "manifest.json").is_file() or time.time() - old.stat().st_mtime > 86400):
                    shutil.rmtree(old)
            return data
        except Exception:
            if generation and generation.exists() and (output / "current").resolve() != generation.resolve():
                shutil.rmtree(generation)  # This attempt only; never remove the displayed snapshot.
            retained_feed = {"state": "unknown", "assessed_at": None}
            try:
                retained_feed = read_json(output / "current/manifest.json", output)["decision_feed"]
                if retained_feed["assessed_at"] and (timestamp(now()) - timestamp(retained_feed["assessed_at"])).total_seconds() > decision_feed.d.FRESH_SECONDS:
                    retained_feed["state"] = "stale"
            except (OSError, ValueError, KeyError, TypeError):
                pass
            atomic_json(output / "health.json", {"ok": False, "attempted_at": now(), "decision_feed": retained_feed, "error": "Publication failed; displaying the last successful snapshot. Inspect publisher service logs."})
            raise


def serve(output, port):
    class Handler(BaseHTTPRequestHandler):
        def do_HEAD(self):
            self.do_GET()  # Same validation and headers, without a response body.

        def do_GET(self):
            try:
                host = urlsplit("//" + self.headers.get("Host", "")).hostname
            except ValueError:
                host = None
            if host not in {"localhost", "127.0.0.1"}:
                self.send_error(403)
                return  # DNS-rebinding protection for a private loopback service.
            path = unquote(urlsplit(self.path).path).lstrip("/") or "index.html"
            if not re.fullmatch(r"(?:index\.html|facilities\.html|style\.css|facilities\.css|status\.js|facilities\.js|health\.json|manifest\.json|doc-[0-9a-f]{20}\.html)", path):
                self.send_error(404)
                return
            file = output / "health.json" if path == "health.json" else output / "current" / path
            try:
                payload = file.read_bytes()
            except OSError:
                self.send_error(503)
                return
            types = {".html": "text/html; charset=utf-8", ".css": "text/css", ".js": "text/javascript", ".json": "application/json"}
            self.send_response(200)
            self.send_header("Content-Type", types[file.suffix])
            self.send_header("Content-Length", str(len(payload)))
            self.send_header("Cache-Control", "no-store")
            self.send_header("X-Content-Type-Options", "nosniff")
            self.send_header("X-Corbanu-Control", "1")
            self.send_header("Referrer-Policy", "no-referrer")
            self.send_header("Content-Security-Policy", "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self' http://127.0.0.1:8770; img-src 'none'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'")
            self.end_headers()
            if self.command != "HEAD":
                self.wfile.write(payload)

        def log_message(self, *_):
            pass  # No URL/query/header/session logging.

    ThreadingHTTPServer(("127.0.0.1", port), Handler).serve_forever()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    build = commands.add_parser("publish")
    build.add_argument("--repo", type=Path, required=True)
    build.add_argument("--state", type=Path, required=True)
    build.add_argument("--output", type=Path, required=True)
    report_cmd = commands.add_parser("report")
    report_cmd.add_argument("--state", type=Path, required=True)
    report_cmd.add_argument("--file", type=Path, required=True)
    server = commands.add_parser("serve")
    server.add_argument("--output", type=Path, required=True)
    server.add_argument("--port", type=int, default=8768)
    slack_cmd = commands.add_parser("decision-slack", help="Explicit manager-only Slack operations; never started by publication")
    slack_cmd.add_argument("--publish-state", type=Path, help="Write a redacted local projection; only with project-status")
    slack_cmd.add_argument("args", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if args.command == "decision-slack":
        # Lazy registration keeps normal publish/serve independent of Slack setup.
        try:
            if args.publish_state is not None:
                projection = argparse.ArgumentParser(description="Local redacted Slack projection; no network")
                projection.add_argument("operation", choices=["project-status"])
                projection.add_argument("--store", type=Path, required=True)
                projection.add_argument("--live", action="store_true", help="Read existing private journals; does not connect")
                options = projection.parse_args(args.args)
                value = decision_feed.project_slack(args.publish_state, options.store, now(), options.live)
                print(json.dumps(value))
            else:
                import decision_manager
                decision_manager.main(args.args)
        except Exception:
            raise SystemExit("Slack operation held; inspect redacted status and retained evidence.") from None
    elif args.command == "publish":
        data = publish(args.repo.resolve(), args.state.resolve(), args.output.resolve())
        print(f"Published {len(data['documents'])} documents; {len(data['runs'])} reported runs; {len(data['problems'])} warnings")
    elif args.command == "report":
        print(report(args.state.resolve(), read_json(args.file, args.file.parent)))
    else:
        serve(args.output.resolve(), args.port)


if __name__ == "__main__":
    main()
