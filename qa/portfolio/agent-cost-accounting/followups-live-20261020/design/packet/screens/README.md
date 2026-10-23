# Screens

Text copies of real product screens captured on developer builds between 2026-10-14 and 2026-10-16, in disposable
profiles. They show what the pages look like; they come from earlier builds than the one under test, and some were made
against local stand-ins (noted below). Paths are shortened to `/Users/tester/run/...`. Only the bottom part of each
`/cost` page fits on one screen; the pages scroll (↑/↓).

| File | What it shows |
| --- | --- |
| review-01-review-finished.txt | A `/review` of uncommitted changes (Z.AI), finished, followed by `/cost`'s first screen |
| review-02-cost-links.txt | The same `/cost` first screen scrolled to its links |
| review-03-request-technical.txt | The technical page of one of that conversation's requests made by the review (its `Turn:` line) |
| s06-01-side-review-technical.txt | Technical page of an automatic-review request made inside `/side` (Z.AI) |
| s07-moonshot-k3-first-screen / -technical | `/cost` for one `kimi-k3` first turn on a person-configured provider `kimi` at api.moonshot.ai (replayed through a stand-in of real usage) |
| s07-moonshot-k2-first-screen / -technical | The same for `kimi-k2.7-code` |
| s08-restart-notice | The session right after restarting on an OpenAI conversation: the warm-up notice |
| s08-restart-first-screen / s08-restart-reopen-warmup-request | `/cost` after that restart: the reopen warm-up line and its request page |
| s08-resume-same-conversation | `/resume` of the conversation already open |
| realtime-01-cost-pages-test-render.txt | `/cost` pages for one realtime session, rendered by a test against a stand-in (fixed 1970 dates and placeholder IDs; each page starts with `===`, and `LINKS` lists the page's links). Not a live screen. |
