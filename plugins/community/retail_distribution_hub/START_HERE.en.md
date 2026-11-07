# Retail Distribution Hub

The warehouse system marks returned goods as new stock.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `retail_distribution_hub_open` or press **Start**.
3. In the terminal run `retail_distribution_hub_start`.
4. Read the mail “Retail Distribution Hub: first lead” in the game mail.
5. Open `http://198.22.30.1/` in the game browser and run `probe 198.22.30.1`.
6. Traverse the gateway and recover the internal archive.

## Hints

Hack mechanic: signedness. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `supply_archive`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `retail_distribution_hub_status` shows progress without resetting anything.
- Reward: 204 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.30.1 retail_distribution_hub:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.30.1 retail_distribution_hub:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_signedness`.

`exploit -f ~/scripts/exp_signedness.adr -s retail_distribution_hub:gateway_entry 198.22.30.1`

Read `/srv/records/supply-ledger.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: networks. Organisation: shop. The plugin folder never touches other saves; created devices and missions are saved with the world.
