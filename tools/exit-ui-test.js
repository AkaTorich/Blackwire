/* Native integration check. Requires --exit-ui-test and an isolated --data-dir.
 * The caller must verify exit code 0 and /tmp/exit-ui-marker.txt in save.json.
 * Run once with --new and once with --load to cover an unchanged loaded game.
 */
(async function () {
  'use strict';
  try {
    var button = document.querySelector('.desk-exit');
    if (!button) throw new Error('Exit button missing');
    var rect = button.getBoundingClientRect();
    var hit = document.elementFromPoint(rect.x + rect.width / 2, rect.y + rect.height / 2);
    if (!button.contains(hit)) throw new Error('Exit button covered');
    Win.toast('Проверка: сохранение и выход через кнопку', 'ok');
    await new Promise(function (resolve) { setTimeout(resolve, 1500); });
    await BW.invoke('sdk_ui_report', {stage: 'exit', result: {pass: true, buttonReachable: true}});
    // This change happens after the report saved the world, before the button saves it.
    await BW.fs_write(null, '/tmp/exit-ui-marker.txt', 'saved by exit button');
    button.click();
    setTimeout(function () {
      BW.invoke('sdk_ui_report', {stage: 'exit', result: {pass: false, error: 'Process stayed open after exit click'}});
    }, 5000);
  } catch (e) {
    await BW.invoke('sdk_ui_report', {stage: 'exit', result: {pass: false, error: String(e)}});
  }
})();
