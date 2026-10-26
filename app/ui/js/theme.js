'use strict';
/* Применение настроек к документу: тема, размер шрифта, прозрачность терминала, стекло. */
var Theme = (function () {
  var root = document.documentElement;

  function store(key, value) {
    try { localStorage.setItem(key, String(value)); } catch (e) { /* без хранилища */ }
  }

  function apply(settings) {
    if (!settings) return;
    var theme = settings.theme || 'swordfish';
    if (root.dataset.theme !== theme) root.dataset.theme = theme;
    store('bw_theme', theme);

    var fs = parseInt(settings.font_size, 10);
    if (!(fs >= 10 && fs <= 24)) fs = 14;
    root.style.setProperty('--font-size', fs + 'px');
    store('bw_font', fs);

    var op = parseFloat(settings.terminal_opacity);
    if (!(op >= 0.2 && op <= 1)) op = 0.85;
    root.style.setProperty('--term-opacity', String(op));

    var wop = parseFloat(settings.window_opacity);
    if (!(wop >= 0.3 && wop <= 1)) wop = 0.72;
    root.style.setProperty('--win-opacity', String(wop));

    var glass = !settings.fx || settings.fx.glass !== false;
    root.classList.toggle('no-glass', !glass);
    store('bw_glass', glass ? '1' : '0');

    root.classList.toggle('no-sound', !settings.sound);
  }

  /* Цвет переменной темы в виде {r,g,b} — нужен холсту */
  function color(name, fallback) {
    var v = getComputedStyle(root).getPropertyValue(name).trim() || fallback || '#39d8ff';
    return parse(v);
  }

  function parse(v) {
    var m;
    if ((m = /^#([0-9a-f]{3})$/i.exec(v))) {
      return { r: parseInt(m[1][0] + m[1][0], 16), g: parseInt(m[1][1] + m[1][1], 16), b: parseInt(m[1][2] + m[1][2], 16) };
    }
    if ((m = /^#([0-9a-f]{6})([0-9a-f]{2})?$/i.exec(v))) {
      return { r: parseInt(m[1].slice(0, 2), 16), g: parseInt(m[1].slice(2, 4), 16), b: parseInt(m[1].slice(4, 6), 16) };
    }
    if ((m = /^rgba?\(\s*([\d.]+)[,\s]+([\d.]+)[,\s]+([\d.]+)/i.exec(v))) {
      return { r: +m[1], g: +m[2], b: +m[3] };
    }
    return { r: 57, g: 216, b: 255 };
  }

  function rgba(c, a) { return 'rgba(' + c.r + ',' + c.g + ',' + c.b + ',' + a + ')'; }

  BW.onSettings(apply);

  return { apply: apply, color: color, parse: parse, rgba: rgba };
})();
