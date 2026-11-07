# Blackwire SDK — plugins in Adder

API version: **1**. Plugin code is written in the game's **Adder** language, extension `.adr`.

## Installing and running

Open "Plugins" on the game desktop. "Open folder" shows the package directory of the current profile: by default `%APPDATA%\top.blackwire.game\plugins`. Put a separate plugin folder there and press "Reload". The folder contains `plugin.json`, the source and, if needed, libraries and assets.

The "Add device example" and "Add district example" buttons install the built-in examples. They do not replace folders that already exist. After installation the card shows the plugin's commands and its own windows. Commands can be run with the buttons or typed in the game terminal.

Disabling a plugin stops calls to its handlers. Machines, files, missions and purchased items already created stay in the world; the plugin's data is kept. Deleting the package folder does not remove the created content from the save either. To return the world to its state before installation, use a save slot made beforehand.

## Your first plugin

Folder `my_counter`:

```json
{
  "id": "my_counter",
  "name": "My counter",
  "version": "1.0.0",
  "api": 1,
  "entry": "main.adr",
  "events": {"on_tick": "tick"},
  "commands": {"counter_show": "show"}
}
```

File `main.adr`:

```python
def tick(event):
    plugin.set("ticks", plugin.get("ticks", 0) + 1)

def show(event):
    print("World ticks:", plugin.get("ticks", 0))
```

Press "Reload", then `counter_show`. The value is saved together with the game.

## Package layout

Required manifest fields: `id`, `name`, `version`, `api`, `entry`. Unknown fields are an error. `id` and function/command names are ASCII letters, digits and underscores; the first character is not a digit, length up to 64. Built-in command names cannot be overridden.

Optional fields:

| Field | Meaning |
| --- | --- |
| name_en | plugin name shown when the game is in English; if absent, `name` is shown |
| permissions | list of permissions |
| events | dict "event → function name" |
| commands | dict "command name → function name" |
| libraries | list of relative paths to Adder sources; prepended to entry in the given order |
| assets | list of relative paths to UTF-8 text assets; read with plugin.asset |
| requires | list of ids of required plugins; sets the load order |

Every handler takes a single dict `event`. The return value of an ordinary function is ignored. `print` goes to the command's response, and errors go to the plugin card. Output of a background handler does not create a notification by itself: use `sys.notify` for that.

For background events the plugin has its own game session: the current machine, user and folder persist between events while the game is running. A plugin command runs in the session of the terminal that invoked it. When the package is disabled or reloaded, its service session is closed together with the scripts it owns.

The source is executed anew on every event. Keep function definitions and constants at the top level; register content in `on_load`. For long-term data use `plugin.get/set`, not Adder global variables. Libraries are also executed anew. Error line numbers refer to the combined text libraries + entry + the handler call.

Plugins are loaded in dependency order, independent ones by id. A missing dependency and a cycle are marked as errors. Use distinct package ids and distinct command names. The `api` version must match the engine version; the `version` field describes the version of the package itself.

## Permissions

| Permission | Capabilities |
| --- | --- |
| no permissions | own storage plugin.get/set, event data, declared assets |
| content | creating/changing the game world, device types, vulnerabilities, catalogs, missions, windows, timers |
| sys | game sys functions: information, notifications, opening applications; sys.kill is forbidden in a handler |
| fs | ordinary operations on virtual files with the rights of the current game session |
| net | ordinary game network actions with the rights of the current game session |
| shell | game shell commands via shell.exec; this also gives access to mechanics implemented as commands |
| time | game host calls time.now |
| network | sending and receiving messages through an already connected multiplayer game |

`content` is author-level power over the game world. It is much broader than the rights of an ordinary character. `net` means the game's fictional network, `network` means the channel of an established multiplayer game. The SDK has no API for reading arbitrary Windows files or running Windows commands.

## Events

