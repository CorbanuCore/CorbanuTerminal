const DEFAULT_MAX_OUTPUT_TOKENS = 4_096;
const ABSOLUTE_MAX_OUTPUT_TOKENS = 32_768;

export interface EstimatedUsage {
  model: string;
  estimatedInputTokens: number;
  maxOutputTokens: number;
  reservedTokens: number;
}

export interface ActualUsage {
  inputTokens: number;
  outputTokens: number;
  cachedInputTokens: number;
  reasoningTokens: number;
  totalTokens: number;
}

export function estimateRequestUsage(body: unknown): EstimatedUsage {
  const request = parseRequest(body);
  const model = typeof request.model === "string" ? request.model.trim() : "";
  if (!model) throw new Error("model is required");
  const maxTokens = parseMaxTokens(request.max_tokens, "max_tokens");
  const maxCompletionTokens = parseMaxTokens(
    request.max_completion_tokens,
    "max_completion_tokens",
  );
  if (
    maxTokens !== undefined &&
    maxCompletionTokens !== undefined &&
    maxTokens !== maxCompletionTokens
  ) {
    throw new Error("max_tokens and max_completion_tokens must match when both are supplied");
  }
  const maxOutputTokens = maxTokens ?? maxCompletionTokens ?? DEFAULT_MAX_OUTPUT_TOKENS;
  const estimatedInputTokens = Math.max(1, Math.ceil((body as Buffer).byteLength / 3));
  return {
    model,
    estimatedInputTokens,
    maxOutputTokens,
    reservedTokens: estimatedInputTokens + maxOutputTokens,
  };
}

export function extractActualUsage(value: unknown): ActualUsage | undefined {
  if (!value || typeof value !== "object" || Array.isArray(value)) return undefined;
  const root = value as Record<string, unknown>;
  const usage = root.usage;
  if (!usage || typeof usage !== "object" || Array.isArray(usage)) return undefined;
  const fields = usage as Record<string, unknown>;
  const inputTokens = readNonNegativeInteger(fields, ["input_tokens", "prompt_tokens"]);
  const outputTokens = readNonNegativeInteger(fields, ["output_tokens", "completion_tokens"]);
  if (inputTokens === undefined || outputTokens === undefined) return undefined;
  const cachedInputTokens =
    readNonNegativeInteger(fields, ["cached_input_tokens", "cache_read_input_tokens"]) ??
    nestedInteger(fields.prompt_tokens_details, "cached_tokens") ??
    0;
  const reasoningTokens =
    readNonNegativeInteger(fields, ["reasoning_tokens"]) ??
    nestedInteger(fields.completion_tokens_details, "reasoning_tokens") ??
    0;
  return {
    inputTokens,
    outputTokens,
    cachedInputTokens,
    reasoningTokens,
    totalTokens: inputTokens + outputTokens,
  };
}

/** Incrementally extracts the last authoritative usage object from SSE data lines. */
export class StreamUsageParser {
  private pending = "";
  private usage?: ActualUsage;

  push(chunk: Uint8Array): void {
    this.pending += Buffer.from(chunk).toString("utf8");
    const lines = this.pending.split(/\r?\n/);
    this.pending = lines.pop() ?? "";
    for (const line of lines) this.inspectLine(line);
  }

  finish(): ActualUsage | undefined {
    if (this.pending) this.inspectLine(this.pending);
    this.pending = "";
    return this.usage;
  }

  private inspectLine(line: string): void {
    const payload = line.startsWith("data:") ? line.slice(5).trim() : "";
    if (!payload || payload === "[DONE]") return;
    try {
      this.usage = extractActualUsage(JSON.parse(payload)) ?? this.usage;
    } catch {
      // Partial and non-JSON SSE events do not carry authoritative usage.
    }
  }
}

function parseRequest(body: unknown): Record<string, unknown> {
  if (!Buffer.isBuffer(body)) {
    throw new Error("inference request Content-Type must be application/json");
  }
  let request: unknown;
  try {
    request = JSON.parse(body.toString("utf8"));
  } catch {
    throw new Error("inference body must be valid JSON");
  }
  if (!request || typeof request !== "object" || Array.isArray(request)) {
    throw new Error("inference body must be a JSON object");
  }
  return request as Record<string, unknown>;
}

function parseMaxTokens(value: unknown, name: string): number | undefined {
  if (value === undefined) return undefined;
  if (
    !Number.isInteger(value) ||
    (value as number) < 1 ||
    (value as number) > ABSOLUTE_MAX_OUTPUT_TOKENS
  ) {
    throw new Error(`${name} must be an integer from 1 through ${ABSOLUTE_MAX_OUTPUT_TOKENS}`);
  }
  return value as number;
}

function readNonNegativeInteger(
  fields: Record<string, unknown>,
  names: readonly string[],
): number | undefined {
  for (const name of names) {
    const value = fields[name];
    if (Number.isSafeInteger(value) && (value as number) >= 0) return value as number;
  }
  return undefined;
}

function nestedInteger(value: unknown, name: string): number | undefined {
  if (!value || typeof value !== "object" || Array.isArray(value)) return undefined;
  return readNonNegativeInteger(value as Record<string, unknown>, [name]);
}
