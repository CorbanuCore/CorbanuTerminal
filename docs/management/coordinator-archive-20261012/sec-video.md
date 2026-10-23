You are the demo-video tooling worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
first (decision 5 is why you exist).

Goal: any sprint worker can produce short videos that demonstrate each core feature end to end in the real TUI,
recorded from the existing tmux functional-test flow, with one command.

1. Find the existing tmux functional testing SOP and harness (search docs/, qa/, scripts/ and AGENTS.md for tmux
   functional/true-TUI procedures; there are completed sprints on this). Build on it; don't invent a parallel one.
2. Tooling: ffmpeg is installed. Install what you need via Homebrew (e.g. asciinema + agg, or vhs). Record the
   tmux pane, render MP4 (H.264, plays in QuickTime) and keep the .cast.
3. Each video: under 90 seconds; a title card with feature name, expected result, product commit sha and date;
   real keystrokes and real output (no mocked screens); idle gaps compressed; final frame shows the result.
   Secrets must never appear on screen: use disposable profiles and fake credentials only.
4. Storage: keep MP4s out of git. Commit the script, the SOP (qa/demos/README.md) and a small per-sprint index file;
   upload MP4s as assets on a GitHub prerelease named `demos` (create it) and link them from the PR body.
5. Prove it: record two demos on current main (e.g. `/permissions` switching to Full Access and back with the
   result shown, and the approval prompt for an outside-workspace write), using GLM 5.2 as the in-product model.
   Watch your own output: extract a few frames with ffmpeg and inspect them to confirm they are legible.
6. PR to main with the script, SOP and index; merge when checks pass. Report the commands, file paths and the two
   demo links.
