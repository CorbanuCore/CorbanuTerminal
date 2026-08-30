// Corbanu-authored reference probes; synthetic values only, not product qualification.
import assert from 'node:assert/strict';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { execFileSync } from 'node:child_process';

const source = process.env.OPENCLAW_REVIEW_SOURCE;
assert(source, 'OPENCLAW_REVIEW_SOURCE must name the pinned OpenClaw checkout');
assert.equal(execFileSync('git', ['-C', source, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(), '13adff02ca3897768d80d2bca18f5acf08c55d91');
assert.equal(execFileSync('git', ['-C', source, 'status', '--porcelain', '--untracked-files=no'], { encoding: 'utf8' }).trim(), '', 'Upstream tracked files must be unchanged');
const upstream = (file) => import(pathToFileURL(path.join(source, file)).href);
const sentinel = await upstream('src/secrets/sentinel.ts');
const registry = await upstream('src/logging/secret-redaction-registry.ts');
const { createSecretEgressBodyTransform } = await upstream('src/secrets/egress-proxy/stream-substitution.ts');
const { createAgentTurnTaintState } = await upstream('src/agents/embedded-agent-runner/run/turn-taint-state.ts');
const results = [];
const check = (name, run) => { run(); results.push({ name, outcome: 'pass' }); };
const raw = 'synthetic-review-credential';
const opaque = sentinel.mintSecretSentinel(raw, { label: 'review-only' });

check('process-local authenticated round trip', () => {
  assert.notEqual(opaque, raw);
  assert.equal(sentinel.resolveSecretSentinel(opaque), raw);
});
check('tampered reference rejected', () => {
  const start = sentinel.SECRET_SENTINEL_PREFIX.length;
  const tampered = opaque.slice(0, start) + (opaque[start] === 'A' ? 'B' : 'A') + opaque.slice(start + 1);
  assert.equal(sentinel.resolveSecretSentinel(tampered), undefined);
});
check('raw opt-out exists', () => {
  const previous = process.env.OPENCLAW_SECRET_SENTINELS;
  try {
    process.env.OPENCLAW_SECRET_SENTINELS = 'off';
    assert.equal(sentinel.mintSecretSentinel(raw, { label: 'review-only' }), raw);
  } finally {
    if (previous === undefined) delete process.env.OPENCLAW_SECRET_SENTINELS;
    else process.env.OPENCLAW_SECRET_SENTINELS = previous;
  }
});
check('short secret omitted by registry', () => {
  registry.registerSecretValueForRedaction('z9Q2x');
  assert.equal(registry.isSecretValueRegisteredForRedaction('z9Q2x'), false);
});
check('registered raw and percent-encoded forms match', () => {
  const value = 'synthetic/credential+with spaces';
  registry.registerSecretValueForRedaction(value);
  assert.equal(registry.redactRegisteredSecretValues(value, () => '[masked]'), '[masked]');
  assert.equal(registry.redactRegisteredSecretValues(encodeURIComponent(value), () => '[masked]'), '[masked]');
});
check('old value evicted after 512 distinct later values', () => {
  const old = 'synthetic-old-credential';
  registry.registerSecretValueForRedaction(old);
  for (let index = 0; index < 512; index++) registry.registerSecretValueForRedaction(`synthetic-later-${index}`);
  assert.equal(registry.isSecretValueRegisteredForRedaction(old), false);
  assert.equal(registry.redactRegisteredSecretValues(old, () => '[masked]'), old);
});
check('separate redaction calls do not carry split values', () => {
  const value = 'synthetic-split-credential';
  registry.registerSecretValueForRedaction(value);
  const joined = [value.slice(0, 10), value.slice(10)].map(chunk => registry.redactRegisteredSecretValues(chunk, () => '[masked]')).join('');
  assert.equal(joined, value);
});

async function transform(chunks, resolveSentinel) {
  let output = '';
  const stream = createSecretEgressBodyTransform({ resolveSentinel, onSubstitution() {} });
  const finished = new Promise(resolve => {
    stream.on('data', chunk => { output += chunk.toString('utf8'); });
    stream.once('error', error => resolve({ output, reason: error.reason }));
    stream.once('end', () => resolve({ output }));
  });
  for (const chunk of chunks) stream.write(chunk);
  stream.end();
  return finished;
}
const split = await transform([...Buffer.from(opaque)].map(byte => Buffer.from([byte])), sentinel.resolveSecretSentinel);
assert.equal(split.output, raw);
assert.equal(split.reason, undefined);
results.push({ name: 'request sentinel substituted across one-byte chunks', outcome: 'pass' });
const late = await transform(['ordinary-prefix:', 'oc-sent-v2.unknown.end'], () => undefined);
assert.equal(late.reason, 'unresolved-sentinel');
assert.equal(late.output, 'ordinary-prefix:');
results.push({ name: 'late refusal does not retract previously emitted request prefix', outcome: 'pass' });
check('new turn helper resets unless caller supplies initial taint', () => {
  const first = createAgentTurnTaintState();
  first.observe({ toolName: 'fixture', argsHash: 'args', resultHash: 'result', resultContentSource: 'network' });
  assert.equal(first.isTainted(), true);
  assert.equal(createAgentTurnTaintState().isTainted(), false);
  assert.equal(createAgentTurnTaintState(true).isTainted(), true);
});
console.log(JSON.stringify({ kind: 'reference-observation-probes', total: results.length, results }, null, 2));
