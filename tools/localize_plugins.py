"""Английская версия 100 плагинов сообщества — поверх русской, в тех же файлах.

Игра выбирает язык сама: шаблоны `tools/plugin_templates/*.adr` спрашивают `plugin.lang()` и берут
поле `<имя>_en`, если оно есть. Скрипт дописывает эти поля и английские инструкции:

  - plugins/catalog.json, plugins/community/<id>/content.json — name_en, story_en, objective_en;
    у узлов os_en и text_en;
  - plugins/community/<id>/plugin.json — name_en (название в окне «Плагины»);
  - plugins/community/<id>/main.adr — копия нужного шаблона;
  - plugins/community/<id>/START_HERE.en.md — инструкция на английском;
  - plugins/README.en.md — английский каталог.

Английские названия, завязки и цели — `tools/translations_plugins_en.json`. Русские тексты не трогаются.
Повторный запуск даёт тот же результат.
"""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
COMMUNITY = ROOT / "plugins/community"
CATALOG = ROOT / "plugins/catalog.json"
EN = ROOT / "tools/translations_plugins_en.json"
TEMPLATES = ROOT / "tools/plugin_templates"
README_EN = ROOT / "plugins/README.en.md"

TEMPLATE_BY_FAMILY = {
    "device": "content", "network": "content", "case": "content", "event": "content",
    "economy": "economy", "utility": "utility",
}
FAMILY_EN = {
    "device": "devices", "network": "networks", "case": "investigations",
    "event": "events", "utility": "tools", "economy": "shops",
}
ORG_KIND_EN = {
    "corporation": "corporation", "bank": "bank", "hospital": "hospital",
    "security_center": "security center", "government": "government agency",
    "university": "university", "media": "media", "shop": "shop", "isp": "ISP",
    "home": "home network", "hosting": "hosting", "underground": "underground group",
}
# Цели инструментов и товаров (в старом английском каталоге их не было).
OBJECTIVE_EN = {
    "Показать свежие журналы.": "Show recent logs.",
    "Увеличить диск на 8 ГБ.": "Add 8 GB of disk.",
    "Показать текущие параметры узла.": "Show the current host parameters.",
    "Показать машины под контролем игрока.": "Show machines under the player's control.",
    "Показать расследование и его стадию.": "Show the investigation and its stage.",
    "Показать финансовое состояние и прайс.": "Show finances and prices.",
    "Собрать непрочитанные письма.": "Collect unread mail.",
    "Увеличить память на 4 ГБ.": "Add 4 GB of memory.",
    "Собрать задания плагинов и их прогресс.": "Collect plugin missions and their progress.",
    "Показать активные службы.": "Show running services.",
    "Показать активные задания.": "Show active missions.",
    "Показать известные организации.": "Show known organisations.",
    "Увеличить диск на 16 ГБ.": "Add 16 GB of disk.",
    "Разобрать текущий каталог.": "Inspect the current directory.",
    "Показать процессы на машине.": "Show processes on the machine.",
    "Показать купленные улучшения и прайс.": "Show bought upgrades and prices.",
    "Показать подозрение и слежку.": "Show heat and surveillance.",
    "Показать прокси-цепочку.": "Show the proxy chain.",
    "Показать карту шлюзов и подсетей.": "Show the map of gateways and subnets.",
    "Показать разведанные узлы и их состояние.": "Show scouted hosts and their state.",
    "Сохранить заметки игрока отчётом.": "Save the player's notes as a report.",
    "Увеличить память на 1 ГБ.": "Add 1 GB of memory.",
    "Увеличить память на 256 МБ.": "Add 256 MB of memory.",
    "Показать уязвимости, добавленные плагинами.": "Show vulnerabilities added by plugins.",
    "Увеличить память на 2 ГБ.": "Add 2 GB of memory.",
    "Показать баланс и долг.": "Show balance and debt.",
    "Увеличить диск на 32 ГБ.": "Add 32 GB of disk.",
    "Сравнить железо разведанных машин.": "Compare hardware of scouted machines.",
    "Показать зарегистрированные типы устройств.": "Show registered device types.",
    "Увеличить диск на 2 ГБ.": "Add 2 GB of disk.",
}


def read_text(path: Path) -> tuple[str, str]:
    """Текст файла и его перевод строки (файлы в репозитории бывают с CRLF)."""
    raw = path.read_bytes().decode("utf-8")
    eol = "\r\n" if "\r\n" in raw else "\n"
    return raw.replace("\r\n", "\n"), eol


def write_text(path: Path, text: str, eol: str = "\n") -> None:
    path.write_bytes(text.replace("\n", eol).encode("utf-8"))


