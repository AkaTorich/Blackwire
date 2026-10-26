'use strict';
/* Реестр приложений и запуск нужного по ?app=. Окна desktop и login рамки не получают. */
var Apps = (function () {
  var registry = {};
  var FRAMELESS = { desktop: true, login: true };
  /* Приложения в одном экземпляре: повторное открытие поднимает существующее окно */
  var SINGLE = { settings: true, help: true, monitor: true, map: true, mail: true, browser: false, files: false };

  var Apps = {
    register: function (id, def) {
      registry[id] = def;
      Apps[id] = def;
    },
    get: function (id) { return registry[id]; },
    ids: function () { return Object.keys(registry); },
    /* Поднять окно приложения, если оно уже открыто, иначе открыть новое */
    openOrFocus: function (app, arg, monitor) {
      if (!SINGLE[app] || arg) {
        return BW.open_app(app, arg == null ? undefined : arg, monitor);
      }
      return BW.list_windows().then(function (list) {
        var found = (list || []).filter(function (x) { return x.app === app; })[0];
        if (found) return BW.focus_window(found.label).then(function () { return found.label; });
        return BW.open_app(app, undefined, monitor);
      }, function () { return BW.open_app(app, undefined, monitor); });
    }
  };

  function unknown(root, id) {
    root.innerHTML = '';
    var box = document.createElement('div');
    box.className = 'unknown-app';
    box.innerHTML = Icons.get('warning', 40) + '<h1>Неизвестное приложение</h1><p></p>';
    box.querySelector('p').textContent = id;
    root.appendChild(box);
  }

  function boot() {
    var root = document.getElementById('root');
    var id = BW.app;
    var def = registry[id];
    if (!def) {
      if (!FRAMELESS[id]) Win.init({ title: 'Blackwire', icon: 'warning' });
      unknown(FRAMELESS[id] ? root : document.querySelector('.win-body'), id);
      Win.bindKeys();
      return;
    }
    var body;
    if (FRAMELESS[id]) {
      body = root;
      body.classList.add('frameless');
      Win.bindKeys();
    } else {
      body = Win.init({ title: def.title || id, icon: def.icon || id, minWidth: def.minWidth, minHeight: def.minHeight });
    }
    document.body.classList.add('app-' + id);
    try {
      def.mount(body, BW.arg);
    } catch (e) {
      console.error('Ошибка приложения ' + id, e);
      Win.toast('Ошибка: ' + (e && e.message ? e.message : e), 'err');
    }
  }

  document.addEventListener('DOMContentLoaded', function () {
    BW.ready.then(boot, boot);
  });

  /* Запрет стандартного контекстного меню WebView — у окон своё */
  document.addEventListener('contextmenu', function (e) {
    var t = e.target;
    if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return;
    e.preventDefault();
  });

  return Apps;
})();
