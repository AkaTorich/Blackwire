# Black Market Exam Papers

Exam custody records show an unauthorized print request before release.

## Start here

1. Put this folder next to your other plugins and press **Reload**.
2. Open the guide window with `black_market_exam_papers_open` or press **Start**.
3. In the terminal run `black_market_exam_papers_start`.
4. Read the mail “Black Market Exam Papers: first lead” in the game mail.
5. Open `http://198.22.48.1/` in the game browser and run `probe 198.22.48.1`.
6. Follow both evidence hosts and establish the custody trail.

## Hints

Hack mechanic: permission_mask. `research` gives the hints and parameters of the specific device.

- If `probe` shows an active firewall, get past it with `firewall <address>`. On a miss, try again and watch your heat.
- On the first host look for the plugin's own service — `exam_archive`.
- Access needs the “entry” weakness, not just a data leak.
- Inside the device, check `/etc/maintenance-note.txt` — it holds the record location.
- `black_market_exam_papers_status` shows progress without resetting anything.
- Reward: 225 ₵, paid once. Running Start again does not duplicate content or reset completion.

## If you want a ready solution

`research -h 3 198.22.48.1 black_market_exam_papers:gateway_entry`

Your own source: `exploit new ~/scripts/my_exploit.adr 198.22.48.1 black_market_exam_papers:gateway_entry`. Fill in the payload using the in-game hints.

Ready editable tool: `market buy exp_permission_mask`.

`exploit -f ~/scripts/exp_permission_mask.adr -s black_market_exam_papers:gateway_entry 198.22.48.1`

Read `/srv/records/exam-custody.csv` on the last host — the preserved record is there.

## Installation and persistence

Family: investigations. Organisation: corporation. The plugin folder never touches other saves; created devices and missions are saved with the world.
