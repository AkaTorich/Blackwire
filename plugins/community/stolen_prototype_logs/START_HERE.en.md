# Stolen Prototype Logs

The prototype bench was used after the inventory team sealed the laboratory.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `stolen_prototype_logs_open` or press **Start**.
3. In the terminal run `stolen_prototype_logs_start`.
4. Read the mail “Stolen Prototype Logs: first lead” in the game mail.
5. Open `http://198.22.43.1/` in the game browser and run `probe 198.22.43.1`.
6. Follow both evidence hosts and establish the custody trail.

## Hints

Hack mechanic: template_injection. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `prototype_store`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `stolen_prototype_logs_status` shows progress without resetting anything.
- Reward: 200 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.43.1 stolen_prototype_logs:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.43.1 stolen_prototype_logs:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_template_injection`.

`exploit -f ~/scripts/exp_template_injection.adr -s stolen_prototype_logs:gateway_entry 198.22.43.1`

Read `/srv/records/bench-access.log` on the last host — the preserved record is there.

## Installation and persistence

Family: investigations. Organisation: media. The plugin folder never touches other saves; created devices and missions are saved with the world.
