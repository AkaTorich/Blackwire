# Farm Irrigation Controllers

Zone 6 waters continuously despite the published irrigation schedule.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `farm_irrigation_controllers_open` or press **Start**.
3. In the terminal run `farm_irrigation_controllers_start`.
4. Read the mail “Farm Irrigation Controllers: first lead” in the game mail.
5. Open `http://198.22.19.1/` in the game browser and run `probe 198.22.19.1`.
6. Audit the controller and recover its operational record.

## Hints

Hack mechanic: weak_rng. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `irrigation_bus`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `farm_irrigation_controllers_status` shows progress without resetting anything.
- Reward: 132 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.19.1 farm_irrigation_controllers:records_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.19.1 farm_irrigation_controllers:records_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_weak_rng`.

`exploit -f ~/scripts/exp_weak_rng.adr -s farm_irrigation_controllers:records_entry 198.22.19.1`

Read `/srv/records/zones.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: devices. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
