# Duplicate Insurance Claims

Two claims reference the same inspection photo and different incident dates.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `duplicate_insurance_claims_open` or press **Start**.
3. In the terminal run `duplicate_insurance_claims_start`.
4. Read the mail “Duplicate Insurance Claims: first lead” in the game mail.
5. Open `http://198.22.47.1/` in the game browser and run `probe 198.22.47.1`.
6. Follow both evidence hosts and establish the custody trail.

## Hints

Hack mechanic: weak_rng. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `claims_store`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `duplicate_insurance_claims_status` shows progress without resetting anything.
- Reward: 220 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.47.1 duplicate_insurance_claims:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.47.1 duplicate_insurance_claims:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_weak_rng`.

`exploit -f ~/scripts/exp_weak_rng.adr -s duplicate_insurance_claims:gateway_entry 198.22.47.1`

Read `/srv/records/claim-pairs.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: investigations. Organisation: media. The plugin folder never touches other saves; created devices and missions are saved with the world.
