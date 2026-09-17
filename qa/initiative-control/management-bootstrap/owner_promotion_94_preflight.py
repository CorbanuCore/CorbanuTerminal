"""Read-only promotion gate. Evidence records are attestations, not authority grants."""
import argparse
import json
import os
from pathlib import Path
import plistlib
import sys
import time

import fable_launcher as f
import owner_daemon as owner
from coordinator import Rejected, TERMINAL, digest
from manager_cycle import ExistingCoordinator
from owner_tmux import freeze_worker_inputs, validate, worker_runtime


ITEMS = {
    1: "manager_bridge_authority",
    2: "fresh_allocations_and_decisions",
    3: "publication_pending",
    4: "isolated_real_worker_qualification",
    5: "quiescent_live_audit",
}


def require(value, reason):
    f.require(value, reason)


def reference(record, key):
    """Hash only explicitly supplied redacted evidence, never a profile."""
    ref = record.get(key, {})
    path = Path(ref.get("path", ""))
    require(path.is_absolute() and path.suffix == ".md", key + "_redacted_evidence_required")
    require(f.file_digest(owner.private_file(path)) == ref.get("sha256"),
            key + "_evidence_digest_mismatch")


def bridge(manifest):
    record = manifest.get("preparation", {})
    require(record.get("bridge") == "owner_tmux.freeze_worker_inputs",
            "manager_preparation_bridge_not_adopted")
    require(record.get("provider") and record.get("policy") == "--yolo",
            "explicit_provider_and_policy_authority_required")
    reference(record, "authority")
    reference(record, "adoption")


class ReadOnlyCoordinator(ExistingCoordinator):
    def snapshot(self):
        with self.connection(readonly=True) as db:
            db.execute("PRAGMA busy_timeout=0")
            row = db.execute("SELECT body FROM state WHERE id=1").fetchone()
            require(row is not None, "uninitialized_state")
            return json.loads(row[0])


def context(schedule):
    root = f.private_dir(schedule)
    receipt = owner.load(root / "installation.json")
    config_path = Path(receipt["pins"]["config"])
    config = owner.load(config_path)
    coordinator = ReadOnlyCoordinator(config["coordinator"])
    return root, receipt, config_path, config, coordinator


def allocations(manifest, ctx, selected):
    _, _, _, config, coordinator = ctx
    state = coordinator.snapshot()
    require(selected and len(set(selected)) == len(selected), "prepared_action_ids_required")
    require(state["manager"] is None, "manager_cycle_unsettled")
    require(state["enabled"], "dispatch_not_enabled")
    record = manifest.get("preparation", {})
    replacements = manifest.get("replacements", {})
    require(set(replacements) == set(selected), "replacement_evidence_for_each_action_required")
    for key in selected:
        action = state["actions"][key]
        require(action["status"] == "prepared" and action["kind"] in owner.WORKER_KINDS,
                "selected_action_not_prepared_worker")
        inputs = action["inputs"]
        bound = freeze_worker_inputs({k: v for k, v in inputs.items() if k != "worker"},
                                     provider=record["provider"], policy=record["policy"])
        require(bound == inputs, "action_not_bound_by_authorized_bridge")
        require(worker_runtime(inputs)["worktree"] in config["worktrees"], "worktree_not_configured")
        allocation = state["allocations"][inputs["allocation"]]
        require(action["allocation_digest"] == digest(allocation), "allocation_digest_mismatch")
        require(inputs == {"allocation": inputs["allocation"], **allocation["inputs"]},
                "allocation_inputs_mismatch")
        replacement = replacements[key]
        old = state["actions"][replacement["old_action"]]
        require(old["id"] != key and old["status"] == "cancelled" and old.get("owner_cancellation"),
                "old_prepared_action_not_replaced")
        require(old["inputs"]["allocation"] == inputs["allocation"]
                and old["allocation_digest"] != action["allocation_digest"]
                and old["manager_run"] != action["manager_run"],
                "fresh_allocation_and_decision_required")
        require(action.get("manager_run"), "accepted_manager_decision_required")
        require(all(a["status"] in TERMINAL | {"prepared"} for a in state["actions"].values()
                    if a["inputs"]["allocation"] == inputs["allocation"]),
                "allocation_reservation_unsettled")
        reference(replacement, "evidence")
        coordinator._executable(state, action)
        coordinator._resources_available(state, action)
    return state


