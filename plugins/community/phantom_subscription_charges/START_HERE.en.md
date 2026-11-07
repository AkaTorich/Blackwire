# Phantom Subscription Charges

Dormant accounts are charged for a service absent from the public price list.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `phantom_subscription_charges_open` or press **Start**.
3. In the terminal run `phantom_subscription_charges_start`.
4. Read the mail “Phantom Subscription Charges: first lead” in the game mail.
5. Open `http://198.22.58.1/` in the game browser and run `probe 198.22.58.1`.
6. Follow both evidence hosts and establish the custody trail.

## Hints

Hack mechanic: signedness. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `billing_archive`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `phantom_subscription_charges_status` shows progress without resetting anything.
- Reward: 275 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.58.1 phantom_subscription_charges:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.58.1 phantom_subscription_charges:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_signedness`.

`exploit -f ~/scripts/exp_signedness.adr -s phantom_subscription_charges:gateway_entry 198.22.58.1`

Read `/srv/records/subscription-ledger.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: investigations. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
