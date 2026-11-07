# Contributing

<<<<<<< HEAD
Corbanu Terminal changes must preserve a coherent product, not merely compile.
Before starting work, read the [Corbanu Terminal development skill](corbanu-terminal-development-skill.md),
the repository-root `AGENTS.md`, and the relevant nested `AGENTS.md` files.
||||||| 413492cd6c
**External contributions are by invitation only**
=======
We welcome community contributions through the [openai/codex issue tracker](https://github.com/openai/codex/issues). Bug reports, root-cause analyses, and feature requests help us understand what matters most and improve Codex.
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

<<<<<<< HEAD
## Start with product intent
||||||| 413492cd6c
At this time, the Codex team does not accept unsolicited code contributions.
=======
**We do not accept external code contributions or pull requests.**
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

<<<<<<< HEAD
- Identify the exact heading and requirement excerpt in the
  [product specification](corbanu-product-spec.md).
- Classify the change as routine, bounded fix, product initiative, or release.
- Use an active plan for a product initiative. At most two product initiatives
  may be active at once.
- Select one single-feature [current sprint](sprints/index.md). Do not implement
  directly from plan prose.
- Record the implementation worktree, branch, base commit, owner, and scope in
  both the plan and the executable sprint.
||||||| 413492cd6c
If you would like to propose a new feature or a change in behavior, please open an issue describing the proposal or upvote an existing enhancement request. We prioritize new features based on community feedback, alignment with our roadmap, and consistency across all Codex surfaces (CLI, IDE extensions, web, etc.).
=======
### Why we do not accept external code contributions
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

<<<<<<< HEAD
If the product specification does not authorize the outcome, obtain the
required product decision before implementation.
||||||| 413492cd6c
If you encounter a bug, please open a bug report or verify that an existing report already covers the issue. If you would like to help, we encourage you to contribute by sharing analysis, reproduction details, root-cause hypotheses, or a high-level outline of a potential fix directly in the issue thread.
=======
Effective changes to Codex require architectural context, an understanding of system-level constraints, and visibility into the project's roadmap. External pull requests often focus on issues that are lower priority, affect a small number of users, or need substantial changes to fit the broader system. Reviewing and iterating on those changes can take more time than implementing a fix directly, diverting attention from higher-priority work.
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

<<<<<<< HEAD
## Keep changes focused
||||||| 413492cd6c
The Codex team may invite an external contributor to submit a pull request when:
=======
Community expertise is most valuable when shared through detailed bug reports, reproduction steps, logs, root-cause analysis, and design discussions in issues. Understanding the problem, identifying the right solution, and prioritizing the work are typically the hard parts; implementation is comparatively straightforward with the help of Codex itself.
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

<<<<<<< HEAD
- Work on one classified outcome at a time.
- Work only the selected sprint's `Remaining` checklist; move verified items to
  `Done` as evidence changes.
- Preserve unrelated changes already present in the worktree.
- Fix failed behavior at the responsible boundary instead of hard-coding the
  reported example.
- Add regression coverage for the general failure class.
- Follow the nearest scoped engineering guidance for every changed path.
||||||| 413492cd6c
- the problem is well understood,
- the proposed approach aligns with the team’s intended solution, and
- the issue is deemed high-impact and high-priority.
=======
For these reasons, we focus community contributions on issue reports, analysis, and feedback, while the Codex team handles code changes.
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

<<<<<<< HEAD
## Prove the final tree
||||||| 413492cd6c
Pull requests that have not been explicitly invited by a member of the Codex team will be closed without review.
=======
### Reporting bugs
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

<<<<<<< HEAD
Run the affected formatting, lint, unit, integration, and snapshot checks after
the last code-changing tool. User-facing interactive behavior also requires a
real Corbanu Terminal TUI run in a PTY with actual keys sent. Smoke tests and
non-interactive `exec` runs are supporting evidence, not substitutes.
||||||| 413492cd6c
**Why we do not generally accept external code contributions**
=======
Before opening a new issue, search the issue tracker to see whether the problem has already been reported. If it has, add any new information to the existing issue.
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

<<<<<<< HEAD
Use the [tmux TUI harness](tmuxHarness.md) for isolated, key-driven terminal
scenarios, stable viewport assertions, cleanup, and failure artifacts.
||||||| 413492cd6c
In the past, the Codex team accepted external pull requests for bug fixes. While we appreciated the effort and engagement from the community, this model did not scale well.
=======
When reporting a bug, include as much relevant detail as possible:
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

<<<<<<< HEAD
Release candidates additionally require live workflows in TensorCash and
Isometric Game, named human acceptance, current user documentation, and the
benchmark and coding-performance gate when due.
||||||| 413492cd6c
Many contributions were made without full visibility into the architectural context, system-level constraints, or near-term roadmap considerations that guide Codex development. Others focused on issues that were low priority or affected a very small subset of users. Reviewing and iterating on these PRs often took more time than implementing the fix directly, and diverted attention from higher-priority work.
=======
- Clear, detailed steps to reproduce the problem.
- Expected and actual behavior.
- Your Codex version, operating system, and other relevant environment details.
- Logs, error messages, or other diagnostic information, with sensitive information removed.
- Root-cause analysis, technical observations, or potential approaches to a fix, if available.
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

<<<<<<< HEAD
## Document only finished behavior
||||||| 413492cd6c
The most valuable contributions consistently came from community members who demonstrated deep understanding of a problem domain. That expertise is most helpful when shared early -- through detailed bug reports, analysis, and design discussion in issues. Identifying the right solution is typically the hard part; implementing it is comparatively straightforward with the help of Codex itself.
=======
### Requesting features
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

<<<<<<< HEAD
User documentation explains the pain solved and the finished user flow.
Unfinished product contracts belong in the product specification or a plan.
Mechanical work in progress belongs in `docs/sprints/current/`; raw release
evidence belongs under `qa/release/<version>/`. Completed sprint records move to
the excluded archive and leave the documentation navigation.

Every feature page cites the exact product-spec heading and short requirement
excerpt it implements.

## Pull requests

A reviewable pull request includes:

- the change class and product citation;
- the plan link when required;
- the sprint id, status, and exact worktree when required;
- a focused explanation of the user outcome;
- final-tree test results;
- true-TUI and live-repository evidence when required;
- documentation changes; and
- any release blocker that remains.

Do not mark work complete while required evidence is missing.

## Licence of contributions

Corbanu Terminal is licensed under [Apache-2.0](license.md), and contributions
come in under the same licence: anything you submit for inclusion is licensed
under Apache-2.0 unless you explicitly say otherwise (section 5 of the
licence). There is no contributor licence agreement (CLA) to sign, and a DCO
`Signed-off-by` line is not required.

## Security reports

Do not place credentials, wallet seeds, private keys, protected financial data,
or exploitable details in a public issue. Use the repository's private security
reporting channel and provide the smallest safe reproduction that identifies
the failed boundary.
||||||| 413492cd6c
For these reasons, we focus external contributions on discussion, analysis, and feedback, and reserve code changes for cases where a targeted invitation makes sense.

### Development workflow

If you are invited by a Codex team member to contribute a PR, here is the recommended development workflow.

- Create a _topic branch_ from `main` - e.g. `feat/interactive-prompt`.
- Keep your changes focused. Multiple unrelated fixes should be opened as separate PRs.
- Ensure your change is free of lint warnings and test failures.

### Guidance for invited code contributions

1. **Start with an issue.** Open a new one or comment on an existing discussion so we can agree on the solution before code is written.
2. **Add or update tests.** A bug fix should generally come with test coverage that fails before your change and passes afterwards. 100% coverage is not required, but aim for meaningful assertions.
3. **Document behavior.** If your change affects user-facing behavior, update the README, inline help (`codex --help`), or relevant example projects.
4. **Keep commits atomic.** Each commit should compile and the tests should pass. This makes reviews and potential rollbacks easier.

### Model metadata updates

When a change updates model catalogs or model metadata (`/models` payloads, presets, or fixtures):

- Set `input_modalities` explicitly for any model that does not support images.
- Keep compatibility defaults in mind: omitted `input_modalities` currently implies text + image support.
- Ensure client surfaces that accept images (for example, TUI paste/attach) consume the same capability signal.
- Add/update tests that cover unsupported-image behavior and warning paths.

### Opening a pull request (by invitation only)

- Fill in the PR template (or include similar information) - **What? Why? How?**
- Include a link to a bug report or enhancement request in the issue tracker
- Run **all** checks locally. Use the root `just` helpers so you stay consistent with the rest of the workspace: `just fmt`, `just fix -p <crate>` for the crate you touched, and the relevant tests (e.g., `just test -p codex-tui` or `just test` if you need a full sweep). CI failures that could have been caught locally slow down the process.
- Make sure your branch is up-to-date with `main` and that you have resolved merge conflicts.
- Mark the PR as **Ready for review** only when you believe it is in a merge-able state.

### Review process

1. One maintainer will be assigned as a primary reviewer.
2. If your invited PR introduces scope or behavior that was not previously discussed and approved, we may close the PR.
3. We may ask for changes. Please do not take this personally. We value the work, but we also value consistency and long-term maintainability.
4. When there is consensus that the PR meets the bar, a maintainer will squash-and-merge.

### Community values

- **Be kind and inclusive.** Treat others with respect; we follow the [Contributor Covenant](https://www.contributor-covenant.org/).
- **Assume good intent.** Written communication is hard - err on the side of generosity.
- **Teach & learn.** If you spot something confusing, open an issue or discussion with suggestions or clarifications.

### Getting help

If you run into problems setting up the project, would like feedback on an idea, or just want to say _hi_ - please open a Discussion topic or jump into the relevant issue. We are happy to help.

Together we can make Codex CLI an incredible tool. **Happy hacking!** :rocket:

### Contributor license agreement (CLA)

All contributors **must** accept the CLA. The process is lightweight:

1. Open your pull request.
2. Paste the following comment (or reply `recheck` if you've signed before):

   ```text
   I have read the CLA Document and I hereby sign the CLA
   ```

3. The CLA-Assistant bot records your signature in the repo and marks the status check as passed.

No special Git commands, email attachments, or commit footers required.

### Security & responsible AI

Have you discovered a vulnerability or have concerns about model output? Please e-mail **security@openai.com** and we will respond promptly.
=======
Open a feature request in the issue tracker, or upvote an existing request that describes the same need. Explain your use case, the behavior you would like, and why it would improve your workflow.

### Community values

- **Be kind and inclusive.** Treat others with respect; we follow the [Contributor Covenant](https://www.contributor-covenant.org/).
- **Assume good intent.** Written communication is hard, so err on the side of generosity.
- **Share what you learn.** Reproduction details, logs, and analysis help the entire community.

### Security

If you discover a security vulnerability, follow the [security policy](../SECURITY.md) instead of reporting it in a public issue.
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
