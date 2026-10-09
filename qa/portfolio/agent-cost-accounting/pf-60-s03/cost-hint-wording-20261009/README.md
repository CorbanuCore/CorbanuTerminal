# /cost date hint and plain billed-cost wording, 2026-10-09

Two presentation changes Travis asked for on 2026-10-09. They apply only to developer-accounting builds; default builds never open `/cost`.

| Change | Capture |
| --- | --- |
| A dimmed hint follows `/cost ` in the composer: `[YYYY-MM-DD]  or  START END hour\|day\|week\|month  (UTC)`. It disappears once an argument is typed, and is hidden when it doesn't fit whole | `01`, `01-hint-ansi` (`ESC[2m` = dim), `02`, `03` |
| The `/cost` view shows the same forms as a dimmed footer note: `Other dates: /cost …` | every page capture |
| The jargon lines are gone. Each page has at most one line about billing, and it names whose bill to check | `10`…`21` |
| A page with no recorded requests keeps its existing estimate line | `30` |

## Method

- **Build:** a debug `corbanu` with `codex-core/developer-accounting,codex-tui/developer-accounting`, built from `914024b6e1`.
- **Turn:** one real GLM 5.2 turn on `zai`, in a disposable home with `CORBANU_TEST_NO_NATIVE_KEYRING=1`. The key was passed only on the `exec` command.
- **Screens:** captured in a tmux TUI resumed with a placeholder key, so the TUI sent no turns. Each page capture combines the lines seen while scrolling, so some lines are out of screen order. Paths are redacted to `<scratch>`.
- **Cost check:** the product shows 0.02937828 USD for 25,085 input tokens (5,248 of them cache reads) and 55 output tokens. At $1.40, $0.26 and $4.40 per 1M tokens, that is the same figure.
- **Key scan:** the real key appears in none of these captures or the scratch home.

## Review

Installed `corbanu exec`, `claude-opus-5-5-plan` on `claude-plan`, `high` effort, `-s read-only`.

- **Round 1** ([review-result.md](review-result.md)): REQUEST CHANGES, 1 Major. A provider that reports charges (OpenRouter) but stated none on a response was said never to report them. Fixed in `914024b6e1`.
- **Round 2** ([review2-result.md](review2-result.md)): APPROVE WITH NITS. Fixed after it: the charge count now skips local-model work as the new lines do, and the refused-request line still points at the bill in case the request stopped mid-response. Neither path appears in these captures.
