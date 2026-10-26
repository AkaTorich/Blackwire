'use strict';
/* Рамка окна, кнопки окна, клавиши, диалоги (подтверждение, ввод), тост и контекстное меню. */
var Win = (function () {
  var tauri = window.__TAURI__ || null;
  var frame = null;       // .win
  var nameEl = null;      // .win-name
  var maxBtn = null;
  var baseTitle = '';
  var modified = false;
  var keysBound = false;

  /* Текущее окно Tauri или заглушка для обычного браузера */
  function current() {
    if (tauri && tauri.window && tauri.window.getCurrentWindow) return tauri.window.getCurrentWindow();
    return {
      minimize: function () { return Promise.resolve(); },
      toggleMaximize: function () { return Promise.resolve(); },
      close: function () { window.close(); return Promise.resolve(); },
      setTitle: function (t) { document.title = t; return Promise.resolve(); },
      isMaximized: function () { return Promise.resolve(false); },
      isFullscreen: function () { return Promise.resolve(false); },
      setFullscreen: function () { return Promise.resolve(); },
      setFocus: function () { return Promise.resolve(); },
      startDragging: function () { return Promise.resolve(); },
      setMinSize: function () { return Promise.resolve(); }
    };
  }

  function el(tag, cls, html) {
    var e = document.createElement(tag);
    if (cls) e.className = cls;
    if (html != null) e.innerHTML = html;
    return e;
  }

  function swallow() {}

  function refreshMaxIcon() {
    if (!maxBtn) return;
    current().isMaximized().then(function (m) {
      maxBtn.innerHTML = Icons.get(m ? 'restore' : 'max', 16);
      maxBtn.title = m ? 'Восстановить' : 'Развернуть';
      if (frame) frame.classList.toggle('maximized', !!m);
    }, swallow);
  }

  /* ---------- рамка ---------- */
  function init(opts) {
    opts = opts || {};
    var root = document.getElementById('root');
    root.innerHTML = '';
    frame = el('div', 'win');
    var header = el('header', 'win-title');
    header.setAttribute('data-tauri-drag-region', '');
    var icon = el('span', 'win-icon', Icons.get(opts.icon || 'hexagon', 16));
    icon.setAttribute('data-tauri-drag-region', '');
    nameEl = el('span', 'win-name');
    nameEl.setAttribute('data-tauri-drag-region', '');
    var btns = el('span', 'win-btns');
    var minB = el('button', 'min', Icons.get('min', 16));
    minB.title = 'Свернуть';
    maxBtn = el('button', 'max', Icons.get('max', 16));
    maxBtn.title = 'Развернуть';
    var closeB = el('button', 'close', Icons.get('close', 16));
    closeB.title = 'Закрыть';
    btns.appendChild(minB); btns.appendChild(maxBtn); btns.appendChild(closeB);
    header.appendChild(icon); header.appendChild(nameEl); header.appendChild(btns);
    var body = el('main', 'win-body');
    frame.appendChild(header); frame.appendChild(body);
    root.appendChild(frame);

    minB.addEventListener('click', function () { BW.sound('click'); current().minimize().catch(swallow); });
    maxBtn.addEventListener('click', function () {
      BW.sound('click');
      current().toggleMaximize().then(refreshMaxIcon, swallow);
    });
    closeB.addEventListener('click', function () { close(); });
    header.addEventListener('dblclick', function (e) {
      if (e.target.closest('.win-btns')) return;
      current().toggleMaximize().then(refreshMaxIcon, swallow);
    });
    window.addEventListener('resize', refreshMaxIcon);

    if (opts.minWidth && opts.minHeight && tauri && tauri.dpi && tauri.dpi.LogicalSize) {
      try {
        current().setMinSize(new tauri.dpi.LogicalSize(opts.minWidth, opts.minHeight)).catch(swallow);
      } catch (e) { /* старый API — размер задаёт Rust */ }
    }

    setTitle(opts.title || 'Blackwire');
    bindKeys();
    refreshMaxIcon();
    BW.sound('open');
    return body;
  }

  function renderName() {
    if (!nameEl) return;
    nameEl.textContent = (modified ? '• ' : '') + baseTitle;
    nameEl.title = baseTitle;
  }

  function setTitle(text) {
    baseTitle = text || '';
    renderName();
    document.title = baseTitle;
    current().setTitle(baseTitle).catch(swallow);
  }

  function setModified(flag) {
    modified = !!flag;
    renderName();
    if (frame) frame.classList.toggle('modified', modified);
  }

  function close() {
    BW.sound('close');
    if (frame) frame.classList.add('closing');
    setTimeout(function () { current().close().catch(swallow); }, 120);
  }

  /* ---------- клавиши ---------- */
  function openApp(app) {
    if (window.Apps && Apps.openOrFocus) Apps.openOrFocus(app, null);
    else BW.open_app(app);
  }

  function bindKeys() {
    if (keysBound) return;
    keysBound = true;
    document.addEventListener('keydown', function (e) {
      var k = e.key;
      if (e.ctrlKey && e.shiftKey && !e.altKey) {
        var code = k.length === 1 ? k.toUpperCase() : k;
        var map = { 'T': 'terminal', 'E': 'editor', 'F': 'files', 'M': 'mail',
          'Е': 'terminal', 'У': 'editor', 'А': 'files', 'Ь': 'mail' };
        if (map[code]) { e.preventDefault(); openApp(map[code]); return; }
      }
      if (k === 'F11') {
        e.preventDefault();
        var w = current();
        w.isFullscreen().then(function (f) { return w.setFullscreen(!f); }).catch(swallow);
        return;
      }
      if (k === 'Escape') {
        if (document.querySelector('.overlay, .menu')) return; // Esc сначала закрывает диалог/меню
        var w2 = current();
        w2.isFullscreen().then(function (f) { if (f) return w2.setFullscreen(false); }).catch(swallow);
        return;
      }
      if (e.ctrlKey && !e.shiftKey && !e.altKey && (k === 'w' || k === 'W' || k === 'ц' || k === 'Ц')) {
        if (BW.app === 'desktop') return;
        e.preventDefault();
        close();
      }
    });
  }

  /* ---------- диалоги ---------- */
  function overlay() {
    var o = el('div', 'overlay');
    document.body.appendChild(o);
    requestAnimationFrame(function () { o.classList.add('show'); });
    return o;
  }

  function removeOverlay(o) {
    o.classList.remove('show');
    setTimeout(function () { if (o.parentNode) o.parentNode.removeChild(o); }, 160);
  }

  function dialog(text, withInput, value) {
    return new Promise(function (resolve) {
      var o = overlay();
      var d = el('div', 'dialog');
      var msg = el('div', 'dialog-text');
      msg.textContent = text;
      d.appendChild(msg);
      var input = null;
      if (withInput) {
        input = el('input', 'input dialog-input');
        input.type = 'text';
        input.value = value || '';
        input.spellcheck = false;
        d.appendChild(input);
      }
      var row = el('div', 'dialog-btns');
      var ok = el('button', 'btn btn-primary');
      ok.textContent = withInput ? 'Готово' : 'Да';
      var cancel = el('button', 'btn');
      cancel.textContent = withInput ? 'Отмена' : 'Нет';
      row.appendChild(cancel); row.appendChild(ok);
      d.appendChild(row);
      o.appendChild(d);

      function finish(v) {
        document.removeEventListener('keydown', onKey, true);
        removeOverlay(o);
        resolve(v);
      }
      function onKey(e) {
        if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); finish(withInput ? null : false); }
        else if (e.key === 'Enter') { e.preventDefault(); e.stopPropagation(); finish(withInput ? input.value : true); }
      }
      document.addEventListener('keydown', onKey, true);
      ok.addEventListener('click', function () { BW.sound('click'); finish(withInput ? input.value : true); });
      cancel.addEventListener('click', function () { BW.sound('click'); finish(withInput ? null : false); });
      o.addEventListener('mousedown', function (e) { if (e.target === o) finish(withInput ? null : false); });
      setTimeout(function () { if (input) { input.focus(); input.select(); } else ok.focus(); }, 30);
    });
  }

  function confirm(text) { return dialog(text, false); }
  function prompt(text, value) { return dialog(text, true, value); }

  /* ---------- тост ---------- */
  var toastHost = null;
  function toast(text, kind) {
    if (!toastHost || !toastHost.parentNode) {
      toastHost = el('div', 'toast-host');
      document.body.appendChild(toastHost);
    }
    var t = el('div', 'toast' + (kind ? ' ' + kind : ''));
    t.textContent = text;
    toastHost.appendChild(t);
    requestAnimationFrame(function () { t.classList.add('show'); });
    setTimeout(function () {
      t.classList.remove('show');
      setTimeout(function () { if (t.parentNode) t.parentNode.removeChild(t); }, 200);
    }, 2600);
  }

  /* ---------- контекстное меню ---------- */
  var openMenus = [];

  function closeMenus() {
    openMenus.forEach(function (m) { if (m.parentNode) m.parentNode.removeChild(m); });
    openMenus = [];
    document.removeEventListener('mousedown', onDocDown, true);
    document.removeEventListener('keydown', onMenuKey, true);
  }
  function onDocDown(e) { if (!e.target.closest('.menu')) closeMenus(); }
  function onMenuKey(e) { if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); closeMenus(); } }

  function place(m, x, y) {
    m.style.left = '0px'; m.style.top = '0px';
    var w = m.offsetWidth, h = m.offsetHeight;
    var vw = window.innerWidth, vh = window.innerHeight;
    if (x + w > vw - 6) x = Math.max(6, vw - w - 6);
    if (y + h > vh - 6) y = Math.max(6, vh - h - 6);
    m.style.left = x + 'px'; m.style.top = y + 'px';
  }

  function buildMenu(items, x, y, level) {
    var m = el('div', 'menu');
    m.dataset.level = String(level);
    items.forEach(function (it) {
      if (it === null) { m.appendChild(el('div', 'menu-sep')); return; }
      var row = el('div', 'menu-item' + (it.disabled ? ' disabled' : '') + (it.children ? ' has-sub' : ''));
      row.innerHTML = '<span class="menu-ico">' + (it.icon ? Icons.get(it.icon, 16) : '') + '</span>' +
        '<span class="menu-label"></span>' +
        (it.children ? '<span class="menu-arrow">' + Icons.get('chevron-right', 14) + '</span>' : '');
      row.querySelector('.menu-label').textContent = it.label;
      if (it.children && !it.disabled) {
        var openSub = function () {
          /* закрыть подменю глубже этого уровня */
          openMenus = openMenus.filter(function (o) {
            if (parseInt(o.dataset.level, 10) > level) { if (o.parentNode) o.parentNode.removeChild(o); return false; }
            return true;
          });
          var r = row.getBoundingClientRect();
          buildMenu(it.children, r.right - 2, r.top - 4, level + 1);
        };
        row.addEventListener('mouseenter', openSub);
        row.addEventListener('click', openSub);
      } else if (!it.disabled) {
        row.addEventListener('mouseenter', function () {
          openMenus = openMenus.filter(function (o) {
            if (parseInt(o.dataset.level, 10) > level) { if (o.parentNode) o.parentNode.removeChild(o); return false; }
            return true;
          });
        });
        row.addEventListener('click', function (e) {
          e.stopPropagation();
          BW.sound('click');
          closeMenus();
          if (typeof it.action === 'function') it.action();
        });
      }
      m.appendChild(row);
    });
    document.body.appendChild(m);
    openMenus.push(m);
    place(m, x, y);
    requestAnimationFrame(function () { m.classList.add('show'); });
    return m;
  }

  function menu(items, x, y) {
    closeMenus();
    buildMenu(items, x, y, 0);
    document.addEventListener('mousedown', onDocDown, true);
    document.addEventListener('keydown', onMenuKey, true);
  }

  return {
    init: init, setTitle: setTitle, setModified: setModified, close: close,
    confirm: confirm, prompt: prompt, toast: toast, menu: menu, closeMenus: closeMenus,
    current: current, bindKeys: bindKeys
  };
})();
