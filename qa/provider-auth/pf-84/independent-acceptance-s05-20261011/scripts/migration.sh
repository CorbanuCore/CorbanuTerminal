#!/bin/bash
# migration.sh : older ledgers and existing homes, created by the pre-S05 baseline (BASE, bddf6292c4).
#  hm: single-account home (flag off, default Z.AI only) -> candidate flag off, then flag on, default only.
#  hb: baseline home with flag on and a named `work` account -> candidate default, then candidate `work`
#      (upgrade to format 4), then the baseline again on the upgraded ledger.
cd "$(dirname "$0")" && . ./env.sh
OUT=$EV/captures/migration.txt; : > "$OUT"
log() { echo "$*" >> "$OUT"; }
snap() { # snap <home> <label> <orig-ids-file>
  local h=$1 db; db=$(ls "$h"/pfterminal_state_*.sqlite)
  log "  [$2] config.toml $(shasum -a 256 "$h/config.toml" | cut -c1-16)  local.age $( [ -f "$h/secrets/local.age" ] && shasum -a 256 "$h/secrets/local.age" | cut -c1-16 || echo absent)"
  log "  [$2] ledger format rows: $(python3 - "$db" <<'PY'
import sqlite3, sys
db = sqlite3.connect(f"file:{sys.argv[1]}?mode=ro", uri=True); out = []
for t in ("draft_accounting_ledger_format", "draft_accounting_ledger_format_3", "draft_accounting_ledger_format_4"):
    try: out += [r[0] for r in db.execute(f"select version from {t}")]
    except sqlite3.OperationalError: pass
print(sorted(out))
PY
) ; attempts $(sqlite3 "$db" 'select count(*) from draft_accounting_attempts'); with account field $(sqlite3 "$db" "select count(*) from draft_accounting_attempts where payload like '%\"account\"%'")"
  if [ -n "$3" ]; then
    log "  [$2] sha256 of the baseline's original attempt rows (id||payload, ordered): $(sqlite3 "$db" "select attempt_id || '|' || payload from draft_accounting_attempts where attempt_id in ($(sed "s/.*/'&'/" "$3" | paste -sd, -)) order by attempt_id" | shasum -a 256 | cut -c1-16)"
  fi
}
x() { # x <bin> <home> <name> args...
  local b=$1 h=$2 name=$3; shift 3
  key provider/zai_api_key env CORBANU_HOME="$h" S="$S" python3 ./cleanexec.py ZAI_API_KEY -- "$b" exec --skip-git-repo-check -C "$S/ws" \
    "$@" "Reply with exactly the word pong and nothing else." > "$S/logs/mig-$name.out" 2> "$S/logs/mig-$name.err"
  local rc=$? who=CAND; [ "$b" = "$BASE" ] && who=BASE
  log "  $name: $who exec $* -> exit=$rc reply=$(tr -d '\n' < "$S/logs/mig-$name.out" | cut -c1-20) $(grep -Ei 'error|ledger|format|refus' "$S/logs/mig-$name.err" | grep -v call_metrics | tail -2 | cut -c1-300 | tr '\n' ' ')"
}

HM=$S/homes/hm; mkdir -p "$HM"
printf 'model = "glm-5.3-flash"\nmodel_provider = "zai"\n' > "$HM/config.toml"
log "== hm: existing single-account home (baseline-created, default Z.AI only, flag off) =="
x "$BASE" "$HM" hm-base1; x "$BASE" "$HM" hm-base2
sqlite3 "$HM"/pfterminal_state_*.sqlite 'select attempt_id from draft_accounting_attempts' > "$S/logs/hm-orig-ids.txt"
snap "$HM" "after baseline" "$S/logs/hm-orig-ids.txt"
x "$CAND" "$HM" hm-cand-off
snap "$HM" "after candidate, flag off" "$S/logs/hm-orig-ids.txt"
x "$CAND" "$HM" hm-cand-on --enable named_accounts
x "$CAND" "$HM" hm-cand-on-default --enable named_accounts --account default
log "  account list (flag on): $(run "$CAND" "$HM" account list --enable named_accounts 2>&1 | grep -v WARNING | tr '\n' ' ')"
snap "$HM" "after candidate, flag on, default only" "$S/logs/hm-orig-ids.txt"
x "$BASE" "$HM" hm-base3
snap "$HM" "after baseline again" "$S/logs/hm-orig-ids.txt"

HB=$S/homes/hb; mkdir -p "$HB"
printf 'model = "glm-5.3-flash"\nmodel_provider = "zai"\n\n[features]\nnamed_accounts = true\n' > "$HB/config.toml"
log ""; log "== hb: baseline home with named account work (flag on) =="
key provider/zai_api_key run_secret_stdin "$BASE" "$HB" account add zai work >/dev/null 2>&1
x "$BASE" "$HB" hb-base-default; x "$BASE" "$HB" hb-base-work --account work
sqlite3 "$HB"/pfterminal_state_*.sqlite 'select attempt_id from draft_accounting_attempts' > "$S/logs/hb-orig-ids.txt"
snap "$HB" "after baseline (default + work)" "$S/logs/hb-orig-ids.txt"
x "$CAND" "$HB" hb-cand-default
snap "$HB" "after candidate default" "$S/logs/hb-orig-ids.txt"
x "$CAND" "$HB" hb-cand-work --account work
snap "$HB" "after candidate work" "$S/logs/hb-orig-ids.txt"
x "$BASE" "$HB" hb-base-after-upgrade
snap "$HB" "after baseline on the format-4 ledger" "$S/logs/hb-orig-ids.txt"
sed "s#$S#\$S#g" "$OUT"
