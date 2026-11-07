# Unlisted Server Rack

Power readings reveal an occupied rack missing from the inventory.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `unlisted_server_rack_open` or press **Start**.
3. In the terminal run `unlisted_server_rack_start`.
4. Read the mail “Unlisted Server Rack: first lead” in the game mail.
5. Open `http://198.22.60.1/` in the game browser and run `probe 198.22.60.1`.
6. Follow both evidence hosts and establish the custody trail.

## Hints

Hack mechanic: use_after_free. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `rack_store`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `unlisted_server_rack_status` shows progress without resetting anything.
- Reward: 285 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.60.1 unlisted_server_rack:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.60.1 unlisted_server_rack:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_use_after_free`.

`exploit -f ~/scripts/exp_use_after_free.adr -s unlisted_server_rack:gateway_entry 198.22.60.1`

Read `/srv/records/capacity-report.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: investigations. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
