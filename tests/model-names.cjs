'use strict';
const assert = require('node:assert/strict');
const { display, ids } = require('../ui/model-names.js');
const { aggregate, group, filter, dimension } = require('../ui/details-data.js');

assert.equal(display('glm-5.3'), 'GLM-5.3');
assert.equal(display('GLM-5.3'), 'GLM-5.3');
for (const model of ['glm-5.3-flash', 'glm-5.3-flashx', 'glm-5.2', 'gpt-6.1-sol']) assert.equal(display(model), model);
assert.deepEqual(ids('GLM-5.3'), ['GLM-5.3', 'glm-5.3'], 'backend filtering retains both raw model IDs');
const rows = [
  { id:'key', model:'glm-5.3', provider:'zhipu', tokens:100, input:10, output:10, cacheRead:80, generationMs:1000, outcome:'success' },
  { id:'plan', model:'GLM-5.3', provider:'zcode', tokens:200, input:20, output:20, cacheRead:160, generationMs:2000, outcome:'success' },
  { id:'flash', model:'glm-5.3-flash', provider:'zhipu', tokens:5, input:5, output:0, cacheRead:0, generationMs:null, outcome:'failed' },
];
const original = JSON.stringify(rows);
const costs = [{rowIds:['key'],cost:1,priced:1},{rowIds:['plan'],cost:2,priced:1}];
const groups = group(rows, row => dimension(row,'model'), costs);
const glm = groups.find(g => g.key === 'GLM-5.3');
assert.equal(groups.length,2);
assert.equal(glm.stats.calls,2);
assert.equal(glm.stats.tokens,300);
assert.equal(glm.stats.cost,3, 'provider prices are added after matching the original requests');
assert.equal(glm.stats.speed,10, 'TPS remains weighted by output and generation time');
assert.equal(aggregate(filter(rows,{model:new Set(['GLM-5.3'])}),costs).cost,3);
assert.equal(filter(rows,{model:new Set(['GLM-5.3']),provider:new Set(['zhipu'])}).length,1, 'provider filtering still distinguishes channels');
assert.equal(JSON.stringify(rows),original, 'display grouping never mutates raw model IDs');

// Exercise the actual sidebar grouping, including its separate priced-row response.
const fs = require('node:fs');
const vm = require('node:vm');
const source = fs.readFileSync(require.resolve('../ui/model-usage.js'),'utf8');
const context = vm.createContext({window:{hudModelNames:{display}},document:{getElementById:()=>({}),querySelector:()=>({})}});
vm.runInContext(source.slice(0,source.indexOf('  function sumModels(')) +
  'globalThis.collectModels=collect;globalThis.setModelCosts=rows=>{mode="cost";costState={hasData:true,rows};};})();',context);
const modelRows = rows.map(r=>({...r,requests:1,outputTokens:r.output,speedSamples:r.generationMs?1:0}));
let sidebar = context.collectModels(modelRows);
assert.equal(sidebar.length,2);
assert.equal(sidebar.find(m=>m.name==='GLM-5.3').tokens,300);
context.setModelCosts([{model:'glm-5.3',cost:1,priced:1},{model:'GLM-5.3',cost:2,priced:1}]);
sidebar=context.collectModels(modelRows);
assert.equal(sidebar.find(m=>m.name==='GLM-5.3').cost,3);
assert.equal(sidebar.find(m=>m.name==='GLM-5.3').priced,2);
console.log('Model display: GLM casing merged, raw IDs and provider pricing preserved.');
