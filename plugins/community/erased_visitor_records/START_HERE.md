# Erased Visitor Records

The front desk and access controller disagree about an after-hours visitor.

## Start here

1. Install this folder through the game's Plugins folder and choose Reload.
2. Open its guide window with `erased_visitor_records_open`, then press Start; or run `erased_visitor_records_start` in the game terminal.
3. Read the new mail titled **Erased Visitor Records: first lead**.
4. Visit `http://198.22.46.1/` in the game's browser and inspect the first host with `probe 198.22.46.1`.
5. Follow both evidence hosts and establish the custody trail.

## Hints

- If probe reports an active firewall, run "firewall <address>" before entry. A failed bypass can be retried; watch your heat.

- Look for the custom visitor_archive_gate service on the first host.
- The access objective requires the entry vulnerability, not only an information leak.
- Private addresses are reachable through the gateway; its web page names the next destination.
- Use `erased_visitor_records_status` for progress without resetting anything.
- Reward: 215 credits, paid once. Repeating Start does not create another copy or reset completion.

Both evidence hosts must be accessed to complete this investigation.

## Optional full solution

`exploit -s erased_visitor_records:gateway_entry 198.22.46.1`

`exploit -s erased_visitor_records:records_entry 10.230.46.2`

Read `/srv/records/visitor-records.csv` on the final host for the preserved record.

## Installation and persistence

Copy this plugin folder into the directory opened by **Plugins → Open folder**, then Reload. It is independent of the other 99 plugins. Created game content remains in the save after disabling a plugin. Names, commands and addresses are unique within this pack.
