"""Manager preparation callsite: bind worker runtime BEFORE allocation registration.

Every worker allocation the manager registers goes through
`owner_tmux.freeze_worker_inputs` with an explicitly supplied provider and
execution policy. Authority is never inferred from the model name, the active
profile or a previous allocation; the caller passes it and cites its record.
The frozen worker block becomes part of the allocation digest, so the next
accepted manager decision must copy these exact inputs.
"""

import copy

from coordinator import Rejected, require
from owner_tmux import freeze_worker_inputs

BRIDGE = "owner_tmux.freeze_worker_inputs"


def prepare_worker(coordinator, *, allocation_id, allocation, provider, policy, replace,
                   expected_revision, evidence):
    """Freeze, then register through the owner API. Returns the registered inputs."""
    require(isinstance(provider, str) and provider, "explicit provider authority required")
    require(policy == "--yolo", "explicit --yolo policy authority required")
    require(isinstance(evidence, dict) and isinstance(evidence.get("authority"), str)
            and evidence["authority"], "provider/policy authority reference required")
    require(isinstance(allocation, dict) and isinstance(allocation.get("inputs"), dict)
            and "allocation" not in allocation["inputs"], "allocation inputs required")
    try:
        frozen = freeze_worker_inputs(allocation["inputs"], provider=provider, policy=policy)
    except Exception as exc:  # LaunchError carries fixed, non-sensitive codes only.
        raise Rejected("worker runtime refused: " + str(exc)) from None
    record = copy.deepcopy(allocation)
    record["inputs"] = frozen
    coordinator.put_allocation(allocation_id, record, replace, expected_revision,
                               {**evidence, "preparation": {"bridge": BRIDGE, "provider": provider,
                                                            "policy": policy}})
    return {"allocation": allocation_id, **frozen}
