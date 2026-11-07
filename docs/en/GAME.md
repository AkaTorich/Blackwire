# Blackwire — player guide

## What this game is

Blackwire simulates a single computer and a fictional network of organisations. The player explores hosts, reads files and mail, writes programs in Adder, completes missions and deals with the consequences of their actions. The network contains corporations, banks, hospitals, providers, security centers, home computers and the hidden hello-world network.

Every app is a real, separate Windows window: it can be minimised, maximised and moved to another monitor. All windows look at the same world state. The terminal, the map, the browser and mail are not separate, independent games.

## Starting out and saves

In a new game, create an identity and, if you like, a password. Load an existing game from the login screen. The main save lives in `%APPDATA%\top.blackwire.game\save.json`. Named slots hold separate snapshots. Make a slot before experimenting with a plugin: the content it creates is saved inside the world.

The world is built from a seed; the same seed produces the same starting network. Game time runs while the app is open. The game autosaves once a minute. Quitting through the game saves it; force-closing from Windows is no substitute.

For a separate test profile, a developer can launch the game with `--data-dir absolute-folder`. The saves, slots, scenarios and plugins of that run then live in the given folder. This option is handy for testing the SDK without touching your own playthrough.

## Desktop and windows

The dock opens the terminal, editor, files window, browser, mail, system monitor, network map, scenario editor, multiplayer, plugins, settings and help. Some apps allow several windows: for example, terminals on different connections and editors for different files.

Minimise and restore windows with the usual Windows controls. Settings let you choose the monitor and arrange windows in a grid, in columns, in rows or as a cascade. Font size and transparency help fit the interface to your screen. The look includes 12 themes and separately adjustable effects.

The "Quit" button on the right of the top bar saves your progress and closes the game at once. The `shutdown` command on your own computer also saves the game and quits; in an SSH session it powers off only the remote host. `reboot` restarts the game system and leaves the game open. The red counter on the mail icon shows only unread Inbox messages and disappears once all mail is read.

Window hotkeys: F11 — full screen; Escape — leave it; Ctrl+W — close the window; Ctrl+Shift+T — terminal; Ctrl+Shift+E — editor; Ctrl+Shift+F — files. For the behaviour of specific editor shortcuts, see the app's own interface.

The system monitor shows the memory used by open game apps and scripts. A new window increases RAM use, closing one frees memory; minimising does not free memory. The "Kill" button ends the app's process and closes its window. When idle, usage may stay constant. When a save is loaded, only windows that are actually open are counted.

## First steps

1. Read the hello-world message in your Inbox. It contains the invitation address for the hidden network.
2. Open the address in the game browser. `.hw` pages belong to the game world.
3. Look through your files and the ready-made scripts in your home folder.
4. Open the terminal and type `help`. For a single command there are `command --help` and `man command`.
5. Scout a known host with ping, scan and probe. Take addresses from the map, mail, `/etc/hosts` and organisations' pages.
6. Complete the mission conditions on the job board at gate.hw/board; checks depend on the actual state of the world.

Not every machine is reachable directly: internal addresses need a route through a gateway. Explore the network step by step, read error messages and files with hints.

## Terminal and sessions

Each terminal has a current machine, user, folder and connection chain. Connecting with ssh changes the current machine; exit returns to the previous one, and on your own machine it closes the terminal.

Examples:

```text
pwd
ls -la
cat readme.txt
cd scripts
adder hello.adr
help ssh
```

The shell supports quotes, environment variables, aliases, file name wildcards, pipelines and redirections. For example, `cat file | grep string` picks out the lines you need, `echo text > file` writes text. Tab helps complete names. History is kept for the current session.

Ordinary commands are described in a separate section, "Terminal commands": it is generated directly by the engine. API 1 plugin commands run separately, without pipelines or redirection.

## Files and permissions

Every machine has its own directory tree. A user's home folder is `/home/name`, root's is `/root`; configuration usually lives in `/etc`, logs in `/var/log`, temporary data in `/tmp`. Files have an owner and read, write and execute permissions.

The files window and the editor work with the current game machine of their session. Missing permissions are an in-game limitation of the account. Administrators and root have extended rights. The chmod, chown, su and sudo commands let you manage permissions where the game provides for it.

Reading, searching and cross-referencing files is often more important than repeating an attack: that is where addresses, logins, notes, keys and information needed for missions are kept.

## Map, addresses and organisations

The map shows hosts and the links between them. Machines known to the player are highlighted; after reconnaissance, more details become available. An organisation has a domain, a gateway, a subnet and a list of devices.

