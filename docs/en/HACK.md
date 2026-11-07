# Blackwire — game exploit mechanics

All mechanics operate on fictional World machines and Session sessions. The network, addresses, files and payloads belong to the game's model. Adder does not touch sockets or processes of the user's operating system.

## The player's path

1. `probe address` shows running services, versions, vulnerability ids and the mechanic.
2. `research address id` reveals the parameters of the specific target and the first hint; `research -h 3 address id` gives detailed hints.
3. `exploit new ~/scripts/mine.adr address id` creates a template and opens the game editor.
4. The player writes an algorithm that builds the payload dictionary from the researched data.
5. `net.probe(address, id, payload)` checks the payload without granting access; feedback explains the error.
6. `net.exploit(address, id, payload)` applies a matching remote payload. For a local one, `net.privesc(payload, id)` is used.

Running the source: `exploit -f ~/scripts/mine.adr -s id address` or `adder ~/scripts/mine.adr address id`. Local: `privesc ~/scripts/mine.adr id`, after obtaining a shell on the target. The exploit/privesc commands without a file and net.exploit/net.privesc without a payload grant no access.

## Research and hints

net.vulns returns id, kind, service, port, version, name, access, local. Local weak spots cannot be researched from another machine. research picks an exact id, a full name, or a service if it is unambiguous. With several matches, it asks you to be more specific.

The profile contains fingerprint for the device and version, layout with the target's data, fields with the payload fields, hints with hints, market with the id of the compatible tool. fingerprint is required in the payload. The profile is deterministic by seed, device and vulnerable version: saving/loading does not change the task, while another machine or version requires new research.

The game offers help research, help exploit, the help window, a template with TODOs in the editor and "Hints" buttons for plugins. Research never returns the correct finished payload: it is built by the player's code or by a purchased Adder tool.

## Vulnerability models

| kind | Research parameters | What the exploit does |
| --- | --- | --- |
| overflow | frame_words, word_size, code_base, entry_offset, canary | Compute the padding length and the address; build buffer from letters A and the suffix `address and canary separated by vertical bars` |
| traversal | mounts, target_file | Climb out of all nested directories with ../ and append the file path without a leading / |
| auth_replay | account, challenge, salt, modulus | Compute the weighted sum of the characters of the string challenge:salt modulo modulus; send user and response |
| query | table, columns | Compose an in-game FIND query with the right table, fields and filter bypass |
| format | stack_depth, argument_base, region_base, record_offset | Compute the argument position and the write address; assemble a positional format |
| protocol | nonce, capabilities | Assemble the HELLO, MODE and OPEN frames in the right order |
| race | resource, window_ms | Compose a check → replace → use schedule inside the game's race window |
| command_chain | command, parser, channel, nonce | Choose the parser's separator and compose a chain with the in-game bw-open |

Detailed format rules are returned by research -h 3; for the author's hint use -h 4. The payload is limited to 64 KiB. The race checks virtual timestamps; no real waiting is needed.

Every built-in vulnerability has a model. Custom device types and vulnerabilities get one from their service/access, or explicitly via exploit.kind in the SDK.

## Buying and modifying sources

The market sells 28 compatible exp_<kind> tools; the full list of mechanics is given below. Research shows the matching id.

`market buy exp_overflow` deducts in-game money and installs `~/scripts/exp_overflow.adr`. The source is open in the editor. If the mechanic doesn't match, the tool names the required item and stops the attempt. It researches the current version itself and builds the payload; defences and limits apply exactly as for your own code.

Purchased personal scripts are reachable from the SSH chain via ~/scripts/. An existing file on the remote machine takes priority and is read with the current user's permissions. For personal tools, only the player's own computer's scripts folder is used. Existing files and modified purchases are not overwritten. A deleted purchased copy of a built-in item can be restored by buying it again at no charge.

Plugins can sell their own source via plugin.catalog("market", ...). Its id can be linked to a vulnerability via exploit.market. A working example: sdk/examples/device_lab.

## Consequences and limits

