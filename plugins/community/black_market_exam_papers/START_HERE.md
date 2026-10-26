# Black Market Exam Papers

Exam custody records show an unauthorized print request before release.

## Start here

1. Install this folder through the game's Plugins folder and choose Reload.
2. Open its guide window with `black_market_exam_papers_open`, then press Start; or run `black_market_exam_papers_start` in the game terminal.
3. Read the new mail titled **Black Market Exam Papers: first lead**.
4. Visit `http://198.22.48.1/` in the game's browser and inspect the first host with `probe 198.22.48.1`.
5. Follow both evidence hosts and establish the custody trail.

## Hints

- If probe reports an active firewall, run "firewall <address>" before entry. A failed bypass can be retried; watch your heat.

- Look for the custom exam_archive_gate service on the first host.
- The access objective requires the entry vulnerability, not only an information leak.
- Private addresses are reachable through the gateway; its web page names the next destination.
- Use `black_market_exam_papers_status` for progress without resetting anything.
- Reward: 225 credits, paid once. Repeating Start does not create another copy or reset completion.

Both evidence hosts must be accessed to complete this investigation.

## Optional full solution

`exploit -s black_market_exam_papers:gateway_entry 198.22.48.1`

`exploit -s black_market_exam_papers:records_entry 10.230.48.2`

Read `/srv/records/exam-custody.csv` on the final host for the preserved record.

## Installation and persistence

Copy this plugin folder into the directory opened by **Plugins → Open folder**, then Reload. It is independent of the other 99 plugins. Created game content remains in the save after disabling a plugin. Names, commands and addresses are unique within this pack.
