# Residential Cameras

Lobby camera footage has a 14-minute gap before the courier arrives.

## Start here

1. Install this folder through the game's Plugins folder and choose Reload.
2. Open its guide window with `residential_cameras_open`, then press Start; or run `residential_cameras_start` in the game terminal.
3. Read the new mail titled **Residential Cameras: first lead**.
4. Visit `http://198.22.1.1/` in the game's browser and inspect the first host with `probe 198.22.1.1`.
5. Audit the controller and recover its operational record.

## Hints

- If probe reports an active firewall, run "firewall <address>" before entry. A failed bypass can be retried; watch your heat.

- Look for the custom video_hub service on the first host.
- The access objective requires the entry vulnerability, not only an information leak.
- After entering the device, inspect /etc/maintenance-note.txt for the record location.
- Use `residential_cameras_status` for progress without resetting anything.
- Reward: 60 credits, paid once. Repeating Start does not create another copy or reset completion.

## Optional full solution

`exploit -s residential_cameras:records_entry 198.22.1.1`

Read `/srv/records/recordings.csv` on the final host for the preserved record.

## Installation and persistence

Copy this plugin folder into the directory opened by **Plugins → Open folder**, then Reload. It is independent of the other 99 plugins. Created game content remains in the save after disabling a plugin. Names, commands and addresses are unique within this pack.
