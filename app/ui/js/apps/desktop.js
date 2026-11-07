'use strict';
/* Рабочий стол: фон на canvas, полоса состояния, док с приложениями и открытыми окнами, уведомления, меню. */
(function () {
  var DOCK_APPS = [
    { id: 'terminal', label: L('Терминал', 'Terminal'), multi: true },
    { id: 'editor', label: L('Редактор', 'Editor'), multi: true },
    { id: 'files', label: L('Файлы', 'Files'), multi: true },
    { id: 'browser', label: L('Браузер', 'Browser'), multi: true },
    { id: 'mail', label: L('Почта', 'Mail') },
    { id: 'monitor', label: L('Монитор', 'System monitor') },
    { id: 'map', label: L('Карта сети', 'Network map') },
    { id: 'plugins', label: L('Плагины', 'Plugins') },
    { id: 'scenario', label: L('Редактор сценариев', 'Scenario editor') },
    { id: 'duel', label: L('Сетевая игра', 'Multiplayer') },
    { id: 'settings', label: L('Настройки', 'Settings') },
    { id: 'help', label: L('Справка', 'Help') }
  ];
  var LAYOUTS = [
    { id: 'grid', label: L('Сетка', 'Grid'), icon: 'grid' },
    { id: 'columns', label: L('Колонки', 'Columns'), icon: 'columns' },
    { id: 'rows', label: L('Ряды', 'Rows'), icon: 'rows' },
    { id: 'cascade', label: L('Каскад', 'Cascade'), icon: 'cascade' }
  ];
  var HISTORY = 60;

  var els = {};
  var meters = {};
  var memTotal = 0;
  var windows = [];
  var mailBadge = null;
  var quitPending = false;

  function quitGame() {
    if (quitPending) return;
    quitPending = true;
    // Native app windows can cover an HTML confirmation on the desktop.
    // The quit command saves the world before closing every game window.
    BW.invoke('quit').catch(function (e) {
      Win.toast(L('Не удалось выйти: ', 'Failed to quit: ') + String(e), 'err');
    }).finally(function () { quitPending = false; });
  }

  function el(tag, cls, html) {
    var e = document.createElement(tag);
    if (cls) e.className = cls;
    if (html != null) e.innerHTML = html;
    return e;
  }

  function swallow() {}

  /* ---------- разметка ---------- */
  function build(root) {
    root.innerHTML = '';
    var desk = el('div', 'desk');
    desk.setAttribute('data-tauri-drag-region', '');
    var canvas = el('canvas', 'desk-fx');
    desk.appendChild(canvas);

    var top = el('header', 'desk-top');
    top.innerHTML =
      '<div class="desk-top-left">' +
        '<span class="desk-brand">BLACKWIRE</span><span class="desk-ver"></span>' +
        '<span class="desk-top-sep"></span>' +
        '<span class="desk-host mono ellipsis">—</span>' +
      '</div>' +
      '<div class="desk-top-center"><span class="desk-clock mono">--:--:--</span><span class="desk-date"></span></div>' +
      '<div class="desk-top-right"></div>';
    desk.appendChild(top);

    var right = top.querySelector('.desk-top-right');
    ['cpu', 'mem', 'net'].forEach(function (k) {
      var m = el('div', 'meter meter-' + k);
      m.innerHTML = '<span class="meter-ico">' + Icons.get(k, 14) + '</span><canvas class="meter-graph"></canvas><span class="meter-val mono">—</span>';
      m.title = k === 'cpu' ? L('Процессор', 'CPU') : k === 'mem' ? L('Память', 'Memory') : L('Сеть', 'Network');
      right.appendChild(m);
      meters[k] = { canvas: m.querySelector('canvas'), val: m.querySelector('.meter-val'), data: [] };
    });

    var exitButton = el('button', 'desk-exit', Icons.get('power', 16) + '<span>' + L('Выйти', 'Quit') + '</span>');
    exitButton.type = 'button';
    exitButton.title = L('Сохранить и выйти из игры', 'Save and quit the game');
    exitButton.setAttribute('aria-label', exitButton.title);
    exitButton.addEventListener('click', quitGame);
    right.appendChild(exitButton);
    right.appendChild(Win.windowControls({ close: quitGame, closeLabel: L('Сохранить и выйти из игры', 'Save and quit the game') }));

    var notify = el('aside', 'desk-notify');
    desk.appendChild(notify);

    var dockWrap = el('footer', 'desk-dockwrap');
    var dock = el('div', 'dock');
    var apps = el('div', 'dock-apps');
    DOCK_APPS.forEach(function (a) {
      var b = el('button', 'dock-btn');
      b.dataset.app = a.id;
      b.dataset.tip = a.label;
      b.innerHTML = Icons.get(a.id, 24);
      if (a.id === 'mail') {
        mailBadge = el('span', 'dock-badge hidden');
        b.appendChild(mailBadge);
      }
      b.addEventListener('click', function () {
        BW.sound('open');
        if (a.multi) BW.open_app(a.id); else Apps.openOrFocus(a.id, null);
      });
      b.addEventListener('contextmenu', function (e) {
        e.preventDefault(); e.stopPropagation();
        Win.menu([{ label: L('Открыть новое окно', 'Open new window'), icon: 'plus', action: function () { BW.open_app(a.id); } }], e.clientX, e.clientY);
      });
      apps.appendChild(b);
    });
    dock.appendChild(apps);
    dock.appendChild(el('div', 'dock-sep hidden'));
    dock.appendChild(el('div', 'dock-wins'));
    dockWrap.appendChild(dock);
    desk.appendChild(dockWrap);
    root.appendChild(desk);

    els.desk = desk; els.canvas = canvas; els.notify = notify;
    els.host = top.querySelector('.desk-host'); els.ver = top.querySelector('.desk-ver');
    els.clock = top.querySelector('.desk-clock'); els.date = top.querySelector('.desk-date');
    els.wins = dock.querySelector('.dock-wins'); els.dockSep = dock.querySelector('.dock-sep');
  }

  /* ---------- открытые окна в доке ---------- */
  function renderWindows() {
    var list = windows.filter(function (w) { return w.app !== 'desktop' && w.app !== 'login'; });
    els.wins.innerHTML = '';
    els.dockSep.classList.toggle('hidden', !list.length);
    els.wins.classList.toggle('hidden', !list.length);
    list.forEach(function (w) {
      var b = el('button', 'dock-win' + (w.minimized ? ' min' : ''));
      b.innerHTML = '<span class="dock-win-ico">' + Icons.get(w.app, 16) + '</span><span class="dock-win-title ellipsis"></span>';
      b.querySelector('.dock-win-title').textContent = w.title || w.app;
      b.title = w.title || w.app;
      b.addEventListener('click', function () { BW.sound('click'); BW.focus_window(w.label).catch(swallow); });
      b.addEventListener('contextmenu', function (e) {
        e.preventDefault(); e.stopPropagation();
        Win.menu([
          { label: L('Поднять', 'Bring to front'), icon: 'up', action: function () { BW.focus_window(w.label).catch(swallow); } },
          null,
          { label: L('Закрыть', 'Close'), icon: 'close', action: function () { BW.close_window(w.label).catch(swallow); } }
        ], e.clientX, e.clientY);
      });
      els.wins.appendChild(b);
    });
  }

  function loadWindows() {
    BW.list_windows().then(function (list) { windows = list || []; renderWindows(); }, swallow);
  }

  /* ---------- почта: непрочитанные ---------- */
  function setUnread(n) {
    if (!mailBadge) return;
    n = parseInt(n, 10) || 0;
    mailBadge.textContent = n > 99 ? '99+' : String(n);
    mailBadge.classList.toggle('hidden', n <= 0);
  }

  function loadUnread() {
    return BW.mail_unread_count().then(setUnread, swallow);
  }

  /* ---------- уведомления ---------- */
  function notify(p) {
    if (!p) return;
    var card = el('div', 'notice');
    card.innerHTML = '<span class="notice-ico">' + Icons.get('bell', 16) + '</span><div class="notice-body"><div class="notice-title ellipsis"></div><div class="notice-text"></div></div>';
    card.querySelector('.notice-title').textContent = p.title || L('Сообщение', 'Message');
    card.querySelector('.notice-text').textContent = p.text || '';
    els.notify.appendChild(card);
    while (els.notify.children.length > 5) els.notify.removeChild(els.notify.firstChild);
    requestAnimationFrame(function () { card.classList.add('show'); });
    BW.sound('notify');
    var timer = null;
    function hide() {
      card.classList.remove('show');
      setTimeout(function () { if (card.parentNode) card.parentNode.removeChild(card); }, 300);
    }
    function arm() { timer = setTimeout(hide, 6000); }
    card.addEventListener('mouseenter', function () { clearTimeout(timer); });
    card.addEventListener('mouseleave', arm);
    card.addEventListener('click', hide);
    arm();
  }

  /* ---------- время и графики ---------- */
  function pad(n) { return (n < 10 ? '0' : '') + n; }

  function setTime(t) {
    if (typeof t === 'number') {
      if (t > 1e9) {
        var d = new Date(t * 1000);
        els.clock.textContent = pad(d.getHours()) + ':' + pad(d.getMinutes()) + ':' + pad(d.getSeconds());
        els.date.textContent = pad(d.getDate()) + '.' + pad(d.getMonth() + 1) + '.' + d.getFullYear();
      } else {
        var s = Math.floor(t), days = Math.floor(s / 86400);
        els.clock.textContent = pad(Math.floor(s / 3600) % 24) + ':' + pad(Math.floor(s / 60) % 60) + ':' + pad(s % 60);
        els.date.textContent = L('день ', 'day ') + (days + 1);
      }
    } else if (t != null) {
      els.clock.textContent = String(t);
    }
  }

  function pushMeter(k, v) {
    var m = meters[k];
    m.data.push(v);
    if (m.data.length > HISTORY) m.data.shift();
  }

  function drawMeter(k, max, label) {
    var m = meters[k], c = m.canvas;
    var dpr = window.devicePixelRatio || 1;
    var w = c.clientWidth || 64, h = c.clientHeight || 22;
    if (c.width !== Math.round(w * dpr) || c.height !== Math.round(h * dpr)) {
      c.width = Math.round(w * dpr); c.height = Math.round(h * dpr);
    }
    var g = c.getContext('2d');
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    g.clearRect(0, 0, w, h);
    var accent = Theme.color('--accent'), accent2 = Theme.color('--accent2');
    var n = m.data.length;
    if (n < 2) { m.val.textContent = label; return; }
    var step = w / (HISTORY - 1);
    var x0 = w - (n - 1) * step;
    g.beginPath();
    for (var i = 0; i < n; i++) {
      var x = x0 + i * step, y = h - 1 - (Math.min(m.data[i], max) / max) * (h - 3);
      if (i === 0) g.moveTo(x, y); else g.lineTo(x, y);
    }
    g.lineWidth = 1.2;
    g.strokeStyle = Theme.rgba(accent, .9);
    g.shadowColor = Theme.rgba(accent, .8);
    g.shadowBlur = 4;
    g.stroke();
    g.shadowBlur = 0;
    g.lineTo(x0 + (n - 1) * step, h); g.lineTo(x0, h); g.closePath();
    var grad = g.createLinearGradient(0, 0, 0, h);
    grad.addColorStop(0, Theme.rgba(accent, .28)); grad.addColorStop(1, Theme.rgba(accent, 0));
    g.fillStyle = grad; g.fill();
    var lx = x0 + (n - 1) * step, ly = h - 1 - (Math.min(m.data[n - 1], max) / max) * (h - 3);
    g.fillStyle = Theme.rgba(accent2, 1);
    g.beginPath(); g.arc(lx, ly, 1.6, 0, Math.PI * 2); g.fill();
    m.val.textContent = label;
  }

  function fmtRate(v) {
    if (v >= 1024) return (v / 1024).toFixed(1) + L(' МБ/с', ' MB/s');
    return Math.round(v) + L(' КБ/с', ' KB/s');
  }

  function tick(p) {
    if (!p) return;
    setTime(p.time);
    if (typeof p.mail_unread === 'number') setUnread(p.mail_unread);
    // Rust шлёт процессор в процентах, память — в мегабайтах (mem_total — тоже)
    var cpu = Math.max(0, Math.min(100, +p.cpu || 0));
    var total = +p.mem_total || memTotal || 0;
    var mem = total ? (+p.mem || 0) / total * 100 : 0;
    var net = (+p.net_in || 0) + (+p.net_out || 0);
    pushMeter('cpu', cpu); pushMeter('mem', mem); pushMeter('net', net);
    drawMeter('cpu', 100, Math.round(cpu) + '%');
    drawMeter('mem', 100, Math.round(mem) + '%');
    var netMax = Math.max(16, Math.max.apply(null, meters.net.data) * 1.1);
    drawMeter('net', netMax, fmtRate(net));
  }

  /* ---------- меню фона ---------- */
  function currentMonitorIndex(mons) {
    var sx = window.screenX + 20, sy = window.screenY + 20;
    for (var i = 0; i < mons.length; i++) {
      var m = mons[i];
      if (sx >= m.x && sx < m.x + m.width && sy >= m.y && sy < m.y + m.height) return m.index;
    }
    return null;
  }

  function showMenu(x, y) {
    BW.monitors().then(function (mons) {
      mons = mons || [];
      var def = BW.settings ? BW.settings.default_monitor : 0;
      var layoutItems = LAYOUTS.map(function (l) {
        return { label: l.label, icon: l.icon, action: function () { BW.arrange_windows(def, l.id).catch(swallow); } };
      });
      var cur = currentMonitorIndex(mons);
      var monItems = mons.map(function (m) {
        return {
          label: (m.name || (L('Монитор ', 'Monitor ') + (m.index + 1))) + ' · ' + m.width + '×' + m.height + (m.primary ? L(' · основной', ' · primary') : ''),
          icon: 'monitor-screen',
          disabled: m.index === cur,
          action: function () {
            BW.desktop_on(m.index, true).then(function () {
              if (cur !== null && cur !== m.index) return BW.desktop_on(cur, false);
            }).catch(swallow);
          }
        };
      });
      Win.menu([
        { label: L('Новый терминал', 'New terminal'), icon: 'terminal', action: function () { BW.open_app('terminal'); } },
        { label: L('Редактор', 'Editor'), icon: 'editor', action: function () { BW.open_app('editor'); } },
        { label: L('Файлы', 'Files'), icon: 'files', action: function () { BW.open_app('files'); } },
        null,
        { label: L('Разложить окна', 'Arrange windows'), icon: 'layout', children: layoutItems },
        { label: L('Настройки', 'Settings'), icon: 'settings', action: function () { Apps.openOrFocus('settings', null); } },
        { label: L('Сменить монитор подложки', 'Move desktop to monitor'), icon: 'monitor-screen', children: monItems.length ? monItems : [{ label: L('Один монитор', 'Single monitor'), disabled: true }] },
        null,
        { label: L('Выйти из игры', 'Quit game'), icon: 'power', action: quitGame }
      ], x, y);
    }, swallow);
  }

  /* ---------- запуск ---------- */
  function mount(root) {
    build(root);
    document.title = 'Blackwire';

    FX.start(els.canvas, BW.settings);
    BW.onSettings(function (s) { FX.update(s); });

    BW.boot().then(function (b) {
      if (!b) return;
      if (b.version) els.ver.textContent = 'v' + b.version;
      if (b.os_name) els.desk.querySelector('.desk-brand').textContent = b.os_name;
      var handle = b.handle || 'ghost';
      BW.sys_status().then(function (st) {
        memTotal = st && st.mem_total || 0;
        els.host.textContent = handle + '@' + (st && st.hostname || 'localhost');
        els.host.title = st && st.ip ? st.ip : '';
      }, function () { els.host.textContent = handle; });
    }, swallow);

    loadWindows();
    // события могут прийти с опозданием и устаревшим списком — всегда перечитываем из Rust;
    // заголовки окон меняются (терминал показывает папку), поэтому ещё и обновляем раз в пару секунд
    BW.listen('windows_changed', loadWindows);
    setInterval(function () { if (!document.hidden) loadWindows(); }, 2000);
    BW.listen('world_tick', tick);
    BW.listen('notify', notify);
    BW.listen('mail_new', loadUnread);
    BW.listen('mail_changed', loadUnread);
    loadUnread();

    els.desk.addEventListener('contextmenu', function (e) {
      if (e.target.closest('.dock, .desk-top, .desk-notify')) return;
      e.preventDefault();
      showMenu(e.clientX, e.clientY);
    });
    els.desk.addEventListener('mousedown', function (e) {
      if (e.button === 0 && !e.target.closest('.dock, .desk-top, .desk-notify, .menu, .overlay')) Win.closeMenus();
    });
    window.addEventListener('resize', function () {
      Object.keys(meters).forEach(function (k) {
        var max = k === 'net' ? Math.max(16, Math.max.apply(null, meters[k].data.concat([0])) * 1.1) : 100;
        drawMeter(k, max, meters[k].val.textContent);
      });
    });
  }

  Apps.register('desktop', { title: L('Рабочий стол', 'Desktop'), icon: 'grid', mount: mount });
})();
