/* Окно финала: показывается при победе (цепочка пройдена + долг закрыт).
   Это не конец — кнопка «Остаться в системе» закрывает окно, мир работает дальше. */
(function () {
  'use strict';

  function el(tag, cls) {
    var e = document.createElement(tag);
    if (cls) e.className = cls;
    return e;
  }

  function mount(body) {
    body.classList.add('finale-body');

    var canvas = el('canvas', 'finale-fx');
    body.appendChild(canvas);
    try { FX.start(canvas, BW.settings || {}); } catch (e) {}
    BW.onSettings(function (s) { try { FX.update(s); } catch (e) {} });

    var wrap = el('div', 'finale-wrap');
    body.appendChild(wrap);

    var logo = el('div', 'finale-logo');
    logo.textContent = 'hello-world';
    wrap.appendChild(logo);

    var sub = el('div', 'finale-sub');
    sub.textContent = 'Добро пожаловать домой';
    wrap.appendChild(sub);

    var textEl = el('div', 'finale-text');
    wrap.appendChild(textEl);

    var statsEl = el('div', 'finale-stats');
    wrap.appendChild(statsEl);

    var btn = el('button', 'btn btn-primary finale-btn');
    btn.textContent = 'Остаться в системе';
    btn.addEventListener('click', function () { BW.sound('click'); Win.close(); });
    wrap.appendChild(btn);

    BW.ending_summary().then(function (s) {
      (s.text || '').split('\n').forEach(function (line) {
        var p = el('p');
        p.textContent = line;
        textEl.appendChild(p);
      });
      function stat(k, v) {
        var d = el('div', 'finale-stat');
        var a = el('span', 'finale-k');
        a.textContent = k;
        var b = el('span', 'finale-v');
        b.textContent = v;
        d.appendChild(a);
        d.appendChild(b);
        statsEl.appendChild(d);
      }
      stat('Задания', s.missions_done + '/' + s.missions_total);
      stat('Долг', 'закрыт');
      stat('Заработано', s.money_earned + ' ₵');
      stat('Выплачено', s.debt_paid + ' ₵');
      stat('Взломано машин', s.machines_owned);
      stat('Просканировано', s.scanned);
      stat('Пик подозрения', Math.round(s.peak_heat || 0));
      stat('Рейдов пережито', s.raids);
    }, function (e) {
      textEl.textContent = String(e);
    });

    window.addEventListener('pagehide', function () { try { FX.stop(); } catch (e) {} });
    try { BW.sound('notify'); } catch (e) {}
  }

  Apps.register('finale', { title: 'hello-world', icon: 'power', minWidth: 560, minHeight: 560, mount: mount });
})();
