// Deterministic UI regressions. No network or service operations.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const source = fs.readFileSync(__dirname + '/facilities.js', 'utf8');

async function run() {
  const message = { textContent: '', dataset: {}, setAttribute() {} };
  const button = { dataset: { facilityAction: 'start', facilityId: 'fixture' },
    addEventListener(_event, listener) { this.click = listener; } };
  const card = { dataset: { facilityId: 'fixture' }, querySelector() { return null; },
    querySelectorAll(selector) { return selector === '[data-facility-action]' ? [button] : []; } };
  let refreshMessage, tick, finishAction;
  let statusFails = false;
  const root = { dataset: { controlEndpoint: 'http://fixture.invalid' },
    querySelector() { return message; }, append(item) { refreshMessage = item; } };
  const response = (ok, payload) => ({ ok, async json() { return payload; } });
  vm.runInNewContext(source, {
    document: {
      querySelector() { return root; },
      querySelectorAll(selector) { return selector === '[data-facility-action]' ? [button] : [card]; },
      createElement() { return { dataset: {} }; },
    },
    window: { confirm() { return true; }, setInterval(callback) { tick = callback; } },
    fetch: async (url) => {
      if (url.endsWith('/action')) return new Promise((resolve) => { finishAction = resolve; });
      if (statusFails) throw new Error('offline');
      return response(true, { ok: true, facilities: { fixture: {
        id: 'fixture', status: 'stopped', actions: { start: true, stop: false },
      } } });
    },
  });
  await tick();
  const pending = button.click();
  await tick();
  assert.equal(message.textContent, 'Starting service…');
  assert.equal(button.disabled, true, 'polling must not enable an in-flight action');
  finishAction(response(false, { ok: false, message: 'Outcome uncertain; check before retrying.' }));
  await pending;
  await tick();
  assert.match(message.textContent, /Outcome uncertain/);
  assert.match(refreshMessage.textContent, /Live status checked/);
  statusFails = true;
  await tick();
  assert.match(message.textContent, /Outcome uncertain/);
  assert.match(refreshMessage.textContent, /bridge unavailable/);
  assert.equal(button.disabled, true);
}
run().then(() => process.stdout.write('Facilities UI regression passed\n')).catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