def qualification(manifest, transport):
    record = manifest.get("qualification", {})
    require(record.get("binary_sha256") == transport["binary_sha256"]
            and record.get("package_digest") == owner.package_digest(),
            "qualification_candidate_mismatch")
    reference(record, "evidence")
    if record.get("mode") == "limited":
        require(record.get("accepted_by") == "Travis Good"
                and record.get("limitation") and record.get("allowed_scope")
                and record.get("remaining_proof"), "named_product_authority_limitation_required")
        reference(record, "acceptance")
        return "LIMITED: " + record["limitation"]
    require(record.get("mode") == "qualified", "real_worker_qualification_required")
    require(record.get("isolated_transport") is True and record.get("isolated_profile") is True
            and record.get("negative_access_probes") is True
            and record.get("mediated_inference") is True,
            "isolated_transport_profile_and_probe_evidence_required")
    require(all(type(record.get(k)) is int and record[k] >= 1
                for k in ("real_ack", "real_start", "real_return")),
            "real_ack_start_return_required")
    require(record.get("independent_reviewer"), "independent_evidence_review_required")
    reference(record, "review")
    return "qualified evidence recorded; authenticity remains the integrator's responsibility"


def technical(manifest, ctx, selected, transport, state):
    root, receipt, config_path, config, coordinator = ctx
    audit = manifest.get("audit", {})
    require(audit.get("dispatchers_quiescent") is True
            and audit.get("raw_key_delivery_stopped") is True
            and audit.get("publishers_quiescent") is True, "dispatchers_and_publishers_not_quiescent")
    require(type(audit.get("observed_at")) in (int, float)
            and 0 <= time.time() - audit["observed_at"] <= 300, "quiescence_attestation_stale")
    require(audit.get("coordinator_revision") == state["revision"], "audit_revision_stale")
    reference(audit, "evidence")
    require(receipt["phase"] == "installed", "installation_not_installed")
    tick = owner.load(root / "tick.json")
    require(tick["hold"] is None
            and not (tick["started_at"] is not None and tick["completed_at"] is None),
            "schedule_recovery_required")
    f.private_dir(root / "logs")
    for name in ("owner.stdout.log", "owner.stderr.log"):
        owner.private_file(root / "logs" / name)
    require("transport" not in config, "fixture_only_start_required")
    require(set(config) == {"coordinator", "worktrees", "package_digest", "manager_enabled"}
            and type(config["manager_enabled"]) is bool, "invalid_config")
    require(state.get("dispatch_control") is None, "initial_partition_required")
    pins = receipt["pins"]
    require(owner.schedule_pins(Path(pins["python"]), Path(pins["runtime"]), config_path,
                                pins["python_sha256"]) == pins, "installation_pin_drift")
    require(type(receipt["interval"]) is int and 1 <= receipt["interval"] <= 30
            and (receipt["interval"] == 30
                 or receipt["label"].startswith("com.corbanu.initiative-owner.test-")),
            "unreviewed_interval")
    domain = owner.installation_domain(receipt)
    sibling = ("user" if domain.startswith("gui/") else "gui/") + str(os.getuid())
    plist = owner.private_file(root / "owner.plist")
    require(f.file_digest(plist) == receipt["plist_sha256"], "plist_drift")
    for location in (domain, sibling):
        presence, detail = owner.service(receipt["label"], location)
        require((presence == "present" and f"path = {plist}\n" in detail)
                if location == domain else presence in {"absent", "domain_absent"},
                "service_ownership_or_domain_conflict")
    # Validate all copy targets and the install asset before removing the job.
    source, runtime = Path(owner.__file__).parent, Path(pins["runtime"])
    require(source.resolve() != runtime.resolve(), "candidate_must_not_be_installed_runtime")
    plistlib.loads((source / "com.corbanu.initiative-owner.plist.in").read_bytes())
    for path in source.glob("*.py"):
        if not path.name.startswith("test_"):
            require(path.is_file() and not path.is_symlink(), "candidate_source_unreadable")
            path.read_bytes()
            target = f.no_links(runtime / path.name)
            if target.exists():
                owner.private_file(target)
    f.no_links(runtime / "com.corbanu.initiative-owner.plist.in")
    require(os.access(runtime, os.W_OK) and os.access(root, os.W_OK)
            and os.access(config_path, os.W_OK), "cutover_paths_not_writable")
    for directory in {root, Path(config["coordinator"]), config_path.parent}:
        require(not any(p.name.endswith(".pending") for p in directory.iterdir()),
                "cutover_pending_write")
    before = owner.activation_status(config_path)
    require(before["complete"] and before["state"] == "armed"
            and before["scope"] == "fixture-only", "armed_fixture_status_required")
    require(not before["recovery_required"] and not before["unresolved_holds"],
            "activation_recovery_or_unresolved_hold")
    with owner.activation_store(config_path, readonly=True) as (_, db, meta):
        require(not db.execute("SELECT 1 FROM operations WHERE phase != 'applied'").fetchone(),
                "reconfigure_requires_settled_operations")
        require(meta["config_digest"] == digest(config)
                and meta["package_digest"] == config["package_digest"], "stored_pin_drift")
    require(not any(a.get("dispatch_owner") == "owner" and a["status"] not in
                    {"prepared", "accepted", "failed", "cancelled"} for a in state["actions"].values()),
            "reconfigure_requires_quiescent_owner")
    for path in config["worktrees"]:
        require(Path(path).is_absolute() and f.no_links(path).is_dir(), "missing_worktree")
    require(len(os.fsencode(Path(transport["runs_dir"]) / ("w-" + "0" * 8) / "s")) < 100,
            "tmux_socket_path_too_long")
    replacement = dict(config, transport=transport, package_digest=owner.package_digest())
    decision = dict(manifest["activation"], scope="tmux-workers",
                    generation=before["generation"] + 2,
                    config_digest=digest(replacement), package_digest=owner.package_digest())
    owner.validate_authority(decision, replacement)
    reference(manifest, "activation_authority")
    assignments = {
        key: {**{field: a.get(field) for field in ("claim", "allocation_digest", "status")},
              "from": coordinator.dispatch_owner(state, a), "to": "owner" if key in selected else "hand"}
        for key, a in state["actions"].items() if a["status"] not in TERMINAL
    }
    require(coordinator.snapshot() == state, "coordinator_changed_during_preflight")
    return dict(config=replacement, decision=decision, assignments=assignments,
                revision=state["revision"], generation=before["generation"])


