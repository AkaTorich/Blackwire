# Ghost Shipping Orders

Containers were billed to an address that no customer account owns.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `ghost_shipping_orders_open` or press **Start**.
3. In the terminal run `ghost_shipping_orders_start`.
4. Read the mail “Ghost Shipping Orders: first lead” in the game mail.
5. Open `http://198.22.45.1/` in the game browser and run `probe 198.22.45.1`.
6. Follow both evidence hosts and establish the custody trail.

## Hints

Hack mechanic: regex_backtracking. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `shipping_store`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `ghost_shipping_orders_status` shows progress without resetting anything.
- Reward: 210 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.45.1 ghost_shipping_orders:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.45.1 ghost_shipping_orders:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_regex_backtracking`.

`exploit -f ~/scripts/exp_regex_backtracking.adr -s ghost_shipping_orders:gateway_entry 198.22.45.1`

Read `/srv/records/ghost-orders.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: investigations. Organisation: media. The plugin folder never touches other saves; created devices and missions are saved with the world.
