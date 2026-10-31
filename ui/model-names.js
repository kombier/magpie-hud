'use strict';

// Presentation only: raw request models remain available for routing, pricing and diagnostics.
(() => {
  const display = value => typeof value === 'string' && /^glm-5\.3$/i.test(value) ? 'GLM-5.3' : value;
  const ids = value => display(value) === 'GLM-5.3' ? ['GLM-5.3', 'glm-5.3'] : [value];
  const api = { display, ids };
  if (typeof module !== 'undefined') module.exports = api;
  else window.hudModelNames = api;
})();
