# Cold Chain Alarm

A simulated refrigerator alarm appears while its monitoring endpoint goes silent.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `cold_chain_alarm_open` or press **Start**.
3. In the terminal run `cold_chain_alarm_start`.
4. Read the mail “Cold Chain Alarm: first lead” in the game mail.
5. Open `http://198.22.96.1/` in the game browser and run `probe 198.22.96.1`.
6. Recover the incident record and follow the timed incident log.

## Hints

Hack mechanic: request_smuggling. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `cold_monitor`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `cold_chain_alarm_status` shows progress without resetting anything.
- Reward: 140 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.96.1 cold_chain_alarm:records_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.96.1 cold_chain_alarm:records_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_request_smuggling`.

`exploit -f ~/scripts/exp_request_smuggling.adr -s cold_chain_alarm:records_entry 198.22.96.1`

Read `/srv/records/cold-chain.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: events. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