| Event | When and what is passed |
| --- | --- |
| on_load | loading, reloading or enabling the package; empty dict |
| on_tick | Game.tick call; dt_ms — milliseconds elapsed |
| on_command | a built-in player command finished; line, session |
| on_hack | a successful remote or local game hack; target, target_host, vuln (object; name in vuln["name"]), outcome, source_ip, monitored, heat_added |
| on_mail | a new incoming mail after a world step: direction=inbox, mail; sending via Game.mail_send: direction=sent, to, subject, body |
| on_mission_done | the story chain advanced; before, active |
| on_gig_done | the number of completed gigs changed; before, done |
| on_purchase | a purchase; catalog=shop/market, id |
| on_achievement | an achievement unlocked; id |
| on_investigation | the surveillance stage changed; before, stage, progress |
| on_scenario_done | the active scenario is complete; scenario |
| on_world | the number of machines or organisations changed; machines, orgs |
| on_file | a change via the editor/file window or the Adder file API; operation and details of the operation |
| on_ui | a button of the plugin's own window; window, action, values |
| on_network | a packet of the established multiplayer game; plugin, channel, data, from |
| on_save | before saving the main profile |
| on_unload | disabling the package or exiting via the game command quit |
| custom.name | an addressed timer event of this plugin; timer, data |

State-change events are produced after a shell action, an Adder API call and a world step. Changes made by a handler do not trigger a synchronous recursive run of another handler. Checks of your own arbitrary conditions are convenient in `on_tick` or by timer. Killing the Windows process directly does not guarantee on_unload; save important data as you go.

## Core functions

All `plugin.*` below can be called only from a plugin. JSON values become ordinary Adder dicts, lists, strings, numbers and null.

| Call | Result / action |
| --- | --- |
| plugin.id() | id of the current plugin |
| plugin.lang() | game language: "ru" or "en"; also allowed in exploit-type profile/validate code |
| plugin.event() | dict of the current event |
| plugin.get(key, default=null) | value of your own saved key |
| plugin.set(key, value) | save a value; key up to 128 characters, data of one plugin up to 64 KB |
| plugin.asset(path) | string of a declared text asset |
| plugin.world() / plugin.world("overview") | full world state / lightweight data for reports |
| plugin.apply(patch) | atomically replace the given sections of the game state after validation |
| plugin.random(lo, hi) | integer in [lo, hi), the world's generator; bounds within ±10^9 |
| sys.lang() | game language "ru" or "en" for plugin code with the sys permission |

Example of changing the economy:

```python
world = plugin.world()
world["finance"]["rate_per_day"] = 1.0
plugin.apply({"finance": world["finance"]})
```

`apply` accepts the sections `machines`, `orgs`, `player`, `missions`, `finance`, `ending`, `life`, `gigs`, `market`, `base`, `achievements`, `scenarios`, `investigation`, `fw_down`, `keyloggers`, `botnet`, `botnet_mined`, `defense`, `down_until`. Several sections can be changed in one call, for example machines and organisations at once. The structure is the same as in `plugin.world()`; examples of real objects are in `sdk/schema`.

The engine checks types, identifiers, references, ports, addresses and routes. You cannot delete an existing machine/organisation, replace the player's identity, delete the user of the active session or create a router loop. To take a device out of service, set `up=false`. The clock, the internal generator, queues and other plugins' data cannot be replaced through apply.

## Device types

| Call | Purpose |
| --- | --- |
| plugin.template(base_kind) | a ready template from the game's real generator |
| plugin.device_type(key, template) | register your own type; returns `plugin_id:key` |
| plugin.spawn(type_id, properties) | create an instance; returns the new numeric id |
| plugin.machine(id) | copy of one machine |
| plugin.update_machine(id, properties) | partial change of a machine with link validation |
| plugin.user(id, name, password, admin=false) | add a game account and a home folder |
| plugin.write(id, path, text) | author write to a virtual file; creates parent folders |

Base kinds: `player`, `home_pc`, `router`, `gateway`, `server`, `workstation`, `database`, `file_server`, `mail`, `web`, `camera`, `badge`, `atm`, `medical_device`, `dns`, `isp`, `honeypot`.

