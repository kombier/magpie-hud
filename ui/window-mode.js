'use strict';

// Only native window events confirm fullscreen; requesting it can start an animation on macOS.
window.hudWindowMode = {
  async bind({ invoke, listen, initial, reportError }) {
    const button = document.getElementById('fullscreen');
    const hud = document.getElementById('hud');
    hud.dataset.nativeControls = String(Boolean(initial.nativeControls));
    let fullscreen = false, requested = null;
    const render = actual => {
      fullscreen = Boolean(actual);
      if (requested === fullscreen) requested = null;
      hud.dataset.fullscreen = String(fullscreen);
      button.setAttribute('aria-pressed', String(fullscreen));
      button.setAttribute('aria-label', fullscreen ? '退出全屏' : '进入全屏');
      button.title = `${fullscreen ? '退出全屏' : '进入全屏'}（⌃⌘F / F11）`;
      button.disabled = requested !== null;
    };
    const set = async target => {
      if (requested !== null || target === fullscreen) return;
      requested = target;
      button.disabled = true;
      try { await invoke('set_fullscreen', { fullscreen: target }); }
      catch (error) { requested = null; render(fullscreen); reportError(error); }
    };
    await listen('fullscreen-changed', event => render(event.payload));
    render(initial.fullscreen);
    button.addEventListener('click', () => set(!fullscreen));
    document.addEventListener('keydown', event => {
      if (event.defaultPrevented || event.repeat) return;
      if ((event.code === 'KeyF' && event.ctrlKey && event.metaKey && !event.altKey && !event.shiftKey)
          || (event.key === 'F11' && !event.ctrlKey && !event.metaKey && !event.altKey && !event.shiftKey)) {
        event.preventDefault();
        void set(!fullscreen);
      } else if (event.key === 'Escape' && fullscreen) {
        event.preventDefault();
        void set(false);
      }
    });
  },
};
