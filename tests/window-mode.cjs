'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');

class Element {
  constructor() { this.dataset = {}; this.attributes = {}; this.listeners = {}; }
  setAttribute(name, value) { this.attributes[name] = value; }
  addEventListener(name, fn) { this.listeners[name] = fn; }
}

(async () => {
  const button = new Element(), hud = new Element(), document = new Element();
  document.getElementById = id => ({ fullscreen: button, hud })[id];
  const window = {};
  vm.runInNewContext(fs.readFileSync(require.resolve('../ui/window-mode.js'), 'utf8'), { window, document });
  const calls = [], errors = [];
  let nativeChange, fail = false;
  await window.hudWindowMode.bind({
    initial: { fullscreen: false },
    invoke: async (command, args) => { if (fail) throw Error('native failure'); calls.push([command, args.fullscreen]); },
    listen: async (name, fn) => { assert.equal(name, 'fullscreen-changed'); nativeChange = fn; },
    reportError: error => errors.push(error),
  });
  button.listeners.click();
  assert.deepEqual(calls, [['set_fullscreen', true]]);
  assert.equal(hud.dataset.fullscreen, 'false', 'request does not pretend the native animation finished');
  assert.equal(button.disabled, true);
  nativeChange({ payload: true });
  assert.equal(button.attributes['aria-label'], '退出全屏');
  assert.equal(button.disabled, false);

  const key = options => {
    const event = { defaultPrevented: false, repeat: false, preventDefault() { this.prevented = true; }, ...options };
    document.listeners.keydown(event);
    return event;
  };
  assert.equal(key({ key: 'Escape', defaultPrevented: true }).prevented, undefined, 'a popup can consume Escape first');
  assert.equal(key({ key: 'Escape' }).prevented, true);
  assert.deepEqual(calls.at(-1), ['set_fullscreen', false]);
  nativeChange({ payload: false });
  key({ code: 'KeyF', ctrlKey: true, metaKey: true });
  assert.deepEqual(calls.at(-1), ['set_fullscreen', true]);
  nativeChange({ payload: true });
  key({ key: 'F11' });
  assert.deepEqual(calls.at(-1), ['set_fullscreen', false]);
  nativeChange({ payload: false });
  fail = true;
  button.listeners.click();
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(errors.length, 1);
  assert.equal(button.disabled, false, 'native refusal permits retry');
  assert.equal(hud.dataset.fullscreen, 'false');
  console.log('Fullscreen: native state, button, shortcuts, Escape priority and failure recovery passed.');
})().catch(error => { console.error(error); process.exitCode = 1; });
