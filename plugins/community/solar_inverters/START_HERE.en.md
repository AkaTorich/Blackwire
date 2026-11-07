# Solar Inverters

One inverter reports midnight generation and conceals a bypass circuit.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `solar_inverters_open` or press **Start**.
3. In the terminal run `solar_inverters_start`.
4. Read the mail “Solar Inverters: first lead” in the game mail.
5. Open `http://198.22.9.1/` in the game browser and run `probe 198.22.9.1`.
6. Audit the controller and recover its operational record.

## Hints

Hack mechanic: token_confusion. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `solar_control`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `solar_inverters_status` shows progress without resetting anything.
- Reward: 92 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.9.1 solar_inverters:records_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.9.1 solar_inverters:records_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_token_confusion`.

`exploit -f ~/scripts/exp_token_confusion.adr -s solar_inverters:records_entry 198.22.9.1`

Read `/srv/records/generation.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: devices. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
