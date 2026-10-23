# Runs the canary proxy; the real Z.AI key is fetched here, on the consuming command only.
. "$(dirname "$0")/../env.sh"
REAL_KEY="$(helper provider/zai_api_key)" exec python3 $S/scripts/canary_proxy.py 18585 $S/proxy/requests.jsonl $S/proxy/canaries.json https://api.z.ai/api/paas/v4 default,main,work
