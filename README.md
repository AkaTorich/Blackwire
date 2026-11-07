# Blackwire
[Русская версия](README.ru.md)

**A hacking simulator with its own operating system, a living game network and the Adder programming language.**

[Download the game](https://github.com/AkaTorich/Blackwire/releases/latest) · [Website](https://game.globthe.top/) · [GitHub Pages](https://akatorich.github.io/Blackwire/) · [Player's guide](docs/en/GAME.md)

After being fired from the corporation, all you have left is a computer, an old router and a debt to the bank. An invitation from the **hello-world** group shows up in your mail. Behind it lie a hidden network, the first missions and a chance to win back your freedom.

Blackwire unfolds on a virtual desktop. The terminal, mail, browser, editor and network map open in separate windows: you can arrange them across the screen and move them between monitors. All applications work with one game world, where machines have owners, services, files, accounts and a history of events.

## What you can do

- **Explore the network.** Find hosts of corporations, banks, clinics and universities, study their services, pass through gateways into internal subnets.
- **Complete missions.** Look for information in mail and files, gain access to systems, deliver data and advance through the hello-world story. After the story you can keep playing freely.
- **Manage the consequences.** Watch your heat, build proxy chains, work with logs and evade security investigations.
- **Upgrade your computer.** Earn money on jobs, buy hardware and tools, set up your own base.
- **Create your own exploits.** Research in-game vulnerabilities, build a payload in Adder and test it against the in-game hints. Ready-made editable tools can be bought on the market; this also works with plugin vulnerabilities. 28 vulnerability models are available. The result can be a guest/user/root login, a data leak or partial access to a file, computer or router; the conditions of a hack are visible during research.
- **Write programs in Adder.** Automate actions, create terminal tools and work with game files and the network. The game has an editor with syntax highlighting and error checking.
- **Play together.** Create multiplayer games: compete for a target or complete a set of missions together.
- **Extend the world with plugins.** Add devices, their vulnerabilities, organisations, missions, events, shops and your own windows.

The look includes 12 themes and settings for fonts, transparency, effects and window layout. The game is bilingual, Russian and English: choose the language on the login screen or in Settings → Language. Computers, addresses and terminal actions belong to the game's fictional world.

## Download and install

Open the [latest release](https://github.com/AkaTorich/Blackwire/releases/latest) and pick the file for your system under **Assets**:

| System | File | Installation |
| --- | --- | --- |
| Windows 10/11, x64 | `Blackwire_<version>_x64-setup.exe` | Run the installer |
| Debian / Ubuntu, x64 | `Blackwire_<version>_amd64.deb` | Install the package via the app manager |
| macOS, Intel and Apple Silicon | `Blackwire_<version>_universal.dmg` | Open the image and move the game to Applications |

The same release also offers separate archives:

- **`Blackwire-SDK-<version>.zip`** — the SDK with documentation, examples, data schemas and plugin checker tools for Windows, Linux and macOS.
- **`Blackwire-Plugins-<version>.zip`** — a set of 100 plugins with instructions for each.
- **`Blackwire-Vulnerability-Plugins-<version>.zip`** — 10 additional story plugins: 30 vulnerabilities, including 10 custom types in Adder. Each plugin's description and START_HERE.md are in Russian.

The game installer also includes the SDK in the `sdk` directory. You don't need to install development tools for ordinary play.

## First steps

1. Launch Blackwire and create a new identity.
2. Open your mail and read the **hello-world** letter.
3. Follow the invitation in the **game browser** — it leads to the first challenge.
4. Look around your home folder: it holds files and ready-made scripts.
5. Open the terminal. The `help` command shows the available commands, and `man command_name` shows help for a specific one.
6. Read the briefings on the job board `gate.hw/board` inside the game. Look for addresses and hints in mail, documents and on the network map.

The game saves automatically once a minute and on a normal exit. Named save slots are available for separate playthroughs and experiments.

Details on the terminal, host access, the economy and investigations are in the [player's guide](docs/en/GAME.md).

## Plugins

Plugins are written in **Adder** and are connected through the **Plugins** app on the game desktop.

1. Download the plugin archive from the [release](https://github.com/AkaTorich/Blackwire/releases/latest) and unpack it.
2. In the game, open **Plugins → Open folder**.
3. Copy the folders of the chosen plugins from the archive's `plugins` directory there. Each folder must contain `plugin.json` and the `main.adr` source.
4. Press **Reload**.
5. Open `START_HERE.md` in the folder of the chosen plugin. It gives the setup, the first step and the commands; hints are also available in the plugin's game window.

Install the plugins you want to try. Generators add their content after the start command given in the instructions. Utilities and shops are available after loading. Created devices and missions stay in the world even after the plugin is disabled.

The set includes devices, organisation networks, investigations, tool panels, shop add-ons and events. Plugin names are in English. The description of each and its start command are collected in the [plugin catalog](plugins/README.en.md).

## Creating your own plugins

**Blackwire SDK, API 1** lets you create content and game logic in Adder:

- your own device types and individual machines with services, users and files;
- vulnerabilities for a device type or a specific device;
- organisations, subnets, missions, rewards, shops and achievements;
- terminal commands, windows, event handlers and timers;
- plugin state saved together with the game, and messaging in a multiplayer game.

Start with the [SDK guide](docs/en/SDK.md) and the [Adder language description](docs/en/ADDER.md). Ready examples: [Device Lab](sdk/examples/device_lab) with devices and vulnerabilities and [District](sdk/examples/district) with a district, a mission, a shop and a window.

The SDK includes `plugin-check`: it checks plugins on a separate game world without loading the player's saves. Launch commands and package contents are described in the [SDK README](sdk/README.en.md).

## Documentation

| Guide | Contents |
| --- | --- |
| [Game](docs/en/GAME.md) | Getting started, applications, network, missions, economy and saves |
| [Terminal commands](docs/en/COMMANDS.md) | Reference of the game shell commands |
| [Adder](docs/en/ADDER.md) | Language syntax, functions and game modules |
| [SDK](docs/en/SDK.md) | Creating plugins, API, events and content generation |
| [Hacking mechanics](docs/HACK.md) | Reconnaissance, access, vulnerabilities and countering defences |
| [Blackwire.chm](Blackwire.chm) | Compiled game and SDK help for Windows |

## Languages

Russian is the primary language, English is the second one; they switch on the fly and the choice is kept in `prefs.json` in the game's data folder.

| What | Where |
| --- | --- |
| Engine language switch, macros `tr!("ru", "en")` and `trf!(…)` | `lang/src/i18n.rs` (imported in every crate) |
| Language in the windows, `L('ru', 'en')` | `app/ui/js/i18n.js`; commands `lang_get` / `lang_set` in `app/src/cmd_game.rs` |
| Engine, world, mission and window texts | in pairs on the spot; data tables carry English fields next to Russian ones |
| English help | `docs/en/` (GAME, COMMANDS, HACK, ADDER, SDK, STORY) |
| English market scripts | `core/src/hack/exploits/en/` and the `*_EN` scripts in `core/src/market.rs` |
| Plugins | `<field>_en` in `content.json`, `name_en` in `plugin.json`, `plugin.lang()` in the templates; `START_HERE.en.md` |
| Generating the English plugin texts | `tools/localize_plugins.py` (100 community plugins), `tools/generate_vulnerability_cases.py` + `tools/vulnerability_cases_en.py` (10 cases) |
| Test | `core/tests/english.rs` — English world, help and command output contain no Cyrillic |

A new world is generated in the chosen language; mail, files and logs of an existing world stay in the language they appeared in.

## Building from source

The engine and the language are written in **Rust**, the shell uses **Tauri 2**, the interface is HTML, CSS and JavaScript. To build you need Rust stable, Python 3.11+ and the [Tauri system dependencies](https://v2.tauri.app/start/prerequisites/).

```sh
git clone https://github.com/AkaTorich/Blackwire.git
cd Blackwire
cargo install tauri-cli --version "^2" --locked
cd app
```

Run the command for your system:

| System | Command |
| --- | --- |
| Windows | `cargo tauri build --bundles nsis` |
| Linux | `cargo tauri build --bundles deb` |
| macOS | `cargo tauri build --target universal-apple-darwin --bundles dmg` |

For a universal macOS build, first install both Rust targets: `rustup target add aarch64-apple-darwin x86_64-apple-darwin`.

The build prepares the SDK automatically and includes it in the installation package. Build settings for all platforms are in [GitHub Actions](.github/workflows/build.yml).

The sources are spread across directories: `lang/` — Adder, `core/` — the game engine, `app/` — the application and interface, `sdk/` — materials for plugin authors, `plugins/` — ready-made plugins, `site/` — the game website (bilingual; GitHub Pages publishes it, `tools/server/update-site.ps1` uploads it to game.globthe.top). Engine and language tests are run from the repository root:

```sh
cargo test --locked -p blackwire-core -p blackwire-lang
```