A public address is reachable from outside. An internal address may be visible only from the organisation's subnet or through a suitable connection chain. Machines power on and off, services restart, the network changes over time. A host on the map is not necessarily open for login.

Plugins may add extra device types. Their own type is shown in the host card and in neofetch; routes, services and accounts keep working by the common rules.

## Reconnaissance

ping checks whether a host replies, scan/nmap shows ports and services, probe links services to known weak spots and extra details. nslookup/dig resolves domain names; whois shows who owns an address; traceroute helps you understand the path.

The service version matters: a vulnerability works against a matching version, and a stopped service does not accept an attack. A plugin may add its own services and vulnerability rules for a whole device type or for a single instance.

## Access and privilege escalation

Exploiting a vulnerability requires your own or a purchased Adder exploit. The `exploit address` command does not grant access by itself.

1. Find the service: `probe address`. Pick the id of a weak spot; local vulnerabilities become visible after you log in to the machine.
2. Research it: `research address id`. You will see the version, the mechanic and the target's parameters. `research -h 3 address id` reveals extra hints.
3. Create the source: `exploit new ~/scripts/my_exploit.adr address id`. The game opens a template in the editor.
4. Fill in the payload following the hints: a buffer, a path, a query, an auth response or a sequence of operations — depending on the mechanic. `net.probe` checks it and explains a rejection.
5. Run it: `exploit -f ~/scripts/my_exploit.adr -s id address`. Only a matching payload produces the vulnerability's result: information, a file, a login, a shell or root.

You can buy a ready-made source: `market` shows the catalogue, `market buy exp_overflow` buys an overflow tool. Research suggests the compatible item. The purchased `.adr` lands in `~/scripts`, is reachable from the SSH chain and is open for editing. It does not cancel defences, routes or version limits. Buying again does not overwrite a modified file.

crack uses the game's password wordlist; ssh connects with a known account. scp moves files between game machines.

A local exploit requires a shell you already have. Run `probe current_host` and `research current_host id`, then `privesc ~/scripts/my_local_exploit.adr id`. A ready-made tool: `market buy exp_race`. The rootkit from the shop adds an editable `rootkit.adr` and lets you research the local entry point even on a fresh kernel.

A wrong payload gives no access. Checks and failed attempts leave traces and may raise heat. Captured logins are visible via creds; obtaining the same login again does not create endless income from selling it.

Everything described here happens only against the model of the fictional Blackwire network.

## Firewalls, honeypots and traces

Some organisations protect hosts with firewalls. A protected host must first be handled with the firewall mechanic the game provides. This can make noise. Honeypots raise the risk of detection.

heat shows your heat. Actions against monitored machines leave traces that security centers and the cyber police read. proxy shows the current chain and the exit address; intermediate machines let you hide your real source.

clearlogs/wipe clears matching log entries if you have the permissions. Having access does not by itself mean there is no evidence. Check heat, trace and messages in your mail.

## Investigation and retaliation

Surveillance goes through stages from quiet and noticed activity to a case, a trace and a raid. Progress depends on traces and heat; without evidence a case can die down. Mail arrives at each transition.

A raid cuts connection chains and temporarily blocks offensive actions. Later, an investigation may lead to retaliation against the player's machine: implants and fines. Your own firewall is switched on with shield; implants shows implants; clean removes them within the game mechanic.

## Keyloggers, botnet and encryption

keylog installs a keylogger on a machine you have already captured and brings in credentials over time. botnet manages a network of captured devices and background income. These tools leave in-game traces and are tied to the world's defences.

encrypt/decrypt works with virtual files and an in-game key. Look for hints and keys in the contents of the world. This is a separate mechanic, not encryption of Windows files.

## Story and missions

The main hello-world chain begins with the invitation. The board shows the active goal, the briefing and the reward. The missions/jobs commands show the list, mission N shows the details. Conditions are checked against the world: a file delivered, access obtained, a mark left, traces removed.

Missions are bound to real machines of the generated world, so target addresses differ between playthroughs. After completion a reply arrives and the next mission opens.

Plugins can add independent missions with their own checks; they are shown together with regular missions but need not change the main chain.

## Gigs and scenarios

gigs shows small paid jobs. After you complete them the list is refilled. It is a source of activity and income after the story events.

The scenario editor lets you set a goal and a reward, save the scenario and run it. The built-in options are: access to a host, delivering a text with a required line, accumulating a sum. One editor scenario is active at a time. Plugins can implement extra conditions with their own missions and Adder code.

