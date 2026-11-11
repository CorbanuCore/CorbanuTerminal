## Windows sandbox

- **Non-admin sandbox: confirming a flagged folder or repository covers
  only what the prompt showed** (D486, #1877 item 5). If another command
  planted or changed the same `.codex`/`.agents` folder, `.git` or
  repository while the startup prompt was on screen, confirming used to
  clear that later change too. Now confirming it is refused with a note,
  it stays flagged, and Corbanu Terminal asks about it again.
