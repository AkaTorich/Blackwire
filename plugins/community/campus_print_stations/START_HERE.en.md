# Campus Print Stations

A confidential exam remained in the shared print queue.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `campus_print_stations_open` or press **Start**.
3. In the terminal run `campus_print_stations_start`.
4. Read the mail “Campus Print Stations: first lead” in the game mail.
5. Open `http://198.22.18.1/` in the game browser and run `probe 198.22.18.1`.
6. Audit the controller and recover its operational record.

## Hints

Hack mechanic: fragment_overlap. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `print_spool`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `campus_print_stations_status` shows progress without resetting anything.
- Reward: 128 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.18.1 campus_print_stations:records_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.18.1 campus_print_stations:records_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_fragment_overlap`.

`exploit -f ~/scripts/exp_fragment_overlap.adr -s campus_print_stations:records_entry 198.22.18.1`

Read `/srv/records/queue.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: devices. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
