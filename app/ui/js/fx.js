'use strict';
/* Фон рабочего стола на canvas: проволочные многогранники, поток шестнадцатеричных цифр,
   «пакеты» по рёбрам, сканлайны. Каждый эффект включается settings.fx.*; пауза при скрытом окне. */
var FX = (function () {
  var canvas = null, ctx = null, settings = null;
  var running = false, raf = 0, lastTs = 0, startTs = 0;
  var w = 0, h = 0, dpr = 1, scale = 1;
  var colors = null;
  var shapes = [];
  var rain = [];
  var packets = [];
  var nextPacket = 0;
  var scanPattern = null;
  var bandY = 0;
  var observer = null;
  var HEX = '0123456789ABCDEF';

  /* ---------- геометрия ---------- */
  function edgesByMinDistance(v) {
    var min = Infinity, i, j;
    for (i = 0; i < v.length; i++) for (j = i + 1; j < v.length; j++) {
      var d = dist(v[i], v[j]);
      if (d < min - 1e-6) min = d;
    }
    var e = [];
    for (i = 0; i < v.length; i++) for (j = i + 1; j < v.length; j++) {
      if (Math.abs(dist(v[i], v[j]) - min) < 1e-3) e.push([i, j]);
    }
    return e;
  }

  function dist(a, b) {
    var dx = a[0] - b[0], dy = a[1] - b[1], dz = a[2] - b[2];
    return Math.sqrt(dx * dx + dy * dy + dz * dz);
  }

  function normalize(v) {
    var r = 0;
    v.forEach(function (p) { r = Math.max(r, Math.sqrt(p[0] * p[0] + p[1] * p[1] + p[2] * p[2])); });
    return v.map(function (p) { return [p[0] / r, p[1] / r, p[2] / r]; });
  }

  function cube() {
    var v = [];
    for (var x = -1; x <= 1; x += 2) for (var y = -1; y <= 1; y += 2) for (var z = -1; z <= 1; z += 2) v.push([x, y, z]);
    return { v: normalize(v), e: edgesByMinDistance(v) };
  }

  function icosahedron() {
    var p = (1 + Math.sqrt(5)) / 2, v = [];
    [-1, 1].forEach(function (a) { [-1, 1].forEach(function (b) {
      v.push([0, a, b * p]); v.push([a, b * p, 0]); v.push([a * p, 0, b]);
    }); });
    return { v: normalize(v), e: edgesByMinDistance(v) };
  }

  function dodecahedron() {
    var p = (1 + Math.sqrt(5)) / 2, q = 1 / p, v = [];
    for (var x = -1; x <= 1; x += 2) for (var y = -1; y <= 1; y += 2) for (var z = -1; z <= 1; z += 2) v.push([x, y, z]);
    [-1, 1].forEach(function (a) { [-1, 1].forEach(function (b) {
      v.push([0, a * q, b * p]); v.push([a * q, b * p, 0]); v.push([a * p, 0, b * q]);
    }); });
    return { v: normalize(v), e: edgesByMinDistance(v) };
  }

  /* Расстановка фигур: доли экрана, размер — доля меньшей стороны */
  function buildShapes() {
    shapes = [
      { geo: icosahedron(), x: 0.74, y: 0.46, size: 0.30, rot: [0.4, 0.2, 0.1], spd: [0.05, 0.08, 0.02], alpha: 1 },
      { geo: cube(), x: 0.19, y: 0.64, size: 0.15, rot: [0.7, 0.3, 0.5], spd: [-0.07, 0.05, 0.03], alpha: 0.8 },
      { geo: dodecahedron(), x: 0.31, y: 0.30, size: 0.11, rot: [0.1, 0.9, 0.2], spd: [0.04, -0.06, 0.05], alpha: 0.7 }
    ];
    shapes.forEach(function (s) { s.proj = []; });
  }

  function project(s, t) {
    var ax = s.rot[0] + s.spd[0] * t, ay = s.rot[1] + s.spd[1] * t, az = s.rot[2] + s.spd[2] * t;
    var cx = Math.cos(ax), sx = Math.sin(ax), cy = Math.cos(ay), sy = Math.sin(ay), cz = Math.cos(az), sz = Math.sin(az);
    var R = s.size * Math.min(w, h);
    var px = s.x * w, py = s.y * h;
    var f0 = 3.2; // расстояние камеры в радиусах
    s.proj = s.geo.v.map(function (p) {
      var x = p[0], y = p[1], z = p[2], y1, z1, x2, z2, x3, y3;
      y1 = y * cx - z * sx; z1 = y * sx + z * cx;
      x2 = x * cy + z1 * sy; z2 = -x * sy + z1 * cy;
      x3 = x2 * cz - y1 * sz; y3 = x2 * sz + y1 * cz;
      var f = f0 / (f0 - z2);
      return { x: px + x3 * R * f, y: py + y3 * R * f, z: z2 };
    });
  }

  /* ---------- цвета ---------- */
  function readColors() {
    colors = {
      accent: Theme.color('--accent', '#39d8ff'),
      accent2: Theme.color('--accent2', '#8ff6ff'),
      glow: Theme.color('--glow', '#39d8ff'),
      fg: Theme.color('--fg', '#dff6ff')
    };
  }

  function rgba(c, a) { return 'rgba(' + c.r + ',' + c.g + ',' + c.b + ',' + a.toFixed(3) + ')'; }

  /* ---------- размер ---------- */
  function resize() {
    if (!canvas) return;
    dpr = Math.min(window.devicePixelRatio || 1, 2.5);
    w = canvas.clientWidth || window.innerWidth;
    h = canvas.clientHeight || window.innerHeight;
    canvas.width = Math.round(w * dpr);
    canvas.height = Math.round(h * dpr);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    scale = Math.max(0.7, Math.min(2.4, Math.min(w, h) / 900));
    rain = [];
    scanPattern = null;
  }

  /* ---------- сканлайны ---------- */
  function makeScanPattern() {
    var step = 3;
    var c = document.createElement('canvas');
    c.width = 1; c.height = step;
    var g = c.getContext('2d');
    g.fillStyle = 'rgba(0,0,0,0.22)';
    g.fillRect(0, 0, 1, 1);
    scanPattern = ctx.createPattern(c, 'repeat');
  }

  function drawScanlines(dt) {
    if (!scanPattern) makeScanPattern();
    ctx.fillStyle = scanPattern;
    ctx.fillRect(0, 0, w, h);
    bandY += dt * 38 * scale;
    if (bandY > h + 200) bandY = -200;
    var g = ctx.createLinearGradient(0, bandY - 120 * scale, 0, bandY + 120 * scale);
    g.addColorStop(0, rgba(colors.accent, 0));
    g.addColorStop(0.5, rgba(colors.accent, 0.035));
    g.addColorStop(1, rgba(colors.accent, 0));
    ctx.fillStyle = g;
    ctx.fillRect(0, bandY - 120 * scale, w, 240 * scale);
  }

  /* ---------- глубина ---------- */
  function drawDepth() {
    var g = ctx.createRadialGradient(w * 0.7, h * 0.45, 0, w * 0.7, h * 0.45, Math.max(w, h) * 0.75);
    g.addColorStop(0, rgba(colors.accent, 0.07));
    g.addColorStop(0.5, rgba(colors.accent, 0.015));
    g.addColorStop(1, 'rgba(0,0,0,0.35)');
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, w, h);
  }

  /* ---------- многогранники ---------- */
  function drawShapes(t) {
    ctx.lineCap = 'round';
    shapes.forEach(function (s) {
      project(s, t);
      var P = s.proj;
      /* свечение */
      ctx.lineWidth = 3.2 * scale;
      ctx.strokeStyle = rgba(colors.glow, 0.05 * s.alpha);
      ctx.beginPath();
      s.geo.e.forEach(function (e) { ctx.moveTo(P[e[0]].x, P[e[0]].y); ctx.lineTo(P[e[1]].x, P[e[1]].y); });
      ctx.stroke();
      /* рёбра: ближние ярче */
      ctx.lineWidth = 1;
      s.geo.e.forEach(function (e) {
        var z = (P[e[0]].z + P[e[1]].z) / 2;
        var a = (0.18 + (z + 1) * 0.22) * s.alpha;
        ctx.strokeStyle = rgba(colors.accent, a);
        ctx.beginPath();
        ctx.moveTo(P[e[0]].x, P[e[0]].y);
        ctx.lineTo(P[e[1]].x, P[e[1]].y);
        ctx.stroke();
      });
      /* вершины */
      P.forEach(function (p) {
        var a = (0.3 + (p.z + 1) * 0.3) * s.alpha;
        ctx.fillStyle = rgba(colors.accent2, a);
        ctx.beginPath();
        ctx.arc(p.x, p.y, 1.4 * scale, 0, Math.PI * 2);
        ctx.fill();
      });
    });
  }

  /* ---------- пакеты по рёбрам ---------- */
  function spawnPacket(t) {
    var s = shapes[Math.floor(Math.random() * shapes.length)];
    var e = s.geo.e[Math.floor(Math.random() * s.geo.e.length)];
    packets.push({ s: s, e: e, t0: t, dur: 0.6 + Math.random() * 0.7, hops: 1 + Math.floor(Math.random() * 3) });
    nextPacket = t + 1.2 + Math.random() * 2.5;
  }

  function drawPackets(t) {
    if (t >= nextPacket) spawnPacket(t);
    packets = packets.filter(function (p) {
      var k = (t - p.t0) / p.dur;
      if (k >= 1) {
        if (p.hops > 1) {
          /* следующий прыжок — по ребру из той же вершины */
          var from = p.e[1];
          var cand = p.s.geo.e.filter(function (e) { return (e[0] === from || e[1] === from) && e !== p.e; });
          if (cand.length) {
            var ne = cand[Math.floor(Math.random() * cand.length)];
            p.e = ne[0] === from ? ne : [ne[1], ne[0]];
            p.t0 = t; p.hops--;
            return true;
          }
        }
        return false;
      }
      var A = p.s.proj[p.e[0]], B = p.s.proj[p.e[1]];
      if (!A || !B) return false;
      for (var i = 0; i < 5; i++) {
        var kk = k - i * 0.035;
        if (kk < 0) break;
        var x = A.x + (B.x - A.x) * kk, y = A.y + (B.y - A.y) * kk;
        var a = (1 - i / 5) * 0.9;
        ctx.fillStyle = rgba(colors.accent2, a);
        ctx.beginPath();
        ctx.arc(x, y, (2.6 - i * 0.4) * scale, 0, Math.PI * 2);
        ctx.fill();
        if (i === 0) {
          ctx.fillStyle = rgba(colors.glow, 0.18);
          ctx.beginPath();
          ctx.arc(x, y, 9 * scale, 0, Math.PI * 2);
          ctx.fill();
        }
      }
      return true;
    });
  }

  /* ---------- поток шестнадцатеричных цифр ---------- */
  function drawRain(dt) {
    var cell = 15 * scale;
    var maxCols = Math.max(6, Math.floor(w / (cell * 7)));
    if (rain.length < maxCols && Math.random() < 0.05) {
      var len = 8 + Math.floor(Math.random() * 16);
      var chars = [];
      for (var c = 0; c < len; c++) chars.push(HEX[Math.floor(Math.random() * 16)]);
      rain.push({
        x: Math.floor(Math.random() * (w / cell)) * cell + cell / 2,
        y: -len * cell,
        speed: (40 + Math.random() * 70) * scale,
        len: len, chars: chars,
        dim: 0.35 + Math.random() * 0.5
      });
    }
    ctx.font = (11 * scale).toFixed(1) + 'px "Cascadia Mono", Consolas, monospace';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    rain = rain.filter(function (col) {
      col.y += col.speed * dt;
      if (Math.random() < 0.08) col.chars[Math.floor(Math.random() * col.len)] = HEX[Math.floor(Math.random() * 16)];
      for (var i = 0; i < col.len; i++) {
        var y = col.y - i * cell;
        if (y < -cell || y > h + cell) continue;
        var k = 1 - i / col.len;
        var a = (i === 0 ? 0.95 : k * k * 0.6) * col.dim;
        ctx.fillStyle = i === 0 ? rgba(colors.accent2, a) : rgba(colors.accent, a);
        ctx.fillText(col.chars[i], col.x, y);
      }
      return col.y - col.len * cell < h + cell;
    });
  }

  /* ---------- цикл ---------- */
  function frame(ts) {
    if (!running) return;
    raf = requestAnimationFrame(frame);
    if (!startTs) { startTs = ts; lastTs = ts; }
    if (ts - lastTs < 15) return; // не чаще ~60 кадров
    var dt = Math.min(0.1, (ts - lastTs) / 1000);
    lastTs = ts;
    var t = (ts - startTs) / 1000;
    var fx = (settings && settings.fx) || {};

    ctx.clearRect(0, 0, w, h);
    drawDepth();
    if (fx.hexrain !== false) drawRain(dt);
    if (fx.wireframe !== false) { drawShapes(t); drawPackets(t); }
    if (fx.scanlines !== false) drawScanlines(dt);
  }

  function onVisibility() {
    if (document.hidden) { pause(); } else if (canvas && !running) { resume(); }
  }

  function pause() {
    running = false;
    if (raf) cancelAnimationFrame(raf);
    raf = 0;
  }

  function resume() {
    if (!canvas || running) return;
    running = true;
    lastTs = 0; startTs = 0;
    raf = requestAnimationFrame(frame);
  }

  function start(cv, s) {
    stop();
    canvas = cv;
    ctx = canvas.getContext('2d');
    settings = s || BW.settings;
    readColors();
    buildShapes();
    resize();
    if (window.ResizeObserver) {
      observer = new ResizeObserver(function () { resize(); });
      observer.observe(canvas);
    } else {
      window.addEventListener('resize', resize);
    }
    document.addEventListener('visibilitychange', onVisibility);
    if (!document.hidden) resume();
  }

  function update(s) {
    settings = s || settings;
    if (ctx) readColors();
  }

  function stop() {
    pause();
    if (observer) { observer.disconnect(); observer = null; }
    window.removeEventListener('resize', resize);
    document.removeEventListener('visibilitychange', onVisibility);
    canvas = null; ctx = null;
    rain = []; packets = [];
  }

  return { start: start, update: update, stop: stop, pause: pause, resume: resume };
})();
