# Online — shared world

Online, every player lives in the same Blackwire network. Organizations, banks, hospitals, gateways and `.hw` sites are shared: what one player changes on a machine, the others will see. Everyone has their own computer (`name-pc`) and router (`name-router`) — machines of the shared network like all the others.

Everything you play for stays your own: the hello-world mission chain, the bank debt, gigs, the market, your base, suspicion and the investigation, achievements, the ending.

## How to log in

1. On the login screen — "Play online".
2. Server address (`game.globthe.top` by default), handle and password (6+ characters).
3. The first time — "New identity on the server", then — "Log in".

The account password is stored on the server only as a salted hash. Your in-game computer's password is issued by the server — it is in your notes (`notes`) and is not your account password.

Your single-player game is not touched online: its save is not written, save slots and the old two-player "Multiplayer" are unavailable online. The server keeps the world and your progress.

## What happens online

| What | How |
| --- | --- |
| Time | runs on the server; your personal clock (debt interest, suspicion decay, mission checks) runs while you are online |
| Terminal, files, mail, map, browser, Adder scripts | executed on the server on your behalf; the windows are yours, on your screen |
| Player chat | the "Player chat" window in the dock: who is online and a shared conversation |
| Logging in from elsewhere | the previous connection is closed |
| Scripts | up to 6 at once; stopped when you go offline |

## Your own server

The server is a separate windowless program `blackwire-server` (a Linux service).

```sh
blackwire-server --port 7777 --data ./data --cert fullchain.pem --key privkey.pem
```

| Option | What it does |
| --- | --- |
| `--port N` | port (7777 by default) |
| `--bind ADDRESS` | address to listen on (0.0.0.0) |
| `--data DIR` | world (`world.json`) and players (`players.json`), written every minute |
| `--cert`, `--key` | TLS certificate; reloaded automatically when the file changes |
| `--seed N` | seed of a new world (first start only) |
| `--lang ru\|en` | language of the shared world |
| `--plugins DIR` | plugins for the server |
| `--max-accounts N`, `--max-scripts N` | limits for accounts and scripts per player |

Without `--cert/--key` the connection is unencrypted — then write the address in the game as `tcp://192.168.0.5:7777` (for your own LAN).

Installing on Linux: build with `bash tools/server/build-server-linux.sh` (a static binary that runs on any x86_64 Linux) and on the server, as root, run `bash install.sh domain port` from the `server/deploy` folder — it creates the `blackwire` user, the `blackwire-server` service, a copy of the site certificate and its daily refresh.

## Inside

| What | Where |
| --- | --- |
| A player's part of the world and swapping it into the shared world | `core/src/mmo.rs` (`PlayerCtx`, `World::mmo_new`, `mmo_add_player`, `Game::swap_player`) |
| Transport: JSON lines over TLS or TCP | crate `wire/` |
| Server: accounts, commands, scripts, clock, chat | crate `server/` (`hub.rs`, `api.rs`, `scripts.rs`, `tick.rs`, `conn.rs`, `accounts.rs`) |
| Online in the game: login, forwarding commands, server events | `app/src/online.rs`; in windows — `BW.online` and `REMOTE` in `app/ui/js/bridge.js` |
| The "Online" card on the login screen | `app/ui/js/apps/login.js` |
| Player chat | `app/ui/js/apps/chat.js`, `app/ui/css/apps/chat.css` |
| Server installation | `server/deploy/` (service, `install.sh`), build — `tools/server/build-server-linux.sh` |
| Checks | `core/tests/mmo.rs`; a live server — `tools/online-smoke.ps1` (unencrypted) and `cargo run -p blackwire-wire --example probe -- address` (TLS like the game, no account); the game — the `--online "address\|name\|password[\|new]"` option |

Why it works this way: all game logic is written for "the player" and their computer `machines[0]`. Before a player's action the server swaps that player's part into the shared world and swaps their computer with a placeholder at index 0; after the action it swaps them back. So the single-player mechanics (hacking, surveillance, missions, money) work online unchanged, and machine ids always match their position in the list.
