# Notes for the next release

Each change on `main` that users will notice adds its own release-note
fragment under `notes/`. There is no shared `RELEASE_NOTES.md` to edit here, so
PRs no longer conflict over release notes (D431). CI (`repo-checks`) fails if
`qa/release/unreleased/RELEASE_NOTES.md` comes back or a fragment is malformed.

## Adding a fragment

Create `notes/<pr-or-issue-number>-<short-slug>.md`, for example
`notes/1796-pending-interrupt.md`. Use lowercase letters, digits, `.`, `_` and
`-`. The file starts with a `## ` section heading, followed by the entries:

```markdown
## Moderate: waiting on and starting agents after untrusted content

- **Interrupting a just-started agent stops its first turn again** (D435).
  ...
```

- To add to an existing section, copy its heading exactly; the entries are
  appended to that section when the notes are assembled. Run
  `python3 scripts/release/release_notes.py preview` to see the headings in use.
- One `## ` section per fragment. `###` subsections are fine; a `# ` title is
  added at release time. Entries added to a section that ends in a `###`
  subsection land under that subsection, so start yours with its own `###`
  heading if it doesn't belong there.
- To change an entry that is already on `main`, edit the fragment that holds it.

Check your fragment with:

```shell
python3 scripts/release/release_notes.py check
python3 scripts/release/release_notes.py preview
```

## At release time

```shell
python3 scripts/release/release_notes.py assemble --version <version>
```

This writes `qa/release/<version>/RELEASE_NOTES.md` (appending after any notes
already there), and deletes the fragments it used. It refuses a section heading
that file already has; merge those by hand. The version follows the release
workflow's pattern (`x.y.z`, `x.y.z-alpha.N` or `x.y.z-beta.N`). Sections
appear in the order of the first fragment, by file name, that uses each heading;
entries within a section follow file-name order. The `0000-NN-*` fragments hold the notes that
were in the old shared file, in their original order.
