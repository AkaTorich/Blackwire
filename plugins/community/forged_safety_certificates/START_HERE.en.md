# Forged Safety Certificates

A certificate was signed by an account whose authority had expired.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `forged_safety_certificates_open` or press **Start**.
3. In the terminal run `forged_safety_certificates_start`.
4. Read the mail “Forged Safety Certificates: first lead” in the game mail.
5. Open `http://198.22.49.1/` in the game browser and run `probe 198.22.49.1`.
6. Follow both evidence hosts and establish the custody trail.

## Hints

Hack mechanic: overflow. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `safety_store`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `forged_safety_certificates_status` shows progress without resetting anything.
- Reward: 230 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.49.1 forged_safety_certificates:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.49.1 forged_safety_certificates:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_overflow`.

`exploit -f ~/scripts/exp_overflow.adr -s forged_safety_certificates:gateway_entry 198.22.49.1`

Read `/srv/records/certificate-chain.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: investigations. Organisation: media. The plugin folder never touches other saves; created devices and missions are saved with the world.
