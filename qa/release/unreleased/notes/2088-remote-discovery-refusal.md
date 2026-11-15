## Windows sandbox

- Moderate and Aggressive now carry the Windows sandbox refusal through
  executor skill/plugin discovery and skill resource reads, including full-disk
  permission profiles and catalogs captured before the live level changed.
  Remote discovery checks the executor before sending roots. The receiving
  executor also rejects marked file operations when the requested default
  Windows sandbox is not set up. Ready remote environments without a sandbox
  also get a startup notice explaining the refusal and setup steps. Permissive
  behavior is unchanged.
