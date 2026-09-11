"""Build conservative per-platform dispositions; never infer whole-case passes.

Execution records are separate from the original design. A passing component
test is recorded as supporting evidence, not a waiver of the remaining steps.
"""
import hashlib
import html
import json
from pathlib import Path

root = Path(__file__).resolve().parent


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def ref(path):
    return {"path": path, "sha256": digest(root / path)}


gaps = {
    1: "Fresh setup fixtures exist on Linux, but the full empty-profile inventory/cancel journey is not covered on both platforms.",
    2: "Synthetic OpenAI expired-account cancel/recovery is exercised on Linux; fresh real-account login, selection and same-session request are not verified here.",
    3: "Linux fake Claude login/compatibility setup is exercised; subscription inference after that login is not verified.",
    4: "Managed-token form/catalog/restart are exercised on both packages; successful subscription inference after setup is not verified.",
    5: "Anthropic masked setup and model catalog are exercised on both packages; actual Anthropic request routing after setup is not verified.",
    6: "Linux custom environment/command/managed routing fixtures are exercised; a bounded complete provider list and requests for every supported provider are missing.",
    7: "Separate Claude/Anthropic setup and catalogs are exercised. Crossed credential types, repair-one/use-both and billing-route receipts are missing.",
    8: "Empty/whitespace inputs stay in the form; a deliberately invalid saved key becomes Enabled · configured on both platforms. Whether this meets the proposed no-false-success expectation is unresolved; stored is not validated. A healthy-other-provider request after each attempt is also missing.",
    9: "Synthetic API form masking/cancel/scrollback pass on both packages; all secret-bearing routes, edit operations and long-token paste variants are not covered.",
    10: "API and account cancellation fixtures run on Linux, and masked cancellation on both; preservation of an unfinished composer message and every route is not proven.",
    11: "Linux rejection/recovery and eligibility fixtures cover subsets; simultaneous checking/expired/disabled status and full menu/chat parity on both platforms are missing.",
    12: "Linux synthetic API/account rejection and same-session recovery are exercised; complete cross-route/profile/platform recovery, message-count and other-provider assertions are missing.",
    13: "Separate setup leaves the selected Claude model unchanged in package probes. Nonselected credential replacement followed by requests to both providers is not proven.",
    14: "Linux actual-key deactivation/reactivation/current-cancel/replacement fixtures run; the complete equivalent Mac sequence is missing.",
    15: "Both packages show distinct Claude and Anthropic catalog entries. Linux duplicate-slug/current-identity tests run; the complete Mac duplicate-custom-provider and effort combination is not proven.",
    16: "Linux Astra model/effort request and restart are covered; switching among incompatible/no-effort models with matching wire evidence is missing.",
    17: "Linux Astra effort cancellation/confirmation/request and exact identity fixtures run; cancel at every provider/model/effort level with an existing conversation on Mac is missing.",
    18: "Synthetic package restart and two real-profile Mac menu launches pass. Native Applications shortcut invocation, disabled-state persistence and live requests in that same full journey are missing.",
    19: "Linux existing-config/custom/resume fixtures run and the real Mac profile opens twice unchanged. Historical migration range and live chat on that existing Mac profile are not proven.",
    20: "Two existing-profile Mac processes reached menus without password entry; no native Applications launch or controlled Keychain deny/cancel/allow recovery was performed. Linux is not the platform for this native case; the overall Mac prerequisite remains open.",
    21: "Linux tests run through SSH with real tmux keys and same-home restart. Client detach/reattach plus the complete account-login handoff sequence is not demonstrated.",
    22: "Both packages complete real Code Mode/shell/MCP calls against synthetic servers. Independent expired tool vs model credentials, inverse cases and live codex_apps reconnect are not proven by that smoke test.",
    23: "Auth rejection/retry fixtures run on Linux. Distinct timeout, offline, rate-limit and server-error status/recovery matrix is missing.",
    24: "Both packages show token guidance at 40 columns. Short-height navigation, long custom names and last-item/resize-back behavior are not fully exercised.",
    25: "Many actual-key open/select/cancel paths run; exhaustive entry/menu boundary and repeat-cycle coverage is not established.",
    26: "No new controlled delayed-health-result UI race journey was executed. Historical unit tests are not a substitute for this proposed observable flow.",
}
design = json.loads((root / "design.json").read_text())
for platform, binary_hash, evidence in (
    ("macOS arm64", "8275923c4ee9c0bfc7e52109f742564c5a429631d765e15b6f0643b4cb7d3669",
     ["mac-package/result.json", "mac-existing/result.json", "mac-inputs-ready/result.json"]),
    ("Linux x86_64", "5ecd62f90574e1b6208625220abf93e431d347c1f2a91c9111c3512e81448062",
     ["linux/tmux-tests.log", "linux/package-tmux/result.json", "linux/inputs-ready/result.json"]),
):
    cases = []
    for case in design["cases"]:
        number = int(case["id"].split("F")[-1])
        cases.append({"id": case["id"], "disposition": "blocked", "summary": gaps[number],
                      "candidate_sha256": binary_hash, "supporting_evidence": [ref(p) for p in evidence]})
    result = {"design_sha256": digest(root / "design.json"), "implementer": "/root",
              "candidate": {"version": "0.1.41", "source": "da77f7c03827d56284d62a3dadb477aed36bb6ce; source-equivalent existing package",
                            "platform": platform, "binary_sha256": binary_hash,
                            "package_manifest": ref("candidate-manifest.md")},
              "cases": cases, "human_acceptance": False,
              "evidence_check": {"agent": "/root/blind_functional_designer", "verdict": "pending"},
              "review_budget": {"used": 7, "limit": 5, "ledger": ref("review-ledger.md"),
                                "extension": {"by": "Travis Good (user)",
                                              "reason": "Explicit approval for two additional passes only",
                                              "artifact": ref("budget-amendment.md")}}}
    name = "results-mac.json" if platform.startswith("macOS") else "results-linux.json"
    (root / name).write_text(json.dumps(result, indent=2) + "\n")

