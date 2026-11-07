# Blackwire terminal commands

The shell understands pipelines (`a | b`), redirections (`>`, `>>`, `2>`, `<`), separators (`;`, `&&`, `||`), variables (`$NAME`, `${NAME}`), the home directory (`~`) and `*`/`?` wildcards in file names. Every command has `--help`.

## Files and directories

| Command | Usage | Purpose |
|---|---|---|
| `ls` | `ls [-l] [-a] [-h] [path…]` | list files |
| `cd` | `cd [path]` | change directory (no argument — home, '-' — back) |
| `pwd` | `pwd` | current directory |
| `cat` | `cat [-n] [file…]` | print a file |
| `less` | `less file` | print a file (same as cat) |
| `more` | `more file` | print a file (same as cat) |
| `touch` | `touch file…` | create an empty file or update its time |
| `mkdir` | `mkdir [-p] dir…` | create a directory |
| `rm` | `rm [-r] [-f] path…` | remove files and directories |
| `rmdir` | `rmdir dir…` | remove an empty directory |
| `mv` | `mv src… dest` | move or rename |
| `cp` | `cp [-r] src… dest` | copy |
| `echo` | `echo [-n] [-e] text…` | print text |
| `printf` | `printf format [args…]` | print text (like echo -e) |
| `find` | `find [path] [-name pattern] [-type f|d]` | search for files |
| `du` | `du [-h] [-s] [path]` | directory sizes |
| `df` | `df [-h]` | disk space |
| `chmod` | `chmod mode file…` | change permissions (755, u+x, rwxr-xr-x) |
| `chown` | `chown user file…` | change owner (root) |
| `file` | `file path…` | identify file type |
| `stat` | `stat path…` | file details |
| `tree` | `tree [-a] [-L depth] [path]` | directory tree |
| `ln` | `ln [-s] target name` | create a copy under another name |

## Text and archives

| Command | Usage | Purpose |
|---|---|---|
| `head` | `head [-n N] [file…]` | first lines |
| `tail` | `tail [-n N] [file…]` | last lines |
| `grep` | `grep [-i] [-n] [-v] [-c] [-r] pattern [file…]` | search for lines |
| `wc` | `wc [-l] [-w] [-c] [file…]` | count lines, words, bytes |
| `sort` | `sort [-r] [-n] [-u] [file…]` | sort lines |
| `uniq` | `uniq [-c] [file]` | drop repeated adjacent lines |
| `cut` | `cut -d delim -f fields [file…]` | cut out fields |
| `tr` | `tr [-d] set1 [set2]` | translate characters |
| `rev` | `rev [file…]` | reverse lines |
| `tee` | `tee [-a] file` | print and write to a file |
| `diff` | `diff file1 file2` | differences between two files |
| `md5` | `md5 [-s string] [file…]` | simplified hash |
| `md5sum` | `md5sum [file…]` | simplified hash |
| `base64` | `base64 [-d] [file]` | encode/decode base64 |
| `hexdump` | `hexdump [-n bytes] file` | file bytes in hex |
| `xxd` | `xxd [-n bytes] file` | file bytes in hex |
| `tar` | `tar -c|-x|-t archive.tar [files…]` | pack/unpack an archive |
| `zip` | `zip archive.zip files…` | pack into an archive |
| `unzip` | `unzip archive.zip [dir]` | unpack an archive |

## System and users

| Command | Usage | Purpose |
|---|---|---|
| `whoami` | `whoami` | user name |
| `id` | `id` | user and permissions |
| `hostname` | `hostname` | machine name |
| `uname` | `uname [-a]` | system info |
| `date` | `date` | world date and time |
| `uptime` | `uptime` | uptime |
| `ps` | `ps [-a]` | processes (with -a — all users) |
| `top` | `top` | load and process snapshot |
| `kill` | `kill pid…` | kill a process |
| `free` | `free [-h]` | memory |
| `env` | `env` | environment variables |
| `export` | `export NAME=value` | set a variable |
| `set` | `set NAME=value` | set a variable |
| `unset` | `unset NAME…` | remove a variable |
| `alias` | `alias [name='command']` | command aliases |
| `unalias` | `unalias name…` | remove an alias |
| `history` | `history [-c]` | command history |
| `clear` | `clear` | clear the screen |
| `help` | `help [command] [-w]` | command table; -w — help window |
| `man` | `man command` | help for a command |
| `which` | `which command…` | where a command lives |
| `passwd` | `passwd [user] new_password` | change password |
| `useradd` | `useradd [-a] name [password]` | add a user (-a — administrator) |
| `userdel` | `userdel name` | delete a user |
| `users` | `users` | list users |
| `su` | `su [user] password` | switch user |
| `sudo` | `sudo command…` | run as root (administrators) |
| `shutdown` | `shutdown` | save and quit the game; over SSH — power off the remote machine |
| `reboot` | `reboot` | reboot the machine |
| `logs` | `logs [-n N] [-f auth|system]` | machine log |
| `exit` | `exit` | disconnect / close the terminal |
| `logout` | `logout` | same as exit |
| `true` | `true` | do nothing, exit code 0 |
| `false` | `false` | do nothing, exit code 1 |

