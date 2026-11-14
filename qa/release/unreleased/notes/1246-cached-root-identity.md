## Security

Linux sandbox cleanup reads cached process-root identities without requesting a filesystem sync. It preserves successful-read ordering and post-read identity checks, and scans mount views without deduplication when an identity is unavailable.
