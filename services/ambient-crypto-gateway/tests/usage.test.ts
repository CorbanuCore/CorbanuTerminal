import assert from "node:assert/strict";
import { describe, test } from "node:test";

import {
  estimateRequestUsage,
  extractActualUsage,
  StreamUsageParser,
} from "../src/usage.js";

describe("inference usage authorization", () => {
  test("charges conservative input and maximum requested output units", () => {
    const body = Buffer.from(JSON.stringify({ model: "z-ai/glm-5.2", messages: [{ role: "user", content: "hello" }], max_tokens: 100 }));
    assert.deepEqual(estimateRequestUsage(body), {
      model: "z-ai/glm-5.2",
      estimatedInputTokens: Math.ceil(body.byteLength / 3),
      maxOutputTokens: 100,
      reservedTokens: Math.ceil(body.byteLength / 3) + 100,
    });
  });

  test("supports adjacent OpenAI token fields and rejects ambiguous or unbounded values", () => {
    assert.equal(
      estimateRequestUsage(Buffer.from('{"model":"z-ai/glm-5.2","max_completion_tokens":25}')).maxOutputTokens,
      25,
    );
    assert.throws(
      () => estimateRequestUsage(Buffer.from('{"model":"z-ai/glm-5.2","max_tokens":10,"max_completion_tokens":20}')),
      /must match/,
    );
    assert.throws(() => estimateRequestUsage(Buffer.from('{"model":"z-ai/glm-5.2","max_tokens":1000000}')), /32768/);
    assert.throws(() => estimateRequestUsage(Buffer.from("not-json")), /valid JSON/);
    assert.throws(() => estimateRequestUsage(undefined), /Content-Type/);
  });

  test("extracts authoritative chat and message usage without double-counting reasoning", () => {
    assert.deepEqual(
      extractActualUsage({
        usage: {
          prompt_tokens: 20,
          completion_tokens: 7,
          prompt_tokens_details: { cached_tokens: 4 },
          completion_tokens_details: { reasoning_tokens: 3 },
        },
      }),
      { inputTokens: 20, outputTokens: 7, cachedInputTokens: 4, reasoningTokens: 3, totalTokens: 27 },
    );
    assert.equal(extractActualUsage({ usage: { total_tokens: 99 } }), undefined);
  });

  test("extracts usage split across streaming SSE chunks", () => {
    const parser = new StreamUsageParser();
    parser.push(Buffer.from('data: {"choices":[{"delta":{"content":"x"}}]}\n\ndata: {"usage":{"input_tokens":'));
    parser.push(Buffer.from('8,"output_tokens":5}}\n\ndata: [DONE]\n\n'));
    assert.equal(parser.finish()?.totalTokens, 13);
  });
});