def dump(obj) -> str:
    return json.dumps(obj, ensure_ascii=False, indent=2) + "\n"


def with_after(d: dict, key: str, new_key: str, value) -> dict:
    """Копия словаря, где new_key стоит сразу после key (порядок полей сохраняется)."""
    out = {}
    for k, v in d.items():
        if k == new_key:
            continue
        out[k] = v
        if k == key:
            out[new_key] = value
    if new_key not in out:
        out[new_key] = value
    return out


def localize(plugin: dict, en: dict) -> dict:
    p = dict(plugin)
    p = with_after(p, "name", "name_en", en["name"])
    p = with_after(p, "story", "story_en", en["story"])
    if "objective" in p:
        obj = en.get("objective") or OBJECTIVE_EN.get(p["objective"])
        if not obj:
            raise SystemExit(f"нет английской цели для {p['id']}: {p['objective']}")
        p = with_after(p, "objective", "objective_en", obj)
    nodes = []
    for node in p.get("nodes", []):
        n = with_after(node, "os", "os_en", f"{en['name']} OS 1.0")
        n = with_after(n, "text", "text_en", (
            f"{en['story']}\n"
            f"Record owner: {p['id']}.\n"
            f"Case reference: BW-{p['number']:03d}.\n"
            f"Status: preserved for in-game review."
        ))
        nodes.append(n)
    if "nodes" in p:
        p["nodes"] = nodes
    return p


def start_here_en(p: dict) -> str:
    pid = p["id"]
    name, story = p["name_en"], p["story_en"]
    family = FAMILY_EN.get(p["family"], p["family"])
    lines = [f"# {name}", "", story, "", "## Start here", "", "1. Put this folder next to your other plugins and press **Reload**."]
    if p["family"] == "utility":
        lines += [
            f"2. Run `{pid}_open` to open the window, or `{pid}_report` to print to the terminal.",
            "3. **Refresh** re-reads the world; **Export report** writes a report to your in-game home folder (`reports/`).",
            "",
            "There is no task — an empty report means no matching hosts or records have been scouted yet.",
        ]
    elif p["family"] == "economy":
        if p.get("mode") == "hardware":
            lines += [
                f"2. Buy the upgrade with `buy {pid}:item` in the game terminal.",
                "3. The upgrade is applied once and kept in the save.",
            ]
        else:
            lines += [
                f"2. Buy the script with `market buy {pid}:item` — it appears in the market.",
                f"3. After buying, run it: `adder ~/scripts/{p.get('file', '')}`.",
            ]
    if p["family"] in ("utility", "economy"):
        lines += [
            "",
            "## Installation and persistence",
            "",
            f"Family: {family}. The plugin folder never touches other saves; the result stays in the save even if the plugin is disabled.",
            "",
        ]
        return "\n".join(lines)

    nodes = p.get("nodes", [])
    first_ip = nodes[0]["ip"] if nodes else ""
    key = nodes[0]["key"] if nodes else "item"
    kind = p.get("exploit_kind", "overflow")
    org_kind = ORG_KIND_EN.get(p.get("org_kind", ""), p.get("org_kind", ""))
    service = p.get("service", "")
    lines = [
        f"# {name}", "", story, "",
        "## Start here", "",
        "1. Put this folder next to your other plugins and press **Reload**.",
        f"2. Open the guide window with `{pid}_open` or press **Start**.",
        f"3. In the terminal run `{pid}_start`.",
        f"4. Read the mail “{name}: first lead” in the game mail.",
        f"5. Open `http://{first_ip}/` in the game browser and run `probe {first_ip}`.",
        f"6. {p.get('objective_en', '')}",
        "",
        "## Hints",
        "",
        f"Hack mechanic: {kind}. `research` gives the hints and parameters of the specific device.",
        "",
        "- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.",
        f"- On the first host look for the plugin's own service — `{service}`.",
        "- Access needs the “entry” weakness, not just a data leak.",
        "- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.",
        f"- `{pid}_status` shows progress without resetting anything.",
    ]
    if p.get("reward"):
        lines.append(f"- Reward: {p['reward']} ₵, paid once. Running Start again does not duplicate content or reset completion.")
    lines += [
        "",
        "## If you want a ready solution",
        "",
        f"`research -h 3 {first_ip} {pid}:{key}_entry`",
        "",
        f"Your own source: `exploit new ~/scripts/my_exploit.adr {first_ip} {pid}:{key}_entry`. Fill in the payload using the in-game hints.",
        "",
        f"Ready editable tool: `market buy exp_{kind}`.",
        "",
        f"`exploit -f ~/scripts/exp_{kind}.adr -s {pid}:{key}_entry {first_ip}`",
        "",
    ]
    if nodes:
        lines += [f"Read `{nodes[-1]['file']}` on the last host — the preserved record is there.", ""]
    lines += [
        "## Installation and persistence",
        "",
        f"Family: {family}. Organisation: {org_kind}. The plugin folder never touches other saves; created devices and missions are saved with the world.",
        "",
    ]
    return "\n".join(lines)


