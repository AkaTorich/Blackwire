# Public Records Office

The public permit portal omits an amendment stored on the internal records host.

## Start here

1. Install this folder through the game's Plugins folder and choose Reload.
2. Open its guide window with `public_records_office_open`, then press Start; or run `public_records_office_start` in the game terminal.
3. Read the new mail titled **Public Records Office: first lead**.
4. Visit `http://198.22.26.1/` in the game's browser and inspect the first host with `probe 198.22.26.1`.
5. Traverse the gateway and recover the internal archive.

## Hints

- If probe reports an active firewall, run "firewall <address>" before entry. A failed bypass can be retried; watch your heat.

- Look for the custom records_store_gate service on the first host.
- The access objective requires the entry vulnerability, not only an information leak.
- Private addresses are reachable through the gateway; its web page names the next destination.
- Use `public_records_office_status` for progress without resetting anything.
- Reward: 180 credits, paid once. Repeating Start does not create another copy or reset completion.

## Optional full solution

`exploit -s public_records_office:gateway_entry 198.22.26.1`

`exploit -s public_records_office:operations_entry 10.230.26.2`

`exploit -s public_records_office:records_entry 10.230.26.3`

Read `/srv/records/permit-register.csv` on the final host for the preserved record.

## Installation and persistence

Copy this plugin folder into the directory opened by **Plugins → Open folder**, then Reload. It is independent of the other 99 plugins. Created game content remains in the save after disabling a plugin. Names, commands and addresses are unique within this pack.