rows = "".join(
    "<tr><td>" + html.escape(case["id"]) + "</td><td>" + html.escape(case["priority"]) +
    "</td><td>Incomplete</td><td>" + html.escape(gaps[int(case["id"].split("F")[-1])]) + "</td></tr>"
    for case in design["cases"]
)
(root / "report.html").write_text('''<!doctype html><html lang="en"><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Independent functional qualification — PF-58</title>
<style>body{margin:0;background:#111318;color:#eee;font:16px/1.5 system-ui}
main{max-width:1100px;margin:auto;padding:40px 24px}h1{line-height:1.1}
a{color:#57d6d0}.notice{padding:20px;border:1px solid #f6c76a;border-radius:12px;color:#f6c76a}
table{width:100%;border-collapse:collapse;margin-top:24px}td,th{text-align:left;padding:14px;border-bottom:1px solid #41444a;vertical-align:top}
td:nth-child(3){color:#f6c76a}td:first-child{white-space:nowrap}small{color:#aaa}
@media(max-width:650px){table{font-size:13px}td,th{padding:8px}}
</style><main><h1>Independent functional qualification</h1>
<p>PF-58 · Provider setup, authentication recovery and model selection · macOS and Linux</p>
<div class="notice"><strong>Not ready for unqualified human testing.</strong>
<p>26 independently proposed cases are preserved. Partial regression passes are not full case passes.
The remaining conditions below have not been waived. “Incomplete” is not a claim that every feature failed.</p></div>
<p>Exact execution receipts: <a href="linux/tmux-tests.log">Linux regression run</a>,
<a href="mac-package/result.json">Mac package TMUX</a>,
<a href="linux/package-tmux/result.json">Linux package TMUX</a>,
<a href="mac-existing/result.json">existing Mac profile</a>.</p>
<p><a href="original-proposal.md">Frozen independent proposal</a> ·
<a href="candidate-manifest.md">Candidate identity</a> ·
<a href="evidence-check.md">Independent evidence check</a> ·
<a href="execution-notes.md">Execution limits</a></p>
<table><thead><tr><th>Case</th><th>Priority</th><th>Full-case status</th><th>Evidence and remaining gap</th></tr></thead><tbody>'''
    + rows + '</tbody></table><p><small>No human acceptance or scope exclusion is inferred. '
    'Two additional review passes were authorized; no further review loop is scheduled.</small></p></main></html>')
