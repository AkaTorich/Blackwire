# Private Clinic Network

A private clinic moved its appointment archive behind a forgotten maintenance gateway.

## Start here

1. Install this folder through the game's Plugins folder and choose Reload.
2. Open its guide window with `private_clinic_network_open`, then press Start; or run `private_clinic_network_start` in the game terminal.
3. Read the new mail titled **Private Clinic Network: first lead**.
4. Visit `http://198.22.21.1/` in the game's browser and inspect the first host with `probe 198.22.21.1`.
5. Traverse the gateway and recover the internal archive.

## Hints

- If probe reports an active firewall, run "firewall <address>" before entry. A failed bypass can be retried; watch your heat.

- Look for the custom patient_archive_gate service on the first host.
- The access objective requires the entry vulnerability, not only an information leak.
- Private addresses are reachable through the gateway; its web page names the next destination.
- Use `private_clinic_network_status` for progress without resetting anything.
- Reward: 150 credits, paid once. Repeating Start does not create another copy or reset completion.

## Optional full solution

`exploit -s private_clinic_network:gateway_entry 198.22.21.1`

`exploit -s private_clinic_network:operations_entry 10.230.21.2`

`exploit -s private_clinic_network:records_entry 10.230.21.3`

Read `/srv/records/patient-audit.csv` on the final host for the preserved record.

## Installation and persistence

Copy this plugin folder into the directory opened by **Plugins → Open folder**, then Reload. It is independent of the other 99 plugins. Created game content remains in the save after disabling a plugin. Names, commands and addresses are unique within this pack.
