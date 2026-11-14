# Generates the synthetic bearer canaries the mock provider recognises (values never real keys).
import json, secrets, sys
S = sys.argv[1]
c = {f"cnry-{n}-{secrets.token_hex(12)}": n for n in ["acct-a", "acct-b", "env", "default"]}
for a in ["default", "work", "ghost"]: c[f"cmdtok-{a}"] = f"cmd:{a}"
open(f"{S}/mock/canaries.json", "w").write(json.dumps(c))
