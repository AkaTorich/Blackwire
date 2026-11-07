# Vanishing Art Inventory

A gallery's private inventory contains a replacement with a copied catalog number.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `vanishing_art_inventory_open` or press **Start**.
3. In the terminal run `vanishing_art_inventory_start`.
4. Read the mail “Vanishing Art Inventory: first lead” in the game mail.
5. Open `http://198.22.51.1/` in the game browser and run `probe 198.22.51.1`.
6. Follow both evidence hosts and establish the custody trail.

## Hints

Hack mechanic: auth_replay. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `art_store`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `vanishing_art_inventory_status` shows progress without resetting anything.
- Reward: 240 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.51.1 vanishing_art_inventory:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.51.1 vanishing_art_inventory:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_auth_replay`.

`exploit -f ~/scripts/exp_auth_replay.adr -s vanishing_art_inventory:gateway_entry 198.22.51.1`

Read `/srv/records/art-inventory.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: investigations. Organisation: media. The plugin folder never touches other saves; created devices and missions are saved with the world.
