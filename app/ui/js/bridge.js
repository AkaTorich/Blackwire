'use strict';
/* Мост в Rust. Один глобальный объект BW: параметры окна, invoke/listen, настройки, звук
   и обёртки для каждой команды из docs/UI-CONTRACT.md. Без Tauri работает как заглушка. */
var BW = (function () {
  var params = new URLSearchParams(location.search);
  var tauri = window.__TAURI__ || null;

  function safeDecode(s) {
    if (s == null) return null;
    try { return decodeURIComponent(s); } catch (e) { return s; }
  }

  var BW = {
    app: params.get('app') || 'desktop',
    arg: safeDecode(params.get('arg')),
    label: params.get('win') || '',
    hasTauri: !!tauri,
    settings: null,
    ready: null
  };

  var DEFAULT_SETTINGS = {
    theme: 'swordfish',
    fx: { wireframe: true, hexrain: true, scanlines: true, glass: true },
    font_size: 14,
    terminal_opacity: 0.85,
    window_opacity: 0.72,
    sound: true,
    default_monitor: 0
  };

  /* ---------- заглушка для просмотра в обычном браузере ---------- */
  var stubThemes = [
    { id: 'swordfish', name: 'Рыба-меч', accent: '#39d8ff' },
    { id: 'ice', name: 'Лёд', accent: '#8fd8ff' },
    { id: 'amber', name: 'Янтарь', accent: '#ffb347' },
    { id: 'phosphor', name: 'Фосфор', accent: '#4cff7a' },
    { id: 'violet', name: 'Фиалка', accent: '#b57cff' },
    { id: 'crimson', name: 'Кармин', accent: '#ff4d6d' },
    { id: 'paper', name: 'Бумага', accent: '#1f5fbf' },
    { id: 'midnight', name: 'Полночь', accent: '#6f9bdc' },
    { id: 'steel', name: 'Сталь', accent: '#9fb6c9' },
    { id: 'gold', name: 'Золото', accent: '#e6c25a' },
    { id: 'aurora', name: 'Аврора', accent: '#5affc8' },
    { id: 'ember', name: 'Уголь', accent: '#ff7a45' }
  ];
  var stubState = {
    settings: JSON.parse(JSON.stringify(DEFAULT_SETTINGS)),
    hasSave: true,
    windows: [
      { label: 'terminal-1', app: 'terminal', title: 'ghost@ghost-pc: ~', minimized: false },
      { label: 'editor-1', app: 'editor', title: 'scan.adr', minimized: true }
    ],
    counter: 2,
    mail: [
      { id: 1, from: 'ops@nullvector.net', subject: 'Первое задание', date: '03:12', unread: true },
      { id: 2, from: 'root@ghost-pc', subject: 'Добро пожаловать', date: '02:40', unread: false }
    ]
  };

  function stubInvoke(cmd, args) {
    args = args || {};
    var s = stubState;
    switch (cmd) {
      case 'boot': return { has_save: s.hasSave, handle: s.hasSave ? 'ghost' : null, os_name: 'BLACKWIRE', version: '0.1.0', settings: s.settings, theme_list: stubThemes };
      case 'new_game': s.hasSave = true; return stubInvoke('boot');
      case 'load_game': return stubInvoke('boot');
      case 'save_game': return 'ok';
      case 'save_slots': return s.slots || (s.slots = []);
      case 'save_to_slot':
        s.slots = (s.slots || []).filter(function (x) { return x.name !== args.name; });
        s.slots.push({ name: args.name, handle: 'ghost', time: '2031-03-14 05:20', money: 420 });
        return s.slots;
      case 'load_slot': return stubInvoke('boot');
      case 'delete_slot':
        s.slots = (s.slots || []).filter(function (x) { return x.name !== args.name; });
        return s.slots;
      case 'scenario_targets': return ['bank-core.corp', 'gate.hw', 'news-portal.net', 'shop-01.net', 'uni-lab.edu'];
      case 'scenario_list': return s.scenarios || (s.scenarios = []);
      case 'scenario_save': {
        var sc = args.scenario || {};
        var goalText = sc.goal && sc.goal.OwnHost ? ('получить доступ к ' + sc.goal.OwnHost.host)
          : sc.goal && sc.goal.Deliver ? ('прислать текст со строкой «' + sc.goal.Deliver.needle + '»')
          : sc.goal && sc.goal.Money ? ('накопить ' + sc.goal.Money.amount + ' ₵') : '';
        s.scenarios = (s.scenarios || []).filter(function (x) { return x.name !== sc.name; });
        s.scenarios.push({ name: sc.name, brief: sc.brief, goal: goalText, reward: sc.reward });
        s.activeScenario = s.activeScenario || null;
        return s.scenarios;
      }
      case 'scenario_delete':
        s.scenarios = (s.scenarios || []).filter(function (x) { return x.name !== args.name; });
        return s.scenarios;
      case 'scenario_start':
        s.activeScenario = (s.scenarios || []).find(function (x) { return x.name === args.name; }) || null;
        return null;
      case 'scenario_stop': s.activeScenario = null; return null;
      case 'scenario_active': return s.activeScenario || null;
      case 'net_host':
        s.net = { role: 'Host', my_name: 'ghost', port: args.port || 7777, error: null, addrs: ['192.168.0.10', '127.0.0.1'],
          snap: { mode: args.mode || 'duel', phase: 'lobby', seed: 0, targets: [], captured: [], winner: null,
            peers: [{ name: 'ghost', stage: 0, score: 0 }] } };
        setTimeout(function () {
          if (!s.net) return;
          s.net.snap.peers.push({ name: 'neo', stage: 0, score: 0 });
          emit('net_event', { snap: s.net.snap });
        }, 1200);
        return { port: s.net.port, addrs: s.net.addrs };
      case 'net_join':
        s.net = { role: 'Client', my_name: 'neo', port: 0, error: null, addrs: [],
          snap: { mode: 'duel', phase: 'lobby', seed: 0, targets: [], captured: [], winner: null,
            peers: [{ name: 'host', stage: 0, score: 0 }, { name: 'neo', stage: 0, score: 0 }] } };
        return null;
      case 'net_start':
        if (s.net) {
          s.net.snap.phase = 'playing';
          s.net.snap.seed = 4242;
          s.net.snap.targets = s.net.snap.mode === 'team' ? ['bank-core.corp', 'news-portal.net', 'uni-lab.edu'] : ['bank-core.corp'];
          emit('net_event', { snap: s.net.snap });
          emit('net_match_start', {});
        }
        return null;
      case 'net_status': return s.net || { role: 'None', my_name: '', port: 0, error: null, addrs: [],
        snap: { mode: 'duel', phase: 'lobby', seed: 0, targets: [], captured: [], winner: null, peers: [] } };
      case 'net_leave': s.net = null; return null;
      case 'login': return args.password === '' || args.password === 'ghost';
      case 'start_desktop': case 'quit': case 'focus_window': case 'close_window':
      case 'arrange_windows': case 'desktop_on': case 'shell_close': case 'script_input':
      case 'script_kill': case 'fs_write': case 'fs_mkdir': case 'fs_delete': case 'fs_rename':
      case 'fs_copy': case 'sys_kill': case 'mail_delete': case 'mail_send':
        if (cmd === 'close_window') s.windows = s.windows.filter(function (w) { return w.label !== args.label; });
        return null;
      case 'open_app': {
        var label = args.app + '-' + (++s.counter);
        s.windows.push({ label: label, app: args.app, title: args.app + (args.arg ? ' ' + args.arg : ''), minimized: false });
        setTimeout(function () { emit('windows_changed', { windows: s.windows }); }, 50);
        return label;
      }
      case 'list_windows': return s.windows;
      case 'monitors': return [{ index: 0, name: 'Основной', x: 0, y: 0, width: screen.width, height: screen.height, scale: window.devicePixelRatio || 1, primary: true }];
      case 'shell_open': return 1;
      case 'shell_exec': return { lines: [{ text: 'заглушка: ' + args.line, kind: 'out', delay_ms: 0 }], actions: [], prompt: { user: 'ghost', host: 'ghost-pc', cwd: '~', is_root: false, hops: ['ghost-pc'] } };
      case 'shell_complete': return [];
      case 'shell_prompt': return { user: 'ghost', host: 'ghost-pc', cwd: '~', is_root: false, hops: ['ghost-pc'] };
      case 'shell_history': return [];
      case 'script_run': return 100;
      case 'fs_list': return [
        { name: 'scan.adr', path: '/home/ghost/scan.adr', is_dir: false, size: 812, modified: 0, owner: 'ghost', mode: 'rw-r--r--' },
        { name: 'notes.txt', path: '/home/ghost/notes.txt', is_dir: false, size: 120, modified: 0, owner: 'ghost', mode: 'rw-r--r--' },
        { name: 'tools', path: '/home/ghost/tools', is_dir: true, size: 0, modified: 0, owner: 'ghost', mode: 'rwxr-xr-x' }
      ];
      case 'fs_read': return '';
      case 'lang_check': return [];
      case 'lang_doc': return '# Язык Adder\n\nДокумент не загружен: нет связи с ядром.\n\n## Переменные\n\nПример:\n\n```\nx = 1\nprint(x)\n```\n';
      case 'shell_doc': return '# Команды терминала\n\n| Команда | Назначение |\n|---|---|\n| `ls` | список файлов |\n| `cd` | сменить папку |\n\n## Сеть\n\n- `scan` — осмотреть сеть\n- `ssh` — подключиться\n';
      case 'sys_status': return { hostname: 'ghost-pc', ip: '10.0.0.12', uptime: 3600, cpu: 12, mem_used: 1024, mem_total: 4096, net_in: 3, net_out: 1, procs: [] };
      case 'net_map': return { nodes: [], links: [] };
      case 'web_get': return { status: 404, title: 'Нет узла', body: '<h1>Нет такого узла</h1>' };
      case 'settings_get': return s.settings;
      case 'settings_set':
        s.settings = args.settings;
        setTimeout(function () { emit('theme_changed', { settings: s.settings }); }, 0);
        return null;
      case 'mail_list': return s.mail;
      case 'mail_read': return { id: args.id, from: 'ops@nullvector.net', to: 'ghost@ghost-pc', subject: 'Первое задание', date: '03:12', body: 'Текст письма недоступен без ядра.' };
      case 'mail_unread_count': return s.mail.filter(function (m) { return m.unread; }).length;
      case 'ending_summary': return { won: true, missions_done: 7, missions_total: 7, debt_cleared: true, money: 420, money_earned: 3250, debt_paid: 3000, machines_owned: 9, scanned: 14, upgrades: 2, peak_heat: 62, raids: 1, time_played: 7200, at_text: '14.03.2031 06:10', text: 'Ты закрыл их бумажку с печатью.\n\nТеперь ты — один из нас.\n\nhello-world' };
      default:
        console.warn('[BW stub] неизвестная команда', cmd, args);
        return null;
    }
  }

  /* ---------- invoke / listen ---------- */
  var localListeners = {};

  function emit(event, payload) {
    var list = localListeners[event] || [];
    list.slice().forEach(function (cb) { try { cb(payload); } catch (e) { console.error(e); } });
  }
  BW.emit = emit;

  BW.invoke = function (cmd, args) {
    args = args || {};
    if (tauri && tauri.core && tauri.core.invoke) {
      return tauri.core.invoke(cmd, args);
    }
    console.info('[BW stub] invoke', cmd, args);
    return new Promise(function (resolve) {
      setTimeout(function () { resolve(stubInvoke(cmd, args)); }, 20);
    });
  };

  /* Возвращает Promise с функцией отписки; cb получает payload. */
  BW.listen = function (event, cb) {
    if (!localListeners[event]) localListeners[event] = [];
    localListeners[event].push(cb);
    var unlistenLocal = function () {
      localListeners[event] = (localListeners[event] || []).filter(function (f) { return f !== cb; });
    };
    if (tauri && tauri.event && tauri.event.listen) {
      /* В Tauri локальный список служит только для эмуляции; реальные события идут напрямую */
      unlistenLocal();
      return tauri.event.listen(event, function (e) { cb(e && e.payload !== undefined ? e.payload : e); });
    }
    return Promise.resolve(unlistenLocal);
  };

  /* ---------- обёртки команд (ключи — camelCase, как ждёт Rust) ---------- */
  var COMMANDS = {
    plugins_doc: [], game_doc: [],
    plugins_panel: ["id"], plugins_ui: ["id", "action", "values"],
    plugins_list: [], plugins_reload: [], plugins_enable: ["id", "enabled"], plugins_run: ["command"], plugins_folder: [], plugins_example: ["name"],
    boot: [], new_game: ['handle', 'seed', 'password'], load_game: [], save_game: [], login: ['password'],
    save_slots: [], save_to_slot: ['name'], load_slot: ['name'], delete_slot: ['name'],
    scenario_targets: [], scenario_list: [], scenario_save: ['scenario'], scenario_delete: ['name'],
    scenario_start: ['name'], scenario_stop: [], scenario_active: [],
    net_host: ['mode', 'port'], net_join: ['addr'], net_start: [], net_status: [], net_leave: [],
    start_desktop: [], quit: [],
    open_app: ['app', 'arg', 'monitor'], focus_window: ['label'], close_window: ['label'],
    list_windows: [], monitors: [], arrange_windows: ['monitor', 'layout'], desktop_on: ['monitor', 'on'],
    shell_open: [], shell_close: ['sessionId'], shell_exec: ['sessionId', 'line'],
    shell_complete: ['sessionId', 'line'], shell_prompt: ['sessionId'], shell_history: ['sessionId'],
    script_run: ['sessionId', 'path', 'args'], script_input: ['pid', 'text'], script_kill: ['pid'],
    fs_list: ['sessionId', 'path'], fs_read: ['sessionId', 'path'], fs_write: ['sessionId', 'path', 'text'],
    fs_mkdir: ['sessionId', 'path'], fs_delete: ['sessionId', 'path'],
    fs_rename: ['sessionId', 'from', 'to'], fs_copy: ['sessionId', 'from', 'to'],
    lang_check: ['src'], lang_doc: [], shell_doc: [],
    sys_status: [], sys_kill: ['pid'], net_map: [], web_get: ['url'],
    settings_get: [], settings_set: ['settings'],
    mail_list: ['folder'], mail_read: ['id'], mail_send: ['to', 'subject', 'body'],
    mail_delete: ['id'], mail_unread_count: [], ending_summary: []
  };

  Object.keys(COMMANDS).forEach(function (name) {
    var keys = COMMANDS[name];
    BW[name] = function () {
      var args = {};
      for (var i = 0; i < keys.length; i++) {
        if (arguments[i] !== undefined) args[keys[i]] = arguments[i];
      }
      return BW.invoke(name, args);
    };
  });

  /* ---------- настройки ---------- */
  var settingsListeners = [];

  function normalize(s) {
    var out = JSON.parse(JSON.stringify(DEFAULT_SETTINGS));
    if (!s || typeof s !== 'object') return out;
    Object.keys(s).forEach(function (k) {
      if (k === 'fx' && s.fx && typeof s.fx === 'object') {
        Object.keys(s.fx).forEach(function (f) { out.fx[f] = !!s.fx[f]; });
      } else if (s[k] !== undefined && s[k] !== null) {
        out[k] = s[k];
      }
    });
    return out;
  }

  function applySettings(s) {
    BW.settings = normalize(s);
    settingsListeners.slice().forEach(function (cb) {
      try { cb(BW.settings); } catch (e) { console.error(e); }
    });
  }

  BW.onSettings = function (cb) {
    settingsListeners.push(cb);
    if (BW.settings) { try { cb(BW.settings); } catch (e) { console.error(e); } }
    return function () {
      settingsListeners = settingsListeners.filter(function (f) { return f !== cb; });
    };
  };

  /* Сохранить настройки: применяет локально сразу, Rust разошлёт theme_changed всем окнам */
  BW.saveSettings = function (patch) {
    var next = normalize(BW.settings || DEFAULT_SETTINGS);
    if (patch) {
      Object.keys(patch).forEach(function (k) {
        if (k === 'fx' && patch.fx) {
          Object.keys(patch.fx).forEach(function (f) { next.fx[f] = !!patch.fx[f]; });
        } else next[k] = patch[k];
      });
    }
    applySettings(next);
    return BW.settings_set(next);
  };

  BW.ready = BW.settings_get().then(function (s) {
    applySettings(s);
  }, function (err) {
    console.warn('settings_get не удался, настройки по умолчанию', err);
    applySettings(DEFAULT_SETTINGS);
  });

  BW.listen('theme_changed', function (p) {
    if (p && p.settings) applySettings(p.settings);
  });

  /* ---------- звук: короткие синтезированные сигналы ---------- */
  var audioCtx = null;

  function ctx() {
    if (audioCtx) return audioCtx;
    var AC = window.AudioContext || window.webkitAudioContext;
    if (!AC) return null;
    try { audioCtx = new AC(); } catch (e) { return null; }
    return audioCtx;
  }

  function tone(ac, type, freq, freqEnd, start, dur, gain) {
    var o = ac.createOscillator();
    var g = ac.createGain();
    o.type = type;
    o.frequency.setValueAtTime(freq, start);
    if (freqEnd) o.frequency.exponentialRampToValueAtTime(freqEnd, start + dur);
    g.gain.setValueAtTime(0.0001, start);
    g.gain.exponentialRampToValueAtTime(gain, start + 0.005);
    g.gain.exponentialRampToValueAtTime(0.0001, start + dur);
    o.connect(g); g.connect(ac.destination);
    o.start(start); o.stop(start + dur + 0.02);
  }

  function noise(ac, start, dur, gain) {
    var len = Math.floor(ac.sampleRate * dur);
    var buf = ac.createBuffer(1, len, ac.sampleRate);
    var d = buf.getChannelData(0);
    for (var i = 0; i < len; i++) d[i] = (Math.random() * 2 - 1) * (1 - i / len);
    var src = ac.createBufferSource();
    src.buffer = buf;
    var f = ac.createBiquadFilter();
    f.type = 'highpass'; f.frequency.value = 2500;
    var g = ac.createGain();
    g.gain.value = gain;
    src.connect(f); f.connect(g); g.connect(ac.destination);
    src.start(start);
  }

  BW.sound = function (name) {
    if (!BW.settings || !BW.settings.sound) return;
    var ac = ctx();
    if (!ac) return;
    if (ac.state === 'suspended') { ac.resume().catch(function () {}); }
    var t = ac.currentTime + 0.01;
    switch (name) {
      case 'click': tone(ac, 'sine', 1400, 900, t, 0.04, 0.035); break;
      case 'key': noise(ac, t, 0.012, 0.03); break;
      case 'open': tone(ac, 'sine', 520, 1040, t, 0.12, 0.04); tone(ac, 'sine', 1040, 1560, t + 0.08, 0.1, 0.03); break;
      case 'close': tone(ac, 'sine', 1040, 400, t, 0.14, 0.04); break;
      case 'error': tone(ac, 'square', 160, 110, t, 0.16, 0.025); tone(ac, 'square', 120, 90, t + 0.12, 0.14, 0.02); break;
      case 'notify': tone(ac, 'triangle', 880, 880, t, 0.09, 0.04); tone(ac, 'triangle', 1320, 1320, t + 0.11, 0.14, 0.035); break;
      default: tone(ac, 'sine', 1000, 1000, t, 0.03, 0.03);
    }
  };

  /* ---------- заглушка мира: тики раз в секунду, когда нет Tauri ---------- */
  if (!tauri) {
    var cpu = 20;
    setInterval(function () {
      cpu = Math.max(2, Math.min(96, cpu + (Math.random() - 0.5) * 18));
      emit('world_tick', {
        time: Math.floor(Date.now() / 1000),
        cpu: cpu,
        mem: 0.35 + Math.sin(Date.now() / 9000) * 0.1,
        net_in: Math.random() * 40,
        net_out: Math.random() * 12
      });
    }, 1000);
    setTimeout(function () {
      emit('notify', { title: 'Сеть', text: 'Обнаружен новый узел 10.0.0.7' });
    }, 4000);
  }

  return BW;
})();
