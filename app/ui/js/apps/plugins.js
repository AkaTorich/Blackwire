'use strict';
/* Окно «Плагины»: список установленных плагинов Adder, их команды и собственные
   окна. Оформление — как у остальной игры (панели, кнопки, подписи словами). */
(function () {
  var PERM_NAMES = {
    content: 'мир и контент',
    sys: 'оболочка и терминал',
    net: 'сеть',
    fs: 'файлы',
    mail: 'почта'
  };

  function el(tag, cls, html) {
    var e = document.createElement(tag);
    if (cls) e.className = cls;
    if (html != null) e.innerHTML = html;
    return e;
  }

  function mount(body) {
    body.classList.add('plug-body');
    var wrap = el('div', 'plug');
    body.appendChild(wrap);

    var head = el('div', 'plug-head');
    head.appendChild(el('span', 'plug-head-ico', Icons.get('plugins', 18)));
    head.appendChild(el('span', 'plug-head-t', '<b>Плагины</b> — свои окна, команды и устройства из пакетов Adder'));
    wrap.appendChild(head);

    var tools = el('div', 'plug-tools');
    wrap.appendChild(tools);
    var pathLine = el('div', 'plug-path mono');
    wrap.appendChild(pathLine);
    var errBox = el('div', 'plug-errs');
    wrap.appendChild(errBox);
    var list = el('div', 'plug-list');
    wrap.appendChild(list);
    var log = el('pre', 'plug-log mono');
    wrap.appendChild(log);

    function toolBtn(label, iconId, action, primary) {
      var cls = 'btn' + (primary ? ' btn-primary' : '') + ' btn-sm';
      var b = el('button', cls, Icons.get(iconId, 14) + '<span>' + label + '</span>');
      b.addEventListener('click', function () {
        BW.sound('click');
        b.disabled = true;
        Promise.resolve().then(action).catch(function (e) { Win.toast('Не удалось: ' + e, 'err'); }).finally(function () { b.disabled = false; });
      });
      tools.appendChild(b);
    }
    toolBtn('Открыть папку', 'folder-open', function () { return BW.plugins_folder(); });
    toolBtn('Перечитать', 'reload', function () { return BW.plugins_reload().then(render); }, true);
    toolBtn('Пример устройств', 'plus', function () { return BW.plugins_example('device_lab').then(render); });
    toolBtn('Пример района', 'plus', function () { return BW.plugins_example('district').then(render); });

    function row(p) {
      var card = el('section', 'panel plug-card');
      var h = el('div', 'panel-head');
      h.innerHTML = Icons.get('plugins', 14) + '<span><b>' + (p.name || p.id) + '</b> · ' + (p.version || '') + '</span>';
      card.appendChild(h);

      var meta = el('div', 'plug-meta');
      var state = el('span', 'plug-state' + (p.enabled ? ' on' : '') + (p.error ? ' err' : ''));
      state.textContent = p.error ? 'Ошибка' : (p.enabled ? 'Включён' : 'Выключен');
      meta.appendChild(state);
      var perms = (p.permissions || []).map(function (code) { return PERM_NAMES[code] || code; }).join(', ') || 'только данные плагина';
      var permSpan = el('span', 'plug-perm');
      permSpan.textContent = 'Доступ: ' + perms;
      meta.appendChild(permSpan);
      card.appendChild(meta);

      if (p.error) {
        var errLine = el('div', 'plug-err');
        errLine.textContent = p.error;
        card.appendChild(errLine);
      }

      var actions = el('div', 'plug-actions');
      var toggle = el('button', 'btn btn-sm', Icons.get(p.enabled ? 'stop' : 'play', 14) + '<span>' + (p.enabled ? 'Выключить' : 'Включить') + '</span>');
      toggle.addEventListener('click', function () {
        BW.sound('click');
        BW.plugins_enable(p.id, !p.enabled).then(render, function (e) { Win.toast('Не удалось: ' + e, 'err'); });
      });
      actions.appendChild(toggle);
      if (p.error) {
        var retry = el('button', 'btn btn-sm', Icons.get('reload', 14) + '<span>Повторить запуск</span>');
        retry.addEventListener('click', function () {
          BW.sound('click');
          BW.plugins_enable(p.id, true).then(render, function (e) { Win.toast('Не удалось: ' + e, 'err'); });
        });
        actions.appendChild(retry);
      }
      (p.windows || []).forEach(function (w) {
        var openB = el('button', 'btn btn-sm btn-primary', Icons.get('max', 14) + '<span>' + (w.title || 'Окно') + '</span>');
        openB.addEventListener('click', function () {
          BW.sound('click');
          Apps.openOrFocus('plugin_panel', w.id);
        });
        actions.appendChild(openB);
      });
      card.appendChild(actions);

      if (p.commands && p.commands.length) {
        var cmds = el('div', 'plug-cmds');
        cmds.appendChild(el('div', 'plug-cmds-h', 'Команды'));
        p.commands.forEach(function (cmd) {
          var cmdRow = el('div', 'plug-cmd');
          var inp = el('input', 'input plug-arg');
          inp.placeholder = 'аргументы (необязательно)';
          var runB = el('button', 'btn btn-sm', Icons.get('terminal', 14) + '<span>' + cmd + '</span>');
          runB.addEventListener('click', function () {
            BW.sound('click');
            runB.disabled = true;
            var line = cmd + (inp.value ? ' ' + inp.value : '');
            BW.plugins_run(line).then(function (lines) {
              log.textContent = (lines || []).map(function (l) { return l.text; }).join('\n');
              return BW.plugins_list().then(render);
            }, function (e) { Win.toast('Не удалось: ' + e, 'err'); }).finally(function () { runB.disabled = false; });
          });
          cmdRow.appendChild(runB);
          cmdRow.appendChild(inp);
          cmds.appendChild(cmdRow);
        });
        card.appendChild(cmds);
      }
      return card;
    }

    function render(info) {
      list.replaceChildren();
      errBox.replaceChildren();
      if (!info) return;
      if (info.folder) pathLine.textContent = info.folder;
      (info.errors || []).forEach(function (e) {
        var p = el('div', 'plug-top-err');
        p.textContent = e;
        errBox.appendChild(p);
      });
      if (!info.plugins || !info.plugins.length) {
        var empty = el('div', 'plug-empty');
        empty.textContent = 'Плагинов пока нет. Открой папку и положи туда папку плагина, или установи пример кнопкой выше.';
        list.appendChild(empty);
        return;
      }
      info.plugins.forEach(function (p) { list.appendChild(row(p)); });
    }

    BW.plugins_list().then(render, function (e) { Win.toast('Не удалось: ' + e, 'err'); });
  }

  Apps.register('plugins', { title: 'Плагины', icon: 'plugins', minWidth: 560, minHeight: 420, mount: mount });
})();
