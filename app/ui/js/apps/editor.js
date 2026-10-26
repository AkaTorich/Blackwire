'use strict';
// Редактор — вкладки файлов, номера строк, textarea поверх подсвеченного pre.
// Подсветка — в editor_highlight.js (EditorHighlight). Файл: js/apps/editor.js, стили: css/apps/editor.css.

const EditorApp = (function () {
  const CHECK_DELAY = 400;   // мс до проверки lang_check после правки
  const INDENT = '    ';

  function el(tag, cls, text) {
    const e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text != null) e.textContent = text;
    return e;
  }

  function iconBtn(icon, title, cls) {
    const b = el('button', 'ed-btn' + (cls ? ' ' + cls : ''));
    b.type = 'button';
    b.title = title;
    b.innerHTML = Icons.get(icon, 16);
    return b;
  }

  function baseName(p) {
    if (!p) return '';
    const i = p.lastIndexOf('/');
    return i < 0 ? p : p.slice(i + 1);
  }

  function lineColOf(text, pos) {
    let line = 1, last = -1;
    for (let i = 0; i < pos; i++) {
      if (text.charCodeAt(i) === 10) { line++; last = i; }
    }
    return { line: line, col: pos - last };
  }

  function lineStartOffset(text, lineNo) {
    let pos = 0;
    for (let l = 1; l < lineNo; l++) {
      const i = text.indexOf('\n', pos);
      if (i < 0) return text.length;
      pos = i + 1;
    }
    return pos;
  }

  function mount(body, arg) {
    const st = {
      tabs: [], active: -1, nextId: 1, untitled: 0,
      checkTimer: null, checkSeq: 0,
      search: { open: false, query: '', ranges: [], cur: -1 },
      lineHeight: 20
    };

    // --- разметка ---
    const root = el('div', 'ed');
    const tabbar = el('div', 'ed-tabbar');
    const tabsList = el('div', 'ed-tabs');
    const tools = el('div', 'ed-tools');
    const btnNew = iconBtn('plus', 'Новый файл — Ctrl+N');
    const btnOpen = iconBtn('folder-open', 'Открыть — Ctrl+O');
    const btnSave = iconBtn('save', 'Сохранить — Ctrl+S');
    const btnFind = iconBtn('search', 'Поиск — Ctrl+F');
    const btnRun = iconBtn('play', 'Запустить — F5', 'ed-btn-run');
    tools.append(btnNew, btnOpen, btnSave, btnFind, btnRun);
    tabbar.append(tabsList, tools);

    const searchBar = el('div', 'ed-search');
    const searchInput = el('input', 'ed-search-input');
    searchInput.type = 'text';
    searchInput.placeholder = 'Найти';
    searchInput.spellcheck = false;
    const searchCount = el('span', 'ed-search-count', '');
    const btnPrev = iconBtn('up', 'Назад — Shift+F3');
    const btnNext = iconBtn('down', 'Далее — F3');
    const btnCloseSearch = iconBtn('close', 'Закрыть — Esc');
    searchBar.append(searchInput, searchCount, btnPrev, btnNext, btnCloseSearch);

    const area = el('div', 'ed-area');
    const gutter = el('div', 'ed-gutter');
    const stack = el('div', 'ed-stack');
    const pre = el('pre', 'ed-pre');
    const ta = el('textarea', 'ed-ta');
    ta.spellcheck = false;
    ta.wrap = 'off';
    ta.setAttribute('autocomplete', 'off');
    ta.setAttribute('autocapitalize', 'off');
    ta.setAttribute('autocorrect', 'off');
    stack.append(pre, ta);
    area.append(gutter, stack);

    const status = el('div', 'ed-status');
    const stPath = el('span', 'ed-st-path', '');
    const stCheck = el('span', 'ed-st-check', '');
    const stMod = el('span', 'ed-st-mod', '');
    const stPos = el('span', 'ed-st-pos', '');
    const stLang = el('span', 'ed-st-lang', '');
    status.append(stPath, stCheck, stMod, stPos, stLang);

    const empty = el('div', 'ed-empty', 'Нет открытых файлов');
    root.append(tabbar, searchBar, area, empty, status);
    body.appendChild(root);

    // --- вкладки ---
    function cur() { return st.active >= 0 ? st.tabs[st.active] : null; }

    function makeTab(path, text) {
      st.untitled += path ? 0 : 1;
      return {
        id: st.nextId++, path: path || '', name: path ? baseName(path) : 'новый' + (st.untitled > 1 ? st.untitled : '') + '.adr',
        text: text || '', modified: false, diags: [], scrollTop: 0, scrollLeft: 0, selStart: 0, selEnd: 0
      };
    }

    function renderTabs() {
      tabsList.textContent = '';
      st.tabs.forEach(function (t, i) {
        const tab = el('div', 'ed-tab' + (i === st.active ? ' active' : '') + (t.modified ? ' modified' : ''));
        tab.title = t.path || t.name;
        const ic = el('span', 'ed-tab-icon');
        ic.innerHTML = Icons.get(EditorHighlight.langFor(t.path || t.name) === 'adder' ? 'file-code' : 'file-text', 14);
        const nm = el('span', 'ed-tab-name', t.name);
        const dot = el('span', 'ed-tab-dot');
        const x = el('button', 'ed-tab-close');
        x.type = 'button';
        x.title = 'Закрыть';
        x.innerHTML = Icons.get('close', 12);
        x.addEventListener('click', function (e) { e.stopPropagation(); closeTab(i); });
        tab.append(ic, nm, dot, x);
        tab.addEventListener('mousedown', function (e) { if (e.button === 1) { e.preventDefault(); closeTab(i); } });
        tab.addEventListener('click', function () { activate(i); });
        tabsList.appendChild(tab);
      });
      const t = cur();
      root.classList.toggle('no-tabs', !t);
      if (t) {
        Win.setTitle(t.name);
        Win.setModified(!!t.modified);
      } else {
        Win.setTitle('Редактор');
        Win.setModified(false);
      }
    }

    function saveState(t) {
      if (!t) return;
      t.text = ta.value;
      t.scrollTop = ta.scrollTop;
      t.scrollLeft = ta.scrollLeft;
      t.selStart = ta.selectionStart;
      t.selEnd = ta.selectionEnd;
    }

    function activate(i) {
      if (i === st.active) { ta.focus(); return; }
      saveState(cur());
      st.active = i;
      const t = cur();
      ta.value = t ? t.text : '';
      st.search.ranges = []; st.search.cur = -1;
      renderTabs();
      refresh();
      if (t) {
        ta.setSelectionRange(t.selStart, t.selEnd);
        ta.scrollTop = t.scrollTop;
        ta.scrollLeft = t.scrollLeft;
        syncScroll();
        if (st.search.open) runSearch(true); else { renderGutter(); renderStatus(); }
        scheduleCheck(0);
      }
      ta.focus();
    }

    function addTab(path, text) {
      const t = makeTab(path, text);
      st.tabs.push(t);
      activate(st.tabs.length - 1);
      return t;
    }

    async function openFile(path) {
      const idx = st.tabs.findIndex(function (t) { return t.path === path; });
      if (idx >= 0) { activate(idx); return; }
      try {
        const text = await BW.fs_read(null, path);
        addTab(path, String(text == null ? '' : text));
      } catch (e) {
        // Нет файла — новая вкладка с этим путём: сохранение создаст файл.
        const t = addTab(path, '');
        t.modified = false;
        Win.toast(String(e));
      }
    }

    async function closeTab(i) {
      const t = st.tabs[i];
      if (!t) return;
      if (i === st.active) saveState(t);
      if (t.modified) {
        const ok = await Win.confirm('Закрыть «' + t.name + '» без сохранения?');
        if (!ok) return;
      }
      st.tabs.splice(i, 1);
      if (st.tabs.length === 0) {
        st.active = -1;
        ta.value = '';
        renderTabs();
        refresh();
        return;
      }
      let ni = st.active;
      if (i < st.active) ni = st.active - 1;
      else if (i === st.active) ni = Math.min(i, st.tabs.length - 1);
      st.active = -2;  // чтобы activate перерисовал
      activate(ni);
    }

    // --- отрисовка ---
    function refresh() {
      const t = cur();
      const text = t ? ta.value : '';
      const marks = st.search.ranges.map(function (r, i) { return { start: r.start, end: r.end, cur: i === st.search.cur }; });
      pre.innerHTML = EditorHighlight.highlight(text, t ? (t.path || t.name) : '', marks) + '\n\n ';
      renderGutter();
      renderStatus();
    }

    function renderGutter() {
      const t = cur();
      const text = ta.value;
      let count = 1;
      for (let i = 0; i < text.length; i++) if (text.charCodeAt(i) === 10) count++;
      const curLine = lineColOf(text, ta.selectionStart).line;
      const errLines = {};
      if (t) for (const d of t.diags) { if (!errLines[d.line]) errLines[d.line] = d.message; }
      gutter.style.width = (Math.max(2, String(count).length) + 1.2) + 'ch';
      const frag = document.createDocumentFragment();
      for (let n = 1; n <= count; n++) {
        const ln = el('div', 'ed-ln' + (n === curLine ? ' cur' : '') + (errLines[n] ? ' err' : ''), String(n));
        if (errLines[n]) ln.title = errLines[n];
        frag.appendChild(ln);
      }
      const spacer = el('div', 'ed-ln ed-ln-spacer', ' ');
      frag.appendChild(spacer);
      gutter.textContent = '';
      gutter.appendChild(frag);
      gutter.scrollTop = ta.scrollTop;
    }

    function renderStatus() {
      const t = cur();
      if (!t) {
        stPath.textContent = ''; stPos.textContent = ''; stMod.textContent = ''; stCheck.textContent = ''; stLang.textContent = '';
        stCheck.className = 'ed-st-check';
        return;
      }
      const lc = lineColOf(ta.value, ta.selectionStart);
      stPath.textContent = t.path || t.name;
      stPath.title = t.path || 'Файл ещё не сохранён';
      stPos.textContent = lc.line + ':' + lc.col;
      stMod.textContent = t.modified ? 'изменено' : '';
      const lang = EditorHighlight.langFor(t.path || t.name);
      stLang.textContent = lang === 'adder' ? 'Adder' : (EditorHighlight.extOf(t.path || t.name).toUpperCase() || 'Текст');
      if (lang !== 'adder') {
        stCheck.textContent = '';
        stCheck.className = 'ed-st-check';
      } else if (!t.diags.length) {
        stCheck.textContent = 'Ошибок нет';
        stCheck.className = 'ed-st-check ok';
      } else {
        const d = t.diags[0];
        stCheck.textContent = 'Строка ' + d.line + ':' + d.col + ' — ' + d.message + (t.diags.length > 1 ? '  (+' + (t.diags.length - 1) + ')' : '');
        stCheck.title = t.diags.map(function (x) { return x.line + ':' + x.col + ' ' + x.message; }).join('\n');
        stCheck.className = 'ed-st-check bad';
      }
    }

    function syncScroll() {
      pre.scrollTop = ta.scrollTop;
      pre.scrollLeft = ta.scrollLeft;
      gutter.scrollTop = ta.scrollTop;
    }

    function measureLineHeight() {
      const lh = parseFloat(getComputedStyle(ta).lineHeight);
      if (lh > 0) st.lineHeight = lh;
    }

    // --- проверка ---
    function scheduleCheck(delay) {
      if (st.checkTimer) clearTimeout(st.checkTimer);
      const t = cur();
      if (!t) return;
      if (EditorHighlight.langFor(t.path || t.name) !== 'adder') { t.diags = []; renderGutter(); renderStatus(); return; }
      const seq = ++st.checkSeq;
      st.checkTimer = setTimeout(function () {
        st.checkTimer = null;
        const src = ta.value;
        BW.lang_check(src).then(function (diags) {
          if (seq !== st.checkSeq || cur() !== t) return;
          t.diags = Array.isArray(diags) ? diags.slice().sort(function (a, b) { return a.line - b.line || a.col - b.col; }) : [];
          renderGutter();
          renderStatus();
        }, function () { /* проверка недоступна — молча */ });
      }, delay == null ? CHECK_DELAY : delay);
    }

    // --- правка текста с сохранением отмены ---
    function replaceRange(start, end, text) {
      ta.setSelectionRange(start, end);
      let ok = false;
      try { ok = document.execCommand('insertText', false, text); } catch (e) { ok = false; }
      if (!ok) {
        ta.setRangeText(text, start, end, 'end');
        onInput();
      }
    }

    function onInput() {
      const t = cur();
      if (!t) return;
      t.text = ta.value;
      if (!t.modified) { t.modified = true; renderTabs(); }
      if (st.search.open && st.search.query) runSearch(true); else refresh();
      scheduleCheck();
    }

    function currentLineInfo() {
      const v = ta.value;
      const pos = ta.selectionStart;
      const ls = v.lastIndexOf('\n', pos - 1) + 1;
      let le = v.indexOf('\n', pos);
      if (le < 0) le = v.length;
      return { start: ls, end: le, text: v.slice(ls, le) };
    }

    function autoIndent() {
      const li = currentLineInfo();
      const head = li.text.slice(0, ta.selectionStart - li.start);
      const indent = (/^[ \t]*/.exec(head) || [''])[0];
      const code = head.replace(/#.*$/, '').trimEnd();
      let add = code.endsWith(':') ? INDENT : '';
      const trimmed = code.trim();
      let ind = indent;
      // после return/break/continue/pass/raise — отступ на шаг назад
      if (/^(return|break|continue|pass|raise)\b/.test(trimmed) && !add) ind = indent.slice(0, Math.max(0, indent.length - INDENT.length));
      replaceRange(ta.selectionStart, ta.selectionEnd, '\n' + ind + add);
    }

    function shiftLines(outdent) {
      const v = ta.value;
      const s0 = ta.selectionStart, e0 = ta.selectionEnd;
      const ls = v.lastIndexOf('\n', s0 - 1) + 1;
      let le = v.indexOf('\n', Math.max(e0 - (e0 > s0 ? 1 : 0), s0));
      if (le < 0) le = v.length;
      const block = v.slice(ls, le);
      const lines = block.split('\n');
      let removedFirst = 0, delta = 0;
      const out = lines.map(function (L, i) {
        if (outdent) {
          const m = /^( {1,4}|\t)/.exec(L);
          const n = m ? m[0].length : 0;
          if (i === 0) removedFirst = n;
          delta -= n;
          return L.slice(n);
        }
        delta += INDENT.length;
        return INDENT + L;
      });
      const text = out.join('\n');
      if (text === block) return;
      replaceRange(ls, le, text);
      if (s0 === e0) {
        const np = outdent ? Math.max(ls, s0 - removedFirst) : s0 + INDENT.length;
        ta.setSelectionRange(np, np);
      } else {
        ta.setSelectionRange(ls, le + delta);
      }
    }

    // --- файлы ---
    async function save(tabIndex) {
      const t = tabIndex == null ? cur() : st.tabs[tabIndex];
      if (!t) return false;
      if (t === cur()) saveState(t);
      let path = t.path;
      if (!path) {
        path = await Win.prompt('Путь для сохранения', '/home/' + t.name);
        if (!path) return false;
        path = path.trim();
        if (!path) return false;
      }
      try {
        await BW.fs_write(null, path, t.text);
      } catch (e) {
        Win.toast('Не сохранено: ' + e);
        BW.sound('error');
        return false;
      }
      t.path = path;
      t.name = baseName(path);
      t.modified = false;
      renderTabs();
      refresh();
      scheduleCheck(0);
      Win.toast('Сохранено: ' + t.name);
      return true;
    }

    async function run() {
      const t = cur();
      if (!t) return;
      if (!(await save())) return;
      try { await BW.open_app('terminal', 'adder ' + t.path); }
      catch (e) { Win.toast(String(e)); }
    }

    async function openByPrompt() {
      const t = cur();
      const start = t && t.path ? t.path.slice(0, t.path.lastIndexOf('/') + 1) : '/home/';
      const p = await Win.prompt('Открыть файл', start);
      if (p && p.trim()) openFile(p.trim());
    }

    // --- поиск ---
    function openSearch() {
      st.search.open = true;
      root.classList.add('search-on');
      const sel = ta.value.slice(ta.selectionStart, ta.selectionEnd);
      if (sel && sel.indexOf('\n') < 0) searchInput.value = sel;
      searchInput.focus();
      searchInput.select();
      runSearch(true);
    }

    function closeSearch() {
      st.search.open = false;
      st.search.ranges = []; st.search.cur = -1;
      root.classList.remove('search-on');
      refresh();
      ta.focus();
    }

    // Пересчитать совпадения; keepPos — выбрать первое после курсора, не прыгая.
    function runSearch(keepPos) {
      const q = searchInput.value;
      st.search.query = q;
      st.search.ranges = [];
      st.search.cur = -1;
      if (q) {
        const hay = ta.value.toLowerCase();
        const needle = q.toLowerCase();
        let i = 0;
        while ((i = hay.indexOf(needle, i)) >= 0 && st.search.ranges.length < 5000) {
          st.search.ranges.push({ start: i, end: i + needle.length });
          i += needle.length;
        }
        if (st.search.ranges.length) {
          const from = ta.selectionStart;
          let k = st.search.ranges.findIndex(function (r) { return r.start >= from; });
          if (k < 0) k = 0;
          st.search.cur = k;
          if (!keepPos) revealMatch();
        }
      }
      searchCount.textContent = q ? (st.search.ranges.length ? (st.search.cur + 1) + ' / ' + st.search.ranges.length : 'нет') : '';
      searchBar.classList.toggle('none', !!q && !st.search.ranges.length);
      refresh();
    }

    function revealMatch() {
      const r = st.search.ranges[st.search.cur];
      if (!r) return;
      ta.setSelectionRange(r.start, r.end);
      scrollToPos(r.start);
    }

    function findNext(dir) {
      if (!st.search.ranges.length) { runSearch(false); return; }
      const n = st.search.ranges.length;
      st.search.cur = (st.search.cur + dir + n) % n;
      searchCount.textContent = (st.search.cur + 1) + ' / ' + n;
      revealMatch();
      refresh();
    }

    function scrollToPos(pos) {
      measureLineHeight();
      const line = lineColOf(ta.value, pos).line;
      const y = (line - 1) * st.lineHeight;
      if (y < ta.scrollTop + st.lineHeight || y > ta.scrollTop + ta.clientHeight - st.lineHeight * 2) {
        ta.scrollTop = Math.max(0, y - ta.clientHeight / 2);
      }
      syncScroll();
    }

    async function gotoLine() {
      const v = await Win.prompt('Перейти к строке', '');
      const n = parseInt(v, 10);
      if (!n || n < 1) return;
      const pos = lineStartOffset(ta.value, n);
      ta.focus();
      ta.setSelectionRange(pos, pos);
      scrollToPos(pos);
      refresh();
    }

    // --- клавиши ---
    function onTaKey(e) {
      if (e.ctrlKey && !e.altKey) {
        switch (e.code) {
          case 'KeyS': e.preventDefault(); save(); return;
          case 'KeyF': e.preventDefault(); openSearch(); return;
          case 'KeyG': e.preventDefault(); gotoLine(); return;
          case 'KeyN': if (!e.shiftKey) { e.preventDefault(); addTab('', ''); } return;
          case 'KeyO': if (!e.shiftKey) { e.preventDefault(); openByPrompt(); } return;
          case 'Tab':
            e.preventDefault();
            if (st.tabs.length > 1) activate((st.active + (e.shiftKey ? st.tabs.length - 1 : 1)) % st.tabs.length);
            return;
          case 'PageDown': e.preventDefault(); if (st.tabs.length > 1) activate((st.active + 1) % st.tabs.length); return;
          case 'PageUp': e.preventDefault(); if (st.tabs.length > 1) activate((st.active + st.tabs.length - 1) % st.tabs.length); return;
          case 'KeyD': {
            // дублировать строку
            e.preventDefault();
            const li = currentLineInfo();
            const p = ta.selectionStart;
            replaceRange(li.end, li.end, '\n' + li.text);
            ta.setSelectionRange(p + li.text.length + 1, p + li.text.length + 1);
            return;
          }
          case 'Slash': {
            // закомментировать / раскомментировать строки
            e.preventDefault();
            toggleComment();
            return;
          }
          default: return;
        }
      }
      switch (e.key) {
        case 'F5': e.preventDefault(); run(); return;
        case 'F3': e.preventDefault(); if (!st.search.open) openSearch(); else findNext(e.shiftKey ? -1 : 1); return;
        case 'Escape': if (st.search.open) { e.preventDefault(); closeSearch(); } return;
        case 'Enter': if (!e.shiftKey) { e.preventDefault(); autoIndent(); } return;
        case 'Tab': {
          e.preventDefault();
          const multi = ta.value.slice(ta.selectionStart, ta.selectionEnd).indexOf('\n') >= 0;
          if (e.shiftKey || multi) shiftLines(e.shiftKey);
          else replaceRange(ta.selectionStart, ta.selectionEnd, INDENT);
          return;
        }
        case 'Backspace': {
          // в начале строки убрать весь шаг отступа
          if (ta.selectionStart !== ta.selectionEnd) return;
          const li = currentLineInfo();
          const head = li.text.slice(0, ta.selectionStart - li.start);
          if (head.length && /^ +$/.test(head) && head.length % INDENT.length === 0) {
            e.preventDefault();
            replaceRange(ta.selectionStart - INDENT.length, ta.selectionStart, '');
          }
          return;
        }
        default: return;
      }
    }

    function toggleComment() {
      const v = ta.value;
      const s0 = ta.selectionStart, e0 = ta.selectionEnd;
      const ls = v.lastIndexOf('\n', s0 - 1) + 1;
      let le = v.indexOf('\n', Math.max(e0 - (e0 > s0 ? 1 : 0), s0));
      if (le < 0) le = v.length;
      const lines = v.slice(ls, le).split('\n');
      const allCommented = lines.every(function (L) { return L.trim() === '' || L.trimStart().startsWith('#'); });
      const out = lines.map(function (L) {
        if (L.trim() === '') return L;
        if (allCommented) return L.replace(/^(\s*)# ?/, '$1');
        const m = /^\s*/.exec(L);
        return m[0] + '# ' + L.slice(m[0].length);
      });
      replaceRange(ls, le, out.join('\n'));
      ta.setSelectionRange(ls, ls + out.join('\n').length);
    }

    function onWindowKey(e) {
      // Клавиши, которые должны работать и когда фокус не в поле текста
      const t = e.target;
      if (t === ta || t === searchInput) return;
      if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA')) return;
      if (e.ctrlKey && !e.shiftKey && !e.altKey) {
        if (e.code === 'KeyS') { e.preventDefault(); save(); }
        else if (e.code === 'KeyF') { e.preventDefault(); openSearch(); }
        else if (e.code === 'KeyN') { e.preventDefault(); addTab('', ''); }
        else if (e.code === 'KeyO') { e.preventDefault(); openByPrompt(); }
        else if (e.code === 'KeyG') { e.preventDefault(); gotoLine(); }
      } else if (e.key === 'F5') { e.preventDefault(); run(); }
    }

    // --- события ---
    ta.addEventListener('input', onInput);
    ta.addEventListener('scroll', syncScroll);
    ta.addEventListener('keydown', onTaKey);
    const posUpdate = function () { renderGutter(); renderStatus(); };
    ta.addEventListener('keyup', posUpdate);
    ta.addEventListener('click', posUpdate);
    ta.addEventListener('select', posUpdate);
    document.addEventListener('selectionchange', function () { if (document.activeElement === ta) posUpdate(); });
    window.addEventListener('keydown', onWindowKey);
    window.addEventListener('resize', syncScroll);

    searchInput.addEventListener('input', function () { runSearch(false); });
    searchInput.addEventListener('keydown', function (e) {
      if (e.key === 'Enter') { e.preventDefault(); findNext(e.shiftKey ? -1 : 1); }
      else if (e.key === 'Escape') { e.preventDefault(); closeSearch(); }
      else if (e.key === 'F3') { e.preventDefault(); findNext(e.shiftKey ? -1 : 1); }
    });
    btnPrev.addEventListener('click', function () { findNext(-1); });
    btnNext.addEventListener('click', function () { findNext(1); });
    btnCloseSearch.addEventListener('click', closeSearch);

    btnNew.addEventListener('click', function () { addTab('', ''); });
    btnOpen.addEventListener('click', openByPrompt);
    btnSave.addEventListener('click', function () { save(); });
    btnFind.addEventListener('click', openSearch);
    btnRun.addEventListener('click', run);
    empty.addEventListener('dblclick', function () { addTab('', ''); });

    BW.listen('fs_changed', function (p) {
      if (!p || !p.path) return;
      st.tabs.forEach(function (t, i) {
        if (t.path !== p.path || t.modified) return;
        BW.fs_read(null, t.path).then(function (text) {
          text = String(text == null ? '' : text);
          if (t.modified || t.text === text) return;
          t.text = text;
          if (i === st.active) {
            const s = ta.selectionStart, top = ta.scrollTop;
            ta.value = text;
            ta.setSelectionRange(Math.min(s, text.length), Math.min(s, text.length));
            ta.scrollTop = top;
            refresh();
            scheduleCheck(0);
          }
        }, function () {});
      });
    });

    BW.onSettings(function () { measureLineHeight(); syncScroll(); });

    Promise.resolve(BW.ready).then(function () {
      measureLineHeight();
      if (arg) openFile(String(arg));
      else addTab('', '');
    });
  }

  return { mount: mount };
})();

Apps.register('editor', {
  title: 'Редактор', icon: 'editor', minWidth: 640, minHeight: 400,
  mount: function (body, arg) { EditorApp.mount(body, arg); }
});
