'use strict';
/* Настройки: темы с предпросмотром, эффекты, шрифт, прозрачность, звук, мониторы, игра, о программе. */
(function () {
  var FALLBACK_THEMES = [
    { id: 'swordfish', name: 'Рыба-меч' }, { id: 'ice', name: 'Лёд' }, { id: 'amber', name: 'Янтарь' },
    { id: 'phosphor', name: 'Фосфор' }, { id: 'violet', name: 'Фиалка' }, { id: 'crimson', name: 'Кармин' },
    { id: 'paper', name: 'Бумага' }, { id: 'midnight', name: 'Полночь' }, { id: 'steel', name: 'Сталь' },
    { id: 'gold', name: 'Золото' }, { id: 'aurora', name: 'Аврора' }, { id: 'ember', name: 'Уголь' }
  ];
  var FX_ITEMS = [
    { key: 'wireframe', label: 'Проволочные фигуры', hint: 'многогранники на рабочем столе' },
    { key: 'hexrain', label: 'Поток цифр', hint: 'шестнадцатеричные колонки' },
    { key: 'scanlines', label: 'Сканлайны', hint: 'тонкие горизонтальные линии' },
    { key: 'glass', label: 'Стекло', hint: 'размытие за окнами' }
  ];
  var LAYOUTS = [
    { id: 'grid', label: 'Сетка' }, { id: 'columns', label: 'Колонки' }, { id: 'rows', label: 'Ряды' }, { id: 'cascade', label: 'Каскад' }
  ];

  var cur = null, bootInfo = null, monitors = [], els = {}, saveTimer = null;

  function el(tag, cls, html) {
    var e = document.createElement(tag);
    if (cls) e.className = cls;
    if (html != null) e.innerHTML = html;
    return e;
  }
  function swallow() {}
  function clone(o) { return JSON.parse(JSON.stringify(o)); }

  function save(immediate) {
    clearTimeout(saveTimer);
    var run = function () { BW.saveSettings(clone(cur)).catch(function (e) { Win.toast('Не сохранилось: ' + e, 'err'); }); };
    if (immediate) run(); else saveTimer = setTimeout(run, 120);
  }

  function section(title, icon) {
    var s = el('section', 'set-section');
    var h = el('h2', 'set-head');
    h.innerHTML = '<span class="set-head-ico">' + Icons.get(icon, 16) + '</span><span></span>';
    h.querySelector('span:last-child').textContent = title;
    s.appendChild(h);
    return s;
  }

  function row(label, control, hint) {
    var r = el('div', 'set-row');
    var text = el('div', 'set-row-text');
    var l = el('div', 'set-row-label'); l.textContent = label; text.appendChild(l);
    if (hint) { var hh = el('div', 'set-row-hint'); hh.textContent = hint; text.appendChild(hh); }
    r.appendChild(text);
    var c = el('div', 'set-row-ctl'); c.appendChild(control); r.appendChild(c);
    return r;
  }

  function toggle(checked, onChange) {
    var lab = el('label', 'toggle');
    var inp = el('input'); inp.type = 'checkbox'; inp.checked = !!checked;
    lab.appendChild(inp); lab.appendChild(el('span', 'toggle-track'));
    inp.addEventListener('change', function () { BW.sound('click'); onChange(inp.checked); });
    return { el: lab, input: inp };
  }

  function slider(min, max, step, value, format, onInput) {
    var wrap = el('div', 'set-slider');
    var inp = el('input', 'slider'); inp.type = 'range'; inp.min = min; inp.max = max; inp.step = step; inp.value = value;
    var val = el('span', 'set-slider-val mono');
    function paint() {
      var pct = (inp.value - min) / (max - min) * 100;
      inp.style.setProperty('--fill', pct.toFixed(1) + '%');
      val.textContent = format(+inp.value);
    }
    inp.addEventListener('input', function () { paint(); onInput(+inp.value); });
    wrap.appendChild(inp); wrap.appendChild(val);
    paint();
    return { el: wrap, input: inp, paint: paint };
  }

  function select(options, value, onChange) {
    var s = el('select', 'input');
    options.forEach(function (o) {
      var op = el('option'); op.value = o.value; op.textContent = o.label;
      if (String(o.value) === String(value)) op.selected = true;
      s.appendChild(op);
    });
    s.addEventListener('change', function () { BW.sound('click'); onChange(s.value); });
    return s;
  }

  /* ---------- темы ---------- */
  function themeTile(t) {
    var b = el('button', 'theme-tile theme-preview');
    b.dataset.theme = t.id;
    b.innerHTML =
      '<div class="tp-win"><div class="tp-title"><i></i><i></i><i></i></div>' +
      '<div class="tp-body"><b></b><b class="w2"></b><b class="w3"></b><span class="tp-cursor"></span></div></div>' +
      '<span class="tp-name ellipsis"></span>';
    b.querySelector('.tp-name').textContent = t.name || t.id;
    b.title = t.name || t.id;
    b.addEventListener('click', function () {
      BW.sound('click');
      cur.theme = t.id;
      markTheme();
      save(true);
    });
    return b;
  }

  function markTheme() {
    if (!els.themes) return;
    Array.prototype.forEach.call(els.themes.children, function (c) {
      c.classList.toggle('selected', c.dataset.theme === cur.theme);
    });
  }

  function buildThemes(list) {
    els.themes.innerHTML = '';
    (list && list.length ? list : FALLBACK_THEMES).forEach(function (t) { els.themes.appendChild(themeTile(t)); });
    markTheme();
  }

  /* ---------- мониторы ---------- */
  function monitorOptions() {
    if (!monitors.length) return [{ value: 0, label: 'Основной' }];
    return monitors.map(function (m) {
      return { value: m.index, label: (m.name || ('Монитор ' + (m.index + 1))) + ' · ' + m.width + '×' + m.height + (m.primary ? ' · основной' : '') };
    });
  }

  function buildMonitors() {
    els.monSection.innerHTML = '';
    els.monSection.appendChild(section('Мониторы', 'monitor-screen').firstChild);
    var def = select(monitorOptions(), cur.default_monitor || 0, function (v) { cur.default_monitor = parseInt(v, 10) || 0; save(true); });
    els.monSection.appendChild(row('Монитор по умолчанию', def, 'новые окна открываются на нём'));

    var arr = el('div', 'set-arrange');
    var layoutSel = select(LAYOUTS.map(function (l) { return { value: l.id, label: l.label }; }), 'grid', swallow);
    var monSel = select(monitorOptions(), cur.default_monitor || 0, swallow);
    var btn = el('button', 'btn', Icons.get('layout', 16) + '<span>Разложить окна</span>');
    btn.addEventListener('click', function () {
      BW.sound('click');
      BW.arrange_windows(parseInt(monSel.value, 10) || 0, layoutSel.value).then(function () { Win.toast('Окна разложены'); }, function (e) { Win.toast('Не удалось: ' + e, 'err'); });
    });
    arr.appendChild(layoutSel); arr.appendChild(monSel); arr.appendChild(btn);
    els.monSection.appendChild(row('Раскладка открытых окон', arr));
  }

  /* ---------- игра ---------- */
  function buildGame(s) {
    var btns = el('div', 'set-btns');
    var saveB = el('button', 'btn btn-primary', Icons.get('save', 16) + '<span>Сохранить игру</span>');
    saveB.addEventListener('click', function () {
      BW.sound('click');
      BW.save_game().then(function () { Win.toast('Игра сохранена', 'ok'); }, function (e) { Win.toast('Не сохранилось: ' + e, 'err'); });
    });
    var loadB = el('button', 'btn', Icons.get('reload', 16) + '<span>Загрузить</span>');
    loadB.addEventListener('click', function () {
      Win.confirm('Загрузить последнее сохранение? Несохранённое пропадёт.').then(function (ok) {
        if (!ok) return;
        BW.load_game().then(function () { Win.toast('Сохранение загружено', 'ok'); }, function (e) { Win.toast('Не загрузилось: ' + e, 'err'); });
      });
    });
    var newB = el('button', 'btn btn-danger', Icons.get('user', 16) + '<span>Новая личность</span>');
    newB.addEventListener('click', function () {
      Win.confirm('Начать заново? Текущее сохранение будет заменено.').then(function (ok) {
        if (!ok) return;
        return Win.prompt('Имя в сети (латиница, 3–16 знаков)', '').then(function (handle) {
          if (handle === null) return;
          handle = handle.trim();
          if (!/^[A-Za-z][A-Za-z0-9_-]{2,15}$/.test(handle)) { Win.toast('Имя не подходит', 'err'); return; }
          return BW.new_game(handle).then(function () { Win.toast('Новый мир построен: ' + handle, 'ok'); }, function (e) { Win.toast('Не удалось: ' + e, 'err'); });
        });
      });
    });
    btns.appendChild(saveB); btns.appendChild(loadB); btns.appendChild(newB);
    s.appendChild(btns);
  }

  /* ---------- сохранения (именованные слоты) ---------- */
  function renderSlots(list) {
    if (!els.slotList) return;
    els.slotList.innerHTML = '';
    if (!list || !list.length) {
      els.slotList.appendChild(el('div', 'set-row-hint', 'Слотов пока нет. «Новый слот» сделает снимок текущей игры.'));
      return;
    }
    list.forEach(function (sl) {
      var r = el('div', 'slot-row');
      var text = el('div', 'slot-text');
      var name = el('div', 'slot-name'); name.textContent = sl.name; text.appendChild(name);
      var meta = el('div', 'slot-meta mono');
      meta.textContent = sl.handle + ' · ' + sl.time + ' · ' + sl.money + ' ₵';
      text.appendChild(meta);
      r.appendChild(text);

      var btns = el('div', 'slot-btns');
      var loadB = el('button', 'btn btn-sm', Icons.get('reload', 14) + '<span>Загрузить</span>');
      loadB.addEventListener('click', function () {
        BW.sound('click');
        Win.confirm('Загрузить слот «' + sl.name + '»? Несохранённое пропадёт.').then(function (ok) {
          if (!ok) return;
          BW.load_slot(sl.name).then(function () { Win.toast('Слот загружен: ' + sl.name, 'ok'); }, function (e) { Win.toast('Не загрузилось: ' + e, 'err'); });
        });
      });
      var overB = el('button', 'btn btn-sm', Icons.get('save', 14) + '<span>Перезаписать</span>');
      overB.addEventListener('click', function () {
        BW.sound('click');
        BW.save_to_slot(sl.name).then(function (ls) { renderSlots(ls); Win.toast('Слот перезаписан', 'ok'); }, function (e) { Win.toast('Не удалось: ' + e, 'err'); });
      });
      var delB = el('button', 'btn btn-sm btn-danger', Icons.get('trash', 14) + '<span></span>');
      delB.title = 'Удалить';
      delB.addEventListener('click', function () {
        BW.sound('click');
        Win.confirm('Удалить слот «' + sl.name + '»?').then(function (ok) {
          if (!ok) return;
          BW.delete_slot(sl.name).then(function (ls) { renderSlots(ls); Win.toast('Слот удалён', 'ok'); }, function (e) { Win.toast('Не удалось: ' + e, 'err'); });
        });
      });
      btns.appendChild(loadB); btns.appendChild(overB); btns.appendChild(delB);
      r.appendChild(btns);
      els.slotList.appendChild(r);
    });
  }

  function refreshSlots() {
    BW.save_slots().then(renderSlots, swallow);
  }

  function buildSlots(s) {
    var newBtn = el('button', 'btn btn-primary', Icons.get('save', 16) + '<span>Новый слот</span>');
    newBtn.addEventListener('click', function () {
      BW.sound('click');
      Win.prompt('Название сохранения', '').then(function (name) {
        if (name === null) return;
        name = (name || '').trim();
        if (!name) { Win.toast('Пустое название', 'err'); return; }
        BW.save_to_slot(name).then(function (ls) { renderSlots(ls); Win.toast('Сохранено: ' + name, 'ok'); }, function (e) { Win.toast('Не удалось: ' + e, 'err'); });
      });
    });
    var head = el('div', 'set-btns');
    head.appendChild(newBtn);
    s.appendChild(head);
    els.slotList = el('div', 'slot-list');
    s.appendChild(els.slotList);
    renderSlots([]);
  }

  /* ---------- о программе ---------- */
  function buildAbout(s) {
    var box = el('div', 'set-about');
    box.innerHTML =
      '<div class="set-about-logo">BLACKWIRE</div>' +
      '<div class="set-about-ver mono"></div>' +
      '<p>Игра о взломе. Каждое приложение — настоящее окно системы; мир живёт в ядре и не зависит от окон.</p>' +
      '<p class="dim">Горячие клавиши: Ctrl+Shift+T — терминал, Ctrl+Shift+E — редактор, Ctrl+Shift+F — файлы, Ctrl+Shift+M — почта, F11 — во весь экран, Ctrl+W — закрыть окно.</p>';
    s.appendChild(box);
    els.aboutVer = box.querySelector('.set-about-ver');
  }

  /* ---------- синхронизация с внешними изменениями ---------- */
  function sync(s) {
    cur = clone(s);
    markTheme();
    FX_ITEMS.forEach(function (f) { if (els.fx[f.key]) els.fx[f.key].input.checked = cur.fx[f.key] !== false; });
    if (els.font) { els.font.input.value = cur.font_size; els.font.paint(); }
    if (els.opacity) { els.opacity.input.value = Math.round(cur.terminal_opacity * 100); els.opacity.paint(); }
    if (els.winop) { els.winop.input.value = Math.round((1 - cur.window_opacity) * 100); els.winop.paint(); }
    if (els.sound) els.sound.input.checked = !!cur.sound;
  }

  /* ---------- запуск ---------- */
  function mount(body) {
    cur = clone(BW.settings);
    els.fx = {};
    body.innerHTML = '';
    var wrap = el('div', 'settings');

    var sTheme = section('Тема', 'grid');
    els.themes = el('div', 'theme-grid');
    sTheme.appendChild(els.themes);
    buildThemes(FALLBACK_THEMES);
    wrap.appendChild(sTheme);

    var sFx = section('Эффекты', 'hexagon');
    FX_ITEMS.forEach(function (f) {
      var t = toggle(cur.fx[f.key] !== false, function (v) { cur.fx[f.key] = v; save(true); });
      els.fx[f.key] = t;
      sFx.appendChild(row(f.label, t.el, f.hint));
    });
    wrap.appendChild(sFx);

    var sView = section('Вид', 'eye');
    els.font = slider(12, 20, 1, cur.font_size, function (v) { return v + ' px'; }, function (v) { cur.font_size = v; save(false); });
    sView.appendChild(row('Размер шрифта', els.font.el, 'для всех окон'));
    els.opacity = slider(40, 100, 5, Math.round(cur.terminal_opacity * 100), function (v) { return v + ' %'; }, function (v) { cur.terminal_opacity = v / 100; save(false); });
    sView.appendChild(row('Плотность фона терминала', els.opacity.el));
    els.winop = slider(0, 70, 5, Math.round((1 - cur.window_opacity) * 100), function (v) { return v + ' %'; }, function (v) { cur.window_opacity = 1 - v / 100; save(false); });
    sView.appendChild(row('Прозрачность окон', els.winop.el, 'сквозь окна виден рабочий стол'));
    wrap.appendChild(sView);

    var sSound = section('Звук', 'sound');
    els.sound = toggle(cur.sound, function (v) { cur.sound = v; save(true); if (v) setTimeout(function () { BW.sound('notify'); }, 150); });
    var soundRow = el('div', 'set-inline');
    soundRow.appendChild(els.sound.el);
    var test = el('button', 'btn', Icons.get('bell', 16) + '<span>Проверить</span>');
    test.addEventListener('click', function () { BW.sound('notify'); });
    soundRow.appendChild(test);
    sSound.appendChild(row('Звуки интерфейса', soundRow, 'короткие сигналы на щелчки и события'));
    wrap.appendChild(sSound);

    els.monSection = section('Мониторы', 'monitor-screen');
    wrap.appendChild(els.monSection);
    buildMonitors();

    var sGame = section('Игра', 'disk');
    buildGame(sGame);
    wrap.appendChild(sGame);

    var sSlots = section('Сохранения', 'save');
    buildSlots(sSlots);
    wrap.appendChild(sSlots);

    var sAbout = section('О программе', 'info');
    buildAbout(sAbout);
    wrap.appendChild(sAbout);

    body.appendChild(wrap);

    BW.boot().then(function (b) {
      bootInfo = b || {};
      if (bootInfo.theme_list && bootInfo.theme_list.length) buildThemes(bootInfo.theme_list);
      if (els.aboutVer) els.aboutVer.textContent = (bootInfo.os_name || 'BLACKWIRE') + (bootInfo.version ? ' · версия ' + bootInfo.version : '');
    }, swallow);
    BW.monitors().then(function (m) { monitors = m || []; buildMonitors(); }, swallow);
    refreshSlots();

    var first = true;
    BW.onSettings(function (s) {
      if (first) { first = false; return; }
      if (JSON.stringify(s) !== JSON.stringify(cur)) sync(s);
    });
  }

  Apps.register('settings', { title: 'Настройки', icon: 'settings', minWidth: 520, minHeight: 400, mount: mount });
})();
