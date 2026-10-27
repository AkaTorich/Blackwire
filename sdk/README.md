# Blackwire SDK

SDK API 1 for Adder plugins. The SDK version matches the game release.

The game installer installs this SDK in the game's `sdk` directory.
The same release on GitHub provides `Blackwire-SDK-<version>.zip` as a separate download.
The ZIP includes checker tools for Windows, Linux and macOS; an installed copy includes the tool for its OS.

## Contents

- `docs/SDK.md` — plugin API, permissions, events and content generation.
- `docs/ADDER.md` — Adder language and game modules.
- `docs/COMMANDS.md` — game terminal commands.
- `docs/GAME.md`, `docs/HACK.md`, `docs/STORY.md` — gameplay and world.
- `Blackwire.chm` — compiled game and SDK help for Windows.
- `examples/device_lab` — devices and per-device vulnerabilities.
- `examples/district` — a district with a quest, shop, achievement, timer and window.
- `schema` — game data examples.
- `bin/<platform>/plugin-check[.exe]` — checker built from the matching game engine.
- `manifest.json` — release version, SDK API, source commit and file checksums.

In the Git source tree the manuals are in `../docs`, and the checker source is
`../core/src/bin/plugin-check.rs`. The build collects these files automatically.

## Try an example

From the SDK directory on Windows:

```powershell
.\bin\windows\plugin-check.exe .\examples --command lab_create --command district_create --ticks 65
```

Linux:

```sh
chmod +x bin/linux/plugin-check
./bin/linux/plugin-check examples --command lab_create --command district_create --ticks 65
```

macOS (Intel and Apple Silicon):

```sh
chmod +x bin/macos/plugin-check
./bin/macos/plugin-check examples --command lab_create --command district_create --ticks 65
```

The checker uses a separate generated game world and does not load player saves.
In the game, open **Plugins**, choose **Open folder**, copy a plugin folder there,
then choose **Reload**. You can also install both examples with the manager's buttons.
On Windows the default plugin folder is `%APPDATA%\top.blackwire.game\plugins`.
Each plugin folder contains `plugin.json` and its Adder source.

## Результаты и условия взлома

`examples/access_lab` — десять вариантов результата взлома и изменяемые условия;
`examples/custom_vuln_type` — авторская проверка payload на Adder.
Начни с `access_create`, затем `access_ready`; подробности — docs/SDK.md.
