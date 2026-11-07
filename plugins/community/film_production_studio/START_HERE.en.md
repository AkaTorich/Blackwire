# Film Production Studio

An unreleased production schedule identifies an unexplained night shoot.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `film_production_studio_open` or press **Start**.
3. In the terminal run `film_production_studio_start`.
4. Read the mail “Film Production Studio: first lead” in the game mail.
5. Open `http://198.22.35.1/` in the game browser and run `probe 198.22.35.1`.
6. Traverse the gateway and recover the internal archive.

## Hints

Hack mechanic: object_reference. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `production_archive`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `film_production_studio_status` shows progress without resetting anything.
- Reward: 234 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.35.1 film_production_studio:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.35.1 film_production_studio:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_object_reference`.

`exploit -f ~/scripts/exp_object_reference.adr -s film_production_studio:gateway_entry 198.22.35.1`

Read `/srv/records/shooting-schedule.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: networks. Organisation: media. The plugin folder never touches other saves; created devices and missions are saved with the world.
