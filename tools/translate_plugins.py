"""Пересобирает все 100 плагинов сообщества по словарю перевода.

Входы:
  - plugins/catalog.json — исходный каталог.
  - tools/translations_plugins.json — ру-перевод: {id: {name, story, objective, service_label}}.
  - tools/plugin_templates/{content,economy,utility}.adr — главные скрипты плагинов.

На выходе переписывает:
  - plugins/community/<id>/plugin.json
  - plugins/community/<id>/main.adr   (копия нужного шаблона)
  - plugins/community/<id>/content.json
  - plugins/community/<id>/START_HERE.md
  - plugins/catalog.json
  - plugins/README.md
"""
from __future__ import annotations
import json
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
COMMUNITY = ROOT / "plugins/community"
CATALOG = ROOT / "plugins/catalog.json"
TRANSLATIONS = ROOT / "tools/translations_plugins.json"
TEMPLATES = ROOT / "tools/plugin_templates"
README = ROOT / "plugins/README.md"

FAMILY_RU = {
    "device": "устройства",
    "network": "сети",
    "case": "расследования",
    "event": "события",
    "utility": "инструменты",
    "economy": "магазины",
}

ORG_KIND_RU = {
    "corporation": "корпорация",
    "bank": "банк",
    "hospital": "больница",
    "security_center": "центр безопасности",
    "government": "ведомство",
    "university": "университет",
    "media": "СМИ",
    "shop": "магазин",
    "isp": "провайдер",
    "home": "домашняя сеть",
    "hosting": "хостинг",
    "underground": "скрытая группа",
}

TEMPLATE_BY_FAMILY = {
    "device": "content",
    "network": "content",
    "case": "content",
    "event": "content",
    "economy": "economy",
    "utility": "utility",
}


