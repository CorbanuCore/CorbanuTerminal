# Runs the canary proxy; the real Z.AI key is fetched here, on the consuming command only.
. "$(dirname "$0")/lib.sh"
REAL_KEY="$(helper provider/zai_api_key)" exec python3 "$(dirname "$0")/canary_proxy.py" 18686 $S/proxy/requests.jsonl $S/proxy/canaries.json https://api.z.ai/api/paas/v4 pdefault,pmain,pwork
