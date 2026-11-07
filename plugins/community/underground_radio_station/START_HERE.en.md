# Underground Radio Station

A broadcast was replaced while the station controller was unattended.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `underground_radio_station_open` or press **Start**.
3. In the terminal run `underground_radio_station_start`.
4. Read the mail “Underground Radio Station: first lead” in the game mail.
5. Open `http://198.22.40.1/` in the game browser and run `probe 198.22.40.1`.
6. Traverse the gateway and recover the internal archive.

## Hints

Hack mechanic: header_injection. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `radio_archive`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `underground_radio_station_status` shows progress without resetting anything.
- Reward: 264 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.40.1 underground_radio_station:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.40.1 underground_radio_station:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_header_injection`.

`exploit -f ~/scripts/exp_header_injection.adr -s underground_radio_station:gateway_entry 198.22.40.1`

Read `/srv/records/broadcast-ledger.txt` on the last host — the preserved record is there.

## Installation and persistence

Family: networks. Organisation: underground group. The plugin folder never touches other saves; created devices and missions are saved with the world.