def evaluate(manifest, schedule, transport_path, selected):
    results, ctx, state, plan = [], None, None, None
    for item in ITEMS:
        try:
            if item == 1:
                bridge(manifest)
            elif item == 2:
                ctx = context(schedule)
                state = allocations(manifest, ctx, selected)
            elif item == 3:
                ctx = ctx or context(schedule)
                owner.publication_preflight(ctx[1]["publish_state"])
            elif item == 4:
                transport = validate(owner.load(transport_path))
                detail = qualification(manifest, transport)
            else:
                require(state is not None, "item_2_required")
                transport = validate(owner.load(transport_path))
                plan = technical(manifest, ctx, selected, transport, state)
            results.append(dict(item=item, name=ITEMS[item], status="PASS",
                                detail=detail if item == 4 else "checked"))
        except Exception as exc:
            # Only fixed validation codes, never arbitrary input contents/credentials.
            reason = str(exc) if isinstance(exc, (f.LaunchError, Rejected)) else type(exc).__name__
            results.append(dict(item=item, name=ITEMS[item], status="UNMET", reason=reason))
    ok = all(row["status"] == "PASS" for row in results)
    return dict(ok=ok, results=results, plan=plan if ok else None,
                warning="Read-only advisory under recorded quiescence; not a reservation or authority grant.")


def from_environment(environ=os.environ):
    path = environ.get("OWNER94_PREFLIGHT")
    manifest = owner.load(Path(path)) if path else {}
    return evaluate(manifest, environ["OWNER80_SCHEDULE"], Path(environ["OWNER80_TRANSPORT"]),
                    environ.get("OWNER80_ACTIONS", "").split())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--schedule", type=Path, required=True)
    parser.add_argument("--transport", type=Path, required=True)
    parser.add_argument("--actions", nargs="+", required=True)
    args = parser.parse_args()
    try:
        result = evaluate(owner.load(args.manifest), args.schedule, args.transport, args.actions)
    except Exception as exc:
        result = dict(ok=False, refusal="preflight_inputs_unreadable", error=type(exc).__name__)
    print(json.dumps(result, sort_keys=True))
    return 0 if result["ok"] else 2


if __name__ == "__main__":
    sys.exit(main())
