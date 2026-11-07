# Public Records Office

The public permit portal omits an amendment stored on the internal records host.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `public_records_office_open` or press **Start**.
3. In the terminal run `public_records_office_start`.
4. Read the mail “Public Records Office: first lead” in the game mail.
5. Open `http://198.22.26.1/` in the game browser and run `probe 198.22.26.1`.
6. Traverse the gateway and recover the internal archive.

## Hints

Hack mechanic: protocol. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `records_store`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `public_records_office_status` shows progress without resetting anything.
- Reward: 180 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.26.1 public_records_office:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.26.1 public_records_office:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_protocol`.

`exploit -f ~/scripts/exp_protocol.adr -s public_records_office:gateway_entry 198.22.26.1`

Read `/srv/records/permit-register.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: networks. Organisation: government agency. The plugin folder never touches other saves; created devices and missions are saved with the world.
