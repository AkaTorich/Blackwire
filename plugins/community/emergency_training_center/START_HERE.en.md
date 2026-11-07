# Emergency Training Center

A simulation alarm was copied into the official incident register.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `emergency_training_center_open` or press **Start**.
3. In the terminal run `emergency_training_center_start`.
4. Read the mail “Emergency Training Center: first lead” in the game mail.
5. Open `http://198.22.34.1/` in the game browser and run `probe 198.22.34.1`.
6. Traverse the gateway and recover the internal archive.

## Hints

Hack mechanic: type_confusion. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `training_archive`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `emergency_training_center_status` shows progress without resetting anything.
- Reward: 228 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.34.1 emergency_training_center:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.34.1 emergency_training_center:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_type_confusion`.

`exploit -f ~/scripts/exp_type_confusion.adr -s emergency_training_center:gateway_entry 198.22.34.1`

Read `/srv/records/drill-record.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: networks. Organisation: hospital. The plugin folder never touches other saves; created devices and missions are saved with the world.
