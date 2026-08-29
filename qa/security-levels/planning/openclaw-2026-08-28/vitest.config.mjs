// Minimal reference harness: no OpenClaw install hooks or Gateway startup.
import path from 'node:path';
import { execFileSync } from 'node:child_process';
const source = process.env.OPENCLAW_REVIEW_SOURCE;
if (!source || !path.isAbsolute(source)) throw new Error('Set absolute OPENCLAW_REVIEW_SOURCE');
const pin = execFileSync('git', ['-C', source, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim();
if (pin !== '13adff02ca3897768d80d2bca18f5acf08c55d91') throw new Error('Wrong OpenClaw source pin');
if (execFileSync('git', ['-C', source, 'status', '--porcelain', '--untracked-files=no'], { encoding: 'utf8' }).trim()) throw new Error('Upstream tracked files must be unchanged');
export default {
  root: source,
  cacheDir: path.join(source, 'node_modules/.vite/reference-review'),
  resolve: { alias: [
    { find: /^@openclaw\/normalization-core$/, replacement: path.join(source, 'packages/normalization-core/src/index.ts') },
    { find: /^@openclaw\/normalization-core\/(.*)$/, replacement: path.join(source, 'packages/normalization-core/src/$1.ts') },
  ] },
  test: {
    include: ['src/security/external-content.test.ts', 'src/agents/embedded-agent-runner/run/turn-taint-state.test.ts'],
    maxWorkers: 1, fileParallelism: false, testTimeout: 10000,
  },
};
