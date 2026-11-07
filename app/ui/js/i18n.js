'use strict';
/* Язык интерфейса: русский или английский. Подключается первым, до остальных скриптов,
   чтобы подписи приложений сразу брались на нужном языке. Тексты пишутся парой на месте:
   L('Файл', 'File'). Выбор хранится в Rust (prefs.json) и дублируется в localStorage. */
var I18N = (function () {
  var code = null;
  try { code = localStorage.getItem('bw_lang'); } catch (e) { /* без localStorage */ }
  if (code !== 'ru' && code !== 'en') {
    var nav = String(navigator.language || 'ru').toLowerCase();
    code = nav.indexOf('ru') === 0 ? 'ru' : 'en';
  }
  document.documentElement.lang = code;

  var I18N = {
    lang: code,
    en: code === 'en',
    /* Запомнить язык в этом профиле WebView (общий для всех окон игры). */
    remember: function (c) {
      c = c === 'en' ? 'en' : 'ru';
      try { localStorage.setItem('bw_lang', c); } catch (e) { /* без localStorage */ }
      return c;
    },
    /* Окна, которые при смене языка не перезагружаются (держат ввод или сеанс). */
    KEEP_ON_SWITCH: { terminal: true, editor: true, duel: true },
    /* Переключатель языка: две кнопки, выбранная подсвечена. Нажатие сохраняет выбор,
       окна перезагружаются по событию lang_changed. */
    picker: function () {
      var box = document.createElement('div');
      box.className = 'lang-picker';
      [['ru', 'Русский'], ['en', 'English']].forEach(function (p) {
        var b = document.createElement('button');
        b.type = 'button';
        b.className = 'btn' + (p[0] === I18N.lang ? ' btn-primary' : '');
        b.textContent = p[1];
        b.addEventListener('click', function () {
          if (p[0] === I18N.lang) return;
          if (window.BW && BW.sound) BW.sound('click');
          BW.setLang(p[0]).then(function () {
            if (I18N.KEEP_ON_SWITCH[BW.app]) location.reload();
          }, function () { location.reload(); });
        });
        box.appendChild(b);
      });
      return box;
    }
  };
  return I18N;
})();

function L(ru, en) { return I18N.en ? en : ru; }
