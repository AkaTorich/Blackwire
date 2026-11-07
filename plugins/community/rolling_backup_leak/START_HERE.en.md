# Rolling Backup Leak

A rotating backup exposes an obsolete manifest before a new snapshot replaces it.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `rolling_backup_leak_open` or press **Start**.
3. In the terminal run `rolling_backup_leak_start`.
4. Read the mail “Rolling Backup Leak: first lead” in the game mail.
5. Open `http://198.22.92.1/` in the game browser and run `probe 198.22.92.1`.
6. Recover the incident record and follow the timed incident log.

## Hints

Hack mechanic: type_confusion. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `backup_sync`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `rolling_backup_leak_status` shows progress without resetting anything.
- Reward: 108 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.92.1 rolling_backup_leak:records_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.92.1 rolling_backup_leak:records_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_type_confusion`.

`exploit -f ~/scripts/exp_type_confusion.adr -s rolling_backup_leak:records_entry 198.22.92.1`

Read `/srv/records/backup-rotation.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: events. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
