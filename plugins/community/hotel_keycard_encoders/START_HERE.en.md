# Hotel Keycard Encoders

The encoder issued a master key during an unstaffed night shift.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `hotel_keycard_encoders_open` or press **Start**.
3. In the terminal run `hotel_keycard_encoders_start`.
4. Read the mail “Hotel Keycard Encoders: first lead” in the game mail.
5. Open `http://198.22.7.1/` in the game browser and run `probe 198.22.7.1`.
6. Audit the controller and recover its operational record.

## Hints

Hack mechanic: object_reference. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `key_issue`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `hotel_keycard_encoders_status` shows progress without resetting anything.
- Reward: 84 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.7.1 hotel_keycard_encoders:records_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.7.1 hotel_keycard_encoders:records_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_object_reference`.

`exploit -f ~/scripts/exp_object_reference.adr -s hotel_keycard_encoders:records_entry 198.22.7.1`

Read `/srv/records/keys.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: devices. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
