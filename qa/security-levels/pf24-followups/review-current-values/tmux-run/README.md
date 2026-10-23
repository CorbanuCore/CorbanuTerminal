# tmux run on GLM 5.2 (`-m glm-5.2 -c model_provider="zai"`)

- **Candidate:** steps A–D ran on `ceb068042b`; step A was repeated on the final `dff1032c51` (`A-configC-review-final.txt`), which adds the launch-settings note.
- **Profile:** disposable, with `security_levels = true` and the workspace trusted.
- **User config (Config C from the frozen code-blind design):**
  - `on-request` approvals;
  - workspace-write with an extra writable root (`<run>/outside`) and network on;
  - `web_search = "live"`;
  - an environment that passes everything through;
  - a custom role `yolo` whose config file sets `approval_policy = "never"`.
- **Earlier run:** a first run on `7a5e9d93df`, before the review fixes, is not kept. Its findings fed the review.

| Step | Capture | Result |
| --- | --- | --- |
| A | `A-configC-review.txt` | Every row shows the real value:<br>• writable roots `<run>/outside`, `/private/tmp` and `$TMPDIR`;<br>• "approved or allow-listed commands can run outside the sandbox";<br>• on-request;<br>• network on and live web search;<br>• secrets passed through;<br>• `yolo` named as changing child settings. |
| B | `B-cancelled.txt` | Esc shows "Cancelled. Nothing changed." No level file is written. |
| C | `C-network-off-review.txt`, `C-escalation-prompt-1.txt`, `C-network-off-escape.txt` | With `network_access = false`, the row reads "off inside the sandbox, on for commands that run outside it". GLM's `curl` failed in the sandbox (000), it asked to escalate, and once approved it got `200`. That matches the row. |
| D | `D-80x24-top.txt`, `D-80x24-scrolled-end.txt` | With Aggressive active and Permissive saved, the Aggressive review shows Aggressive's own values as "now". At 80x24 the footer says "↑/↓ scroll", and scrolling reaches the "Unchanged" note and the restart note. Esc leaves the saved level at Permissive. |
