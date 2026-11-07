# Erased Visitor Records

The front desk and access controller disagree about an after-hours visitor.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `erased_visitor_records_open` or press **Start**.
3. In the terminal run `erased_visitor_records_start`.
4. Read the mail “Erased Visitor Records: first lead” in the game mail.
5. Open `http://198.22.46.1/` in the game browser and run `probe 198.22.46.1`.
6. Follow both evidence hosts and establish the custody trail.

## Hints

Hack mechanic: fragment_overlap. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `visitor_archive`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `erased_visitor_records_status` shows progress without resetting anything.
- Reward: 215 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.46.1 erased_visitor_records:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.46.1 erased_visitor_records:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_fragment_overlap`.

`exploit -f ~/scripts/exp_fragment_overlap.adr -s erased_visitor_records:gateway_entry 198.22.46.1`

Read `/srv/records/visitor-records.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: investigations. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
