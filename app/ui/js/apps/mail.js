'use strict';
// Почта — папки слева, список писем посередине, чтение или форма письма справа.
// Данные — mail_list / mail_read / mail_send / mail_delete, событие mail_new.
// Файл: js/apps/mail.js, стили: css/apps/mail.css.

const MailApp = (function () {
  const FOLDERS = [
    { id: 'inbox', name: 'Входящие', icon: 'inbox' },
    { id: 'sent', name: 'Отправленные', icon: 'send' }
  ];

  function el(tag, cls, text) {
    const e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text != null) e.textContent = text;
    return e;
  }

  function button(label, icon, cls) {
    const b = el('button', 'ml-btn' + (cls ? ' ' + cls : ''));
    b.type = 'button';
    b.title = label;
    b.innerHTML = (icon ? Icons.get(icon, 15) : '') + (label ? '<span>' + label + '</span>' : '');
    return b;
  }

  const ESC = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' };
  function esc(s) { return String(s == null ? '' : s).replace(/[&<>"]/g, function (c) { return ESC[c]; }); }

  function two(n) { return n < 10 ? '0' + n : String(n); }

  function fmtTime(sec, short) {
    if (!sec) return '';
    const d = new Date(sec * 1000);
    const date = two(d.getDate()) + '.' + two(d.getMonth() + 1) + (short ? '' : '.' + d.getFullYear());
    return date + ' ' + two(d.getHours()) + ':' + two(d.getMinutes());
  }

  // Текст письма → HTML: ссылки http:// и адреса почты становятся кликабельными.
  const LINK_RE = /(https?:\/\/[^\s<>"']+)|([\w.+-]+@[\w-]+(?:\.[\w-]+)+)/g;
  function bodyHtml(text) {
    text = String(text == null ? '' : text);
    let html = '', last = 0, m;
    LINK_RE.lastIndex = 0;
    while ((m = LINK_RE.exec(text))) {
      html += esc(text.slice(last, m.index));
      if (m[1]) {
        let url = m[1], trail = '';
        const t = /[.,;:!?)]+$/.exec(url);
        if (t) { trail = t[0]; url = url.slice(0, -trail.length); }
        html += '<a class="ml-link" data-url="' + esc(url) + '">' + esc(url) + '</a>' + esc(trail);
      } else {
        html += '<a class="ml-link ml-addr" data-addr="' + esc(m[2]) + '">' + esc(m[2]) + '</a>';
      }
      last = LINK_RE.lastIndex;
    }
    html += esc(text.slice(last));
    return html;
  }

  function mount(body, arg) {
    const st = {
      folder: 'inbox', mails: { inbox: [], sent: [] }, selected: null, mail: null,
      mode: 'read', mono: false, busy: false
    };

    // --- разметка ---
    const root = el('div', 'ml');
    const bar = el('div', 'ml-bar');
    const btnCompose = button('Написать', 'plus', 'primary');
    const btnReply = button('Ответить', 'reply');
    const btnDelete = button('Удалить', 'trash');
    const btnRefresh = button('Обновить', 'reload');
    const barStat = el('span', 'ml-bar-stat', '');
    bar.append(btnCompose, btnReply, btnDelete, btnRefresh, barStat);

    const cols = el('div', 'ml-cols');
    const folders = el('div', 'ml-folders');
    const folderEls = {};
    for (const f of FOLDERS) {
      const fe = el('div', 'ml-folder' + (f.id === st.folder ? ' cur' : ''));
      fe.title = f.name;
      const ic = el('span', 'ml-folder-icon');
      ic.innerHTML = Icons.get(f.icon, 16);
      const nm = el('span', 'ml-folder-name', f.name);
      const cnt = el('span', 'ml-folder-count', '');
      fe.append(ic, nm, cnt);
      fe.addEventListener('click', function () { setFolder(f.id); });
      folders.appendChild(fe);
      folderEls[f.id] = { el: fe, count: cnt };
    }

    const listCol = el('div', 'ml-listcol');
    const listHead = el('div', 'ml-list-head', FOLDERS[0].name);
    const list = el('div', 'ml-list');
    list.tabIndex = 0;
    const listEmpty = el('div', 'ml-list-empty', 'Писем нет');
    listCol.append(listHead, list, listEmpty);

    const right = el('div', 'ml-right');
    // чтение
    const read = el('div', 'ml-read');
    const rHead = el('div', 'ml-read-head');
    const rSubject = el('div', 'ml-read-subject', '');
    const rMeta = el('div', 'ml-read-meta');
    function metaRow(k) {
      const r = el('div', 'ml-meta-row');
      const kk = el('span', 'ml-meta-k', k);
      const vv = el('span', 'ml-meta-v', '');
      r.append(kk, vv);
      rMeta.appendChild(r);
      return vv;
    }
    const rFrom = metaRow('От');
    const rTo = metaRow('Кому');
    const rTime = metaRow('Время');
    const rTools = el('div', 'ml-read-tools');
    const btnMono = button('Моноширинный', 'file-code', 'small toggle');
    rTools.append(btnMono);
    rHead.append(rSubject, rMeta, rTools);
    const rBody = el('div', 'ml-body');
    read.append(rHead, rBody);
    const readEmpty = el('div', 'ml-read-empty', 'Выберите письмо');

    // форма
    const form = el('form', 'ml-form');
    form.noValidate = true;
    const fTitle = el('div', 'ml-form-title', 'Новое письмо');
    function field(label, placeholder) {
      const row = el('label', 'ml-field');
      const k = el('span', 'ml-field-k', label);
      const inp = el('input', 'ml-input');
      inp.type = 'text';
      inp.spellcheck = false;
      inp.placeholder = placeholder || '';
      inp.setAttribute('autocomplete', 'off');
      row.append(k, inp);
      return { row: row, input: inp };
    }
    const fTo = field('Кому', 'имя@узел');
    const fSubj = field('Тема', '');
    const fBody = el('textarea', 'ml-textarea');
    fBody.spellcheck = false;
    fBody.placeholder = 'Текст письма';
    const fErr = el('div', 'ml-form-err', '');
    const fActions = el('div', 'ml-form-actions');
    const btnSend = button('Отправить', 'send', 'primary');
    btnSend.type = 'submit';
    const btnCancel = button('Отмена', 'close');
    fActions.append(btnSend, btnCancel);
    form.append(fTitle, fTo.row, fSubj.row, fBody, fErr, fActions);

    right.append(read, readEmpty, form);
    cols.append(folders, listCol, right);
    root.append(bar, cols);
    body.appendChild(root);

    // --- данные ---
    function currentList() { return st.mails[st.folder] || []; }

    async function refresh(silent) {
      if (st.busy) return;
      st.busy = true;
      btnRefresh.classList.add('spin');
      try {
        const results = await Promise.all(FOLDERS.map(function (f) {
          return BW.mail_list(f.id).then(function (r) { return Array.isArray(r) ? r : []; }, function () { return null; });
        }));
        FOLDERS.forEach(function (f, i) { if (results[i]) st.mails[f.id] = results[i]; });
        for (const k in st.mails) st.mails[k].sort(function (a, b) { return (b.time || 0) - (a.time || 0); });
        renderFolders();
        renderList();
        if (st.selected != null && !currentList().some(function (m) { return m.id === st.selected; })) {
          st.selected = null;
          st.mail = null;
          renderRead();
        }
      } catch (e) {
        if (!silent) Win.toast(String(e));
      } finally {
        st.busy = false;
        btnRefresh.classList.remove('spin');
      }
    }

    function unreadCount() {
      return (st.mails.inbox || []).filter(function (m) { return !m.read; }).length;
    }

    function renderFolders() {
      const unread = unreadCount();
      folderEls.inbox.count.textContent = unread ? String(unread) : '';
      folderEls.inbox.el.classList.toggle('has-new', unread > 0);
      folderEls.sent.count.textContent = '';
      for (const f of FOLDERS) folderEls[f.id].el.classList.toggle('cur', f.id === st.folder);
      listHead.textContent = FOLDERS.find(function (f) { return f.id === st.folder; }).name;
      barStat.textContent = unread ? unread + ' ' + plural(unread, 'новое', 'новых', 'новых') : '';
      Win.setTitle(unread ? 'Почта (' + unread + ')' : 'Почта');
    }

    function plural(n, a, b, c) {
      const m10 = n % 10, m100 = n % 100;
      if (m10 === 1 && m100 !== 11) return a;
      if (m10 >= 2 && m10 <= 4 && (m100 < 10 || m100 >= 20)) return b;
      return c;
    }

    function renderList() {
      const arr = currentList();
      list.textContent = '';
      const frag = document.createDocumentFragment();
      for (const m of arr) {
        const row = el('div', 'ml-item' + (m.read ? '' : ' unread') + (m.id === st.selected ? ' sel' : ''));
        row.dataset.id = m.id;
        const dot = el('span', 'ml-item-dot');
        const who = el('span', 'ml-item-who', st.folder === 'sent' ? (m.to || '') : (m.from || ''));
        who.title = who.textContent;
        const time = el('span', 'ml-item-time', fmtTime(m.time, true));
        const subj = el('span', 'ml-item-subj', m.subject || '(без темы)');
        subj.title = m.subject || '';
        row.append(dot, who, time, subj);
        row.addEventListener('click', function () { openMail(m.id); });
        row.addEventListener('contextmenu', function (e) {
          e.preventDefault();
          openMail(m.id);
          Win.menu([
            { label: 'Ответить', icon: 'reply', action: reply },
            { label: 'Удалить', icon: 'trash', action: deleteSel }
          ], e.clientX, e.clientY);
        });
        frag.appendChild(row);
      }
      list.appendChild(frag);
      listEmpty.classList.toggle('on', arr.length === 0);
    }

    function setFolder(id) {
      if (st.folder === id) return;
      st.folder = id;
      st.selected = null;
      st.mail = null;
      renderFolders();
      renderList();
      renderRead();
      BW.sound('click');
    }

    async function openMail(id) {
      st.selected = id;
      for (const row of list.children) row.classList.toggle('sel', Number(row.dataset.id) === id || row.dataset.id === String(id));
      let m = null;
      try { m = await BW.mail_read(id); }
      catch (e) { Win.toast(String(e)); return; }
      if (st.selected !== id) return;
      const local = currentList().find(function (x) { return x.id === id; });
      if (local) { local.read = true; }
      if (m && typeof m === 'object') st.mail = m; else st.mail = local;
      if (st.mail) st.mail.read = true;
      st.mode = 'read';
      renderFolders();
      renderList();
      renderRead();
    }

    function renderRead() {
      root.classList.toggle('compose', st.mode === 'compose');
      const m = st.mail;
      read.classList.toggle('on', !!m && st.mode === 'read');
      readEmpty.classList.toggle('on', !m && st.mode === 'read');
      btnReply.disabled = !m;
      btnDelete.disabled = !m;
      if (!m) return;
      rSubject.textContent = m.subject || '(без темы)';
      rFrom.textContent = m.from || '';
      rTo.textContent = m.to || '';
      rTime.textContent = fmtTime(m.time, false);
      rBody.innerHTML = bodyHtml(m.body);
      rBody.classList.toggle('mono', st.mono);
      rBody.scrollTop = 0;
    }

    // --- действия ---
    function compose(to, subject, text) {
      st.mode = 'compose';
      fTitle.textContent = subject && /^re:/i.test(subject) ? 'Ответ' : 'Новое письмо';
      fTo.input.value = to || '';
      fSubj.input.value = subject || '';
      fBody.value = text || '';
      fErr.textContent = '';
      renderRead();
      (to ? (subject ? fBody : fSubj.input) : fTo.input).focus();
      if (to && subject) fBody.setSelectionRange(0, 0);
    }

    function cancelCompose() {
      st.mode = 'read';
      renderRead();
      list.focus();
    }

    function reply() {
      const m = st.mail;
      if (!m) return;
      const subj = /^re:/i.test(m.subject || '') ? m.subject : 'Re: ' + (m.subject || '');
      const quoted = String(m.body || '').split('\n').map(function (L) { return '> ' + L; }).join('\n');
      const head = (m.from || '') + ' писал(а) ' + fmtTime(m.time, false) + ':';
      compose(m.from || '', subj, '\n\n' + head + '\n' + quoted);
    }

    async function send() {
      const to = fTo.input.value.trim(), subject = fSubj.input.value.trim(), text = fBody.value;
      fErr.textContent = '';
      if (!to) { fErr.textContent = 'Укажите получателя'; fTo.input.focus(); return; }
      btnSend.disabled = true;
      try {
        await BW.mail_send(to, subject, text);
        Win.toast('Письмо отправлено: ' + to);
        BW.sound('notify');
        st.mode = 'read';
        renderRead();
        await refresh(true);
      } catch (e) {
        fErr.textContent = String(e);
        BW.sound('error');
      } finally {
        btnSend.disabled = false;
      }
    }

    async function deleteSel() {
      const m = st.mail;
      if (!m) return;
      if (!(await Win.confirm('Удалить письмо «' + (m.subject || 'без темы') + '»?'))) return;
      const arr = currentList();
      const idx = arr.findIndex(function (x) { return x.id === m.id; });
      try { await BW.mail_delete(m.id); }
      catch (e) { Win.toast(String(e)); BW.sound('error'); return; }
      st.selected = null;
      st.mail = null;
      await refresh(true);
      const next = currentList()[Math.min(idx, currentList().length - 1)];
      if (next) openMail(next.id); else renderRead();
    }

    function moveSel(delta) {
      const arr = currentList();
      if (!arr.length) return;
      let i = arr.findIndex(function (x) { return x.id === st.selected; });
      i = i < 0 ? 0 : Math.max(0, Math.min(arr.length - 1, i + delta));
      openMail(arr[i].id);
      const row = list.children[i];
      if (row && row.scrollIntoView) row.scrollIntoView({ block: 'nearest' });
    }

    // --- события ---
    btnCompose.addEventListener('click', function () { compose('', '', ''); });
    btnReply.addEventListener('click', reply);
    btnDelete.addEventListener('click', deleteSel);
    btnRefresh.addEventListener('click', function () { refresh(false); });
    btnMono.addEventListener('click', function () {
      st.mono = !st.mono;
      btnMono.classList.toggle('on', st.mono);
      rBody.classList.toggle('mono', st.mono);
    });
    btnCancel.addEventListener('click', cancelCompose);
    form.addEventListener('submit', function (e) { e.preventDefault(); send(); });
    fBody.addEventListener('keydown', function (e) {
      if (e.ctrlKey && e.key === 'Enter') { e.preventDefault(); send(); }
    });

    rBody.addEventListener('click', function (e) {
      const a = e.target && e.target.closest ? e.target.closest('a.ml-link') : null;
      if (!a) return;
      e.preventDefault();
      if (a.dataset.url) BW.open_app('browser', a.dataset.url).catch(function (err) { Win.toast(String(err)); });
      else if (a.dataset.addr) compose(a.dataset.addr, '', '');
    });

    window.addEventListener('keydown', function (e) {
      const t = e.target;
      const inField = t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA');
      if (e.key === 'Escape' && st.mode === 'compose') { e.preventDefault(); cancelCompose(); return; }
      if (inField) return;
      if (e.ctrlKey || e.altKey) return;
      switch (e.key) {
        case 'F5': e.preventDefault(); refresh(false); return;
        case 'Delete': e.preventDefault(); deleteSel(); return;
        case 'ArrowDown': e.preventDefault(); moveSel(1); return;
        case 'ArrowUp': e.preventDefault(); moveSel(-1); return;
        case 'n': case 'N': case 'т': case 'Т': e.preventDefault(); compose('', '', ''); return;
        case 'r': case 'R': case 'к': case 'К': e.preventDefault(); reply(); return;
        default: return;
      }
    });

    BW.listen('mail_new', function () {
      BW.sound('notify');
      refresh(true);
    });

    Promise.resolve(BW.ready).then(function () {
      renderRead();
      refresh(true).then(function () {
        if (arg) {
          const a = String(arg);
          if (a.indexOf('@') > 0) compose(a, '', '');
          else if (a === 'sent' || a === 'inbox') setFolder(a);
          else if (a === 'compose') compose('', '', '');
        }
      });
    });
  }

  return { mount: mount };
})();

Apps.register('mail', {
  title: 'Почта', icon: 'mail', minWidth: 720, minHeight: 420,
  mount: function (body, arg) { MailApp.mount(body, arg); }
});
