# Feature demo videos

Every sprint gate needs short videos that show each core feature working end to
end in the real TUI (security program decision 5). `scripts/demo_video.py` makes
one with a single command. It drives the candidate in tmux the same way as the
[tmux harness](../../docs/tmuxHarness.md) and the root
[interactive proof rules](../../AGENTS.md#interactive-product-proof): a private
`tmux -L` socket, real keys, text and Enter sent separately, and stable waits
on what is visible. asciinema records the attached pane, and the tool renders an
MP4 from the recording.

## Setup (once)

```sh
brew install tmux asciinema agg ffmpeg   # asciinema 3.x, agg 1.9+
gh auth status                           # needed for --publish
```

Build the candidate you are qualifying, for example
`cargo build -p codex-cli --bin corbanu` with your lane's `CARGO_TARGET_DIR`.

## Record one demo

Use a debug build and export `CORBANU_TEST_NO_NATIVE_KEYRING=1` first; the
script refuses to run without it. That keeps the candidate (and every
`corbanu` an agent command starts) off your real login keychain: the vault
key and login tokens stay in files inside the disposable profile. Release
builds ignore the variable. Only the `vault:` credential lookup runs without
it, because it reads your own vault.

```sh
export CORBANU_TEST_NO_NATIVE_KEYRING=1
python3 scripts/demo_video.py record qa/demos/specs/<demo>.toml \
  --bin "$CARGO_TARGET_DIR/debug/corbanu" --sprint PF-24-S02 --publish
```

This one command:

1. Creates a throwaway run directory under `qa/demos/out/<sprint>/` (ignored by
   git). It holds a disposable Corbanu home, a fresh git workspace, an
   `outside/` directory, a log directory and a fake `HOME`.
2. Starts the candidate in a private tmux server with `RUST_LOG=trace`,
   `log_dir` and `-C <workspace>`. Inherited variables that look like
   credentials are unset. Each spec credential is resolved inside the launcher
   as `VAR="$(corbanu vault auth-helper <label>)"`, so the value never appears
   in the script, argv, the screen or the chat.
3. Records the attached client with `asciinema rec --headless` and runs the
   spec steps. Afterwards it leaves through `/exit`, so logs are flushed before
   the scan.
4. Scans for secrets. The recording must contain no credential value and
   nothing shaped like a key; otherwise nothing is rendered or published.
   Credential values found in private logs or the disposable profile are
   overwritten in place and reported as a `PRODUCT FINDING`.
5. Edits the recording. It cuts at the last step, keeps `pause` holds, squeezes
   `compress` waits, caps other idle gaps at 1 s, and adds a title card. The
   card shows the feature, expected result, product commit (`-dirty` if
   `codex-rs` has uncommitted changes), date and model.
6. Renders `<sprint>-<id>-<sha>-<date>.mp4` (H.264, yuv420p, faststart; plays in
   QuickTime) and keeps both the raw and the edited `.cast`. It fails if the
   video is longer than 90 s, and speeds it up uniformly if it runs past 88 s.
7. Extracts `frames/title.png`, `middle.png` and `final.png`. **Open them before
   you publish** and check that the text is legible and the final frame shows
   the result.
8. With `--publish`, uploads the MP4 and the cast to the
   [`demos` prerelease](https://github.com/CorbanuCore/CorbanuTerminal/releases/tag/demos)
   (creating it if needed) and adds a row to `qa/demos/index/<sprint>.md`.

Without `--publish`, review first and then run
`python3 scripts/demo_video.py publish <run-dir>`. Commit the spec and the
index row with the sprint. Link the MP4 URLs in the PR body. Never commit MP4,
GIF or cast files.

## Write a spec

Specs live in `qa/demos/specs/<id>.toml`; see the two examples there.

| Key | Meaning |
| --- | --- |
| `id`, `feature`, `expected` | File-name slug, title-card heading and the expected result in one or two sentences |
| `model`, `provider` | Passed as `-m` and `-c model_provider=`. Use `glm-5.2` / `zai` (policy for functional cases) |
| `[credentials]` | `ENV_VAR = "<vault label>"`, e.g. `ZAI_API_KEY = "provider/zai_api_key"`. Override with `--credential VAR=vault:LABEL` or `VAR=file:PATH` |
| `config` | Contents of the disposable `config.toml`. The workspace is trusted automatically |
| `[fixtures]` | Workspace files to create (`"path" = "content"`) |
| `args`, `cols`, `rows` | Extra CLI args and terminal size (default 120x36) |
| `[[steps]]` | Exactly one of `type`, `key`, `wait`, `pause` per step |

- `type = "text"`: sends real keys one character at a time, then waits until
  the text is visible. Enter is always its own `key` step.
- `key = "Enter"`: a tmux key name (`Down`, `Up`, `Escape`, `y`, `C-c`).
  `repeat = N` sends it N times.
- `wait = "regex"`: waits until two identical captures match (default
  `timeout = 30`). Add `compress = N` to shorten a slow wait, such as a model
  turn, to N seconds of video.
- `pause = seconds`: a hold kept at real speed so viewers can read. End every
  spec with a `wait` for the visible result and a `pause` of about 3 s.

Placeholders `{workspace}`, `{outside}`, `{home}`, `{logs}` and `{run}` are
expanded in `type`, `wait`, `args`, `config` and fixtures. Prefer relative paths
such as `../outside/x.txt`; they read better on screen. The run directory is
outside the workspace and outside the temporary directories, so writes there
need approval in the default mode.

## Rules for every video

- One core feature per video, under 90 seconds, real keys and real output only.
  Don't edit frames, reuse old recordings or use mocked screens. The title card
  is the only generated content.
- The final frame must show the result: a status line, a file's contents, or
  the denial.
- Use only disposable profiles. Never type, paste or display a real secret. For
  credential features, use synthetic canaries such as `fake-key-0001`, or a
  disposable vault entry.
- Record against the exact candidate commit you are qualifying, after
  `just fmt` and `just fix`.
- If a recording fails, read `failure-screen.txt` and `driver.log` in the run
  directory, fix the spec, and record again. Record model refusals verbatim, as
  the root policy requires.

## Files

| Path | Committed |
| --- | --- |
| `scripts/demo_video.py`, `scripts/test_demo_video.py` | yes |
| `qa/demos/specs/*.toml` | yes |
| `qa/demos/index/<sprint>.md` | yes, one small table per sprint |
| `qa/demos/out/**` (run directories, MP4, casts, frames, logs) | no |

Run the unit tests with `python3 -m unittest scripts.test_demo_video`
(Python 3.11+).
