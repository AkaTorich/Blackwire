# Railway Dispatch Network

A freight route changed after its dispatcher signed off.

## Start here

1. Install this folder through the game's Plugins folder and choose Reload.
2. Open its guide window with `railway_dispatch_network_open`, then press Start; or run `railway_dispatch_network_start` in the game terminal.
3. Read the new mail titled **Railway Dispatch Network: first lead**.
4. Visit `http://198.22.32.1/` in the game's browser and inspect the first host with `probe 198.22.32.1`.
5. Traverse the gateway and recover the internal archive.

## Hints

- If probe reports an active firewall, run "firewall <address>" before entry. A failed bypass can be retried; watch your heat.

- Look for the custom rail_archive_gate service on the first host.
- The access objective requires the entry vulnerability, not only an information leak.
- Private addresses are reachable through the gateway; its web page names the next destination.
- Use `railway_dispatch_network_status` for progress without resetting anything.
- Reward: 216 credits, paid once. Repeating Start does not create another copy or reset completion.

## Optional full solution

`exploit -s railway_dispatch_network:gateway_entry 198.22.32.1`

`exploit -s railway_dispatch_network:operations_entry 10.230.32.2`

`exploit -s railway_dispatch_network:records_entry 10.230.32.3`

Read `/srv/records/dispatch-orders.csv` on the final host for the preserved record.

## Installation and persistence

Copy this plugin folder into the directory opened by **Plugins → Open folder**, then Reload. It is independent of the other 99 plugins. Created game content remains in the save after disabling a plugin. Names, commands and addresses are unique within this pack.
