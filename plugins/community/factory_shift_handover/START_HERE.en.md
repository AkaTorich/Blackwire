# Factory Shift Handover

An unattended control desk receives the next shift's maintenance handover.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `factory_shift_handover_open` or press **Start**.
3. In the terminal run `factory_shift_handover_start`.
4. Read the mail “Factory Shift Handover: first lead” in the game mail.
5. Open `http://198.22.94.1/` in the game browser and run `probe 198.22.94.1`.
6. Recover the incident record and follow the timed incident log.

## Hints

Hack mechanic: mass_assignment. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `shift_control`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `factory_shift_handover_status` shows progress without resetting anything.
- Reward: 124 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.94.1 factory_shift_handover:records_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.94.1 factory_shift_handover:records_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_mass_assignment`.

`exploit -f ~/scripts/exp_mass_assignment.adr -s factory_shift_handover:records_entry 198.22.94.1`

Read `/srv/records/handover.log` on the last host — the preserved record is there.

## Installation and persistence

Family: events. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
