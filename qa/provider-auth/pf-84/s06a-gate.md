# PF-84-S06 slice A gate: AWS profile accounts for Amazon Bedrock

Sprint: [PF-84-S06](../../../docs/sprints/archive/unified-provider-auth/pf-84-s06-aws-and-chatgpt-accounts.md),
slice A (Travis's D4 option (a), 2026-10-11). Behind the default-off `named_accounts` flag.

## What changed

- **Gate lifted for Bedrock only.** `supports_named_accounts` now accepts the built-in Amazon Bedrock
  provider without an `auth.command`. OpenAI sign-in providers, Bedrock behind a command and other
  AWS-configured providers still fail closed (slice B lifts OpenAI).
- **A1 (storage).** An account holds one `aws_profile`: the profile name, validated
  (`[A-Za-z0-9][A-Za-z0-9._+@-]{0,63}`), not secret, shown in clear in `corbanu account list` and
  `/providers` (`Named account · AWS profile \`work\` · fp …`). Added by
  `corbanu account add amazon-bedrock <name> --kind aws-profile --value <profile>` or the `/providers`
  "Add another account" row (plain entry). `corbanu account rename` is new (any kind).
- **A2 (resolution).** A named Bedrock account signs requests with a profile-only provider
  (`ProfileFileCredentialsProvider`): never the managed Bedrock key, `AWS_BEARER_TOKEN_BEDROCK`,
  `AWS_PROFILE`, environment keys or instance credentials. A profile whose `credential_source` is
  `Environment`, `Ec2InstanceMetadata` or `EcsContainer` is refused. Region: `aws.region`, else the
  profile's; it is also given to role chains for STS. The profile name is re-read from the vault on
  every request; the loaded provider is kept per profile for the session.
- **A3 (refusal).** Credentials are resolved once before the first request. A missing profile, a
  profile without a region, a refused `credential_source` or failing credentials end the turn with the
  reason and recovery text; no request is sent and nothing falls back. A `credential_process` helper's
  stderr is cut from the message. An unconfigured account is refused at session start with the
  `--kind aws-profile --value <profile>` command. `corbanu account rename` notes when
  `[provider_accounts]` still selects the old name.
- SSO profiles resolve through the SDK (Travis: allowed; no SSO set-up UI).

## Tests (final head `bffe8d916c`, merged with `origin/main`)

Linux on glitch (`-j10`, the isolated `just test` runner, `CORBANU_TEST_NO_NATIVE_KEYRING=1`):

| Run | Result |
| --- | --- |
| `cargo clippy --tests -D warnings` on vault, aws-auth, login, model-provider, core, cli, tui | clean, plain and `developer-accounting` |
| `-p codex-vault -p codex-aws-auth -p codex-model-provider -p codex-login` | 377 passed |
| `-p codex-cli --test provider_accounts` | 3 passed |
| `-p codex-core --lib -E 'test(provider_accounts) \| test(account)'` | 115 passed; 120 with `developer-accounting` |
| `-p codex-tui --lib` (named accounts, provider manager, status) | 419 passed, plain and `developer-accounting` |

macOS (arm64): `cargo clippy --tests -D warnings` on the same crates clean; debug build used for the
video and the GLM run.

New tests: profile-name validation; profile-only loading with injected AWS files (`work` signs with its
own key id, never `default`; a missing profile errors; a profile without a region is refused; region
override; `credential_source` Environment/Ec2InstanceMetadata/EcsContainer refused; a role chain
without a region reports the region; helper stderr cut from messages);
Bedrock named account ignores the managed key and reports missing profiles with recovery text; the
session-start refusal names `--kind aws-profile`; core stamping (Bedrock accepted, command Bedrock and
custom AWS refused); CLI add/list/rename/remove with validation; `/providers` add method and row text.

## Functional check (tmux, disposable homes)

Script: [slice_a_check.sh](s06/scripts/slice_a_check.sh) with the mock Bedrock Mantle endpoint
[mock_bedrock.py](s06/scripts/mock_bedrock.py), Linux debug build of `bffe8d916c`. Fake `~/.aws` files with
fake key ids only; decoy `AWS_ACCESS_KEY_ID`, `AWS_PROFILE=default` and `AWS_BEARER_TOKEN_BEDROCK` set
for every run. Captures: [s06/slice-a/](s06/slice-a/).

- `exec --account work` was signed with `AKIDWORKPROFILE` (region us-west-2), `--account alt` with
  `AKIDOTHERPROFILE`; the default account still used the bearer token
  ([cli.txt](s06/slice-a/cli.txt), [mock-requests.jsonl](s06/slice-a/mock-requests.jsonl)).
- `--account gone` (profile `nosuchprofile`) failed with the reason and recovery text and sent no
  request; `--account nope` was refused at thread start; with the flag off `--account work` was refused.
- TUI: `/status` shows `· account work`; a turn was signed by `work`'s profile; `/providers` lists the
  Bedrock accounts with profile and fingerprint; "Add another account" took a name and a profile
  ([tui-5](s06/slice-a/tui-5-profile-entry.txt), [tui-6](s06/slice-a/tui-6-saved.txt)); after a
  restart with `[provider_accounts] amazon-bedrock = "prod"` the session ran on `prod`
  ([tui-7](s06/slice-a/tui-7-restart.txt)).
- Leak scan of captures, logs and rollouts for the fake secret keys and the bearer canary: 0 files.

## GLM run, video, review

- GLM 5.3 Flash coordinator (`zai`, real default key from the installed vault helper, never printed),
  macOS debug build: it ran a `corbanu exec` worker on `--account fake` (Bedrock, profile
  `nosuchprofile`); the worker was refused with the reason and recovery text, nothing was sent, and the
  coordinator replied `pong` ([glm-run.txt](s06/slice-a/glm/glm-run.txt),
  [script](s06/scripts/glm_run_a.sh)). Real-key scan of the output and the disposable home: 0 files.
- Video `pf84-aws-profile-account` ([mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s06-pf84-aws-profile-account-bffe8d916c0b-2026-10-11.mp4),
  [index](../../demos/index/PF-84-S06.md)): `/providers` adds Bedrock account `work` (profile
  `work`), *Use for this session*, the mock shows the request signed by `AKIDWORKPROFILE`, `/status`
  names `account work`.
- Independent review (Opus 5.5 High, read-only, installed `corbanu exec`): round 1 REQUEST CHANGES
  (`credential_source` could reach env/IMDS/ECS; role chains without region for STS; helper stderr in
  messages; CLI rename; tests reading real `~/.aws`; double resolution per request), round 2 REQUEST
  CHANGES (the stderr redaction hid the refusal reason), round 3 APPROVE WITH NITS; the three nits are
  fixed in `bffe8d916c`. Reviews: `.codex-work/workers-20261002/pf84-s06/review-a*.result.md` (outside
  the repo).

## Not verified

- **Live Bedrock: pending.** No test AWS account or profile is available (Travis, 2026-10-11). Needs an
  AWS account with Bedrock Mantle access in one region, a least-privilege profile and a spend cap.
- A real SSO profile (`aws sso login`), a real role chain and a `credential_process` helper were not
  exercised; only static-key profiles were (role-chain and refusal paths are unit-tested offline).

## Known limits

- Credentials are fetched for every signature, as on the default SDK path (no credentials cache); for
  SSO or role-chain profiles that is one SSO/STS call per request. Edits to `~/.aws` apply from the next
  session.
- aws-config's own tracing can log a `credential_process` helper's stderr into Corbanu's logs; this
  predates the PR and affects the default path too: [#548](https://github.com/CorbanuCore/CorbanuTerminal/issues/548).
- Throttle and usage rows are keyed as for other named accounts (by name until the fingerprint-keying
  follow-up lands).
