# Expired Badge Incident

A temporary badge remains active after its shift expires.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `expired_badge_incident_open` or press **Start**.
3. In the terminal run `expired_badge_incident_start`.
4. Read the mail “Expired Badge Incident: first lead” in the game mail.
5. Open `http://198.22.93.1/` in the game browser and run `probe 198.22.93.1`.
6. Recover the incident record and follow the timed incident log.

## Hints

Hack mechanic: object_reference. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `badge_control`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `expired_badge_incident_status` shows progress without resetting anything.
- Reward: 116 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.93.1 expired_badge_incident:records_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.93.1 expired_badge_incident:records_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_object_reference`.

`exploit -f ~/scripts/exp_object_reference.adr -s expired_badge_incident:records_entry 198.22.93.1`

Read `/srv/records/badge-status.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: events. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
