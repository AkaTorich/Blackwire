# Deleted Research Attribution

A dataset's attribution changed between its review and publication.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `deleted_research_attribution_open` or press **Start**.
3. In the terminal run `deleted_research_attribution_start`.
4. Read the mail “Deleted Research Attribution: first lead” in the game mail.
5. Open `http://198.22.59.1/` in the game browser and run `probe 198.22.59.1`.
6. Follow both evidence hosts and establish the custody trail.

## Hints

Hack mechanic: length_confusion. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `research_records`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `deleted_research_attribution_status` shows progress without resetting anything.
- Reward: 280 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.59.1 deleted_research_attribution:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.59.1 deleted_research_attribution:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_length_confusion`.

`exploit -f ~/scripts/exp_length_confusion.adr -s deleted_research_attribution:gateway_entry 198.22.59.1`

Read `/srv/records/authorship-history.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: investigations. Organisation: media. The plugin folder never touches other saves; created devices and missions are saved with the world.
