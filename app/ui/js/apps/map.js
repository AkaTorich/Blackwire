'use strict';
// Карта сети — узлы и связи из net_map на canvas: известные ярко с подписью, остальные тускло,
// бегущие импульсы по связям, масштаб колесом, панорама мышью, панель узла справа.
// Файл: js/apps/map.js, стили: css/apps/map.css.

const MapApp = (function () {
  const REFRESH_MS = 10000;
  const WORLD = 1000;       // размер мира после нормализации координат
  const KIND_RU = {
    Player: 'Своя машина', Router: 'Маршрутизатор', Gateway: 'Шлюз', Server: 'Сервер', Workstation: 'Рабочая станция',
    Database: 'База данных', FileServer: 'Файловый сервер', Mail: 'Почтовый сервер', Web: 'Веб-сервер', Camera: 'Камера',
    Badge: 'Пропускная система', Atm: 'Банкомат', MedicalDevice: 'Медицинский прибор', Dns: 'DNS', Isp: 'Провайдер',
    Honeypot: 'Ловушка', HomePc: 'Домашний компьютер'
  };

  function el(tag, cls, text) {
    const e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text != null) e.textContent = text;
    return e;
  }

  function button(label, icon, cls) {
    const b = el('button', 'map-btn' + (cls ? ' ' + cls : ''));
    b.type = 'button';
    b.innerHTML = (icon ? Icons.get(icon, 14) : '') + '<span>' + label + '</span>';
    return b;
  }

  function themeColors() {
    const cs = getComputedStyle(document.documentElement);
    const g = function (v, def) { const x = cs.getPropertyValue(v).trim(); return x || def; };
    return {
      bg: g('--bg', '#050a10'), accent: g('--accent', '#2fe4ff'), accent2: g('--accent2', '#8ab4ff'), fg: g('--fg', '#dfe8f0'),
      dim: g('--fg-dim', '#6b7f8f'), border: g('--panel-border', '#1e3340'), ok: g('--ok', '#4df2a1'),
      warn: g('--warn', '#f2c94c'), err: g('--err', '#ff4d6d'), mono: g('--mono', 'Consolas, monospace')
    };
  }

  // Геометрическая метка по виду узла (в системе координат с центром в узле, радиус r)
  function drawMark(ctx, kind, r) {
    ctx.beginPath();
    switch (kind) {
      case 'Player':
        ctx.arc(0, 0, r, 0, Math.PI * 2); ctx.stroke();
        ctx.beginPath(); ctx.arc(0, 0, r * 0.45, 0, Math.PI * 2); ctx.fill();
        return;
      case 'Router':
        ctx.moveTo(0, -r); ctx.lineTo(r, 0); ctx.lineTo(0, r); ctx.lineTo(-r, 0); ctx.closePath(); ctx.stroke();
        ctx.beginPath(); ctx.moveTo(-r * 0.45, 0); ctx.lineTo(r * 0.45, 0); ctx.moveTo(0, -r * 0.45); ctx.lineTo(0, r * 0.45); ctx.stroke();
        return;
      case 'Gateway':
      case 'Isp':
        polygon(ctx, 6, r, Math.PI / 6); ctx.stroke();
        if (kind === 'Isp') { ctx.beginPath(); polygon(ctx, 6, r * 0.55, Math.PI / 6); ctx.stroke(); }
        return;
      case 'Server':
      case 'FileServer':
        ctx.rect(-r * 0.7, -r, r * 1.4, r * 2); ctx.stroke();
        ctx.beginPath();
        ctx.moveTo(-r * 0.7, -r * 0.33); ctx.lineTo(r * 0.7, -r * 0.33);
        ctx.moveTo(-r * 0.7, r * 0.33); ctx.lineTo(r * 0.7, r * 0.33);
        ctx.stroke();
        if (kind === 'FileServer') { ctx.beginPath(); ctx.arc(r * 0.35, r * 0.66, r * 0.12, 0, Math.PI * 2); ctx.fill(); }
        return;
      case 'Workstation':
      case 'HomePc':
        ctx.rect(-r, -r * 0.8, r * 2, r * 1.3); ctx.stroke();
        ctx.beginPath(); ctx.moveTo(-r * 0.5, r * 0.9); ctx.lineTo(r * 0.5, r * 0.9); ctx.stroke();
        if (kind === 'HomePc') { ctx.beginPath(); ctx.moveTo(-r, -r * 0.8); ctx.lineTo(0, -r * 1.4); ctx.lineTo(r, -r * 0.8); ctx.stroke(); }
        return;
      case 'Database':
        ctx.ellipse(0, -r * 0.6, r * 0.85, r * 0.35, 0, 0, Math.PI * 2); ctx.stroke();
        ctx.beginPath(); ctx.moveTo(-r * 0.85, -r * 0.6); ctx.lineTo(-r * 0.85, r * 0.6);
        ctx.moveTo(r * 0.85, -r * 0.6); ctx.lineTo(r * 0.85, r * 0.6); ctx.stroke();
        ctx.beginPath(); ctx.ellipse(0, r * 0.6, r * 0.85, r * 0.35, 0, 0, Math.PI); ctx.stroke();
        return;
      case 'Mail':
        ctx.rect(-r, -r * 0.7, r * 2, r * 1.4); ctx.stroke();
        ctx.beginPath(); ctx.moveTo(-r, -r * 0.7); ctx.lineTo(0, r * 0.15); ctx.lineTo(r, -r * 0.7); ctx.stroke();
        return;
      case 'Web':
        ctx.arc(0, 0, r, 0, Math.PI * 2); ctx.stroke();
        ctx.beginPath(); ctx.ellipse(0, 0, r * 0.45, r, 0, 0, Math.PI * 2); ctx.stroke();
        ctx.beginPath(); ctx.moveTo(-r, 0); ctx.lineTo(r, 0); ctx.stroke();
        return;
      case 'Camera':
        ctx.arc(0, 0, r, 0, Math.PI * 2); ctx.stroke();
        ctx.beginPath(); ctx.arc(0, 0, r * 0.4, 0, Math.PI * 2); ctx.stroke();
        ctx.beginPath(); ctx.arc(r * 0.15, -r * 0.15, r * 0.1, 0, Math.PI * 2); ctx.fill();
        return;
      case 'Badge':
        roundRect(ctx, -r * 0.7, -r, r * 1.4, r * 2, r * 0.25); ctx.stroke();
        ctx.beginPath(); ctx.arc(0, -r * 0.3, r * 0.3, 0, Math.PI * 2); ctx.stroke();
        ctx.beginPath(); ctx.moveTo(-r * 0.4, r * 0.5); ctx.lineTo(r * 0.4, r * 0.5); ctx.stroke();
        return;
      case 'Atm':
        ctx.rect(-r * 0.8, -r, r * 1.6, r * 2); ctx.stroke();
        ctx.beginPath(); ctx.rect(-r * 0.5, -r * 0.65, r, r * 0.6); ctx.stroke();
        ctx.beginPath(); ctx.moveTo(-r * 0.5, r * 0.4); ctx.lineTo(r * 0.5, r * 0.4); ctx.stroke();
        return;
      case 'MedicalDevice':
        ctx.arc(0, 0, r, 0, Math.PI * 2); ctx.stroke();
        ctx.beginPath(); ctx.moveTo(-r * 0.55, 0); ctx.lineTo(r * 0.55, 0); ctx.moveTo(0, -r * 0.55); ctx.lineTo(0, r * 0.55);
        ctx.lineWidth *= 2; ctx.stroke(); ctx.lineWidth /= 2;
        return;
      case 'Dns':
        polygon(ctx, 3, r, -Math.PI / 2); ctx.stroke();
        ctx.beginPath(); ctx.arc(0, r * 0.2, r * 0.18, 0, Math.PI * 2); ctx.fill();
        return;
      case 'Honeypot':
        polygon(ctx, 3, r, Math.PI / 2); ctx.stroke();
        ctx.beginPath(); ctx.moveTo(0, -r * 0.5); ctx.lineTo(0, r * 0.1); ctx.stroke();
        ctx.beginPath(); ctx.arc(0, r * 0.4, r * 0.1, 0, Math.PI * 2); ctx.fill();
        return;
      default:
        ctx.arc(0, 0, r * 0.6, 0, Math.PI * 2); ctx.stroke();
    }
  }

  function polygon(ctx, n, r, rot) {
    for (let i = 0; i < n; i++) {
      const a = rot + i * Math.PI * 2 / n;
      const x = Math.cos(a) * r, y = Math.sin(a) * r;
      if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
    }
    ctx.closePath();
  }

  function roundRect(ctx, x, y, w, h, r) {
    ctx.moveTo(x + r, y);
    ctx.arcTo(x + w, y, x + w, y + h, r);
    ctx.arcTo(x + w, y + h, x, y + h, r);
    ctx.arcTo(x, y + h, x, y, r);
    ctx.arcTo(x, y, x + w, y, r);
    ctx.closePath();
  }

  function mount(body, arg) {
    const st = {
      nodes: [], links: [], byId: {}, pulses: [],
      scale: 1, ox: 0, oy: 0, fitted: false,
      hover: null, selected: null, showUnknown: false,
      drag: null, colors: themeColors(), raf: 0, timer: null, lastTs: 0, busy: false
    };

    // --- разметка ---
    const root = el('div', 'map');
    const canvas = el('canvas', 'map-canvas');
    const tools = el('div', 'map-tools');
    const btnRefresh = button('Обновить', 'reload');
    const btnUnknown = button('Неизвестные', 'eye-off', 'toggle');
    const btnFit = button('Вся сеть', 'layout');
    const stat = el('span', 'map-stat', '');
    tools.append(btnRefresh, btnUnknown, btnFit, stat);

    const panel = el('div', 'map-panel');
    const pHead = el('div', 'map-panel-head');
    const pName = el('div', 'map-panel-name', '');
    const pClose = el('button', 'map-panel-close');
    pClose.type = 'button';
    pClose.innerHTML = Icons.get('close', 14);
    pHead.append(pName, pClose);
    const pBody = el('div', 'map-panel-body');
    function prow(k) {
      const r = el('div', 'map-prow');
      const kk = el('span', 'map-pk', k);
      const vv = el('span', 'map-pv', '');
      r.append(kk, vv);
      pBody.appendChild(r);
      return { row: r, v: vv };
    }
    const pIp = prow('IP');
    const pKind = prow('Вид');
    const pOrg = prow('Организация');
    const pState = prow('Состояние');
    const pActions = el('div', 'map-panel-actions');
    const bSsh = button('Терминал → ssh', 'terminal', 'primary');
    const bScan = button('Сканировать', 'search');
    const bPing = button('Пинг', 'net');
    pActions.append(bSsh, bScan, bPing);
    panel.append(pHead, pBody, pActions);

    const tip = el('div', 'map-tip', '');
    root.append(canvas, tools, panel, tip);
    body.appendChild(root);

    // --- данные ---
    function normalize(nodes) {
      let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
      for (const n of nodes) {
        const x = Number(n.x) || 0, y = Number(n.y) || 0;
        if (x < minX) minX = x; if (x > maxX) maxX = x;
        if (y < minY) minY = y; if (y > maxY) maxY = y;
      }
      if (!nodes.length) return;
      const spanX = Math.max(1e-6, maxX - minX), spanY = Math.max(1e-6, maxY - minY);
      const span = Math.max(spanX, spanY);
      for (const n of nodes) {
        n.wx = ((Number(n.x) || 0) - minX) / span * WORLD;
        n.wy = ((Number(n.y) || 0) - minY) / span * WORLD;
      }
    }

    async function refresh() {
      if (st.busy) return;
      st.busy = true;
      btnRefresh.classList.add('spin');
      try {
        const m = await BW.net_map();
        const nodes = Array.isArray(m && m.nodes) ? m.nodes : [];
        const links = Array.isArray(m && m.links) ? m.links : [];
        // анимация всегда работает с одинаковым размером мира
        normalize(nodes);
        st.nodes = nodes;
        st.byId = {};
        for (const n of nodes) st.byId[n.id] = n;
        st.links = links.filter(function (l) { return st.byId[l.a] && st.byId[l.b]; });
        rebuildPulses();
        if (st.selected != null) {
          const sel = st.byId[st.selected];
          if (sel) fillPanel(sel); else { st.selected = null; panel.classList.remove('on'); }
        }
        if (!st.fitted) fitView();
        updateStat();
      } catch (e) {
        stat.textContent = String(e);
      } finally {
        st.busy = false;
        btnRefresh.classList.remove('spin');
      }
    }

    function rebuildPulses() {
      const old = {};
      for (const p of st.pulses) old[p.a + '|' + p.b] = p;
      st.pulses = [];
      for (const l of st.links) {
        const a = st.byId[l.a], b = st.byId[l.b];
        if (!a.known || !b.known) continue;
        const key = l.a + '|' + l.b;
        st.pulses.push(old[key] || { a: l.a, b: l.b, t: Math.random(), speed: 0.12 + Math.random() * 0.1, dir: Math.random() < 0.5 ? 1 : -1 });
      }
    }

    function updateStat() {
      const known = st.nodes.filter(function (n) { return n.known; }).length;
      stat.textContent = known + ' известно · ' + st.nodes.length + ' всего';
    }

    // --- вид ---
    function viewSize() { return { w: canvas.clientWidth, h: canvas.clientHeight }; }

    function fitView() {
      const vs = viewSize();
      if (vs.w < 10 || vs.h < 10) return;
      const shown = st.nodes.filter(function (n) { return n.known || st.showUnknown; });
      const list = shown.length ? shown : st.nodes;
      if (!list.length) return;
      let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
      for (const n of list) {
        if (n.wx < minX) minX = n.wx; if (n.wx > maxX) maxX = n.wx;
        if (n.wy < minY) minY = n.wy; if (n.wy > maxY) maxY = n.wy;
      }
      const pad = 90;
      const spanX = Math.max(50, maxX - minX), spanY = Math.max(50, maxY - minY);
      st.scale = Math.min((vs.w - pad * 2) / spanX, (vs.h - pad * 2) / spanY);
      st.scale = Math.max(0.05, Math.min(6, st.scale));
      st.ox = vs.w / 2 - (minX + spanX / 2) * st.scale;
      st.oy = vs.h / 2 - (minY + spanY / 2) * st.scale;
      st.fitted = true;
      st.userMoved = false;
    }

    function toScreen(n) { return { x: n.wx * st.scale + st.ox, y: n.wy * st.scale + st.oy }; }

    function nodeRadius(n) { return n.known ? 11 : 5; }

    function hitTest(mx, my) {
      let best = null, bestD = 1e9;
      for (const n of st.nodes) {
        if (!n.known && !st.showUnknown) continue;
        const p = toScreen(n);
        const d = Math.hypot(p.x - mx, p.y - my);
        const r = nodeRadius(n) + 6;
        if (d < r && d < bestD) { best = n; bestD = d; }
      }
      return best;
    }

    // --- рисование ---
    function draw(ts) {
      const dpr = window.devicePixelRatio || 1;
      const vs = viewSize();
      if (vs.w < 2 || vs.h < 2) return;
      if (canvas.width !== Math.round(vs.w * dpr) || canvas.height !== Math.round(vs.h * dpr)) {
        canvas.width = Math.round(vs.w * dpr);
        canvas.height = Math.round(vs.h * dpr);
      }
      const dt = st.lastTs ? Math.min(0.1, (ts - st.lastTs) / 1000) : 0;
      st.lastTs = ts;
      const c = st.colors;
      const ctx = canvas.getContext('2d');
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.clearRect(0, 0, vs.w, vs.h);

      // фоновая сетка
      const grid = 50 * st.scale;
      if (grid > 14) {
        ctx.strokeStyle = c.border;
        ctx.globalAlpha = 0.35;
        ctx.lineWidth = 1;
        ctx.beginPath();
        for (let x = ((st.ox % grid) + grid) % grid; x < vs.w; x += grid) { ctx.moveTo(Math.round(x) + 0.5, 0); ctx.lineTo(Math.round(x) + 0.5, vs.h); }
        for (let y = ((st.oy % grid) + grid) % grid; y < vs.h; y += grid) { ctx.moveTo(0, Math.round(y) + 0.5); ctx.lineTo(vs.w, Math.round(y) + 0.5); }
        ctx.stroke();
        ctx.globalAlpha = 1;
      }

      // связи
      for (const l of st.links) {
        const a = st.byId[l.a], b = st.byId[l.b];
        const bright = a.known && b.known;
        if (!bright && !st.showUnknown) continue;
        const pa = toScreen(a), pb = toScreen(b);
        const hl = (st.hover != null && (st.hover === a || st.hover === b)) ||
          (st.selected != null && (a.id === st.selected || b.id === st.selected));
        ctx.strokeStyle = bright ? (hl ? c.accent : c.accent2) : c.dim;
        ctx.globalAlpha = bright ? (hl ? 0.95 : 0.55) : 0.2;
        ctx.lineWidth = hl ? 1.5 : 1;
        if (!bright) ctx.setLineDash([3, 5]);
        ctx.beginPath(); ctx.moveTo(pa.x, pa.y); ctx.lineTo(pb.x, pb.y); ctx.stroke();
        ctx.setLineDash([]);
      }
      ctx.globalAlpha = 1;

      // импульсы
      for (const p of st.pulses) {
        const a = st.byId[p.a], b = st.byId[p.b];
        if (!a || !b) continue;
        p.t += p.speed * dt * p.dir;
        if (p.t > 1) { p.t = 1; p.dir = -1; p.speed = 0.1 + Math.random() * 0.12; }
        if (p.t < 0) { p.t = 0; p.dir = 1; p.speed = 0.1 + Math.random() * 0.12; }
        const pa = toScreen(a), pb = toScreen(b);
        const x = pa.x + (pb.x - pa.x) * p.t, y = pa.y + (pb.y - pa.y) * p.t;
        ctx.fillStyle = c.accent;
        ctx.shadowColor = c.accent;
        ctx.shadowBlur = 10;
        ctx.beginPath(); ctx.arc(x, y, 2, 0, Math.PI * 2); ctx.fill();
        ctx.shadowBlur = 0;
      }

      // узлы
      const labelOk = st.scale > 0.35;
      ctx.lineWidth = 1.3;
      ctx.lineJoin = 'round';
      for (const n of st.nodes) {
        if (!n.known && !st.showUnknown) continue;
        const p = toScreen(n);
        if (p.x < -40 || p.y < -40 || p.x > vs.w + 40 || p.y > vs.h + 40) continue;
        const isSel = n.id === st.selected, isHover = n === st.hover;
        if (!n.known) {
          ctx.fillStyle = c.dim;
          ctx.globalAlpha = isHover ? 0.9 : 0.45;
          ctx.beginPath(); ctx.arc(p.x, p.y, isHover ? 4 : 3, 0, Math.PI * 2); ctx.fill();
          ctx.globalAlpha = 1;
          continue;
        }
        const color = n.kind === 'Player' ? c.ok : n.kind === 'Honeypot' ? c.warn : c.accent;
        ctx.save();
        ctx.translate(p.x, p.y);
        if (isSel || isHover) {
          ctx.strokeStyle = color;
          ctx.globalAlpha = isSel ? 0.9 : 0.5;
          ctx.shadowColor = color;
          ctx.shadowBlur = 16;
          ctx.beginPath(); ctx.arc(0, 0, 18, 0, Math.PI * 2); ctx.stroke();
          ctx.globalAlpha = 1;
        }
        // подложка
        ctx.fillStyle = c.bg;
        ctx.globalAlpha = 0.85;
        ctx.shadowBlur = 0;
        ctx.beginPath(); ctx.arc(0, 0, 13, 0, Math.PI * 2); ctx.fill();
        ctx.globalAlpha = 1;
        ctx.strokeStyle = color;
        ctx.fillStyle = color;
        ctx.shadowColor = color;
        ctx.shadowBlur = isHover || isSel ? 14 : 8;
        drawMark(ctx, n.kind, 8);
        ctx.shadowBlur = 0;
        ctx.restore();

        if (labelOk) {
          ctx.font = '11px ' + c.mono;
          ctx.textAlign = 'center';
          ctx.textBaseline = 'top';
          ctx.fillStyle = c.fg;
          ctx.globalAlpha = isSel || isHover ? 1 : 0.9;
          ctx.fillText(n.hostname || n.ip || '', p.x, p.y + 17);
          if (st.scale > 0.6 || isSel || isHover) {
            ctx.fillStyle = c.dim;
            ctx.fillText(n.ip || '', p.x, p.y + 30);
          }
          ctx.globalAlpha = 1;
        }
      }
    }

    function loop(ts) {
      st.raf = 0;
      if (document.hidden) { st.lastTs = 0; return; }
      draw(ts);
      st.raf = requestAnimationFrame(loop);
    }

    function startLoop() { if (!st.raf && !document.hidden) st.raf = requestAnimationFrame(loop); }

    // --- панель узла ---
    function fillPanel(n) {
      pName.textContent = n.hostname || n.ip || ('узел ' + n.id);
      pIp.v.textContent = n.ip || '—';
      pKind.v.textContent = KIND_RU[n.kind] || n.kind || '—';
      const org = n.org_name || n.org || '';
      pOrg.row.classList.toggle('hidden', !org);
      pOrg.v.textContent = org;
      pState.v.textContent = n.known ? 'известен' : 'не исследован';
      pState.v.className = 'map-pv ' + (n.known ? 'ok' : 'dim');
      const isPlayer = n.kind === 'Player';
      bSsh.disabled = isPlayer || !n.ip;
      bScan.disabled = isPlayer || !n.ip;
      bPing.disabled = isPlayer || !n.ip;
      panel.classList.add('on');
    }

    function select(n) {
      st.selected = n ? n.id : null;
      if (n) { fillPanel(n); BW.sound('click'); } else panel.classList.remove('on');
    }

    function openTerminal(cmd) {
      const n = st.selected != null ? st.byId[st.selected] : null;
      if (!n || !n.ip) return;
      BW.open_app('terminal', cmd + ' ' + n.ip).catch(function (e) { Win.toast(String(e)); });
    }

    // --- мышь ---
    function mousePos(e) {
      const r = canvas.getBoundingClientRect();
      return { x: e.clientX - r.left, y: e.clientY - r.top };
    }

    canvas.addEventListener('mousedown', function (e) {
      if (e.button !== 0) return;
      const m = mousePos(e);
      st.drag = { x: m.x, y: m.y, ox: st.ox, oy: st.oy, moved: false };
      canvas.classList.add('dragging');
    });

    window.addEventListener('mousemove', function (e) {
      const m = mousePos(e);
      if (st.drag) {
        const dx = m.x - st.drag.x, dy = m.y - st.drag.y;
        if (Math.abs(dx) > 3 || Math.abs(dy) > 3) st.drag.moved = true;
        if (st.drag.moved) { st.ox = st.drag.ox + dx; st.oy = st.drag.oy + dy; st.userMoved = true; }
        return;
      }
      const h = hitTest(m.x, m.y);
      if (h !== st.hover) {
        st.hover = h;
        canvas.classList.toggle('hover', !!h);
        if (h) {
          tip.textContent = h.known ? (h.hostname || h.ip) + (h.ip && h.hostname ? '  ' + h.ip : '') : 'неизвестный узел';
          tip.classList.add('on');
        } else tip.classList.remove('on');
      }
      if (h) {
        tip.style.left = (m.x + 14) + 'px';
        tip.style.top = (m.y + 14) + 'px';
      }
    });

    window.addEventListener('mouseup', function (e) {
      if (!st.drag) return;
      const wasClick = !st.drag.moved;
      st.drag = null;
      canvas.classList.remove('dragging');
      if (wasClick && e.target === canvas) {
        const m = mousePos(e);
        select(hitTest(m.x, m.y));
      }
    });

    canvas.addEventListener('wheel', function (e) {
      e.preventDefault();
      const m = mousePos(e);
      const factor = e.deltaY < 0 ? 1.15 : 1 / 1.15;
      const ns = Math.max(0.05, Math.min(8, st.scale * factor));
      const f = ns / st.scale;
      st.ox = m.x - (m.x - st.ox) * f;
      st.oy = m.y - (m.y - st.oy) * f;
      st.scale = ns;
      st.userMoved = true;
    }, { passive: false });

    canvas.addEventListener('dblclick', function (e) {
      const m = mousePos(e);
      const n = hitTest(m.x, m.y);
      if (n && n.known && n.kind !== 'Player') { select(n); openTerminal('ssh'); }
    });

    canvas.addEventListener('contextmenu', function (e) {
      e.preventDefault();
      const m = mousePos(e);
      const n = hitTest(m.x, m.y);
      if (n && n.known) {
        select(n);
        if (n.kind === 'Player') return;
        Win.menu([
          { label: 'Терминал → ssh ' + n.ip, icon: 'terminal', action: function () { openTerminal('ssh'); } },
          { label: 'Сканировать', icon: 'search', action: function () { openTerminal('scan'); } },
          { label: 'Пинг', icon: 'net', action: function () { openTerminal('ping'); } }
        ], e.clientX, e.clientY);
      } else {
        Win.menu([
          { label: 'Обновить', icon: 'reload', action: refresh },
          { label: 'Вся сеть', icon: 'layout', action: fitView },
          { label: st.showUnknown ? 'Скрыть неизвестные' : 'Показать неизвестные', icon: st.showUnknown ? 'eye-off' : 'eye', action: toggleUnknown }
        ], e.clientX, e.clientY);
      }
    });

    function toggleUnknown() {
      st.showUnknown = !st.showUnknown;
      btnUnknown.classList.toggle('on', st.showUnknown);
      btnUnknown.innerHTML = Icons.get(st.showUnknown ? 'eye' : 'eye-off', 14) + '<span>Неизвестные</span>';
    }

    // --- кнопки и клавиши ---
    btnRefresh.addEventListener('click', refresh);
    btnUnknown.addEventListener('click', toggleUnknown);
    btnFit.addEventListener('click', fitView);
    pClose.addEventListener('click', function () { select(null); });
    bSsh.addEventListener('click', function () { openTerminal('ssh'); });
    bScan.addEventListener('click', function () { openTerminal('scan'); });
    bPing.addEventListener('click', function () { openTerminal('ping'); });

    window.addEventListener('keydown', function (e) {
      const t = e.target;
      if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA')) return;
      if (e.key === 'F5') { e.preventDefault(); refresh(); }
      else if (e.key === 'Escape') { select(null); }
      else if (e.key === 'Home' || e.key === '0') { fitView(); }
      else if (e.key === '+' || e.key === '=') { zoomCenter(1.2); }
      else if (e.key === '-') { zoomCenter(1 / 1.2); }
    });

    function zoomCenter(factor) {
      const vs = viewSize();
      const ns = Math.max(0.05, Math.min(8, st.scale * factor));
      const f = ns / st.scale;
      st.ox = vs.w / 2 - (vs.w / 2 - st.ox) * f;
      st.oy = vs.h / 2 - (vs.h / 2 - st.oy) * f;
      st.scale = ns;
    }

    document.addEventListener('visibilitychange', function () {
      if (!document.hidden) { startLoop(); refresh(); }
    });
    if (window.ResizeObserver) {
      // пока игрок сам не двигал карту, при каждом изменении окна вписываем сеть заново
      new ResizeObserver(function () {
        if (st.nodes.length && !st.userMoved) fitView();
      }).observe(canvas);
    }
    BW.onSettings(function () { st.colors = themeColors(); });

    Promise.resolve(BW.ready).then(function () {
      Win.setTitle('Карта сети');
      startLoop();
      refresh();
      st.timer = setInterval(function () { if (!document.hidden) refresh(); }, REFRESH_MS);
    });
    window.addEventListener('pagehide', function () {
      if (st.timer) clearInterval(st.timer);
      if (st.raf) cancelAnimationFrame(st.raf);
    });
  }

  return { mount: mount };
})();

Apps.register('map', {
  title: 'Карта сети', icon: 'map', minWidth: 640, minHeight: 420,
  mount: function (body, arg) { MapApp.mount(body, arg); }
});
