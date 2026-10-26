'use strict';
/* Экран входа: загрузка системы (журнал, полоса, логотип), затем карточка входа или новой личности. */
(function () {
  var BOOT_LINES = [
    'BLACKWIRE firmware 2.4 — проверка памяти .......... ok',
    'ядро загружено: /sys/kernel/bw.img',
    'монтирование корня: /dev/vda1 → /  (rw)',
    'монтирование: /home  /tmp  /var/log',
    'сеть: eth0 поднят, запрос адреса ...',
    'сеть: адрес получен',
    'служба ssh: слушает порт 22',
    'служба cron: запущена',
    'служба mail: запущена',
    'оболочка: /bin/sh готова',
    'язык Adder: среда загружена',
    'проверка целостности сохранения ...',
    'графическая подложка: инициализация',
    'готово — переход к входу'
  ];
  var HANDLE_RE = /^[A-Za-z][A-Za-z0-9_-]{2,15}$/;

  var root = null, els = {}, bootInfo = null, bootDone = false, infoReady = false;

  function el(tag, cls, html) {
    var e = document.createElement(tag);
    if (cls) e.className = cls;
    if (html != null) e.innerHTML = html;
    return e;
  }
  function swallow() {}

  /* ---------- разметка ---------- */
  function build() {
    root.innerHTML = '';
    var wrap = el('div', 'login');
    wrap.setAttribute('data-tauri-drag-region', '');
    var canvas = el('canvas', 'login-fx');
    wrap.appendChild(canvas);

    var power = el('button', 'login-power btn-icon', Icons.get('power', 18));
    power.title = 'Выйти';
    power.addEventListener('click', function () { BW.invoke('quit'); });
    wrap.appendChild(power);

    var boot = el('div', 'boot');
    boot.setAttribute('data-tauri-drag-region', '');
    boot.innerHTML =
      '<div class="boot-logo" data-text="BLACKWIRE">BLACKWIRE</div>' +
      '<div class="boot-sub caption">загрузка системы</div>' +
      '<pre class="boot-log mono"></pre>' +
      '<div class="boot-bar"><div class="boot-bar-fill"></div></div>';
    wrap.appendChild(boot);

    var card = el('div', 'login-card hidden');
    wrap.appendChild(card);

    root.appendChild(wrap);
    els.wrap = wrap; els.canvas = canvas; els.boot = boot; els.card = card;
    els.log = boot.querySelector('.boot-log'); els.fill = boot.querySelector('.boot-bar-fill');
  }

  /* ---------- загрузка ---------- */
  function runBoot() {
    var i = 0;
    function next() {
      if (i < BOOT_LINES.length) {
        var line = el('div', 'boot-line');
        line.textContent = '[' + (i * 0.173 + 0.021).toFixed(3) + '] ' + BOOT_LINES[i];
        els.log.appendChild(line);
        els.log.scrollTop = els.log.scrollHeight;
        els.fill.style.width = Math.round((i + 1) / BOOT_LINES.length * 100) + '%';
        BW.sound('key');
        i++;
        setTimeout(next, 90 + Math.random() * 140);
      } else {
        setTimeout(function () { bootDone = true; showCardIfReady(); }, 350);
      }
    }
    setTimeout(next, 400);
  }

  function showCardIfReady() {
    if (!bootDone || !infoReady) return;
    els.boot.classList.add('boot-out');
    setTimeout(function () {
      els.boot.classList.add('hidden');
      renderCard(bootInfo && bootInfo.has_save ? 'login' : 'new');
    }, 320);
  }

  /* ---------- карточка ---------- */
  function renderCard(mode) {
    var c = els.card;
    c.innerHTML = '';
    c.classList.remove('hidden', 'shake');
    c.classList.add('show');
    var os = (bootInfo && bootInfo.os_name) || 'BLACKWIRE';
    var head = el('div', 'login-head');
    head.innerHTML = '<div class="login-logo"></div><div class="login-ver caption"></div>';
    head.querySelector('.login-logo').textContent = os;
    head.querySelector('.login-ver').textContent = bootInfo && bootInfo.version ? 'версия ' + bootInfo.version : '';
    c.appendChild(head);

    if (mode === 'login') renderLogin(c); else renderNew(c);
  }

  function errorLine(c, text) {
    var e = c.querySelector('.login-err');
    e.textContent = text || '';
    e.classList.toggle('hidden', !text);
    if (text) {
      c.classList.remove('shake');
      void c.offsetWidth; // перезапуск анимации
      c.classList.add('shake');
      BW.sound('error');
    }
  }

  function passwordField(placeholder) {
    var wrap = el('div', 'login-pass');
    var input = el('input', 'input mono');
    input.type = 'password'; input.placeholder = placeholder; input.autocomplete = 'off'; input.spellcheck = false;
    var eye = el('button', 'btn-icon login-eye', Icons.get('eye', 16));
    eye.type = 'button'; eye.tabIndex = -1; eye.title = 'Показать';
    eye.addEventListener('click', function () {
      var show = input.type === 'password';
      input.type = show ? 'text' : 'password';
      eye.innerHTML = Icons.get(show ? 'eye-off' : 'eye', 16);
      input.focus();
    });
    wrap.appendChild(input); wrap.appendChild(eye);
    return { wrap: wrap, input: input };
  }

  function renderLogin(c) {
    var user = el('div', 'login-user');
    user.innerHTML = '<span class="login-avatar">' + Icons.get('user', 28) + '</span><span class="login-handle mono ellipsis"></span>';
    user.querySelector('.login-handle').textContent = (bootInfo && bootInfo.handle) || 'user';
    c.appendChild(user);

    var pass = passwordField('пароль');
    c.appendChild(pass.wrap);

    var err = el('div', 'login-err hidden');
    c.appendChild(err);

    var btn = el('button', 'btn btn-primary login-btn');
    btn.innerHTML = Icons.get('lock', 16) + '<span>Войти</span>';
    c.appendChild(btn);

    var links = el('div', 'login-links');
    var fresh = el('button', 'login-link');
    fresh.textContent = 'Новая личность';
    fresh.addEventListener('click', function () {
      Win.confirm('Текущее сохранение будет заменено новым. Продолжить?').then(function (ok) {
        if (ok) renderCard('new');
      });
    });
    links.appendChild(fresh);
    c.appendChild(links);

    var busy = false;
    function submit() {
      if (busy) return;
      busy = true; btn.disabled = true;
      errorLine(c, '');
      BW.login(pass.input.value || '').then(function (ok) {
        if (ok) return success();
        busy = false; btn.disabled = false;
        errorLine(c, 'Неверный пароль');
        pass.input.select();
      }, function (e) {
        busy = false; btn.disabled = false;
        errorLine(c, 'Ошибка входа: ' + (e && e.message ? e.message : e));
      });
    }
    btn.addEventListener('click', submit);
    pass.input.addEventListener('keydown', function (e) { if (e.key === 'Enter') submit(); });
    setTimeout(function () { pass.input.focus(); }, 50);
  }

  function renderNew(c) {
    var title = el('div', 'login-title');
    title.textContent = 'Новая личность';
    c.appendChild(title);

    var name = el('input', 'input mono login-name');
    name.type = 'text'; name.placeholder = 'имя в сети'; name.maxLength = 16; name.spellcheck = false; name.autocomplete = 'off';
    c.appendChild(name);
    var hint = el('div', 'login-hint');
    hint.textContent = 'латиница, цифры, _ и -, от 3 до 16 знаков, первая — буква';
    c.appendChild(hint);

    var pass = passwordField('пароль (можно пустой)');
    c.appendChild(pass.wrap);

    var err = el('div', 'login-err hidden');
    c.appendChild(err);

    var btn = el('button', 'btn btn-primary login-btn');
    btn.innerHTML = Icons.get('plus', 16) + '<span>Создать</span>';
    c.appendChild(btn);

    if (bootInfo && bootInfo.has_save) {
      var links = el('div', 'login-links');
      var back = el('button', 'login-link');
      back.textContent = 'Назад ко входу';
      back.addEventListener('click', function () { renderCard('login'); });
      links.appendChild(back);
      c.appendChild(links);
    }

    name.addEventListener('input', function () {
      name.classList.toggle('bad', name.value.length > 0 && !HANDLE_RE.test(name.value));
    });

    var busy = false;
    function submit() {
      if (busy) return;
      var handle = name.value.trim();
      if (!HANDLE_RE.test(handle)) {
        errorLine(c, 'Имя не подходит: нужна латиница, 3–16 знаков');
        name.focus();
        return;
      }
      busy = true; btn.disabled = true;
      errorLine(c, '');
      btn.querySelector('span').textContent = 'Строим мир…';
      BW.new_game(handle, undefined, pass.input.value || '').then(function (info) {
        if (info) bootInfo = info;
        return success();
      }, function (e) {
        busy = false; btn.disabled = false;
        btn.querySelector('span').textContent = 'Создать';
        errorLine(c, 'Не удалось создать: ' + (e && e.message ? e.message : e));
      });
    }
    btn.addEventListener('click', submit);
    name.addEventListener('keydown', function (e) { if (e.key === 'Enter') pass.input.focus(); });
    pass.input.addEventListener('keydown', function (e) { if (e.key === 'Enter') submit(); });
    setTimeout(function () { name.focus(); }, 50);
  }

  function success() {
    BW.sound('open');
    els.card.classList.add('login-ok');
    return BW.start_desktop().then(function () {
      setTimeout(function () { Win.close(); }, 250);
    }, function (e) {
      els.card.classList.remove('login-ok');
      Win.toast('Не удалось открыть рабочий стол: ' + (e && e.message ? e.message : e), 'err');
    });
  }

  /* ---------- запуск ---------- */
  function mount(r) {
    root = r;
    build();
    document.title = 'Blackwire — вход';
    FX.start(els.canvas, BW.settings);
    BW.onSettings(function (s) { FX.update(s); });
    BW.boot().then(function (b) {
      bootInfo = b || { has_save: false };
      infoReady = true;
      showCardIfReady();
    }, function (e) {
      console.error(e);
      bootInfo = { has_save: false };
      infoReady = true;
      showCardIfReady();
    });
    runBoot();
  }

  Apps.register('login', { title: 'Вход', icon: 'lock', mount: mount });
})();
