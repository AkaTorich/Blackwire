'use strict';
// Подсветка синтаксиса для редактора: язык Adder и простые текстовые форматы.
// Выход — HTML для <pre>: текст разбит на куски {t, c} (текст, класс), затем экранируется.
// Файл: js/apps/editor_highlight.js (подключать перед editor.js).

const EditorHighlight = (function () {
  const KEYWORDS = new Set(('if elif else while for in def return break continue pass lambda try except ' +
    'finally raise global and or not true false null import as').split(' '));
  const CONSTS = new Set(['true', 'false', 'null']);
  const BUILTINS = new Set(('print input len str int float bool list dict range type abs min max sum round ' +
    'sorted reversed enumerate zip any all chr ord hex bin oct repr sleep exit isinstance').split(' '));
  const MODULES = new Set('sys fs net shell plugin math json time random'.split(' '));
  const OPCHARS = '+-*/%=<>!&|^~:';
  const NUM = /0[xX][0-9a-fA-F_]+|\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?/y;
  const PLAIN_EXT = { txt: 1, log: 1, conf: 1, ini: 1, csv: 1, cfg: 1, md: 1, json: 1 };

  const ESC = { '&': '&amp;', '<': '&lt;', '>': '&gt;' };
  function esc(s) { return s.replace(/[&<>]/g, function (c) { return ESC[c]; }); }

  function isIdentStart(c) { return /[A-Za-z_Ѐ-ӿ]/.test(c); }
  function isIdentChar(c) { return /[\wЀ-ӿ]/.test(c); }

  function extOf(path) {
    if (!path) return '';
    const name = path.slice(path.lastIndexOf('/') + 1);
    const i = name.lastIndexOf('.');
    return i < 0 ? '' : name.slice(i + 1).toLowerCase();
  }

  // Какой язык у файла: безымянный и .adr — Adder, иначе простой текст.
  function langFor(path) {
    const ext = extOf(path);
    if (!path || ext === 'adr') return 'adder';
    return 'plain';
  }

  // Строка со смещения `quote` (кавычка), начиная кусок с `start` (учитывает приставку f).
  // Возвращает смещение за концом строки.
  function scanString(src, quote, start, isF, out) {
    const q = src[quote];
    const triple = src.startsWith(q + q + q, quote);
    const qlen = triple ? 3 : 1;
    let j = quote + qlen;
    let segStart = start;
    const n = src.length;
    while (j < n) {
      const c = src[j];
      if (c === '\\') { j += 2; continue; }
      if (!triple && c === '\n') break;                       // незакрытая строка
      if (c === q && (!triple || src.startsWith(q + q + q, j))) { j += qlen; break; }
      if (isF && c === '{') {
        if (src[j + 1] === '{') { j += 2; continue; }        // экранированная скобка
        let depth = 0, k = j;
        while (k < n) {
          const d = src[k];
          if (d === '{') depth++;
          else if (d === '}') { depth--; if (depth === 0) { k++; break; } }
          else if (d === '\n' && !triple) break;
          k++;
        }
        if (j > segStart) out.push({ t: src.slice(segStart, j), c: 'tk-str' });
        out.push({ t: src.slice(j, k), c: 'tk-fexpr' });
        segStart = k;
        j = k;
        continue;
      }
      j++;
    }
    if (j > n) j = n;
    if (j > segStart) out.push({ t: src.slice(segStart, j), c: 'tk-str' });
    return j;
  }

  function tokenizeAdder(src) {
    const out = [];
    const n = src.length;
    let i = 0, plainStart = 0, prevWord = '';
    function flushPlain(to) { if (to > plainStart) out.push({ t: src.slice(plainStart, to), c: '' }); }

    while (i < n) {
      const c = src[i];
      if (c === '#') {
        flushPlain(i);
        let j = src.indexOf('\n', i);
        if (j < 0) j = n;
        out.push({ t: src.slice(i, j), c: 'tk-comment' });
        i = j; plainStart = i; prevWord = '';
        continue;
      }
      if (c === '"' || c === "'") {
        flushPlain(i);
        i = scanString(src, i, i, false, out);
        plainStart = i; prevWord = '';
        continue;
      }
      if ((c === 'f' || c === 'F') && (src[i + 1] === '"' || src[i + 1] === "'") && !(i > 0 && isIdentChar(src[i - 1]))) {
        flushPlain(i);
        i = scanString(src, i + 1, i, true, out);
        plainStart = i; prevWord = '';
        continue;
      }
      if (c >= '0' && c <= '9' && !(i > 0 && isIdentChar(src[i - 1]))) {
        NUM.lastIndex = i;
        const m = NUM.exec(src);
        if (m) {
          flushPlain(i);
          out.push({ t: m[0], c: 'tk-num' });
          i += m[0].length; plainStart = i; prevWord = '';
          continue;
        }
      }
      if (isIdentStart(c)) {
        let j = i + 1;
        while (j < n && isIdentChar(src[j])) j++;
        const w = src.slice(i, j);
        const afterDot = i > 0 && src[i - 1] === '.';
        let cls = '';
        if (!afterDot && KEYWORDS.has(w)) cls = CONSTS.has(w) ? 'tk-const' : 'tk-kw';
        else if (prevWord === 'def') cls = 'tk-def';
        else if (!afterDot && MODULES.has(w)) cls = 'tk-module';
        else if (!afterDot && BUILTINS.has(w)) cls = 'tk-builtin';
        else if (src[j] === '(') cls = 'tk-call';
        else if (afterDot) cls = 'tk-attr';
        if (cls) { flushPlain(i); out.push({ t: w, c: cls }); plainStart = j; }
        prevWord = w;
        i = j;
        continue;
      }
      if (OPCHARS.indexOf(c) >= 0) {
        let j = i + 1;
        while (j < n && OPCHARS.indexOf(src[j]) >= 0) j++;
        flushPlain(i);
        out.push({ t: src.slice(i, j), c: 'tk-op' });
        i = j; plainStart = i; prevWord = '';
        continue;
      }
      if ('()[]{}'.indexOf(c) >= 0) {
        flushPlain(i);
        out.push({ t: c, c: 'tk-br' });
        i++; plainStart = i; prevWord = '';
        continue;
      }
      if (c !== ' ' && c !== '\t') prevWord = '';
      i++;
    }
    flushPlain(n);
    return out;
  }

  // Простой текст: комментарии, секции [..], ключ=значение, числа, строки, слова ошибок.
  const VALUE_RE = /("[^"\n]*"|'[^'\n]*')|((?<![\w.])\d[\d_.:\-]*(?![\w]))|((?<![\wа-яё])(?:error|fail(?:ed|ure)?|fatal|denied|ошибк[а-яё]*|сбой|отказ[а-яё]*)(?![\wа-яё]))|((?<![\wа-яё])(?:warn(?:ing)?|предупрежден[а-яё]*)(?![\wа-яё]))/gi;

  function pushValue(s, out) {
    let last = 0, m;
    VALUE_RE.lastIndex = 0;
    while ((m = VALUE_RE.exec(s))) {
      if (m.index > last) out.push({ t: s.slice(last, m.index), c: '' });
      const cls = m[1] ? 'tk-str' : m[2] ? 'tk-num' : m[3] ? 'tk-err' : 'tk-warn';
      out.push({ t: m[0], c: cls });
      last = VALUE_RE.lastIndex;
      if (m[0].length === 0) VALUE_RE.lastIndex++;
    }
    if (last < s.length) out.push({ t: s.slice(last), c: '' });
  }

  function tokenizePlain(src, ext) {
    const out = [];
    const lines = src.split('\n');
    const kvAllowed = ext !== 'csv' && ext !== 'log' && ext !== 'md';
    for (let li = 0; li < lines.length; li++) {
      const L = lines[li];
      if (li > 0) out.push({ t: '\n', c: '' });
      if (L === '') continue;
      const trimmed = L.trimStart();
      if (trimmed.startsWith('#') || trimmed.startsWith(';') || trimmed.startsWith('//')) {
        out.push({ t: L, c: 'tk-comment' });
        continue;
      }
      if (/^\s*\[[^\]]*\]\s*$/.test(L)) { out.push({ t: L, c: 'tk-kw' }); continue; }
      if (ext === 'md' && /^\s*#{1,6}\s/.test(L)) { out.push({ t: L, c: 'tk-def' }); continue; }
      const kv = kvAllowed ? /^(\s*)([^=:#;\s][^=:]{0,60}?)(\s*[=:]\s*)(.*)$/.exec(L) : null;
      if (kv) {
        if (kv[1]) out.push({ t: kv[1], c: '' });
        out.push({ t: kv[2], c: 'tk-key' });
        out.push({ t: kv[3], c: 'tk-op' });
        pushValue(kv[4], out);
        continue;
      }
      pushValue(L, out);
    }
    return out;
  }

  // Наложить отметки поиска: ranges — [{start, end, cur}] по возрастанию, без пересечений.
  function applyMarks(segs, ranges) {
    if (!ranges || !ranges.length) return segs;
    const out = [];
    let pos = 0, ri = 0;
    for (const s of segs) {
      const segStart = pos, segEnd = pos + s.t.length;
      let cur = segStart;
      while (ri < ranges.length && ranges[ri].end <= cur) ri++;
      let k = ri;
      while (k < ranges.length && ranges[k].start < segEnd) {
        const r = ranges[k];
        const a = Math.max(r.start, cur), b = Math.min(r.end, segEnd);
        if (a > cur) out.push({ t: s.t.slice(cur - segStart, a - segStart), c: s.c });
        if (b > a) {
          const mc = r.cur ? 'tk-found tk-found-cur' : 'tk-found';
          out.push({ t: s.t.slice(a - segStart, b - segStart), c: s.c ? s.c + ' ' + mc : mc });
        }
        cur = b;
        if (r.end <= segEnd) k++; else break;
      }
      if (cur < segEnd) out.push({ t: s.t.slice(cur - segStart), c: s.c });
      pos = segEnd;
    }
    return out;
  }

  function render(segs) {
    let html = '';
    for (const s of segs) {
      if (!s.t) continue;
      html += s.c ? '<span class="' + s.c + '">' + esc(s.t) + '</span>' : esc(s.t);
    }
    return html;
  }

  // Главный вход: текст, путь файла (для выбора языка), отметки поиска → HTML.
  function highlight(text, path, marks) {
    const lang = langFor(path);
    let segs = lang === 'adder' ? tokenizeAdder(text) : tokenizePlain(text, extOf(path));
    segs = applyMarks(segs, marks);
    return render(segs);
  }

  return { highlight: highlight, langFor: langFor, extOf: extOf, esc: esc, PLAIN_EXT: PLAIN_EXT };
})();
