# Rolling Backup Leak

A rotating backup exposes an obsolete manifest before a new snapshot replaces it.

## Start here

1. Install this folder through the game's Plugins folder and choose Reload.
2. Open its guide window with `rolling_backup_leak_open`, then press Start; or run `rolling_backup_leak_start` in the game terminal.
3. Read the new mail titled **Rolling Backup Leak: first lead**.
4. Visit `http://198.22.92.1/` in the game's browser and inspect the first host with `probe 198.22.92.1`.
5. Recover the incident record and follow the timed incident log.

## Hints

- If probe reports an active firewall, run "firewall <address>" before entry. A failed bypass can be retried; watch your heat.

- Look for the custom backup_sync service on the first host.
- The access objective requires the entry vulnerability, not only an information leak.
- After entering the device, inspect /etc/maintenance-note.txt for the record location.
- Use `rolling_backup_leak_status` for progress without resetting anything.
- Reward: 108 credits, paid once. Repeating Start does not create another copy or reset completion.

This incident advances every 15 game seconds for three phases. Its own host may temporarily lose service; it recovers by phase three. The incident does not modify the original game's hosts.

## Optional full solution

`exploit -s rolling_backup_leak:records_entry 198.22.92.1`

Read `/srv/records/backup-rotation.csv` on the final host for the preserved record.

## Installation and persistence

Copy this plugin folder into the directory opened by **Plugins → Open folder**, then Reload. It is independent of the other 99 plugins. Created game content remains in the save after disabling a plugin. Names, commands and addresses are unique within this pack.
