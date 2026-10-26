# Community Internet Exchange

A hidden peer uses an obsolete exchange agreement.

## Start here

1. Install this folder through the game's Plugins folder and choose Reload.
2. Open its guide window with `community_internet_exchange_open`, then press Start; or run `community_internet_exchange_start` in the game terminal.
3. Read the new mail titled **Community Internet Exchange: first lead**.
4. Visit `http://198.22.36.1/` in the game's browser and inspect the first host with `probe 198.22.36.1`.
5. Traverse the gateway and recover the internal archive.

## Hints

- If probe reports an active firewall, run "firewall <address>" before entry. A failed bypass can be retried; watch your heat.

- Look for the custom exchange_archive_gate service on the first host.
- The access objective requires the entry vulnerability, not only an information leak.
- Private addresses are reachable through the gateway; its web page names the next destination.
- Use `community_internet_exchange_status` for progress without resetting anything.
- Reward: 240 credits, paid once. Repeating Start does not create another copy or reset completion.

## Optional full solution

`exploit -s community_internet_exchange:gateway_entry 198.22.36.1`

`exploit -s community_internet_exchange:operations_entry 10.230.36.2`

`exploit -s community_internet_exchange:records_entry 10.230.36.3`

Read `/srv/records/peering-register.csv` on the final host for the preserved record.

## Installation and persistence

Copy this plugin folder into the directory opened by **Plugins → Open folder**, then Reload. It is independent of the other 99 plugins. Created game content remains in the save after disabling a plugin. Names, commands and addresses are unique within this pack.
