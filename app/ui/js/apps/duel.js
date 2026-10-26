'use strict';
/* Сетевая игра: хост открывает порт, остальные подключаются по адресу.
   Режимы — дуэль (кто первым возьмёт узел) и команда (берём цели вместе).
   Локальный адрес — игра по локалке, публичный (с пробросом порта) — интернет. */
(function () {
  var STAGES = ['—', 'виден', 'скан', 'взят'];
  var els = {}, state = { role: 'None', snap: { phase: 'lobby', mode: 'duel', peers: [], targets: [], captured: [], winner: null }, addrs: [], port: 0, my_name: '', error: null };

  function el(tag, cls, html) {
    var e = document.createElement(tag);
    if (cls) e.className = cls;
    if (html != null) e.innerHTML = html;
    return e;
  }

  function apply(st) {
    if (!st) return;
    if (st.role) state.role = st.role;
    if (st.snap) state.snap = st.snap;
    if (st.addrs) state.addrs = st.addrs;
    if (st.port != null) state.port = st.port;
    if (st.my_name) state.my_name = st.my_name;
    state.error = st.error || null;
    render();
  }

  function refresh() { BW.net_status().then(apply, function () {}); }

  /* ---------- экраны ---------- */
  function lobbyChoice() {
    var box = el('div', 'duel-choice');
    box.appendChild(el('div', 'duel-lead', 'Сыграй с другим игроком: по локальной сети или через интернет.'));

    var cols = el('div', 'duel-cols');

    // хост
    var host = el('div', 'duel-card');
    host.appendChild(el('div', 'duel-card-h', Icons.get('net', 16) + '<span>Я создаю игру</span>'));
    var modeSel = el('select', 'input');
    [['duel', 'Дуэль — кто первым возьмёт узел'], ['team', 'Команда — берём цели вместе']].forEach(function (m) {
      var o = el('option'); o.value = m[0]; o.textContent = m[1]; modeSel.appendChild(o);
    });
    host.appendChild(modeSel);
    var portInp = el('input', 'input'); portInp.type = 'number'; portInp.value = '7777'; portInp.min = '1024'; portInp.max = '65535';
    host.appendChild(el('label', 'duel-mini', 'Порт'));
    host.appendChild(portInp);
    var hostBtn = el('button', 'btn btn-primary', Icons.get('play', 16) + '<span>Открыть игру</span>');
    hostBtn.addEventListener('click', function () {
      BW.sound('click');
      BW.net_host(modeSel.value, parseInt(portInp.value, 10) || 7777).then(function (r) {
        Win.toast('Игра открыта на порту ' + (r && r.port), 'ok');
        refresh();
      }, function (e) { Win.toast('Не удалось: ' + e, 'err'); });
    });
    host.appendChild(hostBtn);
    cols.appendChild(host);

    // клиент
    var join = el('div', 'duel-card');
    join.appendChild(el('div', 'duel-card-h', Icons.get('link', 16) + '<span>Я подключаюсь</span>'));
    var addrInp = el('input', 'input'); addrInp.placeholder = 'адрес хоста, напр. 192.168.0.10:7777';
    join.appendChild(el('label', 'duel-mini', 'Адрес хоста'));
    join.appendChild(addrInp);
    join.appendChild(el('div', 'duel-hint', 'По локалке — локальный адрес хоста. Через интернет — его публичный адрес (нужен проброс порта).'));
    var joinBtn = el('button', 'btn btn-primary', Icons.get('forward', 16) + '<span>Подключиться</span>');
    joinBtn.addEventListener('click', function () {
      var addr = (addrInp.value || '').trim();
      if (!addr) { Win.toast('Укажи адрес хоста', 'err'); return; }
      BW.sound('click');
      BW.net_join(addr).then(function () { Win.toast('Подключение…', 'ok'); refresh(); }, function (e) { Win.toast('Не удалось: ' + e, 'err'); });
    });
    join.appendChild(joinBtn);
    cols.appendChild(join);

    box.appendChild(cols);
    return box;
  }

  function roster(withBars) {
    var list = el('div', 'duel-roster');
    (state.snap.peers || []).forEach(function (p) {
      var r = el('div', 'duel-peer' + (p.name === state.my_name ? ' me' : ''));
      var n = el('div', 'duel-peer-n'); n.textContent = p.name + (p.name === state.my_name ? ' (ты)' : ''); r.appendChild(n);
      if (withBars) {
        if (state.snap.mode === 'team') {
          var sc = el('div', 'duel-peer-s'); sc.textContent = 'целей взято: ' + (p.score || 0); r.appendChild(sc);
        } else {
          var bar = el('div', 'duel-bar');
          var fill = el('div', 'duel-bar-f'); fill.style.width = ((p.stage || 0) / 3 * 100) + '%'; bar.appendChild(fill);
          r.appendChild(bar);
          var lab = el('div', 'duel-peer-s'); lab.textContent = STAGES[p.stage || 0]; r.appendChild(lab);
        }
      }
      list.appendChild(r);
    });
    return list;
  }

  function lobbyWaiting() {
    var box = el('div', 'duel-wait');
    if (state.role === 'Host') {
      var addrLine = (state.addrs || []).map(function (a) { return a + ':' + state.port; }).join('   ');
      box.appendChild(el('div', 'duel-lead', 'Игра открыта. Скажи сопернику адрес:'));
      box.appendChild(el('div', 'duel-addr mono', addrLine || ('порт ' + state.port)));
      box.appendChild(el('div', 'duel-hint', 'Режим: ' + (state.snap.mode === 'team' ? 'команда' : 'дуэль') + '. Через интернет дай публичный адрес и пробрось порт ' + state.port + '.'));
    } else {
      box.appendChild(el('div', 'duel-lead', 'Подключились. Ждём, пока хост начнёт партию.'));
    }
    box.appendChild(el('div', 'duel-sub', 'В игре:'));
    box.appendChild(roster(false));

    var btns = el('div', 'duel-actions');
    if (state.role === 'Host') {
      var startBtn = el('button', 'btn btn-primary', Icons.get('play', 16) + '<span>Начать партию</span>');
      if ((state.snap.peers || []).length < 2) { startBtn.disabled = true; startBtn.title = 'нужен хотя бы один соперник'; }
      startBtn.addEventListener('click', function () {
        BW.sound('click');
        BW.net_start().then(function () { Win.toast('Партия началась', 'ok'); refresh(); }, function (e) { Win.toast('Не удалось: ' + e, 'err'); });
      });
      btns.appendChild(startBtn);
    }
    var leave = el('button', 'btn', Icons.get('close', 16) + '<span>Отменить</span>');
    leave.addEventListener('click', doLeave);
    btns.appendChild(leave);
    box.appendChild(btns);
    return box;
  }

  function playing() {
    var box = el('div', 'duel-play');
    var head = el('div', 'duel-lead');
    head.textContent = state.snap.mode === 'team' ? 'Команда: возьмите все цели.' : 'Дуэль: возьми узел первым.';
    box.appendChild(head);

    var tg = el('div', 'duel-targets');
    tg.appendChild(el('div', 'duel-sub', state.snap.mode === 'team' ? 'Цели команды:' : 'Цель:'));
    (state.snap.targets || []).forEach(function (t) {
      var taken = (state.snap.captured || []).indexOf(t) >= 0;
      var row = el('div', 'duel-target' + (taken ? ' taken' : ''));
      row.innerHTML = Icons.get(taken ? 'check' : 'dot', 14) + '<span class="mono"></span>';
      row.querySelector('span').textContent = t;
      tg.appendChild(row);
    });
    box.appendChild(tg);

    box.appendChild(el('div', 'duel-sub', 'Игроки:'));
    box.appendChild(roster(true));

    var btns = el('div', 'duel-actions');
    var term = el('button', 'btn btn-primary', Icons.get('terminal', 16) + '<span>Открыть терминал</span>');
    term.addEventListener('click', function () { BW.sound('click'); BW.open_app('terminal', null, null); });
    btns.appendChild(term);
    var leave = el('button', 'btn btn-danger', Icons.get('close', 16) + '<span>Выйти из партии</span>');
    leave.addEventListener('click', doLeave);
    btns.appendChild(leave);
    box.appendChild(btns);
    return box;
  }

  function done() {
    var box = el('div', 'duel-done');
    var win = state.snap.winner;
    var meWon = state.snap.mode === 'team' ? true : (win === state.my_name);
    box.appendChild(el('div', 'duel-banner ' + (meWon ? 'win' : 'lose'), meWon ? 'Победа' : 'Поражение'));
    var who = state.snap.mode === 'team' ? 'Команда взяла все цели.' : ('Победил ' + (win || '—') + '.');
    box.appendChild(el('div', 'duel-lead', who));
    box.appendChild(roster(true));
    var btns = el('div', 'duel-actions');
    var leave = el('button', 'btn btn-primary', Icons.get('check', 16) + '<span>Готово</span>');
    leave.addEventListener('click', doLeave);
    btns.appendChild(leave);
    box.appendChild(btns);
    return box;
  }

  function doLeave() {
    BW.sound('click');
    BW.net_leave().then(function () { state.role = 'None'; refresh(); }, function () { state.role = 'None'; render(); });
  }

  function render() {
    if (!els.root) return;
    els.root.innerHTML = '';
    if (state.error) {
      var err = el('div', 'duel-err', Icons.get('warning', 16) + '<span></span>');
      err.querySelector('span').textContent = state.error;
      els.root.appendChild(err);
    }
    var phase = state.snap && state.snap.phase;
    if (state.role === 'None') {
      els.root.appendChild(lobbyChoice());
    } else if (phase === 'playing') {
      els.root.appendChild(playing());
    } else if (phase === 'done') {
      els.root.appendChild(done());
    } else {
      els.root.appendChild(lobbyWaiting());
    }
  }

  function mount(body) {
    body.classList.add('duel-body');
    var wrap = el('div', 'duel');
    var head = el('div', 'duel-head');
    head.appendChild(el('span', 'duel-head-ico', Icons.get('duel', 18)));
    head.appendChild(el('span', 'duel-head-t', '<b>Сетевая игра</b> — вдвоём по локалке или через интернет'));
    wrap.appendChild(head);
    els.root = el('div', 'duel-root');
    wrap.appendChild(els.root);
    body.appendChild(wrap);

    render();
    refresh();
    els.un1 = BW.listen('net_event', function (p) { apply(p); });
    els.un2 = BW.listen('net_match_start', function () { refresh(); });
    window.addEventListener('pagehide', function () {
      if (els.un1) els.un1.then(function (f) { f && f(); });
      if (els.un2) els.un2.then(function (f) { f && f(); });
    });
  }

  Apps.register('duel', { title: 'Сетевая игра', icon: 'duel', minWidth: 620, minHeight: 480, mount: mount });
})();
