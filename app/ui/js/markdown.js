'use strict';
/* Мини-парсер Markdown: заголовки, абзацы, списки, таблицы, код, жирный/курсив, ссылки. HTML экранируется. */
var Markdown = (function () {

  function escapeHtml(s) {
    return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
  }

  function slug(text) {
    var s = String(text).toLowerCase().replace(/`/g, '')
      .replace(/[^a-zа-яё0-9\s-]/gi, '').trim().replace(/\s+/g, '-');
    return s || 'h';
  }

  /* Уникальные якоря для заголовков в пределах одного вызова */
  function makeIdGen() {
    var used = {};
    return function (text) {
      var base = slug(text), id = base, n = 2;
      while (used[id]) id = base + '-' + (n++);
      used[id] = true;
      return id;
    };
  }

  /* Строчная разметка: код защищается от остальных замен */
  function inline(s) {
    var codes = [];
    s = s.replace(/`([^`]+)`/g, function (m, c) {
      codes.push('<code>' + escapeHtml(c) + '</code>');
      return '\u0000' + (codes.length - 1) + '\u0000';
    });
    s = escapeHtml(s);
    s = s.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, function (m, t, u) {
      var safe = /^(https?:|home:|mailto:|#|\/)/i.test(u) ? u : '#';
      return '<a href="' + escapeHtml(safe) + '">' + t + '</a>';
    });
    s = s.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
    s = s.replace(/__([^_]+)__/g, '<strong>$1</strong>');
    s = s.replace(/(^|[^*\w])\*([^*\n]+)\*/g, '$1<em>$2</em>');
    s = s.replace(/(^|[^_\w])_([^_\n]+)_(?!\w)/g, '$1<em>$2</em>');
    s = s.replace(/~~([^~]+)~~/g, '<del>$1</del>');
    s = s.replace(/\u0000(\d+)\u0000/g, function (m, i) { return codes[+i]; });
    return s;
  }

  function isTableSep(line) {
    return /^\s*\|?\s*:?-{2,}:?\s*(\|\s*:?-{2,}:?\s*)*\|?\s*$/.test(line);
  }

  function splitRow(line) {
    var t = line.trim();
    if (t.charAt(0) === '|') t = t.slice(1);
    if (t.charAt(t.length - 1) === '|') t = t.slice(0, -1);
    return t.split('|').map(function (c) { return c.trim(); });
  }

  function render(text, opts) {
    opts = opts || {};
    var lines = String(text || '').replace(/\r\n?/g, '\n').split('\n');
    var out = [];
    var genId = makeIdGen();
    var para = [];
    var i = 0;

    function flushPara() {
      if (!para.length) return;
      out.push('<p>' + inline(para.join(' ')) + '</p>');
      para = [];
    }

    /* Список: собирает элементы с учётом вложенности по отступу */
    function parseList(startIndent) {
      var ordered = /^\s*\d+[.)]\s/.test(lines[i]);
      var html = ordered ? '<ol>' : '<ul>';
      while (i < lines.length) {
        var ln = lines[i];
        var m = /^(\s*)([-*+]|\d+[.)])\s+(.*)$/.exec(ln);
        if (!m) {
          /* продолжение пункта с отступом */
          if (/^\s{2,}\S/.test(ln) && html.slice(-5) === '</li>') { html = html.slice(0, -5) + ' ' + inline(ln.trim()) + '</li>'; i++; continue; }
          break;
        }
        var indent = m[1].length;
        if (indent < startIndent) break;
        if (indent > startIndent) {
          html = html.slice(0, -5) + parseList(indent) + '</li>';
          continue;
        }
        var itemOrdered = /\d/.test(m[2]);
        if (itemOrdered !== ordered) break;
        var body = m[3];
        var task = /^\[([ xX])\]\s+(.*)$/.exec(body);
        if (task) {
          body = '<span class="md-check' + (task[1] !== ' ' ? ' on' : '') + '"></span>' + inline(task[2]);
        } else body = inline(body);
        html += '<li>' + body + '</li>';
        i++;
      }
      return html + (ordered ? '</ol>' : '</ul>');
    }

    while (i < lines.length) {
      var line = lines[i];

      /* блок кода */
      var fence = /^\s*```\s*(\w+)?\s*$/.exec(line);
      if (fence) {
        flushPara();
        var lang = fence[1] || '';
        var buf = [];
        i++;
        while (i < lines.length && !/^\s*```\s*$/.test(lines[i])) { buf.push(lines[i]); i++; }
        i++;
        out.push('<pre class="md-code"' + (lang ? ' data-lang="' + escapeHtml(lang) + '"' : '') + '><code>' + escapeHtml(buf.join('\n')) + '</code></pre>');
        continue;
      }

      if (!line.trim()) { flushPara(); i++; continue; }

      /* заголовок */
      var h = /^(#{1,6})\s+(.*?)\s*#*\s*$/.exec(line);
      if (h) {
        flushPara();
        var level = h[1].length;
        var id = genId(h[2]);
        out.push('<h' + level + ' id="' + id + '">' + inline(h[2]) + '</h' + level + '>');
        i++;
        continue;
      }

      /* горизонтальная линия */
      if (/^\s*(-{3,}|\*{3,}|_{3,})\s*$/.test(line)) { flushPara(); out.push('<hr>'); i++; continue; }

      /* цитата */
      if (/^\s*>/.test(line)) {
        flushPara();
        var q = [];
        while (i < lines.length && /^\s*>/.test(lines[i])) { q.push(lines[i].replace(/^\s*>\s?/, '')); i++; }
        out.push('<blockquote>' + render(q.join('\n'), { nested: true }) + '</blockquote>');
        continue;
      }

      /* таблица */
      if (line.indexOf('|') >= 0 && i + 1 < lines.length && isTableSep(lines[i + 1])) {
        flushPara();
        var head = splitRow(line);
        var aligns = splitRow(lines[i + 1]).map(function (c) {
          var l = c.charAt(0) === ':', r = c.charAt(c.length - 1) === ':';
          return l && r ? 'center' : r ? 'right' : '';
        });
        i += 2;
        var t = '<table class="md-table"><thead><tr>';
        head.forEach(function (c, k) { t += '<th' + (aligns[k] ? ' style="text-align:' + aligns[k] + '"' : '') + '>' + inline(c) + '</th>'; });
        t += '</tr></thead><tbody>';
        while (i < lines.length && lines[i].trim() && lines[i].indexOf('|') >= 0) {
          var cells = splitRow(lines[i]);
          t += '<tr>';
          for (var c = 0; c < head.length; c++) {
            t += '<td' + (aligns[c] ? ' style="text-align:' + aligns[c] + '"' : '') + '>' + inline(cells[c] || '') + '</td>';
          }
          t += '</tr>';
          i++;
        }
        out.push(t + '</tbody></table>');
        continue;
      }

      /* список */
      if (/^\s*([-*+]|\d+[.)])\s+/.test(line)) {
        flushPara();
        out.push(parseList(/^(\s*)/.exec(line)[1].length));
        continue;
      }

      para.push(line.trim());
      i++;
    }
    flushPara();
    return out.join('\n');
  }

  /* Оглавление: [{level, text, id}] — те же якоря, что ставит render */
  function headings(text) {
    var res = [];
    var genId = makeIdGen();
    var inCode = false;
    String(text || '').replace(/\r\n?/g, '\n').split('\n').forEach(function (line) {
      if (/^\s*```/.test(line)) { inCode = !inCode; return; }
      if (inCode) return;
      var h = /^(#{1,6})\s+(.*?)\s*#*\s*$/.exec(line);
      if (h) res.push({ level: h[1].length, text: h[2].replace(/[`*_]/g, ''), id: genId(h[2]) });
    });
    return res;
  }

  return { render: render, headings: headings, slug: slug, escape: escapeHtml, inline: inline };
})();
