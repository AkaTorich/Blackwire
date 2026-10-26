# City Housing Agency

An apartment allocation was reassigned without a board decision.

## Start here

1. Install this folder through the game's Plugins folder and choose Reload.
2. Open its guide window with `city_housing_agency_open`, then press Start; or run `city_housing_agency_start` in the game terminal.
3. Read the new mail titled **City Housing Agency: first lead**.
4. Visit `http://198.22.33.1/` in the game's browser and inspect the first host with `probe 198.22.33.1`.
5. Traverse the gateway and recover the internal archive.

## Hints

- If probe reports an active firewall, run "firewall <address>" before entry. A failed bypass can be retried; watch your heat.

- Look for the custom housing_archive_gate service on the first host.
- The access objective requires the entry vulnerability, not only an information leak.
- Private addresses are reachable through the gateway; its web page names the next destination.
- Use `city_housing_agency_status` for progress without resetting anything.
- Reward: 222 credits, paid once. Repeating Start does not create another copy or reset completion.

## Optional full solution

`exploit -s city_housing_agency:gateway_entry 198.22.33.1`

`exploit -s city_housing_agency:operations_entry 10.230.33.2`

`exploit -s city_housing_agency:records_entry 10.230.33.3`

Read `/srv/records/allocation-ledger.csv` on the final host for the preserved record.

## Installation and persistence

Copy this plugin folder into the directory opened by **Plugins → Open folder**, then Reload. It is independent of the other 99 plugins. Created game content remains in the save after disabling a plugin. Names, commands and addresses are unique within this pack.