Research takes into account the route, service availability and the firewall. Probes and rejected attempts are written to the game log, raise heat on monitored targets and may give away a honeypot. A working payload applies only the vulnerability's access: creds, shell, root, read or info.

Shell adds a hop to the current session; found logins are saved in found_creds; read returns only the result of the specific vulnerability. A wrong payload does not change the owner, permissions or the connection chain. A successful local or remote hack fires on_hack for SDK handlers; a payload check fires no such event.

A raid blocks remote attacks and probes but leaves local recovery. A local payload requires a current session on the machine. A stopped/updated service, a wrong version, a foreign fingerprint and a wrong id are rejected.

SSH with a known password and crack with the game wordlist remain separate authentication mechanics. clearlogs/wipe clears traces subject to permissions, proxy shows the chain and the source, heat/trace show the risk of investigation.

## Engine internals

- hack/vuln.rs: detection of built-in and author vulnerabilities.
- hack/payload.rs: stable ids, target selection, the research profile, 28 validators and hints; author types use a check in Adder.
- script_host.rs: net.research, net.probe, net.exploit and net.privesc; checks the payload before changing permissions.
- hack/exploit.rs and hack/privesc.rs: applying the chosen effect. Internal Rust functions without a payload exist for modelling and old engine tests; the game interface does not expose them directly.
- shell/cmds_hack.rs: research, source creation and queueing the Adder script into the normal execution queue.
- market.rs, hack/exploits/*.adr: purchasing and the real source code of the eight tools.
- plugins/content.rs: exploit.kind/difficulty/hint/market rules; plugins/catalog.rs: selling author tools.

No new mandatory fields are required in the save. Definitions of author profiles are stored in the existing plugin vulnerability dictionary. API 1 packages keep declaring content the same way as before; scripts that used the automatic net.exploit without a payload must be moved to the new API.

## Testing

player_exploits.rs runs the real purchased Adder files through the Game queue and checks all built-in vulnerabilities, every model, local privilege escalation, rejection of the old automatic call, faulty payloads, hints, saving/loading and protection of modified sources.

The SDK tests and community_plugins.rs use the same Adder exploits to complete missions. One hundred plugins are checked one after another, each in its own world. A combined load run is not performed on the user's computer.

## Hack results and conditions

`research address id` and `net.research(address, id)` show `result` and `conditions`.
Each condition contains `requirement`, `ok`, `observed`, `hint`. A correct payload
does not replace the conditions: `net.probe` and applying the exploit check them again.
A purchased tool follows the same rules. Conditions are combined with AND.

Extra fields can be set in `plugin.vulnerability`:

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

If `result` is absent, the old `access` field applies. A new `result` takes priority.
The `access` field remains required for compatibility; a local exploit still
escalates to root, and its explicit result can only be `shell`/`root`.

| result.type | Result and allowed methods |
| --- | --- |
| shell | A real terminal login; privilege: guest, user or root. An optional user picks an existing account. |
| creds | A found login and password; an optional user picks the account. |
| read | A one-time file leak; path is required. |
| info | The service banner. |
| computer | Object: hostname, services, users, processes, read, write, list. |
| router | Object: hostname, services, routes, connections. |
| file | Object for a single file: read, write, stat; path is required. |
| partial_shell | Only explicitly listed methods from whoami, hostname, read, list. File permissions are those of the chosen user. |
| partial_computer | An explicit subset of computer methods. |
| partial_router | An explicit subset of router methods. |
| partial_file | An explicit subset of file methods; path is required. |

For partial objects `methods` is required and cannot be empty. For full objects
it is optional and can narrow access. The path must be absolute and normalised,
without `..`. The computer/file objects represent access to data obtained by the exploit
and act as root unless the author explicitly specified an existing `user`.
A partial shell uses a regular user by default, or guest if there is none.
No object adds a connection in the terminal or a host to the list of captured machines.
The router object gives a view of routing, but no implicit control over the whole network.

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
`services`, `users`, `processes`, `routes`, `list` return lists; `connections` a number;
`stat` file details; `write` null. Calling `exec` is not supported:
a full terminal is granted only by the shell result. The object expires after 300 game
seconds; closing the terminal window revokes it. Saving/loading does not keep
access objects: after loading, repeat the hack. A stopped/updated service,
an unreachable host, a blocked channel or a change in the vulnerability's result
interrupt the use of the object. Conditions are checked when access is obtained;
an object already granted is not revoked merely because the required user logged out.
Actions leave traces and raise heat on monitored hosts.

| conditions.type | Parameters and meaning |
| --- | --- |
| active_user | Optional user: the given user is active, otherwise any regular user. |
| active_admin | root or a user with administrator rights is active. init and daemons do not count as a login. |
| users_min / users_max | count: bound on the number of registered accounts, including root. |
| connections_min / connections_max | count: bound on the number of incoming connections. |
| forwarded_ports_min | count: minimum number of forwarded ports; an open service by itself is not forwarding. |
| service_running | service: the given service is running. |
| os_prefix | prefix: the OS starts with this string. |
| file_exists | path: the file or directory exists on the host. |
| time_window | start, end: hours 0..23 on the game clock; start inclusive, end exclusive. 23→7 crosses midnight; equal bounds give an empty window. |
| same_lan | The current host and the target are on the same local network, or one is the other's gateway. |
| plugin_state | key, equals: the value of plugin.get of the vulnerability's owner equals equals. |

At most 16 conditions can be set; count is an integer 0..4096. Absence of `conditions`
adds no restrictions to a custom vulnerability. Unknown types and fields
are rejected at registration. An ordinary player cannot pass "fulfilled conditions"
in the payload: the engine checks the world state on its own.

For scenarios with NPCs and their own network equipment, a plugin with the
`content` permission sets the simulated activity:

```python
plugin.host_activity(mid, {
    "users": ["root", "operator"],
    "connections": 2,
    "forwarded_ports": [7002]
})
state = plugin.host_activity(mid)
plugin.set("door_open", true)
```

This fully replaces the scenario activity on that host. Names must exist;
connections is 0..4096, lists up to 128 items, ports 1..65535. The values are saved.
The player's real sessions are counted on top; closing a session reduces
the number of connections. Logins by NPCs of the living world are counted too and expire after
10 game minutes. System processes do not keep an administrator permanently active.

Working example: `sdk/examples/access_lab` (commands `access_create`, `access_ready`,
`access_reset`). It demonstrates 10 access variants, hints, buying
`exp_protocol` and changing conditions. The `custom_vuln_type` example shows a separate
capability: a fully author-defined payload check type in Adder.

## Extended payload models

The engine supports 28 built-in models: overflow, traversal, auth_replay, query, format, protocol, race, command_chain, integer_overflow, signedness, length_confusion, use_after_free, double_free, type_confusion, object_reference, mass_assignment, token_confusion, request_smuggling, cache_poisoning, header_injection, unsafe_deserialization, prototype_pollution, template_injection, xml_entity, regex_backtracking, fragment_overlap, weak_rng, permission_mask. Each has an `exp_<kind>.adr` and a purchasable tool. These are limited game models: the handlers do not execute real OS commands or arbitrary templates.

### A fully author-defined type in Adder

`kind = plugin.exploit_type("my_lock", {"name":"My Lock", "profile":"describe", "validate":"check"})` registers the type `<plugin.id>:my_lock`. Then pass it to `plugin.vulnerability(..., {"access":"shell", "service":"my_service", "exploit":{"kind":kind,"params":{"code":42}}})`.

`describe(event)` returns `{layout:dictionary, fields:list_of_names, hints:list_of_hints}`. `check(event)` returns a bool or `{ok:bool, feedback:string}`. The event contains seed, device, vulnerability, params, profile; the check also receives payload. Pure computation and reading `plugin.get`, `plugin.asset` are allowed; network calls, changing the world and input are forbidden. The limit is 20000 steps and 150 ms. At most 64 types per plugin. If the plugin is disabled or missing, the type is unavailable; the source code is not saved in the world. The market field of a type definition can point to an item with a ready-made source. Working profile/validate, selling and remote/local invocation are in `examples/custom_vuln_type`.
