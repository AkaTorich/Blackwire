'use strict';
// Монитор — графики CPU/память/сеть на canvas за 120 с, таблица процессов, карточка машины.
// Данные — sys_status раз в секунду. Файл: js/apps/monitor.js, стили: css/apps/monitor.css.

const MonitorApp = (function () {
  const HISTORY = 120;   // секунд истории

  function el(tag, cls, text) {
    const e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text != null) e.textContent = text;
    return e;
  }

  function fmtMb(mb) {
    if (mb == null) return '—';
    if (mb >= 1024) return (mb / 1024).toFixed(2) + ' ГБ';
    return Math.round(mb) + ' МБ';
  }

  function fmtKbs(v) {
    if (v == null) return '—';
    if (v >= 1024) return (v / 1024).toFixed(1) + ' МБ/с';
    return (v < 10 ? v.toFixed(1) : Math.round(v)) + ' кБ/с';
  }

  function two(n) { return n < 10 ? '0' + n : String(n); }

  function fmtUptime(sec) {
    sec = Math.max(0, Math.floor(sec || 0));
    const d = Math.floor(sec / 86400), h = Math.floor(sec % 86400 / 3600), m = Math.floor(sec % 3600 / 60), s = sec % 60;
    return (d ? d + ' д ' : '') + two(h) + ':' + two(m) + ':' + two(s);
  }

  // Цвета темы для canvas
  function themeColors() {
    const cs = getComputedStyle(document.documentElement);
    const g = function (v, def) { const x = cs.getPropertyValue(v).trim(); return x || def; };
    return {
      accent: g('--accent', '#2fe4ff'), accent2: g('--accent2', '#8ab4ff'), fg: g('--fg', '#dfe8f0'),
      dim: g('--fg-dim', '#6b7f8f'), border: g('--panel-border', '#1e3340'), ok: g('--ok', '#4df2a1'),
      warn: g('--warn', '#f2c94c'), err: g('--err', '#ff4d6d'), mono: g('--mono', 'Consolas, monospace')
    };
  }

  // Кольцевая история значений
  function series(n) {
    const a = new Array(n);
    for (let i = 0; i < n; i++) a[i] = null;
    return { data: a, push: function (v) { a.push(v); a.shift(); } };
  }

  // Рисование одного графика. lines — [{data, color}], max — верхняя граница (null — авто)
  function drawChart(canvas, lines, max, colors, unitFmt) {
    const dpr = window.devicePixelRatio || 1;
    const w = canvas.clientWidth, h = canvas.clientHeight;
    if (w < 2 || h < 2) return;
    if (canvas.width !== Math.round(w * dpr) || canvas.height !== Math.round(h * dpr)) {
      canvas.width = Math.round(w * dpr);
      canvas.height = Math.round(h * dpr);
    }
    const ctx = canvas.getContext('2d');
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);
    const padL = 6, padR = 6, padT = 8, padB = 6;
    const gw = w - padL - padR, gh = h - padT - padB;

    let top = max;
    if (top == null) {
      top = 1;
      for (const ln of lines) for (const v of ln.data) if (v != null && v > top) top = v;
      top = niceCeil(top * 1.15);
    }

    // сетка
    ctx.strokeStyle = colors.border;
    ctx.lineWidth = 1;
    ctx.globalAlpha = 0.9;
    ctx.beginPath();
    for (let i = 0; i <= 4; i++) {
      const y = Math.round(padT + gh * i / 4) + 0.5;
      ctx.moveTo(padL, y); ctx.lineTo(padL + gw, y);
    }
    for (let i = 0; i <= 4; i++) {
      const x = Math.round(padL + gw * i / 4) + 0.5;
      ctx.moveTo(x, padT); ctx.lineTo(x, padT + gh);
    }
    ctx.stroke();
    ctx.globalAlpha = 1;

    // подписи шкалы
    ctx.fillStyle = colors.dim;
    ctx.font = '10px ' + colors.mono;
    ctx.textAlign = 'right';
    ctx.textBaseline = 'top';
    ctx.fillText(unitFmt(top), padL + gw - 3, padT + 2);
    ctx.textBaseline = 'bottom';
    ctx.fillText(unitFmt(top / 2), padL + gw - 3, padT + gh / 2 - 1);

    const n = HISTORY;
    const stepX = gw / (n - 1);
    for (const ln of lines) {
      // заливка
      let started = false;
      ctx.beginPath();
      for (let i = 0; i < n; i++) {
        const v = ln.data[i];
        if (v == null) { continue; }
        const x = padL + i * stepX;
        const y = padT + gh - Math.min(1, Math.max(0, v / top)) * gh;
        if (!started) { ctx.moveTo(x, padT + gh); ctx.lineTo(x, y); started = true; }
        else ctx.lineTo(x, y);
      }
      if (!started) continue;
      ctx.lineTo(padL + gw, padT + gh);
      ctx.closePath();
      const grad = ctx.createLinearGradient(0, padT, 0, padT + gh);
      grad.addColorStop(0, ln.color);
      grad.addColorStop(1, 'rgba(0,0,0,0)');
      ctx.globalAlpha = 0.22;
      ctx.fillStyle = grad;
      ctx.fill();
      ctx.globalAlpha = 1;

      // линия со свечением
      ctx.beginPath();
      started = false;
      for (let i = 0; i < n; i++) {
        const v = ln.data[i];
        if (v == null) continue;
        const x = padL + i * stepX;
        const y = padT + gh - Math.min(1, Math.max(0, v / top)) * gh;
        if (!started) { ctx.moveTo(x, y); started = true; } else ctx.lineTo(x, y);
      }
      ctx.strokeStyle = ln.color;
      ctx.lineWidth = 1.5;
      ctx.lineJoin = 'round';
      ctx.shadowColor = ln.color;
      ctx.shadowBlur = 10;
      ctx.stroke();
      ctx.shadowBlur = 0;

      // точка на конце
      const last = ln.data[n - 1];
      if (last != null) {
        const x = padL + gw, y = padT + gh - Math.min(1, Math.max(0, last / top)) * gh;
        ctx.fillStyle = ln.color;
        ctx.shadowColor = ln.color;
        ctx.shadowBlur = 12;
        ctx.beginPath();
        ctx.arc(x, y, 2.5, 0, Math.PI * 2);
        ctx.fill();
        ctx.shadowBlur = 0;
      }
    }
  }

  function niceCeil(v) {
    if (v <= 0) return 1;
    const p = Math.pow(10, Math.floor(Math.log10(v)));
    const m = v / p;
    const k = m <= 1 ? 1 : m <= 2 ? 2 : m <= 5 ? 5 : 10;
    return k * p;
  }

  function mount(body) {
    const st = {
      cpu: series(HISTORY), mem: series(HISTORY), netIn: series(HISTORY), netOut: series(HISTORY),
      status: null, sortKey: 'cpu', sortDir: -1, selected: null, timer: null, colors: themeColors(), busy: false
    };

    // --- разметка ---
    const root = el('div', 'mon');
    const cards = el('div', 'mon-cards');
    function card(title, cls) {
      const c = el('div', 'mon-card ' + cls);
      const head = el('div', 'mon-card-head');
      const t = el('span', 'mon-card-title', title);
      const v = el('span', 'mon-card-value', '—');
      const s = el('span', 'mon-card-sub', '');
      head.append(t, v, s);
      const cv = el('canvas', 'mon-canvas');
      c.append(head, cv);
      cards.appendChild(c);
      return { el: c, value: v, sub: s, canvas: cv };
    }
    const cCpu = card('Процессор', 'cpu');
    const cMem = card('Память', 'mem');
    const cNet = card('Сеть', 'net');
    const legend = el('span', 'mon-legend');
    legend.innerHTML = '<i class="in"></i>вход <i class="out"></i>выход';
    cNet.el.querySelector('.mon-card-head').appendChild(legend);

    const lower = el('div', 'mon-lower');
    const machine = el('div', 'mon-machine');
    const mTitle = el('div', 'mon-machine-title');
    mTitle.innerHTML = Icons.get('cpu', 16) + '<span>Машина</span>';
    const mBody = el('div', 'mon-machine-body');
    machine.append(mTitle, mBody);
    function mrow(k) {
      const r = el('div', 'mon-mrow');
      const kk = el('span', 'mon-mk', k);
      const vv = el('span', 'mon-mv', '—');
      r.append(kk, vv);
      mBody.appendChild(r);
      return vv;
    }
    const mHost = mrow('Имя');
    const mIp = mrow('IP');
    const mUp = mrow('Работает');
    const mMem = mrow('Память');
    const mProcs = mrow('Процессов');
    const mLoad = mrow('Нагрузка');

    const procs = el('div', 'mon-procs');
    const pHead = el('div', 'mon-procs-head');
    const pTitle = el('span', 'mon-procs-title', 'Процессы');
    const pCount = el('span', 'mon-procs-count', '');
    const btnKill = el('button', 'mon-kill');
    btnKill.type = 'button';
    btnKill.innerHTML = Icons.get('stop', 14) + '<span>Снять</span>';
    btnKill.disabled = true;
    pHead.append(pTitle, pCount, btnKill);
    const tableWrap = el('div', 'mon-table-wrap');
    const table = el('table', 'mon-table');
    const thead = el('thead');
    const trh = el('tr');
    const cols = [['pid', 'PID'], ['name', 'Имя'], ['user', 'Пользователь'], ['cpu', 'CPU %'], ['mem', 'Память']];
    for (const c of cols) {
      const th = el('th', 'col-' + c[0], c[1]);
      th.dataset.key = c[0];
      th.addEventListener('click', function () {
        if (st.sortKey === c[0]) st.sortDir = -st.sortDir;
        else { st.sortKey = c[0]; st.sortDir = (c[0] === 'cpu' || c[0] === 'mem') ? -1 : 1; }
        renderProcs();
      });
      trh.appendChild(th);
    }
    thead.appendChild(trh);
    const tbody = el('tbody');
    table.append(thead, tbody);
    tableWrap.appendChild(table);
    procs.append(pHead, tableWrap);
    lower.append(machine, procs);
    root.append(cards, lower);
    body.appendChild(root);

    // --- данные ---
    async function poll() {
      if (st.busy || document.hidden) return;
      st.busy = true;
      try {
        const s = await BW.sys_status();
        st.status = s;
        st.cpu.push(Number(s.cpu) || 0);
        st.mem.push(Number(s.mem_used) || 0);
        st.netIn.push(Number(s.net_in) || 0);
        st.netOut.push(Number(s.net_out) || 0);
        renderAll();
      } catch (e) {
        mHost.textContent = String(e);
      } finally {
        st.busy = false;
      }
    }

    function renderAll() {
      const s = st.status;
      if (!s) return;
      const c = st.colors;
      cCpu.value.textContent = Math.round(s.cpu) + ' %';
      cCpu.sub.textContent = '';
      cMem.value.textContent = fmtMb(s.mem_used);
      cMem.value.title = Math.round(s.mem_used) + ' МБ';
      cMem.sub.textContent = 'из ' + fmtMb(s.mem_total);
      cNet.value.textContent = fmtKbs(s.net_in);
      cNet.sub.textContent = '↑ ' + fmtKbs(s.net_out);
      drawChart(cCpu.canvas, [{ data: st.cpu.data, color: c.accent }], 100, c, function (v) { return Math.round(v) + '%'; });
      drawChart(cMem.canvas, [{ data: st.mem.data, color: c.accent2 }], s.mem_total > 0 ? s.mem_total : null, c, fmtMb);
      drawChart(cNet.canvas, [{ data: st.netIn.data, color: c.accent }, { data: st.netOut.data, color: c.ok }], null, c, fmtKbs);

      mHost.textContent = s.hostname || '—';
      mIp.textContent = s.ip || '—';
      mUp.textContent = fmtUptime(s.uptime);
      mMem.textContent = fmtMb(s.mem_used) + ' / ' + fmtMb(s.mem_total);
      mProcs.textContent = String((s.procs || []).length);
      mLoad.textContent = Math.round(s.cpu) + ' %';
      mLoad.className = 'mon-mv ' + (s.cpu >= 90 ? 'bad' : s.cpu >= 60 ? 'warn' : 'ok');
      renderProcs();
    }

    function sortedProcs() {
      const list = (st.status && st.status.procs ? st.status.procs : []).slice();
      const k = st.sortKey, d = st.sortDir;
      list.sort(function (a, b) {
        let r;
        if (k === 'name' || k === 'user') r = String(a[k] || '').localeCompare(String(b[k] || ''), 'ru');
        else r = (Number(a[k]) || 0) - (Number(b[k]) || 0);
        if (r === 0) r = (a.pid || 0) - (b.pid || 0);
        return r * d;
      });
      return list;
    }

    function renderProcs() {
      const list = sortedProcs();
      for (const th of trh.children) {
        th.classList.toggle('asc', th.dataset.key === st.sortKey && st.sortDir > 0);
        th.classList.toggle('desc', th.dataset.key === st.sortKey && st.sortDir < 0);
      }
      pCount.textContent = list.length ? String(list.length) : '';
      if (st.selected != null && !list.some(function (p) { return p.pid === st.selected; })) st.selected = null;
      const frag = document.createDocumentFragment();
      for (const p of list) {
        const tr = el('tr', p.pid === st.selected ? 'sel' : '');
        tr.dataset.pid = p.pid;
        tr.append(
          el('td', 'col-pid', String(p.pid)),
          el('td', 'col-name', p.name || ''),
          el('td', 'col-user', p.user || ''),
          el('td', 'col-cpu' + (p.cpu >= 50 ? ' hot' : ''), (Number(p.cpu) || 0).toFixed(1)),
          el('td', 'col-mem', fmtMb(p.mem)));
        tr.addEventListener('click', function () {
          st.selected = st.selected === p.pid ? null : p.pid;
          for (const row of tbody.children) row.classList.toggle('sel', Number(row.dataset.pid) === st.selected);
          btnKill.disabled = st.selected == null;
        });
        tr.addEventListener('dblclick', function () { st.selected = p.pid; killSelected(); });
        tr.addEventListener('contextmenu', function (e) {
          e.preventDefault();
          st.selected = p.pid;
          for (const row of tbody.children) row.classList.toggle('sel', Number(row.dataset.pid) === st.selected);
          btnKill.disabled = false;
          Win.menu([{ label: 'Снять процесс ' + p.pid, icon: 'stop', action: killSelected }], e.clientX, e.clientY);
        });
        frag.appendChild(tr);
      }
      tbody.textContent = '';
      tbody.appendChild(frag);
      btnKill.disabled = st.selected == null;
    }

    async function killSelected() {
      if (st.selected == null) return;
      const p = (st.status.procs || []).find(function (x) { return x.pid === st.selected; });
      const name = p ? p.name + ' (' + p.pid + ')' : String(st.selected);
      if (!(await Win.confirm('Снять процесс ' + name + '?'))) return;
      try {
        await BW.sys_kill(st.selected);
        Win.toast('Процесс снят: ' + name);
        st.selected = null;
        poll();
      } catch (e) {
        Win.toast(String(e));
        BW.sound('error');
      }
    }

    function redraw() {
      if (st.status) renderAll();
    }

    // --- события ---
    btnKill.addEventListener('click', killSelected);
    window.addEventListener('keydown', function (e) {
      const t = e.target;
      if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA')) return;
      if (e.key === 'Delete' && st.selected != null) { e.preventDefault(); killSelected(); }
      else if (e.key === 'F5') { e.preventDefault(); poll(); }
      else if (e.key === 'Escape') { st.selected = null; renderProcs(); }
    });
    if (window.ResizeObserver) {
      const ro = new ResizeObserver(function () { redraw(); });
      ro.observe(cards);
    } else {
      window.addEventListener('resize', redraw);
    }
    document.addEventListener('visibilitychange', function () { if (!document.hidden) poll(); });
    BW.onSettings(function () { st.colors = themeColors(); redraw(); });

    Promise.resolve(BW.ready).then(function () {
      poll();
      st.timer = setInterval(poll, 1000);
    });
    window.addEventListener('pagehide', function () { if (st.timer) clearInterval(st.timer); });
  }

  return { mount: mount };
})();

Apps.register('monitor', {
  title: 'Монитор', icon: 'monitor', minWidth: 620, minHeight: 420,
  mount: function (body, arg) { MonitorApp.mount(body, arg); }
});
