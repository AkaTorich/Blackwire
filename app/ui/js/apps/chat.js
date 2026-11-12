'use strict';
/* Чат игроков онлайн-сервера: кто сейчас в сети и общий разговор. */
(function () {
  var els = {}, me = '', lines = [];

  function el(tag, cls, text) {
    var e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text != null) e.textContent = text;
    return e;
  }

  function hhmm(t) {
    if (!t) return '';
    var d = new Date(t * 1000);
    return ('0' + d.getHours()).slice(-2) + ':' + ('0' + d.getMinutes()).slice(-2);
  }

  function addLine(m) {
    lines.push(m);
    if (lines.length > 200) lines.shift();
    var row = el('div', 'chat-line' + (m.from === me ? ' mine' : ''));
    row.appendChild(el('span', 'chat-time mono', hhmm(m.time)));
    row.appendChild(el('span', 'chat-from mono', m.from));
    row.appendChild(el('span', 'chat-text', m.text));
    els.log.appendChild(row);
    while (els.log.children.length > 200) els.log.removeChild(els.log.firstChild);
    els.log.scrollTop = els.log.scrollHeight;
  }

  function setWho(list) {
    els.who.innerHTML = '';
    (list || []).forEach(function (n) {
      var p = el('div', 'chat-who-item mono' + (n === me ? ' me' : ''));
      p.innerHTML = Icons.get('user', 14);
      p.appendChild(el('span', null, n));
      els.who.appendChild(p);
    });
    els.count.textContent = L('В сети: ', 'Online: ') + (list || []).length;
  }

  function send() {
    var text = els.input.value.trim();
    if (!text) return;
    els.input.value = '';
    BW.chat_send(text).catch(function (e) { Win.toast(String(e), 'err'); });
  }

  function mount(body) {
    body.classList.add('chat-body');
    if (!BW.online) {
      body.appendChild(el('div', 'chat-off', L('Чат работает в онлайне: на экране входа выбери «Играть онлайн».', 'Chat works online: choose "Play online" on the login screen.')));
      return;
    }
    var wrap = el('div', 'chat');
    var side = el('div', 'chat-side');
    els.count = el('div', 'chat-count');
    els.who = el('div', 'chat-who');
    side.appendChild(els.count);
    side.appendChild(els.who);
    var main = el('div', 'chat-main');
    els.log = el('div', 'chat-log');
    var bar = el('div', 'chat-bar');
    els.input = el('input', 'input');
    els.input.maxLength = 400;
    els.input.placeholder = L('сообщение всем в сети', 'message to everyone online');
    var btn = el('button', 'btn btn-primary');
    btn.innerHTML = Icons.get('send', 16);
    btn.title = L('Отправить', 'Send');
    btn.addEventListener('click', send);
    els.input.addEventListener('keydown', function (e) { if (e.key === 'Enter') send(); });
    bar.appendChild(els.input);
    bar.appendChild(btn);
    main.appendChild(els.log);
    main.appendChild(bar);
    wrap.appendChild(side);
    wrap.appendChild(main);
    body.appendChild(wrap);

    BW.invoke('online_status').then(function (s) { me = (s && s.handle) || ''; }).finally(function () {
      BW.chat_history().then(function (h) { (h || []).forEach(addLine); }, function () {});
      BW.who().then(setWho, function () {});
    });
    els.un1 = BW.listen('chat', addLine);
    els.un2 = BW.listen('presence', function (p) { if (p && p.online) setWho(p.online); });
    window.addEventListener('pagehide', function () {
      [els.un1, els.un2].forEach(function (u) { if (u) u.then(function (f) { f && f(); }); });
    });
    setTimeout(function () { els.input.focus(); }, 50);
  }

  Apps.register('chat', { title: L('Чат игроков', 'Player chat'), icon: 'chat', minWidth: 420, minHeight: 360, mount: mount });
})();
