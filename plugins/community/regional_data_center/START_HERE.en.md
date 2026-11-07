# Regional Data Center

Rack allocation logs show an unlisted tenant using reserved capacity.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `regional_data_center_open` or press **Start**.
3. In the terminal run `regional_data_center_start`.
4. Read the mail “Regional Data Center: first lead” in the game mail.
5. Open `http://198.22.27.1/` in the game browser and run `probe 198.22.27.1`.
6. Traverse the gateway and recover the internal archive.

## Hints

Hack mechanic: race. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `tenant_archive`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `regional_data_center_status` shows progress without resetting anything.
- Reward: 186 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.27.1 regional_data_center:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.27.1 regional_data_center:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_race`.

`exploit -f ~/scripts/exp_race.adr -s regional_data_center:gateway_entry 198.22.27.1`

Read `/srv/records/rack-allocation.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: networks. Organisation: hosting. The plugin folder never touches other saves; created devices and missions are saved with the world.