## Network

| Command | Usage | Purpose |
|---|---|---|
| `ifconfig` | `ifconfig` | network interfaces |
| `ip` | `ip [addr|route]` | addresses and routes |
| `ping` | `ping address|name` | check connectivity |
| `nslookup` | `nslookup name` | resolve a name to an address |
| `dig` | `dig name` | resolve a name to an address |
| `scan` | `scan address|name` | machine ports and services |
| `nmap` | `nmap address|name` | machine ports and services |
| `ssh` | `ssh user@address password` | connect to a machine |
| `scp` | `scp [user@ip:]src [user@ip:]dest password` | copy a file between machines |
| `netstat` | `netstat` | connections and open ports |
| `whois` | `whois address|name` | who owns an address |
| `traceroute` | `traceroute address|name` | route to a machine |
| `wget` | `wget url [-O file]` | download a page to a file |
| `curl` | `curl url` | print a page |
| `arp` | `arp` | neighbours on the local network |
| `route` | `route` | routing table |

## Hacking

| Command | Usage | Purpose |
|---|---|---|
| `probe` | `probe address` | recon: services, weak spots, users |
| `research` | `research [-h 1..4] address [id]` | research a vulnerability and get hints for your own exploit |
| `exploit` | `exploit -f file.adr [-s id] address`; `exploit new file.adr address [id]` | create or run your own Adder exploit |
| `crack` | `crack [-w wordlist] user@address` | crack a password with a wordlist |
| `privesc` | `privesc file.adr [id]` | run your own local exploit |
| `clearlogs` | `clearlogs [pattern]` | wipe traces from logs (needs permissions) |
| `wipe` | `wipe [pattern]` | wipe traces from logs (same as clearlogs) |
| `proxy` | `proxy` | connection chain and exit address |
| `heat` | `heat` | heat level and evidence |
| `trace` | `trace` | surveillance status: security center investigations |
| `creds` | `creds` | found logins |
| `firewall` | `firewall address` | bypass a host's firewall |
| `keylog` | `keylog [plant address]` | keylogger: plant / view |
| `botnet` | `botnet [add|remove address]` | botnet of captured machines |
| `encrypt` | `encrypt file key` | encrypt a file |
| `decrypt` | `decrypt file key` | decrypt a file |

## Defense

| Command | Usage | Purpose |
|---|---|---|
| `shield` | `shield [on|off]` | your own firewall: repel counterattacks |
| `implants` | `implants` | find implants on your machine |
| `clean` | `clean [path|all]` | clean out implants |

## Base

| Command | Usage | Purpose |
|---|---|---|
| `base` | `base` | your base and rented relays |
| `rent` | `rent` | rent a proxy relay |
| `release` | `release address` | give up a relay |

## Missions

| Command | Usage | Purpose |
|---|---|---|
| `missions` | `missions` | hello-world missions and score |
| `jobs` | `jobs` | missions (same as missions) |
| `mission` | `mission [N]` | mission briefing (no number — current one) |
| `ending` | `ending` | road to freedom: progress and ending |
| `gigs` | `gigs` | gigs: open jobs and income |
| `scenario` | `scenario` | your scenario from the editor: goal and progress |

## Money

| Command | Usage | Purpose |
|---|---|---|
| `wallet` | `wallet` | balance, earnings and debt |
| `debt` | `debt` | loan: balance, rate, payments |
| `pay` | `pay amount|all` | pay off part of the debt |
| `shop` | `shop` | upgrade shop |
| `buy` | `buy id` | buy an upgrade |
| `sell` | `sell` | sell found logins for ₵ |
| `market` | `market [buy id]` | script market on the hw network |

## Adder scripts

| Command | Usage | Purpose |
|---|---|---|
| `adder` | `adder file.adr [args] | -c "code" | --check file` | run an Adder script |

## App windows

| Command | Usage | Purpose |
|---|---|---|
| `edit` | `edit [file]` | open the editor |
| `nano` | `nano [file]` | open the editor |
| `vim` | `vim [file]` | open the editor |
| `open` | `open app [arg]` | open an app window |
| `files` | `files [path]` | files window |
| `browser` | `browser [url]` | browser window |
| `map` | `map` | network map |
| `monitor` | `monitor` | system monitor |
| `settings` | `settings` | settings |
| `mail` | `mail [-l]` | mail (-l — list messages in the terminal) |

## Misc

| Command | Usage | Purpose |
|---|---|---|
| `banner` | `banner text` | big letters |
| `matrix` | `matrix [rows]` | code rain |
| `fortune` | `fortune` | random saying |
| `cal` | `cal [month year]` | calendar |
| `bc` | `bc expression` | calculator |
| `yes` | `yes [text]` | repeat text (20 lines) |
| `sleep` | `sleep seconds` | wait |
| `neofetch` | `neofetch` | machine card |
| `seq` | `seq [from] to` | numbers in order |
| `news` | `news` | city news feed |
| `achievements` | `achievements` | achievements: unlocked and not yet |
| `ach` | `ach` | achievements (same as achievements) |


Total commands: 150.
