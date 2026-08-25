# Research

Internal research artifacts live here rather than in the user-facing MkDocs
tree or release evidence directories. Generated reports stay beside their
source material so local provenance links remain valid.

## Product and security working session

The [`2026-08-23-product-security-session`](2026-08-23-product-security-session/)
packet contains:

- [`repository-guide.html`](2026-08-23-product-security-session/repository-guide.html),
  an onboarding map distilled from the working session and repository docs;
- [`SecurityComparativeAnalysis.html`](2026-08-23-product-security-session/SecurityComparativeAnalysis.html),
  an opinionated comparison of agent-harness security architectures and the
  proposed Corbanu security model;
- [`SecurityComparativeAnalysis.artifact.json`](2026-08-23-product-security-session/SecurityComparativeAnalysis.artifact.json),
  the validated source artifact used to build the portable HTML report; and
- [`Working session - 2026_08_23 11_31 MST - Recording.diarized.md`](2026-08-23-product-security-session/Working%20session%20-%202026_08_23%2011_31%20MST%20-%20Recording.diarized.md),
  the automated diarized transcript. Speaker labels and transcription details
  are machine-generated and should not be treated as identity-confirmed.

## Tmux testing

[`tmux-testing/tmux-testing-report.html`](tmux-testing/tmux-testing-report.html)
is a source-and-design study for a layered TUI test harness. It is research,
not QA evidence: tmux was unavailable on the research workstation, so the
report did not execute the proposed test matrix.

[`tmux-testing/tmuxPlan.html`](tmux-testing/tmuxPlan.html) turns that research
into a bounded engineering work package with functional and non-functional
requirements, implementation stages, required tests, and acceptance gates. It
is the umbrella plan for the completed
[`RW-TMUX-01`](../qa/work-packages/RW-TMUX-01.md) and
[`RW-TMUX-02`](../qa/work-packages/RW-TMUX-02.md) packages and the completed
[`RW-TMUX-03`](../qa/work-packages/RW-TMUX-03.md) package. `RW-TMUX-04` remains
the planned release-matrix adoption increment. These are Routine
execution records, not product sprints or release artifacts.

## Research tools

[`tools/diarize_recording.py`](tools/diarize_recording.py) produced the working
session transcript using MLX Whisper, SpeechBrain, PyTorch, SoundFile, NumPy,
and scikit-learn. Those optional, platform-heavy dependencies are intentionally
not part of the repository's locked general-purpose `scripts` environment.