A custom type is a separate `device_type` with any content built on a base behaviour. For example, a sensor, an industrial controller or a satellite modem gets its own type name, OS, hardware, services, files, users and vulnerabilities. Special behaviour is implemented with Adder handlers. New native graphics primitives or interpreter instructions require changes to the engine itself.

```python
def load(event):
    template = plugin.template("camera")
    template["os"] = "SensorOS 1.0"
    template["org"] = null
    template["router"] = null
    template["services"] = [{"port": 9000, "name": "sensor", "version": "1.0", "running": true, "banner": "Sensor control", "pages": []}]
    plugin.device_type("sensor", template)

def create(event):
    mid = plugin.spawn("my_plugin:sensor", {"hostname": "sensor-1", "ip": "198.18.10.1", "lan_ip": null, "domain": null, "up": true})
    plugin.user(mid, "operator", "sensor123", false)
    plugin.write(mid, "/etc/sensor/config.txt", "mode=normal\n")
```

`id` is assigned automatically, hostname and IP must be unique. For an internal device, set `lan_ip`, `router`, the organisation and the route. Add the id to `player.known_machines` if it should appear on the map right away. The device_type string is shown in the map card and in neofetch; other behaviour rules still use the base kind.

## Custom vulnerabilities

`plugin.vulnerability(key, spec)` registers a rule and returns a namespaced id. The rule applies to existing devices and to new ones as they appear/change. It takes part in the ordinary `probe`, `net.vulns`, `exploit` and local privilege escalation.

| spec field | Purpose |
| --- | --- |
| device_type | optional; only this device type |
| machine | optional; only this numeric device id |
| service | name of the remote service |
| version_prefix | service version prefix; for local — OS prefix; empty means all versions |
| access | creds, shell, root, info or read:/absolute/path |
| local | true for a local kernel vulnerability, false by default |
| handler | optional; name of an Adder function for extra consequences of a successful remote exploit |

If both device_type and machine are given, both conditions must match. Separate vulnerabilities can be registered for any device. For a remote vulnerability the service must be running and match by name/version. An updated service version no longer fits the old rule.

```python
plugin.vulnerability("sensor_default_login", {
    "device_type": "my_plugin:sensor",
    "service": "sensor",
    "version_prefix": "1.",
    "access": "shell"
})
plugin.vulnerability("sensor_42_overload", {
    "machine": 42,
    "service": "sensor",
    "version_prefix": "1.0",
    "access": "info",
    "handler": "overload"
})

def overload(event):
    plugin.update_machine(event["target"], {"up": false})
    plugin.news("One of the sensors stopped responding.")
```

Plugin vulnerabilities are exploited with your own or purchased Adder code, just like the standard ones. A single command without a payload does not grant access. Research: `research address my_plugin:sensor_42_overload`; template: `exploit new ~/scripts/sensor.adr address my_plugin:sensor_42_overload`; run: `exploit -f ~/scripts/sensor.adr -s my_plugin:sensor_42_overload address`.

You can add to the vulnerability description:
```python
"exploit": {
    "kind": "overflow",
    "difficulty": 3,
    "hint": "Check the frame boundary and the address of the service handler.",
    "market": "my_plugin:sensor_exploit"
}
```
kind: one of the 28 built-in models (list below) or a plugin_id:type_id type registered via plugin.exploit_type. difficulty: 1..5. hint is an in-game hint, market is the id of a compatible item. Without exploit the mechanic is chosen by service and access type, so older content declarations keep working. If the author's item is missing, the hint suggests a compatible standard tool.

`net.research(address, id)` returns a profile with layout, fingerprint and hints. The player builds a payload; `net.probe(address, id, payload)` checks it, `net.exploit(address, id, payload)` applies it. Local root requires a shell on the target and `net.privesc(payload, id)`. Each machine and version has its own fingerprint; an old payload will not work on another target. Profiles are reproduced after saving and loading.

Successful remote and local hacks trigger on_hack and the handler of the specific vulnerability. A probe and a rejected payload do not call the success handler. Ordinary routes, firewalls, logging and heat still apply.

