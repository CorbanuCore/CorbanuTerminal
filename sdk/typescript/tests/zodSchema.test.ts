import { describe, expect, it } from "@jest/globals";
import z from "zod";

describe("Zod structured output", () => {
  const schema = z.object({
    summary: z.string(),
    status: z.enum(["ok", "action_required"]),
  });

  it("preserves the sample schema constraints in JSON Schema", () => {
    expect(z.toJSONSchema(schema, { target: "draft-07" })).toEqual({
      $schema: "http://json-schema.org/draft-07/schema#",
      type: "object",
      properties: {
        summary: { type: "string" },
        status: { type: "string", enum: ["ok", "action_required"] },
      },
      required: ["summary", "status"],
      additionalProperties: false,
    });
  });

  it.each(["ok", "action_required"])("accepts status %s", (status) => {
    expect(schema.parse({ summary: "Ready", status })).toEqual({ summary: "Ready", status });
  });

  it.each([
    { status: "ok" },
    { summary: "Ready" },
    { summary: 42, status: "ok" },
    { summary: "Ready", status: "unknown" },
  ])("rejects invalid structured output %j", (value) => {
    expect(schema.safeParse(value).success).toBe(false);
  });
});