def readme_en(catalog: list) -> str:
    counts: dict = {}
    for p in catalog:
        counts[p["family"]] = counts.get(p["family"], 0) + 1
    lines = [
        "# Blackwire community plugins",
        "",
        "[Русская версия](README.md)",
        "",
        "100 independent Adder plugins for Blackwire, SDK API 1. Every plugin is bilingual: it follows the game language (Russian or English).",
        "",
        "## Installation",
        "",
        "1. Unpack `Blackwire-Plugins-<version>.zip`.",
        "2. In the game open **Plugins → Open folder**.",
        "3. Copy the folders of the plugins you want from the archive's `plugins` directory. All 100 work together.",
        "4. Press **Reload** in the game.",
        "5. Open `START_HERE.en.md` inside the plugin folder. The `_open` command opens the guide window right in the game.",
        "",
        "Content is created only after **Start** — loading the pack does not fill the world right away. Tools and shop plugins work as soon as they are loaded. Rewards and upgrades apply once. Created content stays in the save even if the plugin is disabled.",
        "",
        "## Contents",
        "",
        f"{counts.get('device', 0)} device packs, {counts.get('network', 0)} organisation networks, {counts.get('case', 0)} investigations, {counts.get('utility', 0)} tool windows, {counts.get('economy', 0)} shop/market items and {counts.get('event', 0)} timed events. If you run everything, the content packs add 130 devices with their own weak spots, 70 missions and 40 organisations. Devices use unique addresses from 198.22.0.0/16, private networks from 10.230.0.0/16. All addresses belong to the game's fictional world.",
        "",
        "## Plugin catalog",
        "",
        "| # | Plugin | Section | What it adds | Start or command |",
        "|---|---|---|---|---|",
    ]
    for p in catalog:
        family = FAMILY_EN.get(p["family"], p["family"])
        if p["family"] == "utility":
            cmd = f"`{p['id']}_open`"
        elif p["family"] == "economy":
            cmd = f"`{p['id']}_status`"
        else:
            cmd = f"`{p['id']}_start`"
        lines.append(f"| {p['number']} | {p['name_en']} | {family} | {p['story_en']} | {cmd} |")
    lines += [
        "",
        "## Testing",
        "",
        "In the repository `core/tests/community_plugins.rs` checks every plugin on its own and all 100 together on the real game core: commands and window actions, custom vulnerabilities, mission completion, one-time rewards, purchases, market scripts, incident phases, saving and the absence of address conflicts.",
        "",
        "One by one: `cargo test --locked -p blackwire-core --test community_plugins every_plugin -- --test-threads=1`.",
        "",
        "The all-together check is marked `#[ignore]` and runs only on the GitHub runner. Content plugins react only to their own weak spots — someone else's attack does not trigger all 70 handlers.",
        "",
    ]
    return "\n".join(lines)


def main() -> None:
    catalog_text, catalog_eol = read_text(CATALOG)
    catalog = json.loads(catalog_text)
    en = json.loads(EN.read_text(encoding="utf-8"))
    missing = [p["id"] for p in catalog if p["id"] not in en]
    if missing:
        raise SystemExit(f"нет английского текста для: {missing[:5]} … всего {len(missing)}")
    templates = {k: (TEMPLATES / f"{k}.adr").read_text(encoding="utf-8").replace("\r\n", "\n") for k in ("content", "economy", "utility")}

    out = []
    for plugin in catalog:
        p = localize(plugin, en[plugin["id"]])
        out.append(p)
        folder = COMMUNITY / p["id"]

        _, eol = read_text(folder / "content.json")
        write_text(folder / "content.json", dump(p), eol)

        manifest_text, eol = read_text(folder / "plugin.json")
        manifest = with_after(json.loads(manifest_text), "name", "name_en", p["name_en"])
        write_text(folder / "plugin.json", dump(manifest), eol)

        _, eol = read_text(folder / "main.adr")
        write_text(folder / "main.adr", templates[TEMPLATE_BY_FAMILY[p["family"]]], eol)

        _, eol = read_text(folder / "START_HERE.md")
        write_text(folder / "START_HERE.en.md", start_here_en(p), eol)

    write_text(CATALOG, dump(out), catalog_eol)
    write_text(README_EN, readme_en(out))
    print(f"plugins localized: {len(out)}")


if __name__ == "__main__":
    main()
