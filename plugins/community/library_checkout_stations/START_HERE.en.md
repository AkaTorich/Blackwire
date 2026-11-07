# Library Checkout Stations

A restricted atlas was checked out using an expired staff account.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `library_checkout_stations_open` or press **Start**.
3. In the terminal run `library_checkout_stations_start`.
4. Read the mail “Library Checkout Stations: first lead” in the game mail.
5. Open `http://198.22.12.1/` in the game browser and run `probe 198.22.12.1`.
6. Audit the controller and recover its operational record.

## Hints

Hack mechanic: header_injection. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `loan_sync`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `library_checkout_stations_status` shows progress without resetting anything.
- Reward: 104 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.12.1 library_checkout_stations:records_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.12.1 library_checkout_stations:records_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_header_injection`.

`exploit -f ~/scripts/exp_header_injection.adr -s library_checkout_stations:records_entry 198.22.12.1`

Read `/srv/records/loans.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: devices. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
