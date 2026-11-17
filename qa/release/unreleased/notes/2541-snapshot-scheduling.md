## Fixes

- Keep credential filtering for shell snapshots off async workers. A timed-out or cancelled scan refuses the snapshot, and ARM macOS/Linux use runtime-detected SHA acceleration while preserving matching and scan limits.