Selling your own exploit:
```python
plugin.catalog("market", "sensor_exploit", {
    "name": "Sensor Payload",
    "desc": "Sensor overflow exploit; the source can be edited",
    "price": 180,
    "file": "sensor_payload.adr",
    "code": plugin.asset("sensor_payload.adr")
})
```
Add the source to the manifest's assets. `market buy my_plugin:sensor_exploit` installs the editable file into `~/scripts`. An existing file with the same name is not overwritten. A full example with a device, a hint, conditions and a purchase is in sdk/examples/device_lab.

## Services, sites, organisations and mail

| Call | Format |
| --- | --- |
| plugin.service(machine, spec) | port, name, version, running, banner, pages; replaces the service on this port |
| plugin.page(machine, port, spec) | path, title, html; adds/replaces a page of an existing service |
| plugin.org(spec) | name, kind, domain, gateway, subnet, members, description; id is assigned automatically |
| plugin.mail(machine, spec) | from, subject, body, optional to; an incoming mail on the given machine |
| plugin.log(machine, kind, text) | login, login_failed, logout, exec, file_change, service, net, warn |
| plugin.news(text) | a news item in the ordinary city feed |

Create the machines without an organisation first, then the organisation with their ids in members. `plugin.org` updates the org field of the listed machines. Routing is set by router and lan_ip. The organisation kind is one of the standard corporation, bank, hospital, security_center, cyber_police, isp, university, hosting, government, media, shop, home, honeypot, underground.

The site opens in the real game browser. The page HTML goes through the game browser's existing renderer; it is content of the virtual network. For your own interactive tools, use plugin windows.

## Missions and scenarios

`plugin.quest(key, {title, objective, reward})` creates an independent plugin mission. Returns `plugin_id:key`. Registering again does not reset completion. `plugin.quest_complete(id)` pays the reward once and returns true on the first completion, false on a repeat. Any Adder code can check the conditions: the number of devices, a specific file, a service state, a purchase, a timer, a mail, a combination of actions. Missions are visible in missions and on gate.hw/board.

`plugin.gig(spec)` adds a real gig: title, objective, reward, goal. The standard goals `{"Own":{"target":42}}` and `{"Deliver":{"target":42,"path":"/srv/data.txt","needle":"marker"}}` are supported. The id is assigned automatically.

`plugin.scenario(spec)` starts an editor scenario: name, brief, goal, reward. Standard conditions: `{"OwnHost":{"host":"server-name"}}`, `{"Deliver":{"needle":"marker"}}`, `{"Money":{"amount":1000}}`. Only one such scenario is active at a time; your own quests can run in parallel.

The story chain can be extended through the missions section in apply. It uses the real MissionRun and Goal: ReadDeliver, Own, Plant, CoverTracks. Change active and status consistently. For independent additions quest is preferable, so the completion condition of the main story is not changed.

## Shop, market and achievements

`plugin.catalog(kind, key, item)` adds an entry. kind: shop, market, achievements. item contains name, desc, an optional price; for market also file and code. The id is returned as `plugin_id:key`.

Shop entries are visible via shop and gate.hw/shop and are bought with the buy command. The fact of purchase is in player.upgrades. Implement the upgrade's effect in on_purchase or on_tick and store a flag that the effect has already been applied.

Market entries are visible via market and gate.hw/market. Buying with market buy id puts the declared Adder code into ~/scripts/file and charges the price. The file name is only an .adr name without directories. Choose a unique file name for your package.

Achievement entries are visible in achievements/ach. `plugin.unlock("plugin_id:key")` unlocks an achievement. The plugin checks the condition itself. Purchases and rewards use the game's standard balance.

## Timers and a living world

`plugin.timer(key, delay_seconds, event, payload=null, repeat_seconds=0)` creates an addressed timer of this plugin. delay: 1..31536000; event starts with custom. and is declared in the manifest's events. repeat=0 means one-shot. Registering the same key again replaces the timer. `plugin.cancel_timer(key)` cancels it.

Timers are stored in the save and checked on world steps. On a large time jump, missed repeats are not all replayed at once: it fires once, and the next deadline is counted from the current time. A timer does not run while the plugin is disabled. After enabling, an overdue timer fires on the next step.

