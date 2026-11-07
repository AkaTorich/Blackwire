# Marine Research Station

A survey vessel's route conflicts with the downloaded buoy readings.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `marine_research_station_open` or press **Start**.
3. In the terminal run `marine_research_station_start`.
4. Read the mail “Marine Research Station: first lead” in the game mail.
5. Open `http://198.22.39.1/` in the game browser and run `probe 198.22.39.1`.
6. Traverse the gateway and recover the internal archive.

## Hints

Hack mechanic: cache_poisoning. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `marine_archive`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `marine_research_station_status` shows progress without resetting anything.
- Reward: 258 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.39.1 marine_research_station:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.39.1 marine_research_station:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_cache_poisoning`.

`exploit -f ~/scripts/exp_cache_poisoning.adr -s marine_research_station:gateway_entry 198.22.39.1`

Read `/srv/records/buoy-survey.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: networks. Organisation: university. The plugin folder never touches other saves; created devices and missions are saved with the world.
