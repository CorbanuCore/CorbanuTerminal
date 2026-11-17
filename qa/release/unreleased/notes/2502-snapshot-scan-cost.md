## Security

Large shell snapshots can now be checked against isolated credentials without exhausting the old scan budget. The matcher shares work for keys of the same length and reuses exact windows already found clean, while preserving copied-key detection and refusing when protection is unavailable. The bounded scan supports 3 MiB snapshots with eight ordinary credential lengths.