def main() -> None:
    catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
    translations = json.loads(TRANSLATIONS.read_text(encoding="utf-8"))

    missing = [p["id"] for p in catalog if p["id"] not in translations]
    if missing:
        raise SystemExit(f"нет перевода для: {missing[:5]} … всего {len(missing)}")

    templates = {name: (TEMPLATES / f"{name}.adr").read_text(encoding="utf-8") for name in {"content", "economy", "utility"}}

    new_catalog = []
    for plugin in catalog:
        ru = translations[plugin["id"]]
        name = ru["name"]
        story = ru["story"]
        objective = ru["objective"]
        service_label = ru.get("service_label", plugin.get("service", ""))

        plugin["name"] = name
        plugin["story"] = story
        plugin["objective"] = objective

        for node in plugin.get("nodes", []):
            node["os"] = f"{name} OS 1.0"
            node["text"] = (
                f"{story}\n"
                f"Владелец записи: {plugin['id']}.\n"
                f"Номер дела: BW-{plugin['number']:03d}.\n"
                f"Статус: сохранено для разбора в игре."
            )
        new_catalog.append(plugin)

        folder = COMMUNITY / plugin["id"]
        write_plugin_json(folder, plugin, name)
        write_content_json(folder, plugin)
        write_main_adr(folder, plugin, templates)
        write_start_here(folder, plugin, name, story, objective, service_label)

    CATALOG.write_text(
        json.dumps(new_catalog, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    write_readme(new_catalog)
    print(f"переведено плагинов: {len(new_catalog)}")


def write_plugin_json(folder: Path, plugin: dict, name: str) -> None:
    manifest = json.loads((folder / "plugin.json").read_text(encoding="utf-8"))
    manifest["name"] = name
    (folder / "plugin.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


def write_content_json(folder: Path, plugin: dict) -> None:
    (folder / "content.json").write_text(
        json.dumps(plugin, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


def write_main_adr(folder: Path, plugin: dict, templates: dict) -> None:
    kind = TEMPLATE_BY_FAMILY[plugin["family"]]
    (folder / "main.adr").write_text(templates[kind], encoding="utf-8")


def write_start_here(
    folder: Path,
    plugin: dict,
    name: str,
    story: str,
    objective: str,
    service_label: str,
) -> None:
    first_ip = plugin["nodes"][0]["ip"] if plugin.get("nodes") else ""
    reward = plugin.get("reward")
    key = plugin["nodes"][0]["key"] if plugin.get("nodes") else "item"
    pid = plugin["id"]
    family = FAMILY_RU.get(plugin["family"], plugin["family"])
    org_kind = ORG_KIND_RU.get(plugin.get("org_kind", ""), plugin.get("org_kind", ""))
    if plugin["family"] in ("utility", "economy"):
        write_short_guide(folder, plugin, name, story, pid, family)
        return
    lines = [
        f"# {name}",
        "",
        story,
        "",
        "## Начало",
        "",
        "1. Положи эту папку рядом с другими плагинами и нажми «Перечитать».",
        f"2. Открой окно подсказки командой `{pid}_open` или нажми «Старт».",
        f"3. В терминале запусти `{pid}_start`.",
        f"4. Прочитай письмо «{name}: первая зацепка» в игровой почте.",
        f"5. Открой `http://{first_ip}/` в игровом браузере и запусти `probe {first_ip}`.",
        f"6. {objective}",
        "",
        "## Подсказки",
        "",
        "- Если `probe` показывает активный файрвол, обойди его командой `firewall <адрес>`. При промахе попробуй ещё раз и следи за подозрением.",
        f"- На первом узле ищи собственную службу плагина — `{service_label}`.",
        "- Для доступа нужно слабое место «entry», а не просто утечка данных.",
        "- Внутри устройства посмотри `/etc/maintenance-note.txt` — там адрес записи.",
        f"- Команда `{pid}_status` показывает прогресс без сброса.",
    ]
    if reward:
        lines.append(
            f"- Награда: {reward} ₵, выплачивается один раз. Повторный запуск Start не дублирует содержимое и не сбрасывает прохождение."
        )
    lines += [
        "",
        "## Если хочешь готовое решение",
        "",
        f"`research -h 3 {first_ip} {pid}:{key}_entry`",
        f"Свой код: `exploit new ~/scripts/my_exploit.adr {first_ip} {pid}:{key}_entry`.",
        "Готовый инструмент: `market buy exp_overflow`.",
        f"`exploit -f ~/scripts/exp_overflow.adr -s {pid}:{key}_entry {first_ip}`",
        "",
    ]
    if plugin.get("nodes"):
        last = plugin["nodes"][-1]
        lines.append(f"Прочитай `{last['file']}` на последнем узле — там сохранённая запись.")
        lines.append("")
    lines += [
        "## Установка и память",
        "",
        f"Семейство: {family}. Организация: {org_kind}. Папка плагина не трогает чужие сохранения; созданные устройства и задания сохраняются вместе с миром.",
        "",
    ]
    (folder / "START_HERE.md").write_text("\n".join(lines).replace("exp_overflow", "exp_" + plugin.get("exploit_kind", "overflow")), encoding="utf-8")


def write_short_guide(folder: Path, plugin: dict, name: str, story: str, pid: str, family: str) -> None:
    lines = [
        f"# {name}",
        "",
        story,
        "",
        "## Начало",
        "",
        "1. Положи эту папку рядом с другими плагинами и нажми «Перечитать».",
    ]
    if plugin["family"] == "utility":
        lines += [
            f"2. Запусти `{pid}_open` — откроется окно, или `{pid}_report` для вывода в терминал.",
            "3. «Обновить» перечитывает мир; «Выгрузить отчёт» пишет отчёт в домашний каталог внутри игры (`reports/`).",
            "",
            "Задачи нет — пустой отчёт значит, что подходящие узлы или записи ещё не разведаны.",
        ]
    else:
        mode = plugin.get("mode")
        if mode == "hardware":
            command = f"buy {pid}:item"
            lines += [
                f"2. Купи улучшение командой `{command}` в игровом терминале.",
                "3. Улучшение применяется один раз и сохраняется в сейве.",
            ]
        else:
            command = f"market buy {pid}:item"
            file = plugin.get("file", "")
            lines += [
                f"2. Купи скрипт командой `{command}` — он появится в маркете.",
                f"3. После покупки запусти его: `adder ~/scripts/{file}`.",
            ]
    lines += [
        "",
        "## Установка и память",
        "",
        f"Семейство: {family}. Папка плагина не трогает чужие сохранения; результат остаётся в сейве даже если плагин выключить.",
        "",
    ]
    (folder / "START_HERE.md").write_text("\n".join(lines), encoding="utf-8")


def write_readme(catalog: list) -> None:
    counts = {}
    for plugin in catalog:
        counts[plugin["family"]] = counts.get(plugin["family"], 0) + 1
    header = [
        "# Плагины сообщества Blackwire",
        "",
        "100 независимых плагинов на Adder для Blackwire, SDK API 1.",
        "",
        "## Установка",
        "",
        "1. Распакуй `Blackwire-Plugins-<версия>.zip`.",
        "2. В игре открой **Плагины → Открыть папку**.",
        "3. Скопируй туда папки нужных плагинов из каталога `plugins` архива. Все 100 уживаются вместе.",
        "4. Нажми **Перечитать** в игре.",
        "5. Открой `START_HERE.md` внутри папки плагина. Команда `_open` открывает окно подсказки прямо в игре.",
        "",
        "Контент создаётся только после **Старт** — загрузка пакета не наполняет мир сразу. Инструменты и магазинные плагины работают сразу после загрузки. Награда и улучшение применяются один раз. Созданный контент остаётся в сейве, даже если плагин выключить.",
        "",
        "## Состав",
        "",
        f"{counts.get('device', 0)} пакетов устройств, {counts.get('network', 0)} сетей организаций, {counts.get('case', 0)} расследований, {counts.get('utility', 0)} инструментальных окон, {counts.get('economy', 0)} товаров магазина/маркета и {counts.get('event', 0)} событий во времени. Контентные пакеты добавляют 130 устройств с собственными слабыми местами, 70 заданий и 40 организаций, если запустить всё. Устройства используют уникальные адреса из 198.22.0.0/16, частные сети — из 10.230.0.0/16. Все адреса — внутри вымышленного мира игры.",
        "",
        "## Каталог плагинов",
        "",
        "| № | Плагин | Раздел | Что добавляет | Старт или команда |",
        "|---|---|---|---|---|",
    ]
    for plugin in catalog:
        family = FAMILY_RU.get(plugin["family"], plugin["family"])
        if plugin["family"] == "utility":
            cmd = f"`{plugin['id']}_open`"
        elif plugin["family"] == "economy":
            cmd = f"`{plugin['id']}_status`"
        else:
            cmd = f"`{plugin['id']}_start`"
        header.append(f"| {plugin['number']} | {plugin['name']} | {family} | {plugin['story']} | {cmd} |")
    header += [
        "",
        "## Проверка",
        "",
        "В репозитории `core/tests/community_plugins.rs` отдельно проверяет каждый плагин, а также все 100 вместе — на настоящем ядре игры. Проверяются команды и действия окон, собственные уязвимости, выполнение заданий, однократная награда, покупки, скрипты маркета, фазы инцидента, сохранение и отсутствие конфликтов адресов.",
        "",
        "По одному: `cargo test --locked -p blackwire-core --test community_plugins every_plugin -- --test-threads=1`.",
        "",
        "Проверка всех вместе помечена `#[ignore]` и запускается только на раннере GitHub. Контентные плагины реагируют только на свои слабые места — чужая атака не запускает все 70 обработчиков.",
        "",
    ]
    README.write_text("\n".join(header), encoding="utf-8")


if __name__ == "__main__":
    main()
