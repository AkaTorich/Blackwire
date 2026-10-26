'use strict';
// Браузер — страницы игровой сети через web_get, отрисовка в iframe srcdoc со своим стилем.
// Ссылки внутри страницы перехватываются и идут снова через web_get.
// Файл: js/apps/browser.js, стили: css/apps/browser.css.

const BrowserApp = (function () {
  const HOME = 'home://';
  const THEME_VARS = ['--bg', '--bg2', '--panel', '--panel-border', '--fg', '--fg-dim', '--accent', '--accent2',
    '--glow', '--ok', '--warn', '--err', '--mono', '--sans', '--radius', '--sel', '--font-size'];

  function el(tag, cls, text) {
    const e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text != null) e.textContent = text;
    return e;
  }

  function iconBtn(icon, title) {
    const b = el('button', 'br-btn');
    b.type = 'button';
    b.title = title;
    b.innerHTML = Icons.get(icon, 16);
    return b;
  }

  const ESC = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' };
  function esc(s) { return String(s == null ? '' : s).replace(/[&<>"]/g, function (c) { return ESC[c]; }); }

  // --- адреса ---
  function parseUrl(u) {
    const m = /^([a-z][a-z0-9+.-]*):\/\/([^\/?#]*)([^?#]*)(\?[^#]*)?(#.*)?$/i.exec(u);
    if (!m) return null;
    return { scheme: m[1].toLowerCase(), host: m[2], path: m[3] || '/', query: m[4] || '', hash: m[5] || '' };
  }

  function normalizePath(p) {
    const out = [];
    const segs = p.split('/');
    for (let i = 0; i < segs.length; i++) {
      const s = segs[i];
      if (s === '' && i > 0 && i < segs.length - 1) continue;
      if (s === '.') { if (i === segs.length - 1) out.push(''); continue; }
      if (s === '..') { if (out.length > 1) out.pop(); if (i === segs.length - 1) out.push(''); continue; }
      out.push(s);
    }
    let r = out.join('/');
    if (!r.startsWith('/')) r = '/' + r;
    return r;
  }

  // Разрешить href относительно текущего адреса.
  function resolve(base, href) {
    href = String(href || '').trim();
    if (!href) return base;
    if (/^[a-z][a-z0-9+.-]*:/i.test(href)) return href;
    const b = parseUrl(base);
    if (!b) return href;
    const origin = b.scheme + '://' + b.host;
    if (href.startsWith('//')) return b.scheme + ':' + href;
    if (href.startsWith('/')) return origin + normalizePath(href.split('?')[0].split('#')[0]) + tail(href);
    if (href.startsWith('?')) return origin + b.path + href;
    if (href.startsWith('#')) return origin + b.path + b.query + href;
    const dir = b.path.slice(0, b.path.lastIndexOf('/') + 1);
    const plain = href.split('?')[0].split('#')[0];
    return origin + normalizePath(dir + plain) + tail(href);
  }

  function tail(href) {
    const q = href.indexOf('?'), h = href.indexOf('#');
    let start = -1;
    if (q >= 0 && (h < 0 || q < h)) start = q; else if (h >= 0) start = h;
    return start >= 0 ? href.slice(start) : '';
  }

  // Что ввёл пользователь → полный адрес.
  function fromInput(s) {
    s = String(s || '').trim();
    if (!s) return HOME;
    if (s === 'home' || s === 'home:' || s === 'home:/') return HOME;
    if (/^[a-z][a-z0-9+.-]*:\/\//i.test(s)) return s;
    return 'http://' + s;
  }

  function mount(body, arg) {
    const st = { hist: [], idx: -1, url: '', loading: false, seq: 0, title: '' };

    // --- разметка ---
    const root = el('div', 'br');
    const bar = el('div', 'br-bar');
    const btnBack = iconBtn('back', 'Назад — Alt+←');
    const btnFwd = iconBtn('forward', 'Вперёд — Alt+→');
    const btnReload = iconBtn('reload', 'Обновить — F5');
    const btnHome = iconBtn('home', 'Домой');
    const addrWrap = el('div', 'br-addr-wrap');
    const addrIcon = el('span', 'br-addr-icon');
    addrIcon.innerHTML = Icons.get('net', 14);
    const addr = el('input', 'br-addr');
    addr.type = 'text';
    addr.spellcheck = false;
    addr.placeholder = 'адрес';
    addr.setAttribute('autocomplete', 'off');
    addrWrap.append(addrIcon, addr);
    const progress = el('div', 'br-progress');
    bar.append(btnBack, btnFwd, btnReload, btnHome, addrWrap);
    const view = el('div', 'br-view');
    const frame = el('iframe', 'br-frame');
    frame.setAttribute('sandbox', 'allow-scripts');
    frame.setAttribute('title', 'страница');
    view.appendChild(frame);
    const statusLine = el('div', 'br-status');
    const stTitle = el('span', 'br-st-title', '');
    const stInfo = el('span', 'br-st-info', '');
    statusLine.append(stTitle, stInfo);
    root.append(bar, progress, view, statusLine);
    body.appendChild(root);

    // --- стиль для страницы (цвета из текущей темы) ---
    function themeCss() {
      const cs = getComputedStyle(document.documentElement);
      let vars = '';
      for (const v of THEME_VARS) {
        const val = cs.getPropertyValue(v).trim();
        if (val) vars += v + ':' + val + ';';
      }
      return ':root{' + vars + '}' +
        'html,body{margin:0;padding:0;background:var(--bg);color:var(--fg);}' +
        'body{font-family:var(--sans);font-size:var(--font-size,14px);line-height:1.55;padding:22px 28px 40px;max-width:1100px;margin:0 auto;}' +
        '*{box-sizing:border-box;}' +
        'h1,h2,h3,h4{color:var(--accent);font-weight:600;letter-spacing:0.03em;margin:1.1em 0 0.5em;line-height:1.25;text-shadow:0 0 8px var(--glow);}' +
        'h1{font-size:1.7em;margin-top:0.3em;padding-bottom:0.3em;border-bottom:1px solid var(--panel-border);}h2{font-size:1.3em;}h3{font-size:1.1em;}' +
        'p{margin:0.6em 0;}a{color:var(--accent2);text-decoration:none;border-bottom:1px solid transparent;cursor:pointer;}' +
        'a:hover{border-bottom-color:var(--accent2);text-shadow:0 0 6px var(--glow);}' +
        'pre,code,kbd,tt{font-family:var(--mono);font-size:0.95em;}' +
        'pre{background:var(--bg2);border:1px solid var(--panel-border);border-left:2px solid var(--accent);border-radius:var(--radius);padding:10px 14px;overflow:auto;white-space:pre;}' +
        'code{background:var(--bg2);padding:1px 5px;border-radius:3px;}pre code{background:none;padding:0;}' +
        'hr{border:0;border-top:1px solid var(--panel-border);margin:1.2em 0;}' +
        'ul,ol{padding-left:1.6em;}li{margin:0.2em 0;}' +
        'table{border-collapse:collapse;margin:0.8em 0;font-family:var(--mono);font-size:0.92em;max-width:100%;}' +
        'th,td{border:1px solid var(--panel-border);padding:5px 10px;text-align:left;vertical-align:top;}' +
        'th{background:var(--bg2);color:var(--accent);font-weight:600;letter-spacing:0.04em;text-transform:uppercase;font-size:0.85em;}' +
        'tr:hover td{background:var(--panel);}' +
        'blockquote{margin:0.8em 0;padding:4px 14px;border-left:2px solid var(--accent2);color:var(--fg-dim);}' +
        'img{max-width:100%;}' +
        'input,textarea,select,button{font-family:var(--mono);font-size:0.95em;background:var(--bg2);color:var(--fg);border:1px solid var(--panel-border);border-radius:var(--radius);padding:5px 9px;outline:none;}' +
        'input:focus,textarea:focus{border-color:var(--accent);box-shadow:0 0 8px var(--glow);}' +
        'button,input[type=submit]{cursor:pointer;color:var(--accent);border-color:var(--accent);background:transparent;padding:5px 14px;letter-spacing:0.04em;}' +
        'button:hover,input[type=submit]:hover{background:var(--sel);box-shadow:0 0 8px var(--glow);}' +
        'label{color:var(--fg-dim);}form{margin:0.8em 0;}' +
        '.panel,.box,.card{background:var(--panel);border:1px solid var(--panel-border);border-radius:var(--radius);padding:12px 16px;margin:0.8em 0;}' +
        '.dim,small{color:var(--fg-dim);}.ok{color:var(--ok);}.warn{color:var(--warn);}.err,.error{color:var(--err);}.accent{color:var(--accent);}' +
        '::selection{background:var(--sel);}' +
        '::-webkit-scrollbar{width:9px;height:9px;}::-webkit-scrollbar-thumb{background:var(--panel-border);border-radius:4px;}::-webkit-scrollbar-thumb:hover{background:var(--accent);}' +
        '.bw-err{max-width:560px;margin:12vh auto 0;padding:26px 30px;background:var(--panel);border:1px solid var(--panel-border);border-top:2px solid var(--err);border-radius:var(--radius);}' +
        '.bw-err h1{color:var(--err);border:0;text-shadow:0 0 10px var(--err);font-size:1.4em;margin:0 0 0.5em;}' +
        '.bw-err .code{font-family:var(--mono);color:var(--fg-dim);font-size:0.9em;word-break:break-all;}' +
        '.bw-err .body{margin-top:1em;color:var(--fg-dim);white-space:pre-wrap;font-family:var(--mono);font-size:0.9em;max-height:30vh;overflow:auto;}' +
        '.bw-err a{display:inline-block;margin-top:1.2em;}';
    }

    // Скрипт внутри страницы: ссылки и формы → родителю.
    const PAGE_SCRIPT = '(function(){' +
      'document.addEventListener("click",function(e){var a=e.target&&e.target.closest?e.target.closest("a"):null;if(!a)return;' +
      'var h=a.getAttribute("href");if(h==null)return;e.preventDefault();parent.postMessage({type:"bw-nav",href:h},"*");});' +
      'document.addEventListener("submit",function(e){e.preventDefault();var f=e.target;var q=[];' +
      'for(var i=0;i<f.elements.length;i++){var x=f.elements[i];if(!x.name||x.disabled)continue;' +
      'if((x.type==="checkbox"||x.type==="radio")&&!x.checked)continue;if(x.type==="submit"||x.type==="button")continue;' +
      'q.push(encodeURIComponent(x.name)+"="+encodeURIComponent(x.value));}' +
      'var act=f.getAttribute("action")||"";var qs=q.join("&");' +
      'parent.postMessage({type:"bw-nav",href:act+(qs?(act.indexOf("?")>=0?"&":"?")+qs:""),method:(f.method||"get").toLowerCase()},"*");});' +
      'document.addEventListener("keydown",function(e){if(e.key==="F5"||(e.altKey&&(e.key==="ArrowLeft"||e.key==="ArrowRight"))||(e.ctrlKey&&(e.key==="l"||e.key==="L"))){e.preventDefault();parent.postMessage({type:"bw-key",key:e.key,alt:e.altKey,ctrl:e.ctrlKey},"*");}});' +
      'document.addEventListener("DOMContentLoaded",function(){parent.postMessage({type:"bw-title",title:document.title||""},"*");});' +
      '})();';

    function wrap(html) {
      return '<!doctype html><html><head><meta charset="utf-8"><style>' + themeCss() + '</style></head><body>' +
        html + '<script>' + PAGE_SCRIPT + '<\/script></body></html>';
    }

    function errorPage(url, heading, detail, bodyText) {
      return '<div class="bw-err"><h1>' + esc(heading) + '</h1>' +
        '<div class="code">' + esc(url) + '</div>' +
        '<p>' + esc(detail) + '</p>' +
        (bodyText ? '<div class="body">' + esc(bodyText) + '</div>' : '') +
        '<a href="' + esc(HOME) + '">На стартовую</a></div>';
    }

    function stripTags(html) {
      return String(html || '').replace(/<style[\s\S]*?<\/style>|<script[\s\S]*?<\/script>/gi, '').replace(/<[^>]+>/g, ' ').replace(/\s+/g, ' ').trim();
    }

    function statusText(code) {
      switch (code) {
        case 400: return 'Неверный запрос';
        case 401: return 'Нужен вход';
        case 403: return 'Доступ закрыт';
        case 404: return 'Страница не найдена';
        case 500: return 'Ошибка на сервере';
        case 502: return 'Шлюз не отвечает';
        case 503: return 'Служба недоступна';
        default: return 'Ответ ' + code;
      }
    }

    // --- загрузка ---
    function setLoading(on) {
      st.loading = on;
      root.classList.toggle('loading', on);
      btnReload.innerHTML = Icons.get(on ? 'close' : 'reload', 16);
      btnReload.title = on ? 'Остановить — Esc' : 'Обновить — F5';
    }

    function show(html, title, info) {
      frame.srcdoc = wrap(html);
      st.title = title || '';
      stTitle.textContent = st.title;
      stInfo.textContent = info || '';
      Win.setTitle(st.title ? st.title : (st.url || 'Браузер'));
    }

    async function load(url, push) {
      url = String(url);
      const seq = ++st.seq;
      st.url = url;
      addr.value = url;
      if (push !== false) {
        st.hist = st.hist.slice(0, st.idx + 1);
        st.hist.push(url);
        st.idx = st.hist.length - 1;
      }
      updateNav();
      setLoading(true);
      stTitle.textContent = '';
      stInfo.textContent = 'Соединение…';
      let res = null, err = null;
      try { res = await BW.web_get(url); }
      catch (e) { err = String(e); }
      if (seq !== st.seq) return;  // пришёл более новый запрос
      setLoading(false);
      if (err != null) {
        show(errorPage(url, 'Нет ответа', err, ''), 'Нет ответа', 'ошибка');
        BW.sound('error');
        return;
      }
      if (!res || typeof res !== 'object') {
        show(errorPage(url, 'Нет ответа', 'Пустой ответ узла.', ''), 'Нет ответа', 'ошибка');
        return;
      }
      if (res.status !== 200) {
        const text = statusText(res.status);
        show(errorPage(url, text, res.title ? res.title : 'Узел ответил кодом ' + res.status + '.', stripTags(res.body)), text, String(res.status));
        return;
      }
      show(res.body || '', res.title || url, '200');
      BW.sound('open');
    }

    function navigate(href) {
      const target = resolve(st.url || HOME, href);
      load(target, true);
    }

    function updateNav() {
      btnBack.disabled = st.idx <= 0;
      btnFwd.disabled = st.idx >= st.hist.length - 1;
    }

    function goBack() { if (st.idx > 0) { st.idx--; load(st.hist[st.idx], false); } }
    function goFwd() { if (st.idx < st.hist.length - 1) { st.idx++; load(st.hist[st.idx], false); } }
    function reload() { if (st.loading) { st.seq++; setLoading(false); stInfo.textContent = 'остановлено'; return; } if (st.url) load(st.url, false); }
    function goHome() { load(HOME, true); }

    // --- события ---
    window.addEventListener('message', function (e) {
      if (e.source !== frame.contentWindow || !e.data || typeof e.data !== 'object') return;
      const d = e.data;
      if (d.type === 'bw-nav' && typeof d.href === 'string') navigate(d.href);
      else if (d.type === 'bw-title' && d.title && !st.title) { st.title = String(d.title); stTitle.textContent = st.title; Win.setTitle(st.title); }
      else if (d.type === 'bw-key') {
        if (d.key === 'F5') reload();
        else if (d.alt && d.key === 'ArrowLeft') goBack();
        else if (d.alt && d.key === 'ArrowRight') goFwd();
        else if (d.ctrl) { addr.focus(); addr.select(); }
      }
    });

    addr.addEventListener('keydown', function (e) {
      if (e.key === 'Enter') { e.preventDefault(); load(fromInput(addr.value), true); addr.blur(); frame.focus(); }
      else if (e.key === 'Escape') { e.preventDefault(); addr.value = st.url; addr.blur(); }
    });
    addr.addEventListener('focus', function () { addrWrap.classList.add('focus'); });
    addr.addEventListener('blur', function () { addrWrap.classList.remove('focus'); });

    btnBack.addEventListener('click', goBack);
    btnFwd.addEventListener('click', goFwd);
    btnReload.addEventListener('click', reload);
    btnHome.addEventListener('click', goHome);

    window.addEventListener('keydown', function (e) {
      const t = e.target;
      if (t === addr) return;
      if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA')) return;
      if (e.altKey && e.key === 'ArrowLeft') { e.preventDefault(); goBack(); }
      else if (e.altKey && e.key === 'ArrowRight') { e.preventDefault(); goFwd(); }
      else if (e.key === 'F5') { e.preventDefault(); reload(); }
      else if (e.ctrlKey && e.code === 'KeyL') { e.preventDefault(); addr.focus(); addr.select(); }
      else if (e.key === 'Escape' && st.loading) { e.preventDefault(); reload(); }
    });

    // Смена темы — перерисовать страницу с новыми цветами
    BW.onSettings(function () {
      if (st.url && !st.loading) load(st.url, false);
    });

    Promise.resolve(BW.ready).then(function () {
      load(arg ? fromInput(String(arg)) : HOME, true);
    });
  }

  return { mount: mount, resolve: resolve };
})();

Apps.register('browser', {
  title: 'Браузер', icon: 'browser', minWidth: 640, minHeight: 400,
  mount: function (body, arg) { BrowserApp.mount(body, arg); }
});
