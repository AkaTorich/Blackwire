# Cafe Payment Kiosks

Receipt 184 contains a refund to a decommissioned till.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `cafe_payment_kiosks_open` or press **Start**.
3. In the terminal run `cafe_payment_kiosks_start`.
4. Read the mail “Cafe Payment Kiosks: first lead” in the game mail.
5. Open `http://198.22.8.1/` in the game browser and run `probe 198.22.8.1`.
6. Audit the controller and recover its operational record.

## Hints

Hack mechanic: mass_assignment. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `cafe_pay`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `cafe_payment_kiosks_status` shows progress without resetting anything.
- Reward: 88 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.8.1 cafe_payment_kiosks:records_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.8.1 cafe_payment_kiosks:records_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_mass_assignment`.

`exploit -f ~/scripts/exp_mass_assignment.adr -s cafe_payment_kiosks:records_entry 198.22.8.1`

Read `/srv/records/receipts.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: devices. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
