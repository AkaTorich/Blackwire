# Parcel Locker Hubs

Locker 42 opened twice for the same pickup code.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `parcel_locker_hubs_open` or press **Start**.
3. In the terminal run `parcel_locker_hubs_start`.
4. Read the mail “Parcel Locker Hubs: first lead” in the game mail.
5. Open `http://198.22.11.1/` in the game browser and run `probe 198.22.11.1`.
6. Audit the controller and recover its operational record.

## Hints

Hack mechanic: cache_poisoning. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `parcel_dispatch`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `parcel_locker_hubs_status` shows progress without resetting anything.
- Reward: 100 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.11.1 parcel_locker_hubs:records_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.11.1 parcel_locker_hubs:records_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_cache_poisoning`.

`exploit -f ~/scripts/exp_cache_poisoning.adr -s parcel_locker_hubs:records_entry 198.22.11.1`

Read `/srv/records/deliveries.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: devices. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