News, mail, changes to services, files, processes and defence state let you build your own simulation. For reproducible decisions use plugin.random, which is tied to the world's generator.

## Plugin windows

`plugin.window(key, spec)` registers or updates a window. Returns `plugin_id:key`. `plugin.open(id)` opens it as a separate real game window. The window is also available via a button in the plugin card.

```python
plugin.window("control", {
    "title": "Sensor control",
    "text": "Building and maintaining the network",
    "fields": [{"id": "count", "label": "Count", "type": "number", "value": 3}],
    "rows": [["Status", "Ready"]],
    "buttons": [{"id": "generate", "label": "Create"}]
})
```

fields supports text, number, checkbox. rows is a list of table rows. A button press is passed to on_ui: `event["window"]`, `event["action"]`, `event["values"]`. After the handler the window re-reads its description; an update is done with a new plugin.window with the same key. The game's standard look is used, all labels are inserted as text. Arbitrary JS with access to Tauri is not given to the plugin; the logic stays in Adder.

## Multiplayer

`plugin.network()` returns the connection state: role and snap (mode, phase, seed, targets, peers and other fields of the standard multiplayer game). Before connecting, the data may be null. `plugin.net_send(channel, data)` sends up to 16 KB of data through the existing game session. The network permission is required. Reception goes to on_network only for an installed, enabled plugin with the same id and the network permission. from is set by the game transport.

The channel lets you write your own co-op events and data exchange. Package sources, the whole world and user directories are not transferred automatically. Participants must have compatible plugin versions; the rules of shared game logic are implemented by the plugin itself. Each player's income and content creation stay in their local world, as in the standard multiplayer game.

## Checking without windows

`plugin-check.exe` uses the real Blackwire core and creates a separate test world with seed=42 and the player sdk_test. It does not read or change the player's saves.

```text
plugin-check.exe sdk\examples --command district_create --command lab_create --ticks 65
plugin-check.exe plugins-folder --command "command arguments" --ticks 10
```

You can pass the folder of a single package with plugin.json or a parent directory, each subfolder of which is treated as a package. Options come after the path; --command can be repeated, --ticks allows up to 3600 steps. --dump-world path.json writes the test world; --dump-shell path.md writes the help of the engine's real commands. A manifest, handler or command error gives a non-zero exit code.

For development from source:

```text
cargo run -p blackwire-core --bin plugin-check -- sdk/examples --command district_create --ticks 65
cargo test -p blackwire-core plugins::tests
cargo test --workspace --jobs 2
```

Adder-check remains the check for ordinary language scripts. For plugins always use plugin-check: it calls the real handlers, checks the API and sees the real world. A syntax check alone does not prove that a plugin works.

## Errors and API limits

An error stops this plugin; the engine keeps running. Changes to the world, sessions, the UI queue, scheduled scripts and the outgoing network buffer made by the current handler are rolled back. Content created successfully earlier stays. Enabling again or "Reload" repeats on_load.

A handler gets 50000 interpreter steps; the stop cannot be caught with try/except. In addition, time is checked at about 500 ms between periodic checks. These are cooperative limits, not a separate process with a hard memory quota; split large operations into timers and use trusted packages. input does not block the game — it raises an error. sleep interrupts the handler at the next check; use a timer for delays.

Size of entry and of one asset — up to 256 KB; combined source up to 1 MB, text assets up to 2 MB; up to 128 packages and 4096 devices. Command output is limited to 100 lines of 4096 characters.

In API 1 registered plugin commands are run as a standalone command, without pipes and redirection. Inside a plugin shell.exec works with the ordinary capabilities of the built-in shell. A nested synchronous call of another plugin's command is not supported. To share content, use the world, dependencies and events.

## End-to-end examples

`device_lab`: a new camera type, the lab_control service, a type-only vulnerability, local privilege escalation and a vulnerability of a specific instance; commands lab_create and lab_info.

