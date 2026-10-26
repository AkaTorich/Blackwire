'use strict';
// Файлы — дерево папок слева, список или плитка справа, крошки пути, свойства.
// Данные — только из Rust (fs_*), обновление по событию fs_changed.
// Файл: js/apps/files.js, стили: css/apps/files.css.

const FilesApp = (function () {
  function el(tag, cls, text) {
    const e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text != null) e.textContent = text;
    return e;
  }

  function iconBtn(icon, title, cls) {
    const b = el('button', 'fm-btn' + (cls ? ' ' + cls : ''));
    b.type = 'button';
    b.title = title;
    b.innerHTML = Icons.get(icon, 16);
    return b;
  }

  function normPath(p) {
    if (!p) return '/';
    const parts = [];
    for (const s of p.split('/')) {
      if (s === '' || s === '.') continue;
      if (s === '..') { parts.pop(); continue; }
      parts.push(s);
    }
    return '/' + parts.join('/');
  }

  function joinPath(dir, name) { return normPath(dir + '/' + name); }

  function parentOf(p) {
    p = normPath(p);
    if (p === '/') return '/';
    return normPath(p.slice(0, p.lastIndexOf('/')));
  }

  function baseName(p) {
    p = normPath(p);
    return p === '/' ? '/' : p.slice(p.lastIndexOf('/') + 1);
  }

  function fmtSize(n) {
    if (n == null) return '';
    if (n < 1024) return n + ' Б';
    if (n < 1024 * 1024) return (n / 1024).toFixed(n < 10240 ? 1 : 0) + ' КБ';
    if (n < 1024 * 1024 * 1024) return (n / 1048576).toFixed(1) + ' МБ';
    return (n / 1073741824).toFixed(2) + ' ГБ';
  }

  function two(n) { return n < 10 ? '0' + n : String(n); }

  function fmtTime(sec) {
    if (!sec) return '';
    const d = new Date(sec * 1000);
    return two(d.getDate()) + '.' + two(d.getMonth() + 1) + '.' + d.getFullYear() + ' ' + two(d.getHours()) + ':' + two(d.getMinutes());
  }

  function extOf(name) {
    const i = name.lastIndexOf('.');
    return i <= 0 ? '' : name.slice(i + 1).toLowerCase();
  }

  function iconFor(e) {
    if (e.is_dir) return 'folder';
    const x = extOf(e.name);
    if (x === 'adr') return 'file-code';
    if (x === 'txt' || x === 'log' || x === 'conf' || x === 'ini' || x === 'md' || x === 'csv') return 'file-text';
    return 'file';
  }

  function isHidden(name) { return name.charAt(0) === '.'; }

  function mount(body, arg) {
    const st = {
      cwd: '/', entries: [], selected: new Set(), anchor: null,
      view: 'list', showHidden: false, sortKey: 'name', sortDir: 1,
      clip: [], props: false, hist: [], histIdx: -1, loading: false
    };

    // --- разметка ---
    const root = el('div', 'fm');
    const bar = el('div', 'fm-bar');
    const btnBack = iconBtn('back', 'Назад — Alt+←');
    const btnFwd = iconBtn('forward', 'Вперёд — Alt+→');
    const btnUp = iconBtn('up', 'Вверх — Backspace');
    const btnHome = iconBtn('home', 'Домой');
    const btnReload = iconBtn('reload', 'Обновить — F5');
    const sep1 = el('span', 'fm-sep');
    const btnNewDir = iconBtn('folder', 'Новая папка — Ctrl+Shift+N');
    const btnNewFile = iconBtn('file', 'Новый файл — Ctrl+N');
    const btnRename = iconBtn('rename', 'Переименовать — F2');
    const btnDelete = iconBtn('trash', 'Удалить — Del');
    const btnCopy = iconBtn('copy', 'Копировать — Ctrl+C');
    const btnPaste = iconBtn('paste', 'Вставить — Ctrl+V');
    const btnProps = iconBtn('info', 'Свойства — Alt+Enter');
    const sep2 = el('span', 'fm-sep fm-grow');
    const btnHidden = iconBtn('eye-off', 'Скрытые файлы — Ctrl+H');
    const btnView = iconBtn('grid', 'Плитка / список');
    bar.append(btnBack, btnFwd, btnUp, btnHome, btnReload, sep1, btnNewDir, btnNewFile, btnRename, btnDelete, btnCopy, btnPaste, btnProps, sep2, btnHidden, btnView);

    const crumbsBar = el('div', 'fm-crumbs');
    const crumbs = el('div', 'fm-crumbs-list');
    const addr = el('input', 'fm-addr');
    addr.type = 'text';
    addr.spellcheck = false;
    addr.placeholder = '/путь';
    crumbsBar.append(crumbs, addr);

    const main = el('div', 'fm-main');
    const tree = el('div', 'fm-tree');
    const listWrap = el('div', 'fm-list-wrap');
    listWrap.tabIndex = 0;
    const header = el('div', 'fm-head');
    const cols = [['name', 'Имя'], ['size', 'Размер'], ['owner', 'Владелец'], ['mode', 'Права'], ['modified', 'Изменено']];
    for (const c of cols) {
      const h = el('div', 'fm-h fm-c-' + c[0], c[1]);
      h.dataset.key = c[0];
      h.addEventListener('click', function () { setSort(c[0]); });
      header.appendChild(h);
    }
    const list = el('div', 'fm-list');
    const emptyNote = el('div', 'fm-empty', 'Пусто');
    listWrap.append(header, list, emptyNote);
    main.append(tree, listWrap);

    const props = el('div', 'fm-props');
    const propsBody = el('div', 'fm-props-body');
    const propsClose = iconBtn('close', 'Закрыть');
    props.append(propsBody, propsClose);

    const statusBar = el('div', 'fm-status');
    const stCount = el('span', 'fm-st-count', '');
    const stSel = el('span', 'fm-st-sel', '');
    const stClip = el('span', 'fm-st-clip', '');
    statusBar.append(stCount, stSel, stClip);

    root.append(bar, crumbsBar, main, props, statusBar);
    body.appendChild(root);

    // --- дерево ---
    const treeNodes = {};  // путь → узел

    function makeTreeNode(path, name, depth) {
      const node = { path: path, name: name, depth: depth, loaded: false, expanded: false, children: [] };
      const row = el('div', 'fm-tn');
      row.style.paddingLeft = (depth * 14 + 6) + 'px';
      const chev = el('span', 'fm-tn-chev');
      chev.innerHTML = Icons.get('chevron-right', 12);
      const ic = el('span', 'fm-tn-icon');
      ic.innerHTML = Icons.get('folder', 15);
      const nm = el('span', 'fm-tn-name', name);
      nm.title = path;
      row.append(chev, ic, nm);
      const kids = el('div', 'fm-tk');
      node.row = row; node.kids = kids; node.chev = chev; node.ic = ic;
      chev.addEventListener('click', function (e) { e.stopPropagation(); toggleNode(node); });
      row.addEventListener('click', function () { go(path); });
      row.addEventListener('contextmenu', function (e) { e.preventDefault(); treeMenu(node, e.clientX, e.clientY); });
      treeNodes[path] = node;
      return node;
    }

    async function loadNode(node) {
      let entries;
      try { entries = await BW.fs_list(null, node.path); }
      catch (e) { entries = []; }
      const dirs = entries.filter(function (x) { return x.is_dir && (st.showHidden || !isHidden(x.name)); })
        .sort(function (a, b) { return a.name.localeCompare(b.name, 'ru'); });
      node.kids.textContent = '';
      node.children = [];
      for (const d of dirs) {
        const existing = treeNodes[d.path];
        const child = existing || makeTreeNode(d.path, d.name, node.depth + 1);
        node.children.push(child);
        node.kids.append(child.row, child.kids);
      }
      node.loaded = true;
      node.row.classList.toggle('leaf', dirs.length === 0);
    }

    async function expandNode(node) {
      if (!node.loaded) await loadNode(node);
      node.expanded = true;
      node.kids.classList.add('open');
      node.chev.classList.add('open');
      node.ic.innerHTML = Icons.get('folder-open', 15);
    }

    function collapseNode(node) {
      node.expanded = false;
      node.kids.classList.remove('open');
      node.chev.classList.remove('open');
      node.ic.innerHTML = Icons.get('folder', 15);
    }

    function toggleNode(node) {
      if (node.expanded) collapseNode(node); else expandNode(node);
    }

    const rootNode = makeTreeNode('/', '/', 0);
    tree.append(rootNode.row, rootNode.kids);

    // Раскрыть дерево до пути и подсветить его.
    async function revealInTree(path) {
      let node = rootNode;
      await expandNode(node);
      const segs = normPath(path).split('/').filter(Boolean);
      let acc = '';
      for (const s of segs) {
        acc += '/' + s;
        const child = node.children.find(function (c) { return c.path === acc; });
        if (!child) break;
        node = child;
        if (acc !== normPath(path)) await expandNode(node);
      }
      for (const k in treeNodes) treeNodes[k].row.classList.remove('cur');
      const curNode = treeNodes[normPath(path)];
      if (curNode) {
        curNode.row.classList.add('cur');
        if (curNode.row.scrollIntoView) curNode.row.scrollIntoView({ block: 'nearest' });
      }
    }

    async function refreshTreeNode(path) {
      const node = treeNodes[path];
      if (node && node.loaded) {
        await loadNode(node);
        if (node.expanded) node.kids.classList.add('open');
      }
    }

    // --- переходы ---
    async function go(path, noHistory) {
      path = normPath(path);
      if (st.loading) return;
      st.loading = true;
      root.classList.add('loading');
      let entries;
      try {
        entries = await BW.fs_list(null, path);
      } catch (e) {
        st.loading = false;
        root.classList.remove('loading');
        Win.toast(String(e));
        BW.sound('error');
        renderCrumbs();
        return;
      }
      if (!noHistory && st.cwd !== path) {
        st.hist = st.hist.slice(0, st.histIdx + 1);
        st.hist.push(path);
        st.histIdx = st.hist.length - 1;
      }
      st.cwd = path;
      st.entries = Array.isArray(entries) ? entries : [];
      st.selected.clear();
      st.anchor = null;
      st.loading = false;
      root.classList.remove('loading');
      Win.setTitle(path === '/' ? 'Файлы — /' : 'Файлы — ' + baseName(path));
      renderCrumbs();
      renderList();
      renderProps();
      revealInTree(path);
    }

    async function reload() {
      try {
        const entries = await BW.fs_list(null, st.cwd);
        st.entries = Array.isArray(entries) ? entries : [];
        for (const p of Array.from(st.selected)) {
          if (!st.entries.some(function (x) { return x.path === p; })) st.selected.delete(p);
        }
        renderList();
        renderProps();
      } catch (e) {
        Win.toast(String(e));
      }
    }

    function goBack() { if (st.histIdx > 0) { st.histIdx--; go(st.hist[st.histIdx], true); } }
    function goFwd() { if (st.histIdx < st.hist.length - 1) { st.histIdx++; go(st.hist[st.histIdx], true); } }
    function goUp() { if (st.cwd !== '/') go(parentOf(st.cwd)); }

    // --- крошки ---
    function renderCrumbs() {
      crumbs.textContent = '';
      const segs = normPath(st.cwd).split('/').filter(Boolean);
      const home = el('span', 'fm-crumb fm-crumb-root');
      home.innerHTML = Icons.get('monitor-screen', 13);
      home.title = '/';
      home.addEventListener('click', function (e) { e.stopPropagation(); go('/'); });
      crumbs.appendChild(home);
      let acc = '';
      segs.forEach(function (s, i) {
        acc += '/' + s;
        const target = acc;
        const sepEl = el('span', 'fm-crumb-sep');
        sepEl.innerHTML = Icons.get('chevron-right', 11);
        crumbs.appendChild(sepEl);
        const c = el('span', 'fm-crumb' + (i === segs.length - 1 ? ' last' : ''), s);
        c.title = target;
        c.addEventListener('click', function (e) { e.stopPropagation(); go(target); });
        crumbs.appendChild(c);
      });
      addr.value = st.cwd;
      btnBack.disabled = st.histIdx <= 0;
      btnFwd.disabled = st.histIdx >= st.hist.length - 1;
      btnUp.disabled = st.cwd === '/';
    }

    function showAddr() {
      crumbsBar.classList.add('edit');
      addr.value = st.cwd;
      addr.focus();
      addr.select();
    }

    function hideAddr() {
      crumbsBar.classList.remove('edit');
    }

    // --- список ---
    function visibleEntries() {
      const key = st.sortKey, dir = st.sortDir;
      const arr = st.entries.filter(function (e) { return st.showHidden || !isHidden(e.name); });
      arr.sort(function (a, b) {
        if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
        let r = 0;
        if (key === 'name') r = a.name.localeCompare(b.name, 'ru');
        else if (key === 'size') r = (a.size || 0) - (b.size || 0);
        else if (key === 'modified') r = (a.modified || 0) - (b.modified || 0);
        else r = String(a[key] || '').localeCompare(String(b[key] || ''), 'ru');
        if (r === 0) r = a.name.localeCompare(b.name, 'ru');
        return r * dir;
      });
      return arr;
    }

    function setSort(key) {
      if (st.sortKey === key) st.sortDir = -st.sortDir;
      else { st.sortKey = key; st.sortDir = 1; }
      renderList();
    }

    function renderList() {
      const arr = visibleEntries();
      list.textContent = '';
      list.className = 'fm-list ' + (st.view === 'tiles' ? 'tiles' : 'rows');
      header.classList.toggle('hidden', st.view === 'tiles');
      for (const h of header.children) {
        h.classList.toggle('asc', h.dataset.key === st.sortKey && st.sortDir > 0);
        h.classList.toggle('desc', h.dataset.key === st.sortKey && st.sortDir < 0);
      }
      const frag = document.createDocumentFragment();
      for (const e of arr) {
        const row = el('div', (st.view === 'tiles' ? 'fm-tile' : 'fm-row') + (st.selected.has(e.path) ? ' sel' : '') + (isHidden(e.name) ? ' hid' : ''));
        row.dataset.path = e.path;
        const ic = el('span', 'fm-icon' + (e.is_dir ? ' dir' : ''));
        ic.innerHTML = Icons.get(iconFor(e), st.view === 'tiles' ? 36 : 16);
        const nm = el('span', 'fm-name', e.name);
        nm.title = e.name;
        if (st.view === 'tiles') {
          row.append(ic, nm);
        } else {
          const cName = el('div', 'fm-c fm-c-name');
          cName.append(ic, nm);
          row.append(cName,
            el('div', 'fm-c fm-c-size', e.is_dir ? '' : fmtSize(e.size)),
            el('div', 'fm-c fm-c-owner', e.owner || ''),
            el('div', 'fm-c fm-c-mode', e.mode || ''),
            el('div', 'fm-c fm-c-modified', fmtTime(e.modified)));
        }
        row.addEventListener('click', function (ev) { onRowClick(e, ev); });
        row.addEventListener('dblclick', function () { openEntry(e); });
        row.addEventListener('contextmenu', function (ev) {
          ev.preventDefault();
          if (!st.selected.has(e.path)) { st.selected.clear(); st.selected.add(e.path); st.anchor = e.path; renderList(); renderProps(); }
          entryMenu(ev.clientX, ev.clientY);
        });
        frag.appendChild(row);
      }
      list.appendChild(frag);
      emptyNote.classList.toggle('on', arr.length === 0);
      renderStatus(arr.length);
    }

    function updateSelectionClasses() {
      for (const row of list.children) row.classList.toggle('sel', st.selected.has(row.dataset.path));
      renderStatus(list.children.length);
      renderProps();
    }

    function onRowClick(e, ev) {
      const arr = visibleEntries();
      if (ev.shiftKey && st.anchor) {
        const a = arr.findIndex(function (x) { return x.path === st.anchor; });
        const b = arr.findIndex(function (x) { return x.path === e.path; });
        if (a >= 0 && b >= 0) {
          if (!ev.ctrlKey) st.selected.clear();
          for (let i = Math.min(a, b); i <= Math.max(a, b); i++) st.selected.add(arr[i].path);
        }
      } else if (ev.ctrlKey) {
        if (st.selected.has(e.path)) st.selected.delete(e.path); else st.selected.add(e.path);
        st.anchor = e.path;
      } else {
        st.selected.clear();
        st.selected.add(e.path);
        st.anchor = e.path;
      }
      updateSelectionClasses();
    }

    function selectedEntries() {
      return st.entries.filter(function (e) { return st.selected.has(e.path); });
    }

    function openEntry(e) {
      BW.sound('open');
      if (e.is_dir) go(e.path);
      else BW.open_app('editor', e.path).catch(function (err) { Win.toast(String(err)); });
    }

    function moveSelection(delta, extend) {
      const arr = visibleEntries();
      if (!arr.length) return;
      let idx = arr.findIndex(function (x) { return x.path === st.anchor; });
      if (idx < 0) idx = delta > 0 ? -1 : arr.length;
      idx = Math.max(0, Math.min(arr.length - 1, idx + delta));
      const e = arr[idx];
      if (!extend) st.selected.clear();
      st.selected.add(e.path);
      st.anchor = e.path;
      updateSelectionClasses();
      const row = list.querySelector('[data-path="' + e.path.replace(/"/g, '\\"') + '"]');
      if (row && row.scrollIntoView) row.scrollIntoView({ block: 'nearest' });
    }

    function renderStatus(count) {
      stCount.textContent = count + ' ' + plural(count, 'элемент', 'элемента', 'элементов');
      const sel = selectedEntries();
      if (sel.length) {
        const total = sel.reduce(function (s, e) { return s + (e.is_dir ? 0 : (e.size || 0)); }, 0);
        stSel.textContent = 'выбрано ' + sel.length + (total ? ' · ' + fmtSize(total) : '');
      } else stSel.textContent = '';
      stClip.textContent = st.clip.length ? 'в буфере: ' + st.clip.length : '';
    }

    function plural(n, a, b, c) {
      const m10 = n % 10, m100 = n % 100;
      if (m10 === 1 && m100 !== 11) return a;
      if (m10 >= 2 && m10 <= 4 && (m100 < 10 || m100 >= 20)) return b;
      return c;
    }

    // --- свойства ---
    function renderProps() {
      root.classList.toggle('props-on', st.props);
      if (!st.props) return;
      propsBody.textContent = '';
      const sel = selectedEntries();
      const e = sel.length === 1 ? sel[0] : null;
      function row(k, v) {
        const r = el('div', 'fm-prop');
        r.append(el('span', 'fm-prop-k', k), el('span', 'fm-prop-v', v));
        propsBody.appendChild(r);
      }
      if (!e) {
        if (sel.length > 1) {
          const total = sel.reduce(function (s, x) { return s + (x.is_dir ? 0 : (x.size || 0)); }, 0);
          row('Выбрано', sel.length + ' ' + plural(sel.length, 'элемент', 'элемента', 'элементов'));
          row('Размер', fmtSize(total));
        } else {
          row('Папка', st.cwd);
          row('Содержимое', st.entries.length + ' ' + plural(st.entries.length, 'элемент', 'элемента', 'элементов'));
        }
        return;
      }
      row('Имя', e.name);
      row('Путь', e.path);
      row('Тип', e.is_dir ? 'Папка' : (extOf(e.name) ? 'Файл .' + extOf(e.name) : 'Файл'));
      row('Размер', e.is_dir ? '—' : fmtSize(e.size) + (e.size >= 1024 ? ' (' + e.size + ' Б)' : ''));
      row('Владелец', e.owner || '—');
      row('Права', e.mode || '—');
      row('Изменено', fmtTime(e.modified) || '—');
    }

    // --- действия ---
    async function newFolder() {
      const name = await Win.prompt('Имя новой папки', 'Новая папка');
      if (!name || !name.trim()) return;
      try { await BW.fs_mkdir(null, joinPath(st.cwd, name.trim())); await reload(); }
      catch (e) { Win.toast(String(e)); BW.sound('error'); }
    }

    async function newFile() {
      const name = await Win.prompt('Имя нового файла', 'новый.adr');
      if (!name || !name.trim()) return;
      const path = joinPath(st.cwd, name.trim());
      try { await BW.fs_write(null, path, ''); await reload(); selectPath(path); }
      catch (e) { Win.toast(String(e)); BW.sound('error'); }
    }

    function selectPath(path) {
      st.selected.clear();
      st.selected.add(path);
      st.anchor = path;
      updateSelectionClasses();
    }

    async function renameSel() {
      const sel = selectedEntries();
      if (sel.length !== 1) return;
      const e = sel[0];
      const name = await Win.prompt('Новое имя', e.name);
      if (!name || !name.trim() || name.trim() === e.name) return;
      try { await BW.fs_rename(null, e.path, joinPath(parentOf(e.path), name.trim())); await reload(); }
      catch (err) { Win.toast(String(err)); BW.sound('error'); }
    }

    async function deleteSel() {
      const sel = selectedEntries();
      if (!sel.length) return;
      const text = sel.length === 1 ? 'Удалить «' + sel[0].name + '»?' : 'Удалить ' + sel.length + ' ' + plural(sel.length, 'элемент', 'элемента', 'элементов') + '?';
      if (!(await Win.confirm(text))) return;
      for (const e of sel) {
        try { await BW.fs_delete(null, e.path); }
        catch (err) { Win.toast(String(err)); BW.sound('error'); }
      }
      await reload();
    }

    function copySel() {
      const sel = selectedEntries();
      if (!sel.length) return;
      st.clip = sel.map(function (e) { return e.path; });
      renderStatus(list.children.length);
      Win.toast('Скопировано: ' + st.clip.length);
    }

    async function pasteClip() {
      if (!st.clip.length) return;
      for (const src of st.clip) {
        let dst = joinPath(st.cwd, baseName(src));
        if (dst === src || st.entries.some(function (e) { return e.path === dst; })) {
          const nm = baseName(src);
          const i = nm.lastIndexOf('.');
          dst = joinPath(st.cwd, i > 0 ? nm.slice(0, i) + ' (копия)' + nm.slice(i) : nm + ' (копия)');
        }
        try { await BW.fs_copy(null, src, dst); }
        catch (err) { Win.toast(String(err)); BW.sound('error'); }
      }
      await reload();
    }

    function toggleProps() { st.props = !st.props; renderProps(); }
    function toggleHidden() {
      st.showHidden = !st.showHidden;
      btnHidden.innerHTML = Icons.get(st.showHidden ? 'eye' : 'eye-off', 16);
      btnHidden.classList.toggle('on', st.showHidden);
      renderList();
      // дерево: перечитать загруженные узлы
      for (const k in treeNodes) if (treeNodes[k].loaded) refreshTreeNode(k);
    }
    function toggleView() {
      st.view = st.view === 'list' ? 'tiles' : 'list';
      btnView.innerHTML = Icons.get(st.view === 'list' ? 'grid' : 'layout', 16);
      renderList();
    }

    function openTerminalHere() {
      BW.open_app('terminal', 'cd ' + st.cwd).catch(function (e) { Win.toast(String(e)); });
    }

    function entryMenu(x, y) {
      const sel = selectedEntries();
      const one = sel.length === 1 ? sel[0] : null;
      const items = [];
      if (one) items.push({ label: one.is_dir ? 'Открыть' : 'Открыть в редакторе', icon: one.is_dir ? 'folder-open' : 'editor', action: function () { openEntry(one); } });
      if (one && one.is_dir) items.push({ label: 'Терминал здесь', icon: 'terminal', action: function () { BW.open_app('terminal', 'cd ' + one.path).catch(function () {}); } });
      if (items.length) items.push(null);
      items.push({ label: 'Копировать', icon: 'copy', action: copySel });
      if (one) items.push({ label: 'Переименовать', icon: 'rename', action: renameSel });
      items.push({ label: 'Удалить', icon: 'trash', action: deleteSel });
      items.push(null);
      items.push({ label: 'Свойства', icon: 'info', action: function () { if (!st.props) toggleProps(); else renderProps(); } });
      Win.menu(items, x, y);
    }

    function bgMenu(x, y) {
      Win.menu([
        { label: 'Новая папка', icon: 'folder', action: newFolder },
        { label: 'Новый файл', icon: 'file', action: newFile },
        null,
        { label: 'Вставить', icon: 'paste', action: pasteClip },
        { label: 'Обновить', icon: 'reload', action: reload },
        null,
        { label: 'Терминал здесь', icon: 'terminal', action: openTerminalHere },
        { label: st.view === 'list' ? 'Плитка' : 'Список', icon: st.view === 'list' ? 'grid' : 'layout', action: toggleView },
        { label: st.showHidden ? 'Скрыть скрытые' : 'Показать скрытые', icon: st.showHidden ? 'eye-off' : 'eye', action: toggleHidden },
        { label: 'Свойства папки', icon: 'info', action: function () { st.selected.clear(); updateSelectionClasses(); if (!st.props) toggleProps(); } }
      ], x, y);
    }

    function treeMenu(node, x, y) {
      Win.menu([
        { label: 'Открыть', icon: 'folder-open', action: function () { go(node.path); } },
        { label: node.expanded ? 'Свернуть' : 'Развернуть', icon: node.expanded ? 'chevron-down' : 'chevron-right', action: function () { toggleNode(node); } },
        null,
        { label: 'Терминал здесь', icon: 'terminal', action: function () { BW.open_app('terminal', 'cd ' + node.path).catch(function () {}); } }
      ], x, y);
    }

    // --- клавиши ---
    function onKey(e) {
      const t = e.target;
      if (t === addr) return;
      if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA')) return;
      if (e.altKey) {
        if (e.key === 'ArrowLeft') { e.preventDefault(); goBack(); }
        else if (e.key === 'ArrowRight') { e.preventDefault(); goFwd(); }
        else if (e.key === 'Enter') { e.preventDefault(); toggleProps(); }
        return;
      }
      if (e.ctrlKey) {
        switch (e.code) {
          case 'KeyC': e.preventDefault(); copySel(); return;
          case 'KeyV': e.preventDefault(); pasteClip(); return;
          case 'KeyH': e.preventDefault(); toggleHidden(); return;
          case 'KeyL': e.preventDefault(); showAddr(); return;
          case 'KeyA': e.preventDefault(); for (const x of visibleEntries()) st.selected.add(x.path); updateSelectionClasses(); return;
          case 'KeyN': e.preventDefault(); if (e.shiftKey) newFolder(); else newFile(); return;
          default: return;
        }
      }
      switch (e.key) {
        case 'F2': e.preventDefault(); renameSel(); return;
        case 'F5': e.preventDefault(); reload(); return;
        case 'Delete': e.preventDefault(); deleteSel(); return;
        case 'Backspace': e.preventDefault(); goUp(); return;
        case 'Enter': {
          e.preventDefault();
          const sel = selectedEntries();
          if (sel.length === 1) openEntry(sel[0]);
          return;
        }
        case 'ArrowDown': e.preventDefault(); moveSelection(1, e.shiftKey); return;
        case 'ArrowUp': e.preventDefault(); moveSelection(-1, e.shiftKey); return;
        case 'Home': e.preventDefault(); moveSelection(-100000, e.shiftKey); return;
        case 'End': e.preventDefault(); moveSelection(100000, e.shiftKey); return;
        case 'Escape': e.preventDefault(); st.selected.clear(); updateSelectionClasses(); return;
        default: return;
      }
    }

    // --- события ---
    btnBack.addEventListener('click', goBack);
    btnFwd.addEventListener('click', goFwd);
    btnUp.addEventListener('click', goUp);
    btnHome.addEventListener('click', function () { go(st.home); });
    btnReload.addEventListener('click', reload);
    btnNewDir.addEventListener('click', newFolder);
    btnNewFile.addEventListener('click', newFile);
    btnRename.addEventListener('click', renameSel);
    btnDelete.addEventListener('click', deleteSel);
    btnCopy.addEventListener('click', copySel);
    btnPaste.addEventListener('click', pasteClip);
    btnProps.addEventListener('click', toggleProps);
    btnHidden.addEventListener('click', toggleHidden);
    btnView.addEventListener('click', toggleView);
    propsClose.addEventListener('click', toggleProps);

    crumbsBar.addEventListener('click', function (e) { if (e.target === crumbsBar || e.target === crumbs) showAddr(); });
    addr.addEventListener('keydown', function (e) {
      if (e.key === 'Enter') { e.preventDefault(); const p = addr.value.trim(); hideAddr(); listWrap.focus(); if (p) go(p); }
      else if (e.key === 'Escape') { e.preventDefault(); hideAddr(); listWrap.focus(); }
    });
    addr.addEventListener('blur', hideAddr);

    list.addEventListener('click', function (e) {
      if (e.target === list) { st.selected.clear(); updateSelectionClasses(); }
    });
    listWrap.addEventListener('contextmenu', function (e) {
      if (e.target === list || e.target === listWrap || e.target === emptyNote) { e.preventDefault(); bgMenu(e.clientX, e.clientY); }
    });
    listWrap.addEventListener('keydown', onKey);
    window.addEventListener('keydown', function (e) {
      if (e.target === listWrap) return;  // уже обработано
      onKey(e);
    });

    BW.listen('fs_changed', function (p) {
      if (!p || !p.path) return;
      const path = normPath(p.path);
      const parent = parentOf(path);
      if (parent === st.cwd || path === st.cwd) reload();
      refreshTreeNode(parent);
      if (treeNodes[path]) refreshTreeNode(path);
    });

    st.home = arg ? normPath(String(arg)) : '/';
    Promise.resolve(BW.ready).then(function () {
      listWrap.focus();
      go(st.home);
    });
  }

  return { mount: mount };
})();

Apps.register('files', {
  title: 'Файлы', icon: 'files', minWidth: 640, minHeight: 400,
  mount: function (body, arg) { FilesApp.mount(body, arg); }
});
