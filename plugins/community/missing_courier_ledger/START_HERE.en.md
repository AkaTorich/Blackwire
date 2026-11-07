# Missing Courier Ledger

A courier vanished between two depots; the handoff ledger names the final receiving clerk.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `missing_courier_ledger_open` or press **Start**.
3. In the terminal run `missing_courier_ledger_start`.
4. Read the mail “Missing Courier Ledger: first lead” in the game mail.
5. Open `http://198.22.41.1/` in the game browser and run `probe 198.22.41.1`.
6. Follow both evidence hosts and establish the custody trail.

## Hints

Hack mechanic: unsafe_deserialization. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `courier_records`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `missing_courier_ledger_status` shows progress without resetting anything.
- Reward: 190 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.41.1 missing_courier_ledger:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.41.1 missing_courier_ledger:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_unsafe_deserialization`.

`exploit -f ~/scripts/exp_unsafe_deserialization.adr -s missing_courier_ledger:gateway_entry 198.22.41.1`

Read `/srv/records/handoff-ledger.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: investigations. Organisation: media. The plugin folder never touches other saves; created devices and missions are saved with the world.