`district`: generates a gateway and an internal archive, an organisation, a subnet, a site, files, accounts, a mail and a mission; adds an upgrade to the shop, a script to the market and an achievement; creates its own window and recurring news; a vulnerability handler writes consequences into the virtual world. Commands district_create, district_open, district_status.

The sources of both packages are included in the SDK and the manual. They show the real formats of the current API.

## Lightweight reports

For status panels use `plugin.world("overview")`. It returns the sections player, orgs, finance, investigation and a list of machines with the fields id, hostname, ip, lan_ip, router, kind, os, up, ram_mb, disk_mb, services, custom_vulns, the last four logs entries and mail headers (from, subject, read). The plugins section contains quests, catalogs and device_types with kind and os only.

This mode does not copy virtual file systems, user databases or full device templates. For one full machine use `plugin.machine(id)`. Calling `plugin.world()` without arguments still returns the full state for editing. An unknown mode raises an error.

The `event["vuln"]` field of the on_hack event is the description object of the vulnerability found. To check its name, compare `event["vuln"]["name"]` with the full name of the form `plugin_id:rule_id`.

## Exploit results and conditions

`research address id` and `net.research(address, id)` show `result` and `conditions`.
Each condition contains `requirement`, `ok`, `observed`, `hint`. A correct payload
does not replace the conditions: `net.probe` and applying the exploit check them again.
A purchased tool obeys the same rules. Conditions are combined with AND.

In `plugin.vulnerability` you can set additional fields:

```python
plugin.vulnerability("report", {
    "device_type": device,
    "service": "http", "version_prefix": "1.", "access": "info",
    "exploit": {"kind": "traversal"},
    "result": {"type": "partial_file", "path": "/tmp/report.txt", "methods": ["read", "stat"]},
    "conditions": [
        {"type": "active_admin"},
        {"type": "connections_min", "count": 2},
        {"type": "plugin_state", "key": "door_open", "equals": true}
    ]
})
```

If `result` is absent, the older `access` field applies. A new `result` takes priority.
The `access` field remains required for compatibility; a local exploit still
escalates to root, and its explicit result can only be `shell`/`root`.

| result.type | Result and allowed methods |
| --- | --- |
| shell | A real terminal login; privilege: guest, user or root. An optional user selects an existing account. |
| creds | A found login and password; an optional user selects the account. |
| read | A one-time file leak; path is required. |
| info | The service banner. |
| computer | Object: hostname, services, users, processes, read, write, list. |
| router | Object: hostname, services, routes, connections. |
| file | Object of a single file: read, write, stat; path is required. |
| partial_shell | Only explicitly listed methods from whoami, hostname, read, list. File rights are those of the chosen user. |
| partial_computer | An explicit subset of computer methods. |
| partial_router | An explicit subset of router methods. |
| partial_file | An explicit subset of file methods; path is required. |

For partial objects `methods` is required and cannot be empty. For full objects
it is optional and can narrow access. The path must be absolute and normalised,
without `..`. computer/file objects represent data access obtained by the exploit
and act as root unless the author explicitly specifies an existing `user`.
A partial shell uses an ordinary user by default, or guest if there is none.
No object adds a connection in the terminal or a host to the list of captured machines.
A router object gives a view of routing, but not implicit control of the whole network.

For an object, `net.exploit` returns `{ok:true, access, handle, methods, host, expires_in:300}`.
Further actions are performed **in the same terminal session**:

```python
r = net.exploit(address, selector, payload)
if r["ok"] and "handle" in r:
    print(net.access(r["handle"], "read"))  # file/partial_file: only the granted file
    # net.access(r["handle"], "write", "new text")
    # computer: net.access(handle, "read", "/tmp/report.txt")
    # computer: net.access(handle, "write", "/tmp/report.txt", "new text")
    # partial_shell: net.access(handle, "list", "/home/operator")
```

