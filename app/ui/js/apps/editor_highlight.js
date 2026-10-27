'use strict';
// Только разметка: исходный текст никогда не изменяется и не исполняется.
const EditorHighlight = (function () {
  const KEYWORDS = new Set('and or not if elif else while for in def return lambda try except finally raise pass break continue global as'.split(' '));
  const CONSTS = new Set(['true', 'false', 'null']);
  const BUILTINS = new Set('print input len str int float bool list dict range type abs min max sum round sorted reversed enumerate zip any all chr ord hex bin oct repr sleep exit'.split(' '));
  const MODULES = new Set('sys fs net shell plugin math json time random'.split(' '));
  const NUM = /0[xX][0-9a-fA-F_]+|\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?/y;
  const PLAIN_EXT = { txt:1, log:1, conf:1, ini:1, csv:1, cfg:1, md:1, json:1 };
  const escape = { '&':'&amp;', '<':'&lt;', '>':'&gt;' };
  function esc(s) { return String(s).replace(/[&<>]/g, function (c) { return escape[c]; }); }
  function identStart(c) { return !!c && /^[\p{L}_]$/u.test(c); }
  function identChar(c) { return !!c && /^[\p{L}\p{N}_]$/u.test(c); }
  function charAt(s, i) { return i < s.length ? String.fromCodePoint(s.codePointAt(i)) : ''; }
  function extOf(path) {
    const name = String(path || '').split(/[\\/]/).pop();
    const i = name.lastIndexOf('.');
    return i < 0 ? '' : name.slice(i + 1).toLowerCase();
  }
  function langFor(path) {
    const ext = extOf(path);
    return !path || ext === 'adr' ? 'adder' : ext === 'json' ? 'json' : 'plain';
  }
  function stringEnd(src, quote) {
    const q = src[quote], triple = src.startsWith(q + q + q, quote), size = triple ? 3 : 1;
    let i = quote + size;
    while (i < src.length) {
      if (src[i] === '\\') { i = Math.min(i + 2, src.length); continue; }
      if (!triple && src[i] === '\n') break;
      if (src.startsWith(q.repeat(size), i)) return i + size;
      i++;
    }
    return i;
  }
  function fExpressionEnd(src, start, triple) {
    let depth = 1, i = start + 1;
    while (i < src.length) {
      const c = src[i];
      if (!triple && c === '\n') break;
      if (c === '"' || c === "'") { i = stringEnd(src, i); continue; }
      if (c === '{') depth++;
      else if (c === '}') { depth--; if (depth === 0) return i + 1; }
      i++;
    }
    return i;
  }
  function stringTokens(src, start, quote, f, out, depth) {
    const q = src[quote], triple = src.startsWith(q + q + q, quote), size = triple ? 3 : 1;
    let i = quote + size, literal = start;
    while (i < src.length) {
      const c = src[i];
      if (c === '\\') { i = Math.min(i + 2, src.length); continue; }
      if (!triple && c === '\n') break;
      if (src.startsWith(q.repeat(size), i)) { i += size; break; }
      if (f && c === '{') {
        if (src[i + 1] === '{') { i += 2; continue; }
        const end = fExpressionEnd(src, i, triple);
        if (i > literal) out.push({t:src.slice(literal, i), c:'tk-str'});
        const closed = end > i && src[end - 1] === '}';
        out.push({t:'{', c:'tk-fexpr'});
        const expression = src.slice(i + 1, closed ? end - 1 : end);
        if (depth < 24) out.push.apply(out, tokenizeAdder(expression, depth + 1));
        else out.push({t:expression, c:'tk-fexpr'});
        if (closed) out.push({t:'}', c:'tk-fexpr'});
        i = end; literal = end;
        continue;
      }
      i++;
    }
    if (i > literal) out.push({t:src.slice(literal, i), c:'tk-str'});
    return i;
  }
  function tokenizeAdder(src, depth) {
    depth = depth || 0;
    const out = [];
    let i = 0, previous = '', previousSymbol = '';
    function push(t, c) { out.push({t:t, c:c || ''}); }
    while (i < src.length) {
      const c = charAt(src, i);
      if (/\s/.test(c)) {
        let end = i + c.length;
        while (end < src.length && /\s/.test(src[end])) end++;
        const space = src.slice(i, end); push(space);
        if (space.indexOf('\n') >= 0) { previous = ''; previousSymbol = ''; }
        i = end; continue;
      }
      if (c === '#') {
        let end = src.indexOf('\n', i); if (end < 0) end = src.length;
        push(src.slice(i, end), 'tk-comment'); i = end; previous = ''; continue;
      }
      if (c === '"' || c === "'" || ((c === 'f' || c === 'F') && (src[i+1] === '"' || src[i+1] === "'"))) {
        const f = c === 'f' || c === 'F';
        i = stringTokens(src, i, i + (f ? 1 : 0), f, out, depth); previous = ''; previousSymbol = 'string'; continue;
      }
      if (/\d/.test(c)) {
        NUM.lastIndex = i;
        const number = NUM.exec(src);
        if (number) { push(number[0], 'tk-num'); i += number[0].length; previous = ''; previousSymbol = 'number'; continue; }
      }
      if (identStart(c)) {
        let end = i + c.length;
        while (end < src.length && identChar(charAt(src, end))) end += charAt(src, end).length;
        const word = src.slice(i, end), afterDot = previousSymbol === '.';
        let next = end; while (next < src.length && /[ \t]/.test(src[next])) next++;
        let cls = '';
        if (previous === 'def') cls = 'tk-def';
        else if (!afterDot && CONSTS.has(word)) cls = 'tk-const';
        else if (!afterDot && KEYWORDS.has(word)) cls = 'tk-kw';
        else if (!afterDot && MODULES.has(word)) cls = 'tk-module';
        else if (!afterDot && BUILTINS.has(word)) cls = 'tk-builtin';
        else if (src[next] === '(') cls = 'tk-call';
        else if (afterDot) cls = 'tk-attr';
        push(word, cls); previous = word; previousSymbol = 'name'; i = end; continue;
      }
      push(c, '()[]{}'.indexOf(c) >= 0 ? 'tk-br' : '+-*/%=<>!&|^~:.,'.indexOf(c) >= 0 ? 'tk-op' : '');
      previous = ''; previousSymbol = c; i += c.length;
    }
    return out;
  }
  const JSON_VALUE = /-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?|true\b|false\b|null\b/y;
  function tokenizeJSON(src) {
    const out = []; let i = 0;
    while (i < src.length) {
      const c = src[i];
      if (c === '"') {
        const end = stringEnd(src, i); let next = end;
        while (next < src.length && /\s/.test(src[next])) next++;
        out.push({t:src.slice(i, end), c:src[next] === ':' ? 'tk-key' : 'tk-str'}); i = end; continue;
      }
      JSON_VALUE.lastIndex = i;
      const match = JSON_VALUE.exec(src);
      if (match) { out.push({t:match[0], c:CONSTS.has(match[0]) ? 'tk-const' : 'tk-num'}); i += match[0].length; continue; }
      out.push({t:c, c:'{}[]'.indexOf(c) >= 0 ? 'tk-br' : ':,'.indexOf(c) >= 0 ? 'tk-op' : ''}); i++;
    }
    return out;
  }
  const VALUE_RE = /("(?:\\.|[^"\\\n])*"|'(?:\\.|[^'\\\n])*')|((?<![\w.])\d[\d_.:\-]*(?![\w]))|((?<![\wа-яё])(?:error|fail(?:ed|ure)?|fatal|denied|ошибк[а-яё]*|сбой|отказ[а-яё]*)(?![\wа-яё]))|((?<![\wа-яё])(?:warn(?:ing)?|предупрежден[а-яё]*)(?![\wа-яё]))/gi;
  function values(text, out) {
    let last = 0, m; VALUE_RE.lastIndex = 0;
    while ((m = VALUE_RE.exec(text))) {
      if (m.index > last) out.push({t:text.slice(last, m.index), c:''});
      out.push({t:m[0], c:m[1] ? 'tk-str' : m[2] ? 'tk-num' : m[3] ? 'tk-err' : 'tk-warn'}); last = VALUE_RE.lastIndex;
    }
    if (last < text.length) out.push({t:text.slice(last), c:''});
  }
  function tokenizePlain(src, ext) {
    const out = [];
    src.split('\n').forEach(function (line, index) {
      if (index) out.push({t:'\n', c:''});
      if (/^\s*(?:#|;|\/\/)/.test(line)) { out.push({t:line,c:'tk-comment'}); return; }
      if (ext === 'md' && /^\s*#{1,6}\s/.test(line)) { out.push({t:line,c:'tk-def'}); return; }
      if (/^\s*\[[^\]]*\]\s*$/.test(line)) { out.push({t:line,c:'tk-kw'}); return; }
      const kv = !['csv','log','md'].includes(ext) && /^(\s*)([^=:#;\s][^=:]{0,60}?)(\s*[=:]\s*)(.*)$/.exec(line);
      if (kv) { out.push({t:kv[1],c:''},{t:kv[2],c:'tk-key'},{t:kv[3],c:'tk-op'}); values(kv[4],out); }
      else values(line,out);
    });
    return out;
  }
  function applyMarks(segs, ranges) {
    if (!ranges || !ranges.length) return segs;
    const out = []; let pos = 0, ri = 0;
    for (const s of segs) {
      const start = pos, end = pos + s.t.length; let cur = start;
      while (ri < ranges.length && ranges[ri].end <= cur) ri++;
      let k = ri;
      while (k < ranges.length && ranges[k].start < end) {
        const r = ranges[k], a = Math.max(r.start, cur), b = Math.min(r.end, end);
        if (a > cur) out.push({t:s.t.slice(cur-start,a-start),c:s.c});
        if (b > a) out.push({t:s.t.slice(a-start,b-start),c:(s.c ? s.c+' ' : '')+'tk-found'+(r.cur ? ' tk-found-cur' : '')});
        cur = b; if (r.end <= end) k++; else break;
      }
      if (cur < end) out.push({t:s.t.slice(cur-start),c:s.c}); pos = end;
    }
    return out;
  }
  function highlight(text, path, marks, enabled) {
    const lang = langFor(path);
    const segs = enabled === false ? [{t:text,c:''}] : lang === 'adder' ? tokenizeAdder(text) : lang === 'json' ? tokenizeJSON(text) : tokenizePlain(text,extOf(path));
    return applyMarks(segs,marks).map(function (s) { return s.c ? '<span class="'+s.c+'">'+esc(s.t)+'</span>' : esc(s.t); }).join('');
  }
  return {highlight:highlight,langFor:langFor,extOf:extOf,esc:esc,PLAIN_EXT:PLAIN_EXT};
})();
