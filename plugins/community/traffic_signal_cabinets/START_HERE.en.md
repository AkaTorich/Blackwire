# Traffic Signal Cabinets

The maintenance schedule contains an unauthorized priority route.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `traffic_signal_cabinets_open` or press **Start**.
3. In the terminal run `traffic_signal_cabinets_start`.
4. Read the mail “Traffic Signal Cabinets: first lead” in the game mail.
5. Open `http://198.22.20.1/` in the game browser and run `probe 198.22.20.1`.
6. Audit the controller and recover its operational record.

## Hints

Hack mechanic: permission_mask. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `signal_control`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `traffic_signal_cabinets_status` shows progress without resetting anything.
- Reward: 136 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.20.1 traffic_signal_cabinets:records_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.20.1 traffic_signal_cabinets:records_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_permission_mask`.

`exploit -f ~/scripts/exp_permission_mask.adr -s traffic_signal_cabinets:records_entry 198.22.20.1`

Read `/srv/records/phases.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: devices. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
