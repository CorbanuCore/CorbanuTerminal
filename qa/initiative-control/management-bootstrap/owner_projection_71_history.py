"""Read only retained publication/export artifacts; emit counts and hashes, not content."""
import hashlib
import json
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
publications = []
for manifest in sorted((root / "site/releases").glob("build-*/manifest.json")):
    value = json.loads(manifest.read_bytes())
    page = manifest.with_name("index.html").read_bytes()
    health = value.get("decision_feed", {})
    publications.append(dict(
        generation=manifest.parent.name, published_at=value.get("published_at"),
        tree_digest=value.get("source", {}).get("tree_digest"),
        feed_pin=value.get("source", {}).get("decision_feed", {}).get("digest"),
        feed_revision=health.get("revision"), input_status=health.get("input_status"),
        open_count=health.get("open_count"), slack=health.get("slack"),
        card_ids=sorted(set(re.findall(r'id="decision-([^"]+)"', page.decode()))),
        manifest_sha256=hashlib.sha256(manifest.read_bytes()).hexdigest(),
        page_sha256=hashlib.sha256(page).hexdigest()))
exports = []
paths = sorted(root.glob("export.*/source/decision-feed.json"))
paths += sorted(root.glob("incoming/upload-*/source/decision-feed.json"))
for path in paths:
    raw = path.read_bytes()
    value = json.loads(raw)
    feed = value.get("feed")
    slack = value.get("slack")
    manifest = path.parent.parent / "state/source.json"
    source = json.loads(manifest.read_bytes()) if manifest.exists() else {}
    records = {(item["id"], record["revision"]) for item in (feed or {}).get("decisions", [])
               for record in item["revisions"]}
    latest = {(item["id"], item["revisions"][-1]["revision"])
              for item in (feed or {}).get("decisions", [])}
    rows = {(row["id"], row["revision"]) for row in (slack or {}).get("decisions", [])}
    exports.append(dict(
        artifact=str(path.relative_to(root)), sha256=hashlib.sha256(raw).hexdigest(),
        tree_digest=source.get("tree_digest"), collected_at=source.get("collected_at"),
        input_status=value.get("status"), feed_revision=(feed or {}).get("revision"),
        decisions=len(latest), decision_ids=sorted(key[0] for key in latest),
        revisions=len(records), slack_status=value.get("slack_status"),
        projected_rows=len(rows),
        missing_latest=sorted(latest - rows) if slack else None,
        missing_revisions=sorted(records - rows) if slack else None,
        slack_bytes=len(json.dumps(slack, sort_keys=True, separators=(",", ":")).encode()) if slack else None))
for publication in publications:
    matches = [value for value in exports if value["sha256"] == publication["feed_pin"]
               and value["tree_digest"] == publication["tree_digest"]]
    publication["matching_exports"] = [value["artifact"] for value in matches]
    for value in matches:
        publication["missing_cards"] = sorted(
            set(value["decision_ids"]) - set(publication["card_ids"]))
print(json.dumps(dict(root=str(root), publications=publications, exports=exports), indent=2))