## Money, debt and the shop

wallet shows your balance, debt shows the loan and the rate. pay amount or pay all pays off the debt with the funds available. Interest accrues in game time. When it is fully repaid, a confirmation arrives.

shop shows upgrades, buy id purchases them. Built-in items include a wordlist, a map, hardware, a VPN, trace cleaning and a rootkit. Their effects are tied to other systems of the world. sell sells credentials you have found.

market is a separate catalogue of ready-made Adder scripts. market buy id pays for a script and puts it in ~/scripts. Run it with adder. Plugins can add items and scripts to both built-in catalogues.

## Your base and achievements

base shows rented relays. rent creates a relay with access; release gives it up. Rent requires funds up front and then over game time. If you run short of money, the relay may be seized. These are real machines of the model: connections pass through them and tools are kept on them.

achievements/ach shows achievements. They unlock from events and the state of the world and are saved. Plugins can register their own achievements.

## The ending and life after it

The main victory requires completing the hello-world chain and paying off the debt. The game shows the outcome of your path and a closing message. After that the world keeps running: you can explore the network, do gigs, write scripts, build content and use plugins.

## Adder and the editor

Adder uses indentation, variables, lists, dictionaries, functions, loops and error handling. Game functions live in sys, fs, net, shell. The built-in math, json, time and random are described in the language section. A regular script receives its arguments in sys.args.

```python
# hello.adr — details about your machine
print("Machine:", sys.hostname())
print("User:", sys.whoami())
print("Address:", net.my_ip())
```

Save the file in your home folder and run `adder path.adr`. Syntax checking is available in the editor and via `adder --check file`. An error points to the line. While running, a script is a game process; its output is visible in the terminal.

The language resembles Python, but it has its own interpreter and its own API. Python libraries, import os and access to the Windows file system do not work here. The SDK section describes the additional plugin module for extension authors.

## Multiplayer

The multiplayer window lets you create a match or join one by address. Modes: a duel over a goal and a co-op run through a set of goals. Participants build the starting network from a shared seed; the host coordinates progress. Before the start, a slot named "до-сети" is created so you can return to your previous playthrough.

Playing over a local network requires a reachable host address. Internet play depends on the host's network setup. The SDK does not change these requirements. Plugins can exchange messages through an existing match if participants have compatible packages installed.

## Plugins

The "Plugins" window shows a package's name, version, permissions, state and error. It has enable/disable, reload, commands, command arguments and buttons for the plugin's own windows.

The "Device Lab" example creates cameras with their own vulnerabilities. "Line-7 District" creates a linked network with missions, a website, a shop, a script, an achievement, a control window and background news. The buttons are enough to work with the examples; their sources show how to write your own extensions.

Detailed formats, events, functions, limits and the checking utility are described in the Blackwire SDK section.

## If something doesn't work

If a host doesn't respond: check the address, the power state, the route and the required port. If you lack permissions: check the current user and machine. If a mission isn't counted: reread the exact condition and make sure the action was performed on the specified target.

If a plugin shows an error: copy it together with the line number; check the manifest, the API version and the permissions. After fixing it, press "Reload". To test logic, use a separate profile or plugin-check so your main playthrough is not changed.

If a window is too cramped: enlarge it or adjust the font size. The full reference for the language and the commands is available both inside the game and in this CHM.

## Different hack results

`research address id` shows the result in advance: a guest/user/root login, data,
an account or an object with a limited set of actions. A correct exploit
may fail without an active user, the required number of connections or
a fulfilled scenario condition. `research` shows the current values and hints.
Buying a ready-made exploit does not cancel these conditions. Partial access to a file or
services does not capture the whole computer. The Access Lab tutorial plugin shows the difference.

## System windows and switching

Every game app, the background and the login window are available to the system window switcher and the taskbar or Dock. Game-style buttons minimise, maximise and close the corresponding window. The background's close button saves the game and exits it. While the game is active, the background stays above other programs and the game apps sit above it. When you switch to another program, the game gives it the foreground. Minimised windows are not restored by a mere change of focus. Button grouping and the switching method are determined by the system settings.

## Highlighting in the editor

Adder (.adr) and JSON files are highlighted on opening and while typing. Colours distinguish keywords, functions, game APIs, strings, numbers and comments; expressions inside f-strings are highlighted too. The button with the code icon in the editor toolbar turns highlighting on and off. Search, selection and saving work with the source text, without colour markup. The light theme uses separate high-contrast colours.
