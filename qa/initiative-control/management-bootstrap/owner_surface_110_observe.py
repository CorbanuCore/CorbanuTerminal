"""Disposable rendered-state capture; supporting evidence, not human acceptance."""
from owner_assurance_109_observe import observe


if __name__ == "__main__":
    for scenario in ("healthy", "held-waiting", "quarantined", "expired",
                     "supervisor-stale", "supervisor-unreadable", "saved-snapshot-aged"):
        observe(scenario)
