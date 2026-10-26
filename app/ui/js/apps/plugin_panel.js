'use strict';
/* Окно, которое открывает сам плагин: берёт из плагина заголовок, текст,
   таблицу и кнопки, рисует их в оформлении игры. */
(function () {
  function el(tag, cls, html) {
    var e = document.createElement(tag);
    if (cls) e.className = cls;
    if (html != null) e.innerHTML = html;
    return e;
  }

  function mount(body, id) {
    body.classList.add('plug-panel-body');
    var wrap = el('div', 'plug-panel');
    body.appendChild(wrap);
    var content = el('div', 'plug-panel-main');
    wrap.appendChild(content);
    var log = el('pre', 'plug-panel-log mono');
    wrap.appendChild(log);

    function render(spec) {
      content.replaceChildren();
      if (!spec || typeof spec !== 'object') {
        content.appendChild(el('div', 'plug-empty', 'Плагин не передал данные для окна.'));
        return;
      }
      if (spec.title) content.appendChild(el('h2', 'plug-panel-title', spec.title));
      if (spec.text) {
        var p = el('p', 'plug-panel-text');
        p.textContent = spec.text;
        content.appendChild(p);
      }

      var values = {};
      (spec.fields || []).forEach(function (f) {
        var row = el('label', 'plug-field');
        row.appendChild(el('span', 'plug-field-lbl', f.label || f.id));
        var type = ['number', 'checkbox', 'text'].indexOf(f.type) >= 0 ? f.type : 'text';
        var input = el('input', 'input');
        input.type = type;
        if (type === 'checkbox') {
          input.checked = !!f.value;
        } else {
          input.value = f.value == null ? '' : f.value;
        }
        row.appendChild(input);
        values[f.id] = input;
        content.appendChild(row);
      });

      if (spec.rows && spec.rows.length) {
        var tbl = el('table', 'plug-panel-table');
        spec.rows.forEach(function (r) {
          var tr = el('tr');
          var cells = Array.isArray(r) ? r : [r];
          cells.forEach(function (v) {
            var td = el('td');
            if (typeof v === 'boolean') {
              td.textContent = v ? 'да' : 'нет';
              td.className = v ? 'ok' : 'off';
            } else {
              td.textContent = String(v);
            }
            tr.appendChild(td);
          });
          tbl.appendChild(tr);
        });
        content.appendChild(tbl);
      }

      var btnRow = el('div', 'plug-panel-btns');
      (spec.buttons || []).forEach(function (b, i) {
        var btn = el('button', 'btn' + (i === 0 ? ' btn-primary' : '') + ' btn-sm', '<span>' + (b.label || b.id) + '</span>');
        btn.addEventListener('click', function () {
          BW.sound('click');
          btn.disabled = true;
          var data = {};
          Object.keys(values).forEach(function (k) {
            var v = values[k];
            data[k] = v.type === 'checkbox' ? v.checked : (v.type === 'number' ? Number(v.value) : v.value);
          });
          BW.plugins_ui(id, b.id, data).then(function (lines) {
            log.textContent = (lines || []).map(function (l) { return l.text; }).join('\n');
            return refresh();
          }, function (e) { Win.toast('Не удалось: ' + e, 'err'); }).finally(function () { btn.disabled = false; });
        });
        btnRow.appendChild(btn);
      });
      content.appendChild(btnRow);
    }

    function refresh() {
      return BW.plugins_panel(id).then(render, function (e) {
        content.replaceChildren();
        var err = el('div', 'plug-err');
        err.textContent = String(e);
        content.appendChild(err);
      });
    }

    refresh();
  }

  Apps.register('plugin_panel', { title: 'Окно плагина', icon: 'plugins', minWidth: 560, minHeight: 380, mount: mount });
})();
