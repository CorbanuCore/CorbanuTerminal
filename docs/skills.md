# Skills

Corbanu Terminal inherits Codex skills and ships bundled system skills.

## Built-In Skill Loading

Bundled skills are installed into:

```text
$CODEX_HOME/skills/.system/
```

For a fresh Corbanu Terminal home, that is:

```text
$HOME/.corbanu/skills/.system/
```

An upgraded installation that reuses a lone `$HOME/.pfterminal` home keeps its
skills there. An explicit `CODEX_HOME` remains authoritative.

These skills load regardless of the directory where the user starts
Corbanu Terminal. User-installed global skills can also live in:

```text
$HOME/.agents/skills/
```

Repo-scoped skills can live in:

```text
<repo>/.agents/skills/
```

## Current Bundled Skills

Corbanu Terminal currently includes the inherited Codex system skills plus
Corbanu Terminal's frontend design skill:

| Skill             | Purpose                                             |
| ----------------- | --------------------------------------------------- |
| `frontend-design` | Browser frontend design and implementation guidance |
| `imagegen`        | Generate or edit raster images                      |
| `openai-docs`     | Reference OpenAI/Codex docs                         |
| `plugin-creator`  | Scaffold Codex plugins                              |
| `skill-creator`   | Create or update a skill                            |
| `skill-installer` | Install curated or GitHub-hosted skills             |

Use:

```text
/skills
```

to browse loaded skills in the TUI.

For inherited Codex skills behavior, see:

<https://developers.openai.com/codex/skills>
