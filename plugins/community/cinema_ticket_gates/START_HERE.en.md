# Cinema Ticket Gates

A reseller issued duplicate passes for an already full screening.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `cinema_ticket_gates_open` or press **Start**.
3. In the terminal run `cinema_ticket_gates_start`.
4. Read the mail “Cinema Ticket Gates: first lead” in the game mail.
5. Open `http://198.22.16.1/` in the game browser and run `probe 198.22.16.1`.
6. Audit the controller and recover its operational record.

## Hints

Hack mechanic: xml_entity. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `ticket_gate`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `cinema_ticket_gates_status` shows progress without resetting anything.
- Reward: 120 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.16.1 cinema_ticket_gates:records_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.16.1 cinema_ticket_gates:records_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_xml_entity`.

`exploit -f ~/scripts/exp_xml_entity.adr -s cinema_ticket_gates:records_entry 198.22.16.1`

Read `/srv/records/admissions.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: devices. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
