# Execution scope and provenance

The designer's proposal was frozen before these runs. Every original case is
retained. An unresolved case means its complete expected journey is not proven;
it does not mean that every supporting assertion failed. No case is silently
excluded and no human checklist box is checked automatically.

## Newly executed checks

- Mac package actual-key TMUX: code host + shell + MCP round trips, three
  concurrent processes, same-home restart, Ctrl+C cancellation/recovery,
  inert permissions/security cancellation, narrow token guidance, Claude
  catalog after masked setup/restart and independent Anthropic API catalog.
- Mac existing profile: two bounded process launches, provider/model menus,
  consistent visible provider/model state and no password entry by the tester.
  Configuration equality is an assertion reported by the runner's successful
  exit, not a separately supplied before/after receipt. This uses the stable target
  from tmux, not native Applications-window automation or a model request.
- Both platforms: new F08/F09 input probes keep empty/whitespace entry in the
  form, preserve masking during cancellation and keep a canary out of chat
  scrollback. An arbitrary invalid key becomes “Enabled · configured”; no
  remote authentication validation is demonstrated. No inference is attempted
  by this input probe. Its endpoint is localhost port 1, not a live provider.
- Linux: existing Rust real-key provider management, convergence, Claude setup
  and reauthentication suite, all binary aliases pinned to the packaged CLI;
  then the same package-level host/shell/MCP smoke used on Mac.

The first input-probe attempts (`mac-inputs`, `linux/inputs`) stopped at the
disposable working-directory trust prompt. The corrected driver pre-trusts only
its own empty disposable test directory; new `*-ready` artifacts supersede those
fixture failures. No product source changed. Input-probe captures also include
the product's temporary-home PATH-alias warning: these captures cannot qualify
normal installed startup. Separate package-tool runs prove their own tool calls.

The Linux test log is an execution receipt, not independent raw proof of each
request or all original expectations. Success captures pin the package hash and
visible state. They do not expose live credentials. Package smoke captures are
PTY text, not pixel screenshots. Historical screenshot provenance stays separate.

## Deliberately unclaimed

No live provider accounts were added or replaced. No Keychain ACLs were changed,
no native security dialog was accepted, no paid-provider credential was copied
between billing routes, and no original user terminal window was automated.
Native deny/allow recovery and full account/browser login remain unverified.
The existing Escape-stream cancellation issue is not qualified by Ctrl+C.
Security/memory cases outside this provider packet remain governed by the wider
human guide. Neither a new release nor merge to main is authorized by this run.
