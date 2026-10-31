'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');

// 调用实际 HUD 来源渲染函数；最小 DOM 只承载节点，不模拟会话或统计逻辑。
class Element {
  constructor() { this.children = []; this.dataset = {}; this.className = ''; this.textContent = ''; }
  setAttribute() {}
  append(...children) { children.forEach(child => { child.parent = this; this.children.push(child); }); }
  replaceChildren(...children) { this.children = []; this.append(...children); }
  remove() { this.parent.children = this.parent.children.filter(child => child !== this); }
  querySelector(selector) {
    return this.children.find(child => child.className === selector.slice(1))
      || this.children.map(child => child.querySelector(selector)).find(Boolean);
  }
}
const list = new Element();
const context = vm.createContext({
  document: { createElement: () => new Element() },
  byId: id => { assert.equal(id, 'source-list'); return list; },
  sourceViews: new Map(), hud: new Element(), selector: { value: '*' }, selectedAgent: '*',
  snapshot: { agentStats: [{ agent: 'hermes', daily: { requests: 5 } }, { agent: 'codex' }] },
  sourceIcon: () => new Element(), maskAccount: value => value,
  sessionIsActive: session => session.active === true,
});
const source = fs.readFileSync(require.resolve('../ui/hud.js'), 'utf8');
for (const name of ['sourceName', 'sourceKey', 'callerName', 'filterAgent', 'renderSources']) {
  const start = source.indexOf(`  function ${name}(`);
  assert.ok(start >= 0, name);
  const end = source.indexOf('\n  }\n', start) + 5;
  vm.runInContext(source.slice(start, end), context);
}
const codex = { id: 'codex-session', agent: 'codex', active: true };
const hermesKey = '["Hermes Agent",""]';
context.renderSources([codex]);
assert.equal(context.sourceViews.size, 2, 'sessionless Hermes remains visible alongside a Codex session');
assert.equal(context.sourceViews.get(hermesKey).querySelector('.source-copy').children[0].textContent, 'Hermes Agent');
assert.equal(context.sourceViews.get(hermesKey).dataset.active, 'false', 'completed usage does not fabricate a live session');
assert.equal(context.sourceViews.get('["Codex",""]').dataset.active, 'true');

context.selectedAgent = hermesKey;
context.renderSources([]);
assert.deepEqual([...context.sourceViews.keys()], [hermesKey], 'Hermes can be selected without any session card');

context.selectedAgent = '*';
context.selector.value = codex.id;
context.renderSources([codex]);
assert.deepEqual([...context.sourceViews.keys()], ['["Codex",""]'], 'a specific session stays scoped to its source');

context.selector.value = '*';
context.snapshot.agentStats = [];
context.renderSources([]);
assert.equal(context.sourceViews.size, 0, 'expired sources do not leave stale nodes');
assert.equal(list.textContent, '等待请求');
console.log('Source rendering: sessionless usage, Agent filtering, active state and session scope passed.');
