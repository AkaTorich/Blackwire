'use strict';
/* Справка: оглавление слева, Markdown справа, поиск с подсветкой, якоря по заголовкам. */
(function () {
  var BOOKS = [
    { id: 'game', title: 'Руководство игрока', icon: 'file-code', load: function () { return BW.game_doc(); } },
    { id: 'sdk', title: 'SDK плагинов', icon: 'plugins', load: function () { return BW.plugins_doc(); } },
    { id: 'lang', title: 'Язык Adder', icon: 'file-code', load: function () { return BW.lang_doc(); } },
    { id: 'shell', title: 'Команды терминала', icon: 'terminal', load: function () { return BW.shell_doc(); } }
  ];

  var els = {}, baseHtml = {}, marks = [], markIndex = -1, headingEls = [], searchTimer = null;

  function el(tag, cls, html) {
    var e = document.createElement(tag);
    if (cls) e.className = cls;
    if (html != null) e.innerHTML = html;
    return e;
  }

  /* ---------- разметка ---------- */
  function build(body) {
    body.innerHTML = '';
    var bar = el('div', 'toolbar help-bar');
    var side = el('button', 'btn-icon', Icons.get('menu', 18));
    side.title = 'Оглавление';
    side.addEventListener('click', function () { els.help.classList.toggle('side-hidden'); });
    bar.appendChild(side);
    var sbox = el('div', 'help-search');
    sbox.innerHTML = '<span class="help-search-ico">' + Icons.get('search', 15) + '</span>';
    var inp = el('input', 'input'); inp.type = 'text'; inp.placeholder = 'Поиск по справке'; inp.spellcheck = false;
    sbox.appendChild(inp);
    var cnt = el('span', 'help-count mono');
    sbox.appendChild(cnt);
    bar.appendChild(sbox);
    var prev = el('button', 'btn-icon', Icons.get('up', 16)); prev.title = 'Предыдущее (Shift+Enter)';
    var next = el('button', 'btn-icon', Icons.get('down', 16)); next.title = 'Следующее (Enter)';
    bar.appendChild(prev); bar.appendChild(next);
    body.appendChild(bar);

    var help = el('div', 'help');
    var aside = el('aside', 'help-side');
    var toc = el('nav', 'help-toc');
    aside.appendChild(toc);
    var content = el('article', 'help-content');
    content.innerHTML = '<div class="help-loading dim">Загрузка справки…</div>';
    help.appendChild(aside); help.appendChild(content);
    body.appendChild(help);

    els.help = help; els.toc = toc; els.content = content; els.search = inp; els.count = cnt;

    prev.addEventListener('click', function () { step(-1); });
    next.addEventListener('click', function () { step(1); });
    inp.addEventListener('input', function () {
      clearTimeout(searchTimer);
      searchTimer = setTimeout(function () { search(inp.value); }, 160);
    });
    inp.addEventListener('keydown', function (e) {
      if (e.key === 'Enter') { e.preventDefault(); step(e.shiftKey ? -1 : 1); }
      else if (e.key === 'Escape' && inp.value) { e.preventDefault(); e.stopPropagation(); inp.value = ''; search(''); }
    });
    document.addEventListener('keydown', function (e) {
      if (e.ctrlKey && !e.shiftKey && (e.key === 'f' || e.key === 'F' || e.key === 'а' || e.key === 'А')) { e.preventDefault(); inp.focus(); inp.select(); }
    });
    content.addEventListener('click', function (e) {
      var a = e.target.closest('a');
      if (!a) return;
      e.preventDefault();
      var href = a.getAttribute('href') || '';
      if (href.charAt(0) !== '#') return;
      var sec = a.closest('.help-book');
      var id = (sec ? sec.dataset.book + '-' : '') + href.slice(1);
      scrollToId(id);
    });
    content.addEventListener('scroll', onScroll);
    if (body.clientWidth && body.clientWidth < 640) help.classList.add('side-hidden');
  }

  /* ---------- содержимое ---------- */
  function render(docs) {
    els.content.innerHTML = '';
    els.toc.innerHTML = '';
    headingEls = [];
    BOOKS.forEach(function (b, i) {
      var md = typeof docs[i] === 'string' ? docs[i] : '# ' + b.title + '\n\nТекст недоступен.';
      var sec = el('section', 'help-book');
      sec.id = 'book-' + b.id;
      sec.dataset.book = b.id;
      sec.innerHTML = Markdown.render(md);
      Array.prototype.forEach.call(sec.querySelectorAll('h1,h2,h3,h4,h5,h6'), function (h) {
        h.id = b.id + '-' + h.id;
        headingEls.push(h);
      });
      els.content.appendChild(sec);
      baseHtml[b.id] = sec.innerHTML;

      var bookItem = el('button', 'toc-book');
      bookItem.innerHTML = '<span class="toc-ico">' + Icons.get(b.icon, 15) + '</span><span class="ellipsis"></span>';
      bookItem.querySelector('.ellipsis').textContent = b.title;
      bookItem.addEventListener('click', function () { scrollToEl(sec); });
      els.toc.appendChild(bookItem);
      Markdown.headings(md).forEach(function (h) {
        if (h.level > 3 || h.level === 1) return;
        var item = el('button', 'toc-item lvl-' + h.level);
        item.textContent = h.text;
        item.title = h.text;
        item.dataset.target = b.id + '-' + h.id;
        item.addEventListener('click', function () { scrollToId(item.dataset.target); });
        els.toc.appendChild(item);
      });
    });
    onScroll();
  }

  function scrollToId(id) {
    var t = document.getElementById(id);
    if (t) scrollToEl(t);
  }

  function scrollToEl(t) {
    var c = els.content;
    var top = t.getBoundingClientRect().top - c.getBoundingClientRect().top + c.scrollTop - 10;
    c.scrollTo({ top: Math.max(0, top), behavior: 'smooth' });
    t.classList.remove('flash');
    void t.offsetWidth;
    t.classList.add('flash');
  }

  var scrollRaf = 0;
  function onScroll() {
    if (scrollRaf) return;
    scrollRaf = requestAnimationFrame(function () {
      scrollRaf = 0;
      var c = els.content, cTop = c.getBoundingClientRect().top + 24;
      var active = null;
      for (var i = 0; i < headingEls.length; i++) {
        if (headingEls[i].getBoundingClientRect().top <= cTop) active = headingEls[i]; else break;
      }
      Array.prototype.forEach.call(els.toc.querySelectorAll('.toc-item'), function (it) {
        var on = !!active && it.dataset.target === active.id;
        it.classList.toggle('active', on);
        if (on) {
          var r = it.getBoundingClientRect(), tr = els.toc.getBoundingClientRect();
          if (r.top < tr.top || r.bottom > tr.bottom) it.scrollIntoView({ block: 'nearest' });
        }
      });
    });
  }

  /* ---------- поиск ---------- */
  function restore() {
    Array.prototype.forEach.call(els.content.querySelectorAll('.help-book'), function (sec) {
      if (baseHtml[sec.dataset.book] !== undefined && sec.querySelector('mark')) sec.innerHTML = baseHtml[sec.dataset.book];
    });
    headingEls = Array.prototype.slice.call(els.content.querySelectorAll('h1,h2,h3,h4,h5,h6'));
    marks = []; markIndex = -1;
  }

  function search(term) {
    restore();
    term = (term || '').trim();
    if (term.length < 2) {
      els.count.textContent = '';
      els.toc.classList.remove('filtering');
      Array.prototype.forEach.call(els.toc.children, function (c) { c.classList.remove('dimmed'); });
      return;
    }
    var lower = term.toLowerCase();
    var walker = document.createTreeWalker(els.content, NodeFilter.SHOW_TEXT, {
      acceptNode: function (n) {
        if (!n.nodeValue.trim()) return NodeFilter.FILTER_REJECT;
        if (n.nodeValue.toLowerCase().indexOf(lower) < 0) return NodeFilter.FILTER_REJECT;
        return NodeFilter.FILTER_ACCEPT;
      }
    });
    var nodes = [];
    while (walker.nextNode()) nodes.push(walker.currentNode);
    nodes.forEach(function (node) {
      var text = node.nodeValue, lowerText = text.toLowerCase();
      var frag = document.createDocumentFragment();
      var pos = 0, idx;
      while ((idx = lowerText.indexOf(lower, pos)) >= 0) {
        if (idx > pos) frag.appendChild(document.createTextNode(text.slice(pos, idx)));
        var m = el('mark');
        m.textContent = text.slice(idx, idx + term.length);
        frag.appendChild(m);
        marks.push(m);
        pos = idx + term.length;
      }
      if (pos < text.length) frag.appendChild(document.createTextNode(text.slice(pos)));
      node.parentNode.replaceChild(frag, node);
    });
    /* в оглавлении притушить разделы без совпадений */
    var hit = {};
    marks.forEach(function (m) {
      /* ближайший заголовок перед совпадением — по порядку в документе */
      var owner = null;
      for (var i = 0; i < headingEls.length; i++) {
        var h = headingEls[i];
        if (h === m.closest('h1,h2,h3,h4,h5,h6') || (h.compareDocumentPosition(m) & Node.DOCUMENT_POSITION_FOLLOWING)) owner = h;
        else break;
      }
      if (owner) hit[owner.id] = true;
    });
    Array.prototype.forEach.call(els.toc.querySelectorAll('.toc-item'), function (it) {
      it.classList.toggle('dimmed', !hit[it.dataset.target]);
    });
    els.toc.classList.add('filtering');
    if (marks.length) { markIndex = 0; focusMark(); }
    else els.count.textContent = '0';
  }

  function focusMark() {
    marks.forEach(function (m, i) { m.classList.toggle('cur', i === markIndex); });
    els.count.textContent = (markIndex + 1) + ' / ' + marks.length;
    var m = marks[markIndex];
    if (m) {
      var c = els.content;
      var top = m.getBoundingClientRect().top - c.getBoundingClientRect().top + c.scrollTop - c.clientHeight / 3;
      c.scrollTo({ top: Math.max(0, top), behavior: 'smooth' });
    }
  }

  function step(dir) {
    if (!marks.length) return;
    markIndex = (markIndex + dir + marks.length) % marks.length;
    focusMark();
  }

  /* ---------- запуск ---------- */
  function mount(body, arg) {
    build(body);
    Promise.all(BOOKS.map(function (b) { return b.load().catch(function () { return null; }); })).then(function (docs) {
      render(docs);
      if (arg) {
        var book = BOOKS.filter(function (b) { return b.id === arg; })[0];
        if (book) scrollToId('book-' + book.id);
        else { els.search.value = arg; search(arg); }
      }
    });
  }

  Apps.register('help', { title: 'Справка', icon: 'help', minWidth: 520, minHeight: 360, mount: mount });
})();
