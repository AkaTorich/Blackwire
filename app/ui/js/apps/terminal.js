'use strict';
// Терминал — окно-вид на сеанс оболочки, который живёт в Rust.
// Вывод печатается строками по виду `kind`, строка ввода рисуется сама (без <input>),
// клавиши ловятся на окне. Файл: js/apps/terminal.js, стили: css/apps/terminal.css.

const TerminalApp = (function () {
  const MAX_ROWS = 4000;        // сколько строк вывода держим в окне
  const TRIM_ROWS = 500;        // сколько старых строк убираем за раз

  const KIND_CLASS = {
    normal: 'k-normal', error: 'k-error', dim: 'k-dim', accent: 'k-accent',
    success: 'k-success', warn: 'k-warn', heading: 'k-heading'
  };

  function el(tag, cls, text) {
    const e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text != null) e.textContent = text;
    return e;
  }

  function promptString(p) {
    if (!p) return '$ ';
    return p.user + '@' + p.host + ':' + p.cwd + (p.is_root ? '#' : '$') + ' ';
  }

  function commonPrefix(list) {
    if (!list.length) return '';
    let pref = list[0];
    for (let i = 1; i < list.length && pref; i++) {
      const s = list[i];
      let k = 0;
      while (k < pref.length && k < s.length && pref[k] === s[k]) k++;
      pref = pref.slice(0, k);
    }
    return pref;
  }

  function mount(body, arg) {
    const st = {
      sid: null,            // номер сеанса оболочки
      prompt: null,         // Prompt из Rust
      line: '', cur: 0,     // строка ввода и позиция курсора
      history: [], histIdx: -1, histDraft: '',
      busy: false,          // ввод закрыт: идёт печать или работает скрипт
      printing: false,      // идёт печать строк с задержками
      scriptPid: null,      // pid скрипта этого окна
      inputWanted: false,   // скрипт ждёт input()
      inputPrompt: '',
      skip: false,          // Ctrl+C во время печати — пропустить задержки
      wake: null, timer: null,
      closed: false
    };

    // --- разметка ---
    const root = el('div', 'term');
    const bg = el('div', 'term-bg');
    const scroll = el('div', 'term-scroll');
    const out = el('div', 'term-out');
    const hops = el('div', 'term-hops');
    const line = el('div', 'term-line');
    const promptEl = el('span', 'term-prompt');
    const before = el('span', 'term-text');
    const cursor = el('span', 'term-cursor');
    const after = el('span', 'term-text');
    line.append(promptEl, before, cursor, after);
    scroll.append(out, hops, line);
    root.append(bg, scroll);
    body.appendChild(root);

    // --- вывод ---
    function scrollBottom() {
      scroll.scrollTop = scroll.scrollHeight;
    }

    function addRow(text, kind) {
      const cls = KIND_CLASS[kind] || 'k-normal';
      const parts = String(text == null ? '' : text).split('\n');
      for (const part of parts) {
        const row = el('div', 'term-row ' + cls, part === '' ? '​' : part);
        out.appendChild(row);
      }
      if (out.childElementCount > MAX_ROWS) {
        for (let i = 0; i < TRIM_ROWS && out.firstChild; i++) out.removeChild(out.firstChild);
      }
      scrollBottom();
    }

    // Эхо команды: подсказка цветом, команда обычным.
    function echoCommand(promptText, text, accentPrompt) {
      const row = el('div', 'term-row k-cmd');
      row.appendChild(el('span', accentPrompt ? 'term-prompt term-prompt-script' : 'term-prompt' + (st.prompt && st.prompt.is_root ? ' root' : ''), promptText));
      row.appendChild(el('span', '', text));
      out.appendChild(row);
      scrollBottom();
    }

    function clearScreen() {
      out.textContent = '';
      scrollBottom();
    }

    // --- строка ввода ---
    function renderPrompt() {
      const p = st.prompt;
      if (st.inputWanted) {
        promptEl.textContent = st.inputPrompt;
        promptEl.className = 'term-prompt term-prompt-script';
        hops.textContent = '';
        hops.classList.remove('on');
        return;
      }
      promptEl.textContent = promptString(p);
      promptEl.className = 'term-prompt' + (p && p.is_root ? ' root' : '');
      if (p && p.hops && p.hops.length > 1) {
        hops.textContent = p.hops.join(' → ');
        hops.classList.add('on');
      } else {
        hops.textContent = '';
        hops.classList.remove('on');
      }
      if (p) Win.setTitle(p.user + '@' + p.host + ': ' + p.cwd);
    }

    function renderLine() {
      const hidden = st.busy && !st.inputWanted;
      root.classList.toggle('busy', hidden);
      if (hidden) return;
      before.textContent = st.line.slice(0, st.cur);
      const ch = st.line.charAt(st.cur);
      cursor.textContent = ch === '' ? ' ' : ch;
      after.textContent = st.line.slice(st.cur + 1);
      scrollBottom();
    }

    function setLine(text, cur) {
      st.line = text;
      st.cur = cur == null ? text.length : Math.max(0, Math.min(cur, text.length));
      renderLine();
    }

    function insertText(text) {
      text = String(text).replace(/\r/g, '');
      // Многострочная вставка: первая строка в поле, остальные — следом как команды не выполняем,
      // вставляем как есть одной строкой (переносы заменяем пробелами).
      text = text.replace(/\n+/g, ' ');
      setLine(st.line.slice(0, st.cur) + text + st.line.slice(st.cur), st.cur + text.length);
    }

    // --- печать с задержками ---
    function wait(ms) {
      return new Promise(function (resolve) {
        st.wake = resolve;
        st.timer = setTimeout(function () { st.timer = null; st.wake = null; resolve(); }, ms);
      });
    }

    function wakeUp() {
      if (st.timer) { clearTimeout(st.timer); st.timer = null; }
      if (st.wake) { const r = st.wake; st.wake = null; r(); }
    }

    async function printLines(lines) {
      if (!lines || !lines.length) return;
      st.printing = true;
      st.skip = false;
      try {
        for (const ln of lines) {
          if (st.closed) return;
          if (ln.delay_ms > 0 && !st.skip) await wait(ln.delay_ms);
          addRow(ln.text, ln.kind);
          if (ln.kind === 'error') BW.sound('error');
        }
      } finally {
        st.printing = false;
        st.skip = false;
      }
    }

    // --- выполнение ---
    async function refreshPrompt() {
      if (st.sid == null) return;
      try {
        st.prompt = await BW.shell_prompt(st.sid);
        renderPrompt();
      } catch (e) { /* сеанс мог закрыться */ }
    }

    async function handleActions(actions) {
      for (const a of actions || []) {
        switch (a.type) {
          case 'clear':
            clearScreen();
            break;
          case 'exit':
            shutdown();
            Win.close();
            return;
          case 'open_app':
            try { await BW.open_app(a.app, a.arg == null ? null : a.arg); }
            catch (e) { addRow(String(e), 'error'); }
            break;
          case 'notify':
            Win.toast(a.title ? a.title + ': ' + a.text : a.text);
            BW.sound('notify');
            break;
          case 'script_started':
            st.scriptPid = a.pid;
            st.busy = true;
            st.inputWanted = false;
            renderLine();
            replayOrphans(a.pid);
            break;
          default:
            break;
        }
      }
    }

    async function runLine(text) {
      if (st.sid == null) return;
      echoCommand(promptString(st.prompt), text, false);
      st.busy = true;
      renderLine();
      let res = null;
      try {
        res = await BW.shell_exec(st.sid, text);
      } catch (e) {
        addRow(String(e), 'error');
        BW.sound('error');
      }
      if (st.closed) return;
      if (res) {
        await printLines(res.lines);
        if (res.prompt) { st.prompt = res.prompt; renderPrompt(); }
        await handleActions(res.actions);
      }
      if (st.scriptPid == null) {
        st.busy = false;
        renderLine();
      }
    }

    function submit() {
      const text = st.line;
      if (st.inputWanted) {
        // ответ на input() скрипта
        echoCommand(st.inputPrompt, text, true);
        st.inputWanted = false;
        st.busy = true;
        setLine('', 0);
        renderPrompt();
        BW.script_input(st.scriptPid, text).catch(function (e) { addRow(String(e), 'error'); });
        return;
      }
      setLine('', 0);
      st.histIdx = -1;
      st.histDraft = '';
      if (text.trim() !== '') {
        if (st.history[st.history.length - 1] !== text) st.history.push(text);
      }
      BW.sound('key');
      runLine(text);
    }

    // --- скрипты (события по pid этого окна) ---
    // Скрипт может напечатать и даже закончиться раньше, чем терминал узнает его pid
    // (ответ shell_exec печатается с паузами). Такие события копим по pid и проигрываем потом.
    const orphans = {};
    function stash(kind, p) {
      if (!p || p.pid == null) return;
      (orphans[p.pid] = orphans[p.pid] || []).push({ kind: kind, p: p });
      if (Object.keys(orphans).length > 20) delete orphans[Object.keys(orphans)[0]];
    }
    function replayOrphans(pid) {
      const list = orphans[pid] || [];
      delete orphans[pid];
      list.forEach(function (ev) {
        if (ev.kind === 'out') onScriptOut(ev.p);
        else if (ev.kind === 'done') onScriptDone(ev.p);
        else if (ev.kind === 'input') onScriptInputWanted(ev.p);
      });
    }

    function onScriptOut(p) {
      if (!p) return;
      if (p.pid !== st.scriptPid) { stash('out', p); return; }
      addRow(p.text, 'normal');
    }

    function onScriptDone(p) {
      if (!p) return;
      if (p.pid !== st.scriptPid) { stash('done', p); return; }
      if (!p.ok && p.error) {
        addRow(p.line ? 'Ошибка (строка ' + p.line + '): ' + p.error : p.error, 'error');
        BW.sound('error');
      }
      st.scriptPid = null;
      st.inputWanted = false;
      st.busy = false;
      refreshPrompt().then(function () { renderLine(); });
      renderLine();
    }

    function onScriptInputWanted(p) {
      if (!p) return;
      if (p.pid !== st.scriptPid) { stash('input', p); return; }
      st.inputWanted = true;
      st.inputPrompt = p.prompt || '';
      st.busy = false;
      setLine('', 0);
      renderPrompt();
      renderLine();
    }

    // --- клавиши ---
    function interrupt() {
      if (st.printing) { st.skip = true; wakeUp(); return; }
      if (st.scriptPid != null) {
        addRow('^C', 'dim');
        BW.script_kill(st.scriptPid).catch(function () {});
        return;
      }
      const sel = window.getSelection ? String(window.getSelection()) : '';
      if (sel) { copyText(sel); return; }
      if (st.line) echoCommand(promptString(st.prompt), st.line + '^C', false);
      st.histIdx = -1;
      setLine('', 0);
    }

    function copyText(text) {
      if (!text) return;
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(function () { Win.toast('Скопировано'); }, function () {});
      }
    }

    function copySelection() {
      const sel = window.getSelection ? String(window.getSelection()) : '';
      if (sel) copyText(sel);
    }

    function paste() {
      if (st.busy && !st.inputWanted) return;
      if (!navigator.clipboard || !navigator.clipboard.readText) return;
      navigator.clipboard.readText().then(function (t) { if (t) insertText(t); }, function () {
        Win.toast('Буфер обмена недоступен');
      });
    }

    function historyMove(dir) {
      if (!st.history.length) return;
      if (st.histIdx === -1) {
        if (dir < 0) return;
        st.histDraft = st.line;
        st.histIdx = st.history.length;
      }
      let idx = st.histIdx - dir;  // dir=1 — вверх (к старым), dir=-1 — вниз
      if (idx < 0) idx = 0;
      if (idx >= st.history.length) {
        st.histIdx = -1;
        setLine(st.histDraft, null);
        return;
      }
      st.histIdx = idx;
      setLine(st.history[idx], null);
    }

    function wordLeft() {
      let i = st.cur;
      while (i > 0 && st.line[i - 1] === ' ') i--;
      while (i > 0 && st.line[i - 1] !== ' ') i--;
      return i;
    }

    function wordRight() {
      let i = st.cur;
      while (i < st.line.length && st.line[i] !== ' ') i++;
      while (i < st.line.length && st.line[i] === ' ') i++;
      return i;
    }

    async function complete() {
      if (st.sid == null || st.inputWanted) return;
      let list;
      try { list = await BW.shell_complete(st.sid, st.line.slice(0, st.cur)); }
      catch (e) { return; }
      if (!Array.isArray(list) || !list.length) return;
      const head = st.line.slice(0, st.cur);
      const tail = st.line.slice(st.cur);
      // Варианты могут быть целыми строками или только последним словом.
      const wholeLine = list.every(function (s) { return s.startsWith(head); });
      const sp = head.lastIndexOf(' ');
      const word = wholeLine ? head : head.slice(sp + 1);
      const base = wholeLine ? '' : head.slice(0, sp + 1);
      if (list.length === 1) {
        let s = list[0];
        if (!s.endsWith('/') && !s.endsWith(' ')) s += ' ';
        setLine(base + s + tail, (base + s).length);
        return;
      }
      const pref = commonPrefix(list);
      if (pref.length > word.length) {
        setLine(base + pref + tail, (base + pref).length);
      }
      const shown = list.map(function (s) { return wholeLine ? s.slice(base.length) : s; });
      echoCommand(promptString(st.prompt), st.line, false);
      addRow(shown.join('  '), 'dim');
    }

    function onKey(e) {
      const t = e.target;
      // Диалоги каркаса (Win.prompt и т. п.) — не вмешиваемся.
      if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return;
      if (e.altKey) return;

      if (e.ctrlKey) {
        if (e.shiftKey) {
          if (e.code === 'KeyC') { e.preventDefault(); copySelection(); }
          if (e.code === 'KeyV') { e.preventDefault(); paste(); }
          return;  // остальное с Shift — каркасу (Ctrl+Shift+T и т. п.)
        }
        switch (e.code) {
          case 'KeyC': e.preventDefault(); interrupt(); return;
          case 'KeyL': e.preventDefault(); clearScreen(); return;
          case 'KeyV': e.preventDefault(); paste(); return;
          case 'KeyU':
            e.preventDefault();
            if (!st.busy || st.inputWanted) setLine(st.line.slice(st.cur), 0);
            return;
          case 'KeyK':
            e.preventDefault();
            if (!st.busy || st.inputWanted) setLine(st.line.slice(0, st.cur), st.cur);
            return;
          case 'KeyA': e.preventDefault(); if (!st.busy || st.inputWanted) setLine(st.line, 0); return;
          case 'KeyE': e.preventDefault(); if (!st.busy || st.inputWanted) setLine(st.line, st.line.length); return;
          case 'ArrowLeft': e.preventDefault(); setLine(st.line, wordLeft()); return;
          case 'ArrowRight': e.preventDefault(); setLine(st.line, wordRight()); return;
          case 'Backspace':
            e.preventDefault();
            if (!st.busy || st.inputWanted) { const i = wordLeft(); setLine(st.line.slice(0, i) + st.line.slice(st.cur), i); }
            return;
          default: return;  // Ctrl+W, Ctrl+Shift+… — каркасу
        }
      }

      if (st.busy && !st.inputWanted) {
        if (e.key === 'Enter' || e.key.length === 1) e.preventDefault();
        return;
      }

      switch (e.key) {
        case 'Enter': e.preventDefault(); submit(); return;
        case 'Backspace':
          e.preventDefault();
          if (st.cur > 0) setLine(st.line.slice(0, st.cur - 1) + st.line.slice(st.cur), st.cur - 1);
          return;
        case 'Delete':
          e.preventDefault();
          if (st.cur < st.line.length) setLine(st.line.slice(0, st.cur) + st.line.slice(st.cur + 1), st.cur);
          return;
        case 'ArrowLeft': e.preventDefault(); setLine(st.line, st.cur - 1); return;
        case 'ArrowRight': e.preventDefault(); setLine(st.line, st.cur + 1); return;
        case 'ArrowUp': e.preventDefault(); if (!st.inputWanted) historyMove(1); return;
        case 'ArrowDown': e.preventDefault(); if (!st.inputWanted) historyMove(-1); return;
        case 'Home': e.preventDefault(); setLine(st.line, 0); return;
        case 'End': e.preventDefault(); setLine(st.line, st.line.length); return;
        case 'Tab': e.preventDefault(); complete(); return;
        case 'Escape': e.preventDefault(); return;
        case 'PageUp': e.preventDefault(); scroll.scrollTop -= scroll.clientHeight * 0.8; return;
        case 'PageDown': e.preventDefault(); scroll.scrollTop += scroll.clientHeight * 0.8; return;
        default: break;
      }
      if (e.key.length === 1) {
        e.preventDefault();
        insertText(e.key);
      }
    }

    // --- запуск и остановка ---
    function shutdown() {
      if (st.closed) return;
      st.closed = true;
      wakeUp();
      if (st.sid != null) {
        try { BW.shell_close(st.sid).catch(function () {}); } catch (e) { /* окно уже уходит */ }
      }
    }

    async function open() {
      try {
        st.sid = await BW.shell_open();
      } catch (e) {
        addRow('Оболочка недоступна: ' + e, 'error');
        return;
      }
      try { st.prompt = await BW.shell_prompt(st.sid); } catch (e) { st.prompt = null; }
      renderPrompt();
      try {
        const h = await BW.shell_history(st.sid);
        if (Array.isArray(h)) st.history = h.slice();
      } catch (e) { /* истории может не быть */ }
      renderLine();
      if (arg) await runLine(String(arg));
    }

    window.addEventListener('keydown', onKey);
    window.addEventListener('blur', function () { root.classList.add('unfocused'); });
    window.addEventListener('focus', function () { root.classList.remove('unfocused'); });
    window.addEventListener('beforeunload', shutdown);
    window.addEventListener('pagehide', shutdown);
    BW.listen('script_out', onScriptOut);
    BW.listen('script_done', onScriptDone);
    BW.listen('script_input_wanted', onScriptInputWanted);

    Promise.resolve(BW.ready).then(open);
  }

  return { mount: mount };
})();

Apps.register('terminal', {
  title: 'Терминал', icon: 'terminal', minWidth: 520, minHeight: 320,
  mount: function (body, arg) { TerminalApp.mount(body, arg); }
});
