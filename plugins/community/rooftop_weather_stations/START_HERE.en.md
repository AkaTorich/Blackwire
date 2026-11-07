# Rooftop Weather Stations

Wind readings stay identical while independent stations report a storm.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `rooftop_weather_stations_open` or press **Start**.
3. In the terminal run `rooftop_weather_stations_start`.
4. Read the mail “Rooftop Weather Stations: first lead” in the game mail.
5. Open `http://198.22.5.1/` in the game browser and run `probe 198.22.5.1`.
6. Audit the controller and recover its operational record.

## Hints

Hack mechanic: double_free. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `weather_feed`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `rooftop_weather_stations_status` shows progress without resetting anything.
- Reward: 76 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.5.1 rooftop_weather_stations:records_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.5.1 rooftop_weather_stations:records_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_double_free`.

`exploit -f ~/scripts/exp_double_free.adr -s rooftop_weather_stations:records_entry 198.22.5.1`

Read `/srv/records/readings.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: devices. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
