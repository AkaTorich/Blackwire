'use strict';
/* Значки — свои SVG в одном стиле: линии 1.6 px, stroke=currentColor, без заливок, viewBox 24. */
var Icons = (function () {
  var FILE = '<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5"/>';
  var LINK = '<path d="M10 14a4 4 0 0 1 0-5.7l2.3-2.3a4 4 0 0 1 5.7 5.7l-1 1"/><path d="M14 10a4 4 0 0 1 0 5.7l-2.3 2.3a4 4 0 0 1-5.7-5.7l1-1"/>';
  var EYE = '<path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="3"/>';
  var MAIL = '<rect x="3" y="5" width="18" height="14" rx="2"/><path d="M3 7l9 6 9-6"/>';

  var PATHS = {
    /* приложения */
    'terminal': '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M7 9l3 3-3 3M12 15h5"/>',
    'editor': '<path d="M13 4H6a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-7"/><path d="M18 3l3 3-8 8H10v-3z"/>',
    'files': '<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
    'browser': '<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18"/>',
    'monitor': '<rect x="3" y="4" width="18" height="12" rx="2"/><path d="M8 20h8M12 16v4M6 11h3l2-4 2 6 2-3h3"/>',
    'map': '<circle cx="5" cy="6" r="2"/><circle cx="19" cy="6" r="2"/><circle cx="12" cy="18" r="2"/><circle cx="12" cy="10" r="2"/><path d="M6.7 7.2l3.5 1.8M17.3 7.2l-3.5 1.8M12 12v4"/>',
    'settings': '<path d="M4 7h9M17 7h3M4 12h3M11 12h9M4 17h11M19 17h1"/><circle cx="15" cy="7" r="2"/><circle cx="9" cy="12" r="2"/><circle cx="17" cy="17" r="2"/>',
    'help': '<circle cx="12" cy="12" r="9"/><path d="M9.5 9.5a2.5 2.5 0 1 1 3.6 2.3c-.8.4-1.1 1-1.1 1.7"/><path d="M12 17h.01"/>',
    'plugins': '<path d="M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM14 14h6v6h-6z"/>',
    'scenario': FILE + '<path d="M8.5 13.5l2 2 4-4"/>',
    'duel': '<circle cx="6" cy="12" r="2.5"/><circle cx="18" cy="12" r="2.5"/><path d="M8.5 12h7M12 4v3M12 17v3"/>',
    'mail': MAIL,
    'mail-open': '<path d="M3 10l9-6 9 6v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><path d="M3 10l9 6 9-6"/>',
    'send': '<path d="M21 3L3 10.5l8 2.5 2.5 8z"/><path d="M21 3l-10 10"/>',
    'reply': '<path d="M9 7L4 12l5 5"/><path d="M4 12h9a6 6 0 0 1 6 6v1"/>',
    'attachment': '<path d="M16.5 7.5l-7 7a2.5 2.5 0 0 0 3.5 3.5l7.5-7.5a5 5 0 0 0-7-7L6 11a7 7 0 0 0 10 10"/>',
    'inbox': '<path d="M4 4h16v16H4z"/><path d="M4 14h5l1.5 2h3L15 14h5"/>',
    /* файлы и папки */
    'folder': '<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
    'folder-open': '<path d="M3 8a2 2 0 0 1 2-2h4l2 2h7a2 2 0 0 1 2 2v1"/><path d="M3 11h17.4a1 1 0 0 1 .96 1.3l-1.8 6a1 1 0 0 1-.96.7H5a2 2 0 0 1-2-2z"/>',
    'file': FILE,
    'file-code': FILE + '<path d="M10 12.5l-2 2 2 2M14 12.5l2 2-2 2"/>',
    'file-text': FILE + '<path d="M8 13h8M8 17h6"/>',
    /* окно */
    'close': '<path d="M6 6l12 12M18 6L6 18"/>',
    'min': '<path d="M5 12h14"/>',
    'max': '<rect x="5" y="5" width="14" height="14" rx="1.5"/>',
    'restore': '<rect x="4" y="8" width="12" height="12" rx="1.5"/><path d="M8 8V5.5A1.5 1.5 0 0 1 9.5 4h9A1.5 1.5 0 0 1 20 5.5v9a1.5 1.5 0 0 1-1.5 1.5H16"/>',
    /* действия */
    'play': '<path d="M7 5v14l11-7z"/>',
    'stop': '<rect x="6" y="6" width="12" height="12" rx="1.5"/>',
    'save': '<path d="M5 3h11l3 3v15H5z"/><path d="M8 3v5h7V3M8 21v-7h8v7"/>',
    'search': '<circle cx="11" cy="11" r="6"/><path d="M20 20l-4.5-4.5"/>',
    'plus': '<path d="M12 5v14M5 12h14"/>',
    'trash': '<path d="M4 7h16M9 7V4h6v3M6 7l1 13h10l1-13M10 11v6M14 11v6"/>',
    'rename': '<path d="M12 4v16M9.5 4h5M9.5 20h5"/><path d="M4 8h4v8H4M16 8h4v8h-4"/>',
    'back': '<path d="M15 5l-7 7 7 7"/>',
    'forward': '<path d="M9 5l7 7-7 7"/>',
    'reload': '<path d="M20 12a8 8 0 1 1-2.3-5.7"/><path d="M20 4v5h-5"/>',
    'home': '<path d="M4 11l8-7 8 7v9a1 1 0 0 1-1 1h-4v-6h-6v6H5a1 1 0 0 1-1-1z"/>',
    'power': '<path d="M12 3v9"/><path d="M6.6 6.6a8 8 0 1 0 10.8 0"/>',
    'user': '<circle cx="12" cy="8" r="4"/><path d="M4 21a8 8 0 0 1 16 0"/>',
    'lock': '<rect x="5" y="11" width="14" height="10" rx="2"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/>',
    'key': '<circle cx="8" cy="14" r="4"/><path d="M11 11l9-9M16 4l3 3M13 7l3 3"/>',
    'cpu': '<rect x="6" y="6" width="12" height="12" rx="1.5"/><rect x="10" y="10" width="4" height="4"/><path d="M9 3v3M15 3v3M9 18v3M15 18v3M3 9h3M3 15h3M18 9h3M18 15h3"/>',
    'mem': '<rect x="3" y="7" width="18" height="9" rx="1.5"/><path d="M7 7v9M12 7v9M17 7v9M6 16v2M10 16v2M14 16v2M18 16v2"/>',
    'net': '<circle cx="12" cy="5" r="2"/><circle cx="5" cy="19" r="2"/><circle cx="19" cy="19" r="2"/><path d="M12 7v4M12 11l-5.6 6.4M12 11l5.6 6.4"/>',
    'bell': '<path d="M6 16v-5a6 6 0 0 1 12 0v5l1.5 2h-15z"/><path d="M10 21h4"/>',
    'grid': '<rect x="4" y="4" width="7" height="7" rx="1"/><rect x="13" y="4" width="7" height="7" rx="1"/><rect x="4" y="13" width="7" height="7" rx="1"/><rect x="13" y="13" width="7" height="7" rx="1"/>',
    'columns': '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M9 4v16M15 4v16"/>',
    'rows': '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 9.5h18M3 14.5h18"/>',
    'cascade': '<rect x="3" y="8" width="13" height="12" rx="1.5"/><path d="M7 8V5.5A1.5 1.5 0 0 1 8.5 4H19.5A1.5 1.5 0 0 1 21 5.5v10a1.5 1.5 0 0 1-1.5 1.5H16"/>',
    'copy': '<rect x="9" y="9" width="11" height="11" rx="1.5"/><path d="M5 15V5a1 1 0 0 1 1-1h10"/>',
    'paste': '<rect x="5" y="5" width="14" height="16" rx="1.5"/><path d="M9 5V3h6v2M9 11h6M9 15h6"/>',
    'check': '<path d="M5 12.5l4.5 4.5L19 7"/>',
    'warning': '<path d="M12 3l10 18H2z"/><path d="M12 10v5M12 18h.01"/>',
    'info': '<circle cx="12" cy="12" r="9"/><path d="M12 11v6M12 7.5h.01"/>',
    'chevron-right': '<path d="M9 6l6 6-6 6"/>',
    'chevron-down': '<path d="M6 9l6 6 6-6"/>',
    'chevron-left': '<path d="M15 6l-6 6 6 6"/>',
    'up': '<path d="M12 19V5M5 12l7-7 7 7"/>',
    'down': '<path d="M12 5v14M5 12l7 7 7-7"/>',
    'link': LINK,
    'unlink': LINK + '<path d="M4 4l16 16"/>',
    'eye': EYE,
    'eye-off': EYE + '<path d="M4 4l16 16"/>',
    'logout': '<path d="M10 4H5a1 1 0 0 0-1 1v14a1 1 0 0 0 1 1h5"/><path d="M14 8l4 4-4 4M9 12h9"/>',
    'layout': '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 10h18M10 10v10"/>',
    'monitor-screen': '<rect x="3" y="4" width="18" height="12" rx="2"/><path d="M8 20h8M12 16v4"/>',
    'clock': '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
    'disk': '<circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="2.5"/><path d="M12 3a9 9 0 0 1 9 9"/>',
    'hexagon': '<path d="M12 3l7.8 4.5v9L12 21l-7.8-4.5v-9z"/>',
    'sound': '<path d="M4 10v4h3l4 3V7l-4 3z"/><path d="M15 9a4 4 0 0 1 0 6M17.5 6.5a7.5 7.5 0 0 1 0 11"/>',
    'sound-off': '<path d="M4 10v4h3l4 3V7l-4 3z"/><path d="M15 9.5l5 5M20 9.5l-5 5"/>',
    'dot': '<circle cx="12" cy="12" r="3"/>',
    'menu': '<path d="M4 7h16M4 12h16M4 17h16"/>',
    'swap': '<path d="M4 8h14l-3-3M20 16H6l3 3"/>'
  };

  var EMPTY = '<circle cx="12" cy="12" r="7"/>';

  function get(name, size) {
    size = size || 18;
    var body = PATHS[name] || EMPTY;
    return '<svg class="ico" width="' + size + '" height="' + size + '" viewBox="0 0 24 24" fill="none" ' +
      'stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">' +
      body + '</svg>';
  }

  function has(name) { return Object.prototype.hasOwnProperty.call(PATHS, name); }

  function names() { return Object.keys(PATHS); }

  return { get: get, has: has, names: names };
})();
