'use strict';
/* Сетевая игра: хост открывает порт, остальные подключаются по адресу.
   Режимы — дуэль (кто первым возьмёт узел) и команда (берём цели вместе).
   Локальный адрес — игра по локалке, публичный (с пробросом порта) — интернет. */
(function () {
  var STAGES = L(['—', 'виден', 'скан', 'взят'], ['—', 'seen', 'scanned', 'captured']);
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
    box.appendChild(el('div', 'duel-lead', L('Сыграй с другим игроком: по локальной сети или через интернет.', 'Play against another player: over the local network or the internet.')));

    var cols = el('div', 'duel-cols');

    // хост
    var host = el('div', 'duel-card');
    host.appendChild(el('div', 'duel-card-h', Icons.get('net', 16) + '<span>' + L('Я создаю игру', 'I host a game') + '</span>'));
    var modeSel = el('select', 'input');
    [['duel', L('Дуэль — кто первым возьмёт узел', 'Duel — first to capture the host wins')], ['team', L('Команда — берём цели вместе', 'Team — capture the targets together')]].forEach(function (m) {
      var o = el('option'); o.value = m[0]; o.textContent = m[1]; modeSel.appendChild(o);
    });
    host.appendChild(modeSel);
    var portInp = el('input', 'input'); portInp.type = 'number'; portInp.value = '7777'; portInp.min = '1024'; portInp.max = '65535';
    host.appendChild(el('label', 'duel-mini', L('Порт', 'Port')));
    host.appendChild(portInp);
    var hostBtn = el('button', 'btn btn-primary', Icons.get('play', 16) + '<span>' + L('Открыть игру', 'Open game') + '</span>');
    hostBtn.addEventListener('click', function () {
      BW.sound('click');
      BW.net_host(modeSel.value, parseInt(portInp.value, 10) || 7777).then(function (r) {
        Win.toast(L('Игра открыта на порту ', 'Game open on port ') + (r && r.port), 'ok');
        refresh();
      }, function (e) { Win.toast(L('Не удалось: ', 'Failed: ') + e, 'err'); });
    });
    host.appendChild(hostBtn);
    cols.appendChild(host);

    // клиент
    var join = el('div', 'duel-card');
    join.appendChild(el('div', 'duel-card-h', Icons.get('link', 16) + '<span>' + L('Я подключаюсь', 'I join a game') + '</span>'));
    var addrInp = el('input', 'input'); addrInp.placeholder = L('адрес хоста, напр. 192.168.0.10:7777', 'host address, e.g. 192.168.0.10:7777');
    join.appendChild(el('label', 'duel-mini', L('Адрес хоста', 'Host address')));
    join.appendChild(addrInp);
    join.appendChild(el('div', 'duel-hint', L('По локалке — локальный адрес хоста. Через интернет — его публичный адрес (нужен проброс порта).', 'On a LAN — the local address of the host. Over the internet — its public address (port forwarding required).')));
    var joinBtn = el('button', 'btn btn-primary', Icons.get('forward', 16) + '<span>' + L('Подключиться', 'Connect') + '</span>');
    joinBtn.addEventListener('click', function () {
      var addr = (addrInp.value || '').trim();
      if (!addr) { Win.toast(L('Укажи адрес хоста', 'Enter the host address'), 'err'); return; }
      BW.sound('click');
      BW.net_join(addr).then(function () { Win.toast(L('Подключение…', 'Connecting…'), 'ok'); refresh(); }, function (e) { Win.toast(L('Не удалось: ', 'Failed: ') + e, 'err'); });
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
      var n = el('div', 'duel-peer-n'); n.textContent = p.name + (p.name === state.my_name ? L(' (ты)', ' (you)') : ''); r.appendChild(n);
      if (withBars) {
        if (state.snap.mode === 'team') {
          var sc = el('div', 'duel-peer-s'); sc.textContent = L('целей взято: ', 'targets captured: ') + (p.score || 0); r.appendChild(sc);
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
      box.appendChild(el('div', 'duel-lead', L('Игра открыта. Скажи сопернику адрес:', 'Game open. Give your opponent the address:')));
      box.appendChild(el('div', 'duel-addr mono', addrLine || (L('порт ', 'port ') + state.port)));
      box.appendChild(el('div', 'duel-hint', L('Режим: ', 'Mode: ') + (state.snap.mode === 'team' ? L('команда', 'team') : L('дуэль', 'duel')) + L('. Через интернет дай публичный адрес и пробрось порт ', '. Over the internet, give your public address and forward port ') + state.port + '.'));
    } else {
      box.appendChild(el('div', 'duel-lead', L('Подключились. Ждём, пока хост начнёт партию.', 'Connected. Waiting for the host to start the match.')));
    }
    box.appendChild(el('div', 'duel-sub', L('В игре:', 'In game:')));
    box.appendChild(roster(false));

    var btns = el('div', 'duel-actions');
    if (state.role === 'Host') {
      var startBtn = el('button', 'btn btn-primary', Icons.get('play', 16) + '<span>' + L('Начать партию', 'Start match') + '</span>');
      if ((state.snap.peers || []).length < 2) { startBtn.disabled = true; startBtn.title = L('нужен хотя бы один соперник', 'at least one opponent is needed'); }
      startBtn.addEventListener('click', function () {
        BW.sound('click');
        BW.net_start().then(function () { Win.toast(L('Партия началась', 'Match started'), 'ok'); refresh(); }, function (e) { Win.toast(L('Не удалось: ', 'Failed: ') + e, 'err'); });
      });
      btns.appendChild(startBtn);
    }
    var leave = el('button', 'btn', Icons.get('close', 16) + '<span>' + L('Отменить', 'Cancel') + '</span>');
    leave.addEventListener('click', doLeave);
    btns.appendChild(leave);
    box.appendChild(btns);
    return box;
  }

  function playing() {
    var box = el('div', 'duel-play');
    var head = el('div', 'duel-lead');
    head.textContent = state.snap.mode === 'team' ? L('Команда: возьмите все цели.', 'Team: capture all targets.') : L('Дуэль: возьми узел первым.', 'Duel: capture the host first.');
    box.appendChild(head);

    var tg = el('div', 'duel-targets');
    tg.appendChild(el('div', 'duel-sub', state.snap.mode === 'team' ? L('Цели команды:', 'Team targets:') : L('Цель:', 'Target:')));
    (state.snap.targets || []).forEach(function (t) {
      var taken = (state.snap.captured || []).indexOf(t) >= 0;
      var row = el('div', 'duel-target' + (taken ? ' taken' : ''));
      row.innerHTML = Icons.get(taken ? 'check' : 'dot', 14) + '<span class="mono"></span>';
      row.querySelector('span').textContent = t;
      tg.appendChild(row);
    });
    box.appendChild(tg);

    box.appendChild(el('div', 'duel-sub', L('Игроки:', 'Players:')));
    box.appendChild(roster(true));

    var btns = el('div', 'duel-actions');
    var term = el('button', 'btn btn-primary', Icons.get('terminal', 16) + '<span>' + L('Открыть терминал', 'Open terminal') + '</span>');
    term.addEventListener('click', function () { BW.sound('click'); BW.open_app('terminal', null, null); });
    btns.appendChild(term);
    var leave = el('button', 'btn btn-danger', Icons.get('close', 16) + '<span>' + L('Выйти из партии', 'Leave match') + '</span>');
    leave.addEventListener('click', doLeave);
    btns.appendChild(leave);
    box.appendChild(btns);
    return box;
  }

  function done() {
    var box = el('div', 'duel-done');
    var win = state.snap.winner;
    var meWon = state.snap.mode === 'team' ? true : (win === state.my_name);
    box.appendChild(el('div', 'duel-banner ' + (meWon ? 'win' : 'lose'), meWon ? L('Победа', 'Victory') : L('Поражение', 'Defeat')));
    var who = state.snap.mode === 'team' ? L('Команда взяла все цели.', 'The team captured all targets.') : (L('Победил ', 'Winner: ') + (win || '—') + '.');
    box.appendChild(el('div', 'duel-lead', who));
    box.appendChild(roster(true));
    var btns = el('div', 'duel-actions');
    var leave = el('button', 'btn btn-primary', Icons.get('check', 16) + '<span>' + L('Готово', 'Done') + '</span>');
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
    head.appendChild(el('span', 'duel-head-t', L('<b>Сетевая игра</b> — вдвоём по локалке или через интернет', '<b>Multiplayer</b> — two players over a LAN or the internet')));
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

  Apps.register('duel', { title: L('Сетевая игра', 'Multiplayer'), icon: 'duel', minWidth: 620, minHeight: 480, mount: mount });
})();
