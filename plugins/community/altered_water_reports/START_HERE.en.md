# Altered Water Reports

The signed laboratory results differ from the values on the public dashboard.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `altered_water_reports_open` or press **Start**.
3. In the terminal run `altered_water_reports_start`.
4. Read the mail “Altered Water Reports: first lead” in the game mail.
5. Open `http://198.22.44.1/` in the game browser and run `probe 198.22.44.1`.
6. Follow both evidence hosts and establish the custody trail.

## Hints

Hack mechanic: xml_entity. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `water_records`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `altered_water_reports_status` shows progress without resetting anything.
- Reward: 205 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.44.1 altered_water_reports:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.44.1 altered_water_reports:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_xml_entity`.

`exploit -f ~/scripts/exp_xml_entity.adr -s altered_water_reports:gateway_entry 198.22.44.1`

Read `/srv/records/sample-chain.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: investigations. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
