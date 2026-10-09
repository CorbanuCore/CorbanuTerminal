#!/bin/bash
# remote side: C1 placeholder attempts for all 21 built-in providers in one disposable home (no real credentials)
R=<rtx-scratch>; H=$R/homes/h-c1; mkdir -p $H; printf 'model = "glm-5.2"\nmodel_provider = "zai"\n' > $H/config.toml
cd $R/work/repo
(python3 $R/local_mock.py --port 11434 --log $R/logs/mock-ollama.jsonl &) ; (python3 $R/local_mock.py --port 1234 --log $R/logs/mock-lmstudio.jsonl &); sleep 1
run(){ tag=$1; kv=$2; shift 2; env "$kv=placeholder-not-a-real-key" RUST_LOG=warn CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$H CORBANU_HOME=$H PFTERMINAL_HOME=$H $R/corbanu-acct exec --json --skip-git-repo-check -s read-only "$@" "Reply with exactly: ok" </dev/null > $R/logs/$tag.jsonl 2> $R/logs/$tag.err; echo "$tag exit=$? $(grep -o '"message":"[^"]*' $R/logs/$tag.jsonl | tail -1 | cut -c1-140)"; }
run c1-openai CODEX_API_KEY -c 'model_provider="openai"' -m gpt-5.4
run c1-anthropic ANTHROPIC_API_KEY -c 'model_provider="anthropic"' -m claude-opus-5-5
run c1-claude-plan X -c 'model_provider="claude-plan"' -m claude-opus-5-5-plan
run c1-ambient AMBIENT_API_KEY -c 'model_provider="ambient"' -m test-model
run c1-pfterminal-plan CORBANU_API_KEY -c 'model_provider="pfterminal-plan"' -m test-model
run c1-pfterminal-plan-anthropic CORBANU_API_KEY -c 'model_provider="pfterminal-plan-anthropic"' -m x-ai/grok-4.7
run c1-kimi-code KIMI_API_KEY -c 'model_provider="kimi-code"' -m k3
run c1-zai ZAI_API_KEY -c 'model_provider="zai"' -m glm-5.2
run c1-zai-anthropic ZAI_API_KEY -c 'model_provider="zai-anthropic"' -m glm-5.2
run c1-openrouter OPENROUTER_API_KEY -c 'model_provider="openrouter"' -m z-ai/glm-5.2
run c1-openrouter-anthropic OPENROUTER_API_KEY -c 'model_provider="openrouter-anthropic"' -m x-ai/grok-4.7
run c1-deepseek DEEPSEEK_API_KEY -c 'model_provider="deepseek"' -m deepseek-flash
run c1-meta MODEL_API_KEY -c 'model_provider="meta"' -m muse-spark-1.1
run c1-baseten BASETEN_API_KEY -c 'model_provider="baseten"' -m test-model
run c1-baseten-anthropic BASETEN_API_KEY -c 'model_provider="baseten-anthropic"' -m x-ai/grok-4.7
run c1-vercel AI_GATEWAY_API_KEY -c 'model_provider="vercel"' -m vercel/moonshotai/kimi-k3
run c1-vercel-anthropic AI_GATEWAY_API_KEY -c 'model_provider="vercel-anthropic"' -m x-ai/grok-4.7
run c1-vercel-anthropic-fast AI_GATEWAY_API_KEY -c 'model_provider="vercel-anthropic-fast"' -m x-ai/grok-4.7
run c1-amazon-bedrock AWS_BEARER_TOKEN_BEDROCK -c 'model_provider="amazon-bedrock"' -c 'model_providers.amazon-bedrock.aws.region="us-east-1"' -m openai.gpt-5.4
run c1-ollama X -c 'model_provider="ollama"' -m mock-model
run c1-lmstudio X -c 'model_provider="lmstudio"' -m mock-model
pkill -f local_mock.py
