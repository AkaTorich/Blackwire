# Blackwire community plugins

[Русская версия](README.md)

100 independent Adder plugins for Blackwire, SDK API 1. Every plugin is bilingual: it follows the game language (Russian or English).

## Installation

1. Unpack `Blackwire-Plugins-<version>.zip`.
2. In the game open **Plugins → Open folder**.
3. Copy the folders of the plugins you want from the archive's `plugins` directory. All 100 work together.
4. Press **Reload** in the game.
5. Open `START_HERE.en.md` inside the plugin folder. The `_open` command opens the guide window right in the game.

Content is created only after **Start** — loading the pack does not fill the world right away. Tools and shop plugins work as soon as they are loaded. Rewards and upgrades apply once. Created content stays in the save even if the plugin is disabled.

## Contents

20 device packs, 20 organisation networks, 20 investigations, 15 tool windows, 15 shop/market items and 10 timed events. If you run everything, the content packs add 130 devices with their own weak spots, 70 missions and 40 organisations. Devices use unique addresses from 198.22.0.0/16, private networks from 10.230.0.0/16. All addresses belong to the game's fictional world.

## Plugin catalog

| # | Plugin | Section | What it adds | Start or command |
|---|---|---|---|---|
| 1 | Residential Cameras | devices | Lobby camera footage has a 14-minute gap before the courier arrives. | `residential_cameras_start` |
| 2 | Parking Terminals | devices | Space B17 was billed twice; reconcile the terminal transaction log. | `parking_terminals_start` |
| 3 | Transit Card Readers | devices | A maintenance card appears on three buses at the same timestamp. | `transit_card_readers_start` |
| 4 | Smart Door Locks | devices | Apartment 407 has an obsolete guest access token. | `smart_door_locks_start` |
| 5 | Rooftop Weather Stations | devices | Wind readings stay identical while independent stations report a storm. | `rooftop_weather_stations_start` |
| 6 | Warehouse Barcode Scanners | devices | Pallet CR-204 moved to a loading dock without an outbound scan. | `warehouse_barcode_scanners_start` |
| 7 | Hotel Keycard Encoders | devices | The encoder issued a master key during an unstaffed night shift. | `hotel_keycard_encoders_start` |
| 8 | Cafe Payment Kiosks | devices | Receipt 184 contains a refund to a decommissioned till. | `cafe_payment_kiosks_start` |
| 9 | Solar Inverters | devices | One inverter reports midnight generation and conceals a bypass circuit. | `solar_inverters_start` |
| 10 | Water Quality Sensors | devices | Three sample batches disagree with the signed calibration record. | `water_quality_sensors_start` |
| 11 | Parcel Locker Hubs | devices | Locker 42 opened twice for the same pickup code. | `parcel_locker_hubs_start` |
| 12 | Library Checkout Stations | devices | A restricted atlas was checked out using an expired staff account. | `library_checkout_stations_start` |
| 13 | EV Charging Controllers | devices | A charging bay kept billing after its contactor was disconnected. | `ev_charging_controllers_start` |
| 14 | Elevator Control Panels | devices | The service panel recorded an unscheduled stop on the closed floor. | `elevator_control_panels_start` |
| 15 | Factory Inspection Cameras | devices | Rejected batch K12 was relabeled as accepted after inspection. | `factory_inspection_cameras_start` |
| 16 | Cinema Ticket Gates | devices | A reseller issued duplicate passes for an already full screening. | `cinema_ticket_gates_start` |
| 17 | Medical Refrigeration Units | devices | The overnight temperature alarm was acknowledged before it was raised. | `medical_refrigeration_units_start` |
| 18 | Campus Print Stations | devices | A confidential exam remained in the shared print queue. | `campus_print_stations_start` |
| 19 | Farm Irrigation Controllers | devices | Zone 6 waters continuously despite the published irrigation schedule. | `farm_irrigation_controllers_start` |
| 20 | Traffic Signal Cabinets | devices | The maintenance schedule contains an unauthorized priority route. | `traffic_signal_cabinets_start` |
| 21 | Private Clinic Network | networks | A private clinic moved its appointment archive behind a forgotten maintenance gateway. | `private_clinic_network_start` |
| 22 | Freight Port Network | networks | Dock records disagree with the container inventory held in the dispatch archive. | `freight_port_network_start` |
| 23 | University Research Network | networks | A laboratory grant was redirected through a retired research project. | `university_research_network_start` |
| 24 | Independent Newsroom | networks | An editor needs the original notes behind a silently changed public report. | `independent_newsroom_start` |
| 25 | Credit Union Branch | networks | An internal ledger contains a loan approved after its signatory left the bank. | `credit_union_branch_start` |
| 26 | Public Records Office | networks | The public permit portal omits an amendment stored on the internal records host. | `public_records_office_start` |
| 27 | Regional Data Center | networks | Rack allocation logs show an unlisted tenant using reserved capacity. | `regional_data_center_start` |
| 28 | Neighborhood ISP | networks | The subscriber archive records a private route absent from the provider map. | `neighborhood_isp_start` |
| 29 | Museum Security Network | networks | A protected exhibit was moved while the security roster was empty. | `museum_security_network_start` |
| 30 | Retail Distribution Hub | networks | The warehouse system marks returned goods as new stock. | `retail_distribution_hub_start` |
| 31 | Municipal Power Office | networks | A planned outage was removed from the internal maintenance calendar. | `municipal_power_office_start` |
| 32 | Railway Dispatch Network | networks | A freight route changed after its dispatcher signed off. | `railway_dispatch_network_start` |
| 33 | City Housing Agency | networks | An apartment allocation was reassigned without a board decision. | `city_housing_agency_start` |
| 34 | Emergency Training Center | networks | A simulation alarm was copied into the official incident register. | `emergency_training_center_start` |
| 35 | Film Production Studio | networks | An unreleased production schedule identifies an unexplained night shoot. | `film_production_studio_start` |
| 36 | Community Internet Exchange | networks | A hidden peer uses an obsolete exchange agreement. | `community_internet_exchange_start` |
| 37 | Cooperative Workshop | networks | The shared tool ledger records equipment that never reached the workshop. | `cooperative_workshop_start` |
| 38 | Regional Court Archive | networks | A sealed case appears in the index with an altered custody date. | `regional_court_archive_start` |
| 39 | Marine Research Station | networks | A survey vessel's route conflicts with the downloaded buoy readings. | `marine_research_station_start` |
| 40 | Underground Radio Station | networks | A broadcast was replaced while the station controller was unattended. | `underground_radio_station_start` |
| 41 | Missing Courier Ledger | investigations | A courier vanished between two depots; the handoff ledger names the final receiving clerk. | `missing_courier_ledger_start` |
| 42 | Counterfeit Payroll Trail | investigations | Payroll adjustments match a vendor account that should have been closed. | `counterfeit_payroll_trail_start` |
| 43 | Stolen Prototype Logs | investigations | The prototype bench was used after the inventory team sealed the laboratory. | `stolen_prototype_logs_start` |
| 44 | Altered Water Reports | investigations | The signed laboratory results differ from the values on the public dashboard. | `altered_water_reports_start` |
| 45 | Ghost Shipping Orders | investigations | Containers were billed to an address that no customer account owns. | `ghost_shipping_orders_start` |
| 46 | Erased Visitor Records | investigations | The front desk and access controller disagree about an after-hours visitor. | `erased_visitor_records_start` |
| 47 | Duplicate Insurance Claims | investigations | Two claims reference the same inspection photo and different incident dates. | `duplicate_insurance_claims_start` |
| 48 | Black Market Exam Papers | investigations | Exam custody records show an unauthorized print request before release. | `black_market_exam_papers_start` |
| 49 | Forged Safety Certificates | investigations | A certificate was signed by an account whose authority had expired. | `forged_safety_certificates_start` |
| 50 | Diverted Relief Supplies | investigations | Relief parcels were marked delivered before reaching the regional depot. | `diverted_relief_supplies_start` |
| 51 | Vanishing Art Inventory | investigations | A gallery's private inventory contains a replacement with a copied catalog number. | `vanishing_art_inventory_start` |
| 52 | Tampered Election Kiosks | investigations | A fictional city kiosk reports maintenance performed by a retired service account. | `tampered_election_kiosks_start` |
| 53 | Leaked Patient Referrals | investigations | An anonymized referral index was exported outside its scheduled reporting window. | `leaked_patient_referrals_start` |
| 54 | Corrupted Sensor Calibration | investigations | Sensor measurements are plausible but their calibration signatures do not agree. | `corrupted_sensor_calibration_start` |
| 55 | Unauthorized Night Shift | investigations | Three badge holders entered during a shift that staffing records say never existed. | `unauthorized_night_shift_start` |
| 56 | Hidden Warehouse Tenant | investigations | A sealed storage room receives power under a nonexistent lease. | `hidden_warehouse_tenant_start` |
| 57 | Swapped Construction Bids | investigations | The submitted bid differs from the revision kept by the review committee. | `swapped_construction_bids_start` |
| 58 | Phantom Subscription Charges | investigations | Dormant accounts are charged for a service absent from the public price list. | `phantom_subscription_charges_start` |
| 59 | Deleted Research Attribution | investigations | A dataset's attribution changed between its review and publication. | `deleted_research_attribution_start` |
| 60 | Unlisted Server Rack | investigations | Power readings reveal an occupied rack missing from the inventory. | `unlisted_server_rack_start` |
| 61 | Known Host Inventory | tools | List discovered hosts and their live status. | `known_host_inventory_open` |
| 62 | Owned Machine Inventory | tools | List machines already under the player's control. | `owned_machine_inventory_open` |
| 63 | Service Exposure Map | tools | Inspect running services on discovered machines. | `service_exposure_map_open` |
| 64 | Custom Vulnerability Ledger | tools | List plugin-defined weaknesses on discovered devices. | `custom_vulnerability_ledger_open` |
| 65 | Network Route Atlas | tools | Show gateway relationships and internal addresses. | `network_route_atlas_open` |
| 66 | Device Type Catalog | tools | List registered plugin device types and firmware. | `device_type_catalog_open` |
| 67 | Quest Progress Board | tools | Summarize independent plugin quests and rewards. | `quest_progress_board_open` |
| 68 | Mail Triage Desk | tools | Summarize unread mail on the player's computer. | `mail_triage_desk_open` |
| 69 | Hardware Capacity Report | tools | Compare memory and storage on discovered machines. | `hardware_capacity_report_open` |
| 70 | Budget Planning Desk | tools | Show cash, loan balance and available upgrade prices. | `budget_planning_desk_open` |
| 71 | Investigation Status Desk | tools | Display the current in-game investigation state. | `investigation_status_desk_open` |
| 72 | Incident Log Viewer | tools | Review recent logs on owned machines. | `incident_log_viewer_open` |
| 73 | Organization Directory | tools | List discovered organizations and their gateways. | `organization_directory_open` |
| 74 | Session Notes Exporter | tools | Export player notes into an in-game report. | `session_notes_exporter_open` |
| 75 | Upgrade Inventory Desk | tools | List purchased upgrades and registered shop items. | `upgrade_inventory_desk_open` |
| 76 | Pocket Memory Upgrade | shops | Add 256 MB of memory for lightweight terminal work. | `pocket_memory_upgrade_status` |
| 77 | Developer Memory Upgrade | shops | Add 1 GB of memory for script development. | `developer_memory_upgrade_status` |
| 78 | Workstation Memory Upgrade | shops | Add 2 GB of memory for parallel game workloads. | `workstation_memory_upgrade_status` |
| 79 | Analyst Memory Upgrade | shops | Add 4 GB of memory for a larger virtual workstation. | `analyst_memory_upgrade_status` |
| 80 | Pocket Storage Upgrade | shops | Add 2 GB of virtual disk capacity. | `pocket_storage_upgrade_status` |
| 81 | Archive Storage Upgrade | shops | Add 8 GB of virtual disk capacity for evidence. | `archive_storage_upgrade_status` |
| 82 | Research Storage Upgrade | shops | Add 16 GB of virtual disk capacity for research. | `research_storage_upgrade_status` |
| 83 | Cold Storage Upgrade | shops | Add 32 GB of virtual disk capacity for long-term archives. | `cold_storage_upgrade_status` |
| 84 | Host Identity Tool | shops | Report the current host, address and access level. | `host_identity_tool_status` |
| 85 | Route Trace Tool | shops | Report the current connection chain and proxy exit. | `route_trace_tool_status` |
| 86 | Cashflow Summary Tool | shops | Report available funds and outstanding debt. | `cashflow_summary_tool_status` |
| 87 | Mission Checklist Tool | shops | Print the active mission checklist. | `mission_checklist_tool_status` |
| 88 | Process Survey Tool | shops | Print process ownership, CPU use and memory use. | `process_survey_tool_status` |
| 89 | Directory Audit Tool | shops | Inspect file ownership and permissions in the current directory. | `directory_audit_tool_status` |
| 90 | Heat Watch Tool | shops | Report suspicion and investigation progress. | `heat_watch_tool_status` |
| 91 | Storm Relay Outage | events | A coastal relay loses its uplink as a simulated storm approaches. | `storm_relay_outage_start` |
| 92 | Rolling Backup Leak | events | A rotating backup exposes an obsolete manifest before a new snapshot replaces it. | `rolling_backup_leak_start` |
| 93 | Expired Badge Incident | events | A temporary badge remains active after its shift expires. | `expired_badge_incident_start` |
| 94 | Factory Shift Handover | events | An unattended control desk receives the next shift's maintenance handover. | `factory_shift_handover_start` |
| 95 | Radio Interference Burst | events | A radio controller reports a short interference burst and a failed relay test. | `radio_interference_burst_start` |
| 96 | Cold Chain Alarm | events | A simulated refrigerator alarm appears while its monitoring endpoint goes silent. | `cold_chain_alarm_start` |
| 97 | Parcel Queue Surge | events | A parcel router accumulates delayed pickups during an intake surge. | `parcel_queue_surge_start` |
| 98 | Battery Reserve Warning | events | A backup battery controller reports a declining reserve during a grid drill. | `battery_reserve_warning_start` |
| 99 | Audit Window Countdown | events | A temporary audit service is available during a short scheduled inspection. | `audit_window_countdown_start` |
| 100 | Maintenance Credential Rollover | events | A service gateway rotates its maintenance credentials at the end of a shift. | `maintenance_credential_rollover_start` |

## Testing

In the repository `core/tests/community_plugins.rs` checks every plugin on its own and all 100 together on the real game core: commands and window actions, custom vulnerabilities, mission completion, one-time rewards, purchases, market scripts, incident phases, saving and the absence of address conflicts.

One by one: `cargo test --locked -p blackwire-core --test community_plugins every_plugin -- --test-threads=1`.

The all-together check is marked `#[ignore]` and runs only on the GitHub runner. Content plugins react only to their own weak spots — someone else's attack does not trigger all 70 handlers.
