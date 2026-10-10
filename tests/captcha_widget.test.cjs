const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const source = fs.readFileSync(require('node:path').join(__dirname, '../assets/captcha-interactive.js'), 'utf8');

function page(provider) {
  const elements = new Map();
  const callbacks = [];
  let removed = 0;
  const element = () => ({ style: {}, append() {}, appendChild() {}, remove() {} });
  const document = { getElementById: id => elements.get(id), createElement: element,
    body: { appendChild(box) { elements.set(box.id, box); } } };
  const api = { render(_, options) { callbacks.push(options); return callbacks.length; },
    remove() { removed++; }, execute() { throw new Error('manual mode must not execute challenges'); } };
  const window = { __sunoCliCaptchaConfig: { provider, sitekey: 'test-key' }, hcaptcha: api, turnstile: api };
  const context = vm.createContext({ window, document });
  return { window, callbacks, run: () => vm.runInContext(source, context), removed: () => removed };
}

for (const provider of [1, 2]) {
  test(`provider ${provider}: waits for human callback and never executes a challenge`, () => {
    const p = page(provider); p.run();
    assert.equal(p.window.__sunoCliCaptchaState.status, 'pending');
    assert.equal(p.callbacks[0].size, 'normal');
    p.callbacks[0].callback('single-use-test-response');
    assert.equal(p.window.__sunoCliCaptchaState.status, 'solved');
    assert.equal(p.window.__sunoCliCaptchaState.token, 'single-use-test-response');
    p.callbacks[0]['expired-callback']();
    assert.equal(p.window.__sunoCliCaptchaState.status, 'error');
    assert.equal(p.window.__sunoCliCaptchaState.token, undefined);
  });
  test(`provider ${provider}: next round removes prior widget and ignores stale callbacks`, () => {
    const p = page(provider); p.run();
    const old = p.callbacks[0]; old.callback('first'); p.run();
    assert.equal(p.removed(), 1);
    assert.equal(p.window.__sunoCliCaptchaState.status, 'pending');
    old.callback('replayed'); old['expired-callback']();
    assert.equal(p.window.__sunoCliCaptchaState.status, 'pending');
    p.callbacks[1].callback('fresh');
    assert.equal(p.window.__sunoCliCaptchaState.token, 'fresh');
  });
  test(`provider ${provider}: challenge failure never returns a token`, () => {
    const p = page(provider); p.run(); p.callbacks[0]['error-callback']();
    p.callbacks[0].callback('late');
    assert.equal(p.window.__sunoCliCaptchaState.status, 'error');
    assert.equal(p.window.__sunoCliCaptchaState.token, undefined);
  });
}
