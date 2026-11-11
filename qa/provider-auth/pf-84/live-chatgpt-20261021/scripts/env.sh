# Environment used for every product run in this record (sourced, not executed).
# <L> is the throwaway run dir .codex-work/pf84-live1 (deleted after the run).
L=<L>
B=/Volumes/CorbanuDrive/Corbanu/.codex-work/integration-build-20260922a/integration-package-dev/bin/corbanu  # installed signed build, 2f9d05c51e
export HOME=$L/hhome CODEX_HOME=$L/home CORBANU_HOME=$L/home PFTERMINAL_HOME=$L/home
export CORBANU_TEST_NO_NATIVE_KEYRING=1 TMPDIR=$L/tmp   # build has debug assertions, so the keyring guard applies
unset OPENAI_API_KEY CODEX_API_KEY CODEX_ACCESS_TOKEN
c() { sandbox-exec -f $L/nokeychain.sb "$B" "$@"; }
# Home config: cli_auth_credentials_store = "file"; [features] named_accounts = true
# Default login: `printf 'sk-proj-FAKE…' | c login --with-api-key` (a fake key, never valid)
# Z.AI (switch step only): ZAI_API_KEY from the operator vault helper at use time, never written to the home.