Other methods take no arguments. `hostname` and `whoami` return a string;
`services`, `users`, `processes`, `routes`, `list` return lists; `connections` returns a number;
`stat` returns file details; `write` returns null. Calling `exec` is not supported:
only a shell result grants a full terminal. The object expires after 300 game
seconds, closing the terminal window revokes it. Save/load does not keep
access objects: after loading, repeat the hack. Stopping/updating the service,
the host becoming unreachable, the link being blocked or the vulnerability result changing
interrupt use of the object. Conditions are checked when access is obtained;
an object already granted is not revoked just because the required user logged out.
Actions leave traces and raise heat on monitored hosts.

| conditions.type | Parameters and meaning |
| --- | --- |
| active_user | Optional user: the given user is active, otherwise any ordinary user. |
| active_admin | root or a user with administrator rights is active. init and daemons do not count as a login. |
| users_min / users_max | count: bound on the number of registered accounts, including root. |
| connections_min / connections_max | count: bound on the number of incoming connections. |
| forwarded_ports_min | count: minimum of forwarded ports; an open service by itself is not a forward. |
| service_running | service: the given service is running. |
| os_prefix | prefix: the OS starts with this string. |
| file_exists | path: a file or directory exists on the host. |
| time_window | start, end: hours 0..23 on the game clock; start is inclusive, end is exclusive. 23→7 crosses midnight; equal bounds give an empty window. |
| same_lan | The current host and the target are on the same local network, or one is the gateway of the other. |
| plugin_state | key, equals: the value of plugin.get of the vulnerability's owner equals equals. |

At most 16 conditions can be set; count is an integer 0..4096. The absence of `conditions`
adds no restrictions to a custom vulnerability. Unknown types and fields
are rejected at registration. An ordinary player cannot pass "fulfilled conditions"
in a payload: the engine checks the world state on its own.

For scenarios with NPCs and custom network hardware, a plugin with the
`content` permission sets simulated activity:

```python
plugin.host_activity(mid, {
    "users": ["root", "operator"],
    "connections": 2,
    "forwarded_ports": [7002]
})
state = plugin.host_activity(mid)
plugin.set("door_open", true)
```

This fully replaces the scenario activity on that host. The names must exist;
connections — 0..4096, lists up to 128 elements, ports 1..65535. The values are saved.
Real player sessions are counted in addition; closing a session decreases
the number of connections. Logins of living-world NPCs are counted too and expire after
10 game minutes. System processes do not make an administrator permanently active.

Working example: `sdk/examples/access_lab` (commands `access_create`, `access_ready`,
`access_reset`). It demonstrates 10 access variants, hints, buying
`exp_protocol` and changing conditions. The `custom_vuln_type` example shows a separate
capability: a fully author-defined payload check type in Adder.

## Extended payload models

The engine supports 28 built-in models: overflow, traversal, auth_replay, query, format, protocol, race, command_chain, integer_overflow, signedness, length_confusion, use_after_free, double_free, type_confusion, object_reference, mass_assignment, token_confusion, request_smuggling, cache_poisoning, header_injection, unsafe_deserialization, prototype_pollution, template_injection, xml_entity, regex_backtracking, fragment_overlap, weak_rng, permission_mask. Each has an `exp_<kind>.adr` and a purchasable tool. These are limited game models: the handlers do not run real OS commands or arbitrary templates.

### A fully custom type in Adder

`kind = plugin.exploit_type("my_lock", {"name":"My Lock", "profile":"describe", "validate":"check"})` registers the type `<plugin.id>:my_lock`. Then pass it to `plugin.vulnerability(..., {"access":"shell", "service":"my_service", "exploit":{"kind":kind,"params":{"code":42}}})`.

`describe(event)` returns `{layout:dict, fields:list_of_names, hints:list_of_hints}`. `check(event)` returns a bool or `{ok:bool, feedback:string}`. The event contains seed, device, vulnerability, params, profile; the check also receives payload. Pure computation and reading `plugin.get`, `plugin.asset`, `plugin.lang` are allowed; network calls, world changes and input are forbidden. The limit is 20000 steps and 150 ms. At most 64 types per plugin. If the plugin is disabled or missing, the type is unavailable; the source is not stored in the world. The market field of the type definition can point to an item with a ready source. Working profile/validate, selling and remote/local use are in `examples/custom_vuln_type`.
