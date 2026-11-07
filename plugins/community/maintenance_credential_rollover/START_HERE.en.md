# Maintenance Credential Rollover

A service gateway rotates its maintenance credentials at the end of a shift.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `maintenance_credential_rollover_open` or press **Start**.
3. In the terminal run `maintenance_credential_rollover_start`.
4. Read the mail “Maintenance Credential Rollover: first lead” in the game mail.
5. Open `http://198.22.100.1/` in the game browser and run `probe 198.22.100.1`.
6. Recover the incident record and follow the timed incident log.

## Hints

Hack mechanic: prototype_pollution. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `maintenance_hub`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `maintenance_credential_rollover_status` shows progress without resetting anything.
- Reward: 172 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.100.1 maintenance_credential_rollover:records_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.100.1 maintenance_credential_rollover:records_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_prototype_pollution`.

`exploit -f ~/scripts/exp_prototype_pollution.adr -s maintenance_credential_rollover:records_entry 198.22.100.1`

Read `/srv/records/credential-notice.txt` on the last host — the preserved record is there.

## Installation and persistence

Family: events. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
