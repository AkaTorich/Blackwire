# Freight Port Network

Dock records disagree with the container inventory held in the dispatch archive.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `freight_port_network_open` or press **Start**.
3. In the terminal run `freight_port_network_start`.
4. Read the mail “Freight Port Network: first lead” in the game mail.
5. Open `http://198.22.22.1/` in the game browser and run `probe 198.22.22.1`.
6. Traverse the gateway and recover the internal archive.

## Hints

Hack mechanic: traversal. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `cargo_registry`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `freight_port_network_status` shows progress without resetting anything.
- Reward: 156 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.22.1 freight_port_network:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.22.1 freight_port_network:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_traversal`.

`exploit -f ~/scripts/exp_traversal.adr -s freight_port_network:gateway_entry 198.22.22.1`

Read `/srv/records/cargo-manifest.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: networks. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
