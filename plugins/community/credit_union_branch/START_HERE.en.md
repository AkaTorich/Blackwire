# Credit Union Branch

An internal ledger contains a loan approved after its signatory left the bank.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `credit_union_branch_open` or press **Start**.
3. In the terminal run `credit_union_branch_start`.
4. Read the mail “Credit Union Branch: first lead” in the game mail.
5. Open `http://198.22.25.1/` in the game browser and run `probe 198.22.25.1`.
6. Traverse the gateway and recover the internal archive.

## Hints

Hack mechanic: format. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `loan_archive`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `credit_union_branch_status` shows progress without resetting anything.
- Reward: 174 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.25.1 credit_union_branch:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.25.1 credit_union_branch:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_format`.

`exploit -f ~/scripts/exp_format.adr -s credit_union_branch:gateway_entry 198.22.25.1`

Read `/srv/records/loan-ledger.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: networks. Organisation: bank. The plugin folder never touches other saves; created devices and missions are saved with the world.
