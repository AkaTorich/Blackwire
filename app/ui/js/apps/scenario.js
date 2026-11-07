'use strict';
/* Редактор сценариев: собрать своё задание (цель, условие, награда), сохранить
   файлом и запустить. Внизу — список сохранённых со «Старт» и удалением. */
(function () {
  var GOALS = [
    { id: 'own', label: L('Получить доступ к узлу', 'Gain access to a host') },
    { id: 'deliver', label: L('Прислать текст с нужной строкой', 'Send a text with the required string') },
    { id: 'money', label: L('Накопить деньги', 'Save up money') }
  ];

  var els = {}, targets = [];

  function el(tag, cls, html) {
    var e = document.createElement(tag);
    if (cls) e.className = cls;
    if (html != null) e.innerHTML = html;
    return e;
  }

  function field(label, control, hint) {
    var f = el('div', 'scn-field');
    var l = el('label', 'scn-label'); l.textContent = label; f.appendChild(l);
    f.appendChild(control);
    if (hint) { var h = el('div', 'scn-hint'); h.textContent = hint; f.appendChild(h); }
    return f;
  }

  function goalFromForm() {
    var kind = els.goal.value;
    if (kind === 'own') {
      var host = (els.host.value || '').trim();
      if (!host) return { err: L('выбери узел-цель', 'pick a target host') };
      return { goal: { OwnHost: { host: host } } };
    }
    if (kind === 'deliver') {
      var needle = (els.needle.value || '').trim();
      if (needle.length < 3) return { err: L('строка-улика слишком короткая', 'evidence string is too short') };
      return { goal: { Deliver: { needle: needle } } };
    }
    var amount = parseInt(els.amount.value, 10);
    if (!(amount > 0)) return { err: L('сумма должна быть больше нуля', 'amount must be greater than zero') };
    return { goal: { Money: { amount: amount } } };
  }

  function showGoalInputs() {
    var kind = els.goal.value;
    els.hostField.style.display = kind === 'own' ? '' : 'none';
    els.needleField.style.display = kind === 'deliver' ? '' : 'none';
    els.amountField.style.display = kind === 'money' ? '' : 'none';
  }

  function refreshList() {
    BW.scenario_list().then(renderList, function () {});
    BW.scenario_active().then(function (a) {
      els.active.textContent = a ? (L('Активен: ', 'Active: ') + a.name) : L('Активного сценария нет', 'No active scenario');
      els.stopBtn.style.display = a ? '' : 'none';
    }, function () {});
  }

  function renderList(list) {
    els.list.innerHTML = '';
    if (!list || !list.length) {
      els.list.appendChild(el('div', 'scn-hint', L('Пока ни одного. Собери сценарий выше и нажми «Сохранить».', 'None yet. Build a scenario above and press "Save".')));
      return;
    }
    list.forEach(function (sc) {
      var r = el('div', 'scn-row');
      var t = el('div', 'scn-row-text');
      var n = el('div', 'scn-name'); n.textContent = sc.name; t.appendChild(n);
      var g = el('div', 'scn-goal'); g.textContent = sc.goal + ' · ' + sc.reward + ' ₵'; t.appendChild(g);
      if (sc.brief) { var b = el('div', 'scn-brief'); b.textContent = sc.brief; t.appendChild(b); }
      r.appendChild(t);
      var btns = el('div', 'scn-btns');
      var startB = el('button', 'btn btn-sm btn-primary', Icons.get('play', 14) + '<span>' + L('Старт', 'Start') + '</span>');
      startB.addEventListener('click', function () {
        BW.sound('click');
        BW.scenario_start(sc.name).then(function () { Win.toast(L('Сценарий запущен: ', 'Scenario started: ') + sc.name, 'ok'); refreshList(); }, function (e) { Win.toast(L('Не удалось: ', 'Failed: ') + e, 'err'); });
      });
      var delB = el('button', 'btn btn-sm btn-danger', Icons.get('trash', 14) + '<span></span>');
      delB.title = L('Удалить', 'Delete');
      delB.addEventListener('click', function () {
        BW.sound('click');
        Win.confirm(L('Удалить сценарий «' + sc.name + '»?', 'Delete scenario "' + sc.name + '"?')).then(function (ok) {
          if (!ok) return;
          BW.scenario_delete(sc.name).then(function (ls) { renderList(ls); Win.toast(L('Удалён', 'Deleted'), 'ok'); }, function (e) { Win.toast(L('Не удалось: ', 'Failed: ') + e, 'err'); });
        });
      });
      btns.appendChild(startB); btns.appendChild(delB);
      r.appendChild(btns);
      els.list.appendChild(r);
    });
  }

  function save() {
    var name = (els.name.value || '').trim();
    if (!name) { Win.toast(L('Дай сценарию имя', 'Give the scenario a name'), 'err'); return; }
    var g = goalFromForm();
    if (g.err) { Win.toast(g.err, 'err'); return; }
    var reward = parseInt(els.reward.value, 10);
    if (!(reward >= 0)) reward = 0;
    var scenario = { name: name, brief: (els.brief.value || '').trim(), goal: g.goal, reward: reward };
    BW.scenario_save(scenario).then(function (ls) { renderList(ls); Win.toast(L('Сохранён: ', 'Saved: ') + name, 'ok'); }, function (e) { Win.toast(L('Не сохранилось: ', 'Not saved: ') + e, 'err'); });
  }

  function mount(body) {
    body.classList.add('scn-body');
    var wrap = el('div', 'scn');

    var head = el('div', 'scn-head');
    head.appendChild(el('span', 'scn-head-ico', Icons.get('scenario', 18)));
    head.appendChild(el('span', 'scn-head-t', L('<b>Свой сценарий</b> — задай цель и награду', '<b>Custom scenario</b> — set a goal and a reward')));
    wrap.appendChild(head);

    var form = el('div', 'scn-form');

    els.name = el('input', 'input'); els.name.placeholder = L('Короткое имя', 'Short name');
    form.appendChild(field(L('Имя', 'Name'), els.name));

    els.brief = el('textarea', 'input scn-area'); els.brief.rows = 3; els.brief.placeholder = L('О чём задание (пара строк)', 'What the mission is about (a couple of lines)');
    form.appendChild(field(L('Описание', 'Description'), els.brief));

    els.goal = el('select', 'input');
    GOALS.forEach(function (g) { var o = el('option'); o.value = g.id; o.textContent = g.label; els.goal.appendChild(o); });
    els.goal.addEventListener('change', showGoalInputs);
    form.appendChild(field(L('Цель', 'Goal'), els.goal));

    els.host = el('select', 'input');
    els.hostField = field(L('Узел-цель', 'Target host'), els.host, L('из известных публичных узлов мира', 'from the known public hosts of the world'));
    form.appendChild(els.hostField);

    els.needle = el('input', 'input'); els.needle.placeholder = L('Строка, которую надо слить', 'The string to leak');
    els.needleField = field(L('Строка-улика', 'Evidence string'), els.needle, L('засчитается, когда она уйдёт в «Отправленных»', 'counts once it shows up in "Sent"'));
    form.appendChild(els.needleField);

    els.amount = el('input', 'input'); els.amount.type = 'number'; els.amount.min = '1'; els.amount.value = '1000';
    els.amountField = field(L('Сумма, ₵', 'Amount, ₵'), els.amount);
    form.appendChild(els.amountField);

    els.reward = el('input', 'input'); els.reward.type = 'number'; els.reward.min = '0'; els.reward.value = '300';
    form.appendChild(field(L('Награда, ₵', 'Reward, ₵'), els.reward));

    var saveBtn = el('button', 'btn btn-primary', Icons.get('save', 16) + '<span>' + L('Сохранить', 'Save') + '</span>');
    saveBtn.addEventListener('click', function () { BW.sound('click'); save(); });
    var saveRow = el('div', 'scn-save'); saveRow.appendChild(saveBtn);
    form.appendChild(saveRow);

    wrap.appendChild(form);

    var listHead = el('div', 'scn-list-head');
    listHead.appendChild(el('span', null, L('Сохранённые сценарии', 'Saved scenarios')));
    els.active = el('span', 'scn-active'); listHead.appendChild(els.active);
    els.stopBtn = el('button', 'btn btn-sm', Icons.get('stop', 14) + '<span>' + L('Снять активный', 'Stop active') + '</span>');
    els.stopBtn.style.display = 'none';
    els.stopBtn.addEventListener('click', function () {
      BW.sound('click');
      BW.scenario_stop().then(function () { Win.toast(L('Сценарий снят', 'Scenario stopped'), 'ok'); refreshList(); }, function (e) { Win.toast(L('Не удалось: ', 'Failed: ') + e, 'err'); });
    });
    listHead.appendChild(els.stopBtn);
    wrap.appendChild(listHead);

    els.list = el('div', 'scn-list');
    wrap.appendChild(els.list);

    body.appendChild(wrap);

    showGoalInputs();
    BW.scenario_targets().then(function (t) {
      targets = t || [];
      els.host.innerHTML = '';
      if (!targets.length) { var o = el('option'); o.value = ''; o.textContent = L('— узлов пока нет —', '— no hosts yet —'); els.host.appendChild(o); }
      targets.forEach(function (h) { var o = el('option'); o.value = h; o.textContent = h; els.host.appendChild(o); });
    }, function () {});
    refreshList();
  }

  Apps.register('scenario', { title: L('Редактор сценариев', 'Scenario editor'), icon: 'scenario', minWidth: 640, minHeight: 460, mount: mount });
})();
