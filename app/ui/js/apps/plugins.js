'use strict';
/* Окно «Плагины»: список установленных плагинов Adder, их команды и собственные
   окна. Оформление — как у остальной игры (панели, кнопки, подписи словами). */
(function () {
  var PERM_NAMES = {
    content: L('мир и контент', 'world and content'),
    sys: L('оболочка и терминал', 'shell and terminal'),
    net: L('сеть', 'network'),
    fs: L('файлы', 'files'),
    mail: L('почта', 'mail')
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
    head.appendChild(el('span', 'plug-head-t', L('<b>Плагины</b> — свои окна, команды и устройства из пакетов Adder', '<b>Plugins</b> — custom windows, commands and devices from Adder packages')));
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
        Promise.resolve().then(action).catch(function (e) { Win.toast(L('Не удалось: ', 'Failed: ') + e, 'err'); }).finally(function () { b.disabled = false; });
      });
      tools.appendChild(b);
    }
    toolBtn(L('Открыть папку', 'Open folder'), 'folder-open', function () { return BW.plugins_folder(); });
    toolBtn(L('Перечитать', 'Reload'), 'reload', function () { return BW.plugins_reload().then(render); }, true);
    toolBtn(L('Пример устройств', 'Devices example'), 'plus', function () { return BW.plugins_example('device_lab').then(render); });
    toolBtn(L('Пример района', 'District example'), 'plus', function () { return BW.plugins_example('district').then(render); });
    toolBtn(L('Свой тип уязвимости', 'Custom vulnerability type'), 'plus', function () { return BW.plugins_example('custom_vuln_type').then(render); });
    toolBtn(L('Результаты и условия', 'Results and conditions'), 'plus', function () { return BW.plugins_example('access_lab').then(render); });

    function row(p) {
      var card = el('section', 'panel plug-card');
      card.dataset.pluginId = p.id;
      var h = el('div', 'panel-head');
      h.innerHTML = Icons.get('plugins', 14);
      var heading = el('span');
      var name = el('b');
      name.textContent = p.name || p.id;
      heading.appendChild(name);
      heading.appendChild(document.createTextNode(' · ' + (p.version || '')));
      h.appendChild(heading);
      card.appendChild(h);

      var meta = el('div', 'plug-meta');
      var state = el('span', 'plug-state' + (p.enabled ? ' on' : '') + (p.error ? ' err' : ''));
      state.textContent = p.error ? L('Ошибка', 'Error') : (p.enabled ? L('Включён', 'Enabled') : L('Выключен', 'Disabled'));
      meta.appendChild(state);
      var perms = (p.permissions || []).map(function (code) { return PERM_NAMES[code] || code; }).join(', ') || L('только данные плагина', 'plugin data only');
      var permSpan = el('span', 'plug-perm');
      permSpan.textContent = L('Доступ: ', 'Access: ') + perms;
      meta.appendChild(permSpan);
      card.appendChild(meta);

      if (p.error) {
        var errLine = el('div', 'plug-err');
        errLine.textContent = p.error;
        card.appendChild(errLine);
      }

      var actions = el('div', 'plug-actions');
      var toggle = el('button', 'btn btn-sm', Icons.get(p.enabled ? 'stop' : 'play', 14) + '<span>' + (p.enabled ? L('Выключить', 'Disable') : L('Включить', 'Enable')) + '</span>');
      toggle.addEventListener('click', function () {
        BW.sound('click');
        BW.plugins_enable(p.id, !p.enabled).then(render, function (e) { Win.toast(L('Не удалось: ', 'Failed: ') + e, 'err'); });
      });
      actions.appendChild(toggle);
      if (p.error) {
        var retry = el('button', 'btn btn-sm', Icons.get('reload', 14) + '<span>' + L('Повторить запуск', 'Retry start') + '</span>');
        retry.addEventListener('click', function () {
          BW.sound('click');
          BW.plugins_enable(p.id, true).then(render, function (e) { Win.toast(L('Не удалось: ', 'Failed: ') + e, 'err'); });
        });
        actions.appendChild(retry);
      }
      (p.windows || []).forEach(function (w) {
        var openB = el('button', 'btn btn-sm btn-primary', Icons.get('max', 14));
        var windowTitle = el('span');
        windowTitle.textContent = w.title || L('Окно', 'Window');
        openB.appendChild(windowTitle);
        openB.addEventListener('click', function () {
          BW.sound('click');
          Apps.openOrFocus('plugin_panel', w.id);
        });
        actions.appendChild(openB);
      });
      card.appendChild(actions);

      if (p.commands && p.commands.length) {
        var cmds = el('div', 'plug-cmds');
        cmds.appendChild(el('div', 'plug-cmds-h', L('Команды', 'Commands')));
        p.commands.forEach(function (cmd) {
          var cmdRow = el('div', 'plug-cmd');
          var inp = el('input', 'input plug-arg');
          inp.placeholder = L('аргументы (необязательно)', 'arguments (optional)');
          var runB = el('button', 'btn btn-sm', Icons.get('terminal', 14));
          var commandName = el('span');
          commandName.textContent = cmd;
          runB.appendChild(commandName);
          runB.addEventListener('click', function () {
            BW.sound('click');
            runB.disabled = true;
            var line = cmd + (inp.value ? ' ' + inp.value : '');
            BW.plugins_run(line).then(function (lines) {
              log.textContent = (lines || []).map(function (l) { return l.text; }).join('\n');
              return BW.plugins_list().then(render);
            }, function (e) { Win.toast(L('Не удалось: ', 'Failed: ') + e, 'err'); }).finally(function () { runB.disabled = false; });
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
        empty.textContent = L('Плагинов пока нет. Открой папку и положи туда папку плагина, или установи пример кнопкой выше.', 'No plugins yet. Open the folder and put a plugin folder in it, or install an example with a button above.');
        list.appendChild(empty);
        return;
      }
      info.plugins.forEach(function (p) { list.appendChild(row(p)); });
    }

    BW.plugins_list().then(render, function (e) { Win.toast(L('Не удалось: ', 'Failed: ') + e, 'err'); });
  }

  Apps.register('plugins', { title: L('Плагины', 'Plugins'), icon: 'plugins', minWidth: 560, minHeight: 420, mount: mount });
})();
