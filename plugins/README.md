# Blackwire Community Plugins

100 independent Adder plugins for Blackwire 0.1.1, SDK API 1.

## Installation

1. Extract `Blackwire-Plugins-0.1.1.zip`.
2. In the game open **Plugins → Open folder**.
3. Copy the desired folders from the archive's `plugins` directory into that folder. All 100 can coexist.
4. Choose **Reload** in the game.
5. Read `START_HERE.md` inside a plugin folder. Its `_open` command opens an in-game guide.

Content is created only after **Start**; loading the pack does not immediately fill the world with new districts. Utility and shop plugins are ready after loading. Each mission reward and hardware upgrade applies once. Created content persists in the saved game even if the plugin is disabled.

## Contents

20 device packs, 20 organization networks, 20 investigations, 15 utility panels, 15 shop/market additions and 10 timed incidents. The 70 content packs add 130 devices with their own weaknesses, 70 quests and 40 organizations when all are started. Devices use unique addresses in 198.22.0.0/16 and private networks in 10.230.0.0/16. Every address refers to the fictional game world.

## Plugin catalog

| # | Plugin | Category | What it adds | Start or use |
|---|---|---|---|---|
| 1 | Residential Cameras | device | Lobby camera footage has a 14-minute gap before the courier arrives. | `residential_cameras_start` |
| 2 | Parking Terminals | device | Space B17 was billed twice; reconcile the terminal transaction log. | `parking_terminals_start` |
| 3 | Transit Card Readers | device | A maintenance card appears on three buses at the same timestamp. | `transit_card_readers_start` |
| 4 | Smart Door Locks | device | Apartment 407 has an obsolete guest access token. | `smart_door_locks_start` |
| 5 | Rooftop Weather Stations | device | Wind readings stay identical while independent stations report a storm. | `rooftop_weather_stations_start` |
| 6 | Warehouse Barcode Scanners | device | Pallet CR-204 moved to a loading dock without an outbound scan. | `warehouse_barcode_scanners_start` |
| 7 | Hotel Keycard Encoders | device | The encoder issued a master key during an unstaffed night shift. | `hotel_keycard_encoders_start` |
| 8 | Cafe Payment Kiosks | device | Receipt 184 contains a refund to a decommissioned till. | `cafe_payment_kiosks_start` |
| 9 | Solar Inverters | device | One inverter reports midnight generation and conceals a bypass circuit. | `solar_inverters_start` |
| 10 | Water Quality Sensors | device | Three sample batches disagree with the signed calibration record. | `water_quality_sensors_start` |
| 11 | Parcel Locker Hubs | device | Locker 42 opened twice for the same pickup code. | `parcel_locker_hubs_start` |
| 12 | Library Checkout Stations | device | A restricted atlas was checked out using an expired staff account. | `library_checkout_stations_start` |
| 13 | EV Charging Controllers | device | A charging bay kept billing after its contactor was disconnected. | `ev_charging_controllers_start` |
| 14 | Elevator Control Panels | device | The service panel recorded an unscheduled stop on the closed floor. | `elevator_control_panels_start` |
| 15 | Factory Inspection Cameras | device | Rejected batch K12 was relabeled as accepted after inspection. | `factory_inspection_cameras_start` |
| 16 | Cinema Ticket Gates | device | A reseller issued duplicate passes for an already full screening. | `cinema_ticket_gates_start` |
| 17 | Medical Refrigeration Units | device | The overnight temperature alarm was acknowledged before it was raised. | `medical_refrigeration_units_start` |
| 18 | Campus Print Stations | device | A confidential exam remained in the shared print queue. | `campus_print_stations_start` |
| 19 | Farm Irrigation Controllers | device | Zone 6 waters continuously despite the published irrigation schedule. | `farm_irrigation_controllers_start` |
| 20 | Traffic Signal Cabinets | device | The maintenance schedule contains an unauthorized priority route. | `traffic_signal_cabinets_start` |
| 21 | Private Clinic Network | network | A private clinic moved its appointment archive behind a forgotten maintenance gateway. | `private_clinic_network_start` |
| 22 | Freight Port Network | network | Dock records disagree with the container inventory held in the dispatch archive. | `freight_port_network_start` |
| 23 | University Research Network | network | A laboratory grant was redirected through a retired research project. | `university_research_network_start` |
| 24 | Independent Newsroom | network | An editor needs the original notes behind a silently changed public report. | `independent_newsroom_start` |
| 25 | Credit Union Branch | network | An internal ledger contains a loan approved after its signatory left the bank. | `credit_union_branch_start` |
| 26 | Public Records Office | network | The public permit portal omits an amendment stored on the internal records host. | `public_records_office_start` |
| 27 | Regional Data Center | network | Rack allocation logs show an unlisted tenant using reserved capacity. | `regional_data_center_start` |
| 28 | Neighborhood ISP | network | The subscriber archive records a private route absent from the provider map. | `neighborhood_isp_start` |
| 29 | Museum Security Network | network | A protected exhibit was moved while the security roster was empty. | `museum_security_network_start` |
| 30 | Retail Distribution Hub | network | The warehouse system marks returned goods as new stock. | `retail_distribution_hub_start` |
| 31 | Municipal Power Office | network | A planned outage was removed from the internal maintenance calendar. | `municipal_power_office_start` |
| 32 | Railway Dispatch Network | network | A freight route changed after its dispatcher signed off. | `railway_dispatch_network_start` |
| 33 | City Housing Agency | network | An apartment allocation was reassigned without a board decision. | `city_housing_agency_start` |
| 34 | Emergency Training Center | network | A simulation alarm was copied into the official incident register. | `emergency_training_center_start` |
| 35 | Film Production Studio | network | An unreleased production schedule identifies an unexplained night shoot. | `film_production_studio_start` |
| 36 | Community Internet Exchange | network | A hidden peer uses an obsolete exchange agreement. | `community_internet_exchange_start` |
| 37 | Cooperative Workshop | network | The shared tool ledger records equipment that never reached the workshop. | `cooperative_workshop_start` |
| 38 | Regional Court Archive | network | A sealed case appears in the index with an altered custody date. | `regional_court_archive_start` |
| 39 | Marine Research Station | network | A survey vessel's route conflicts with the downloaded buoy readings. | `marine_research_station_start` |
| 40 | Underground Radio Station | network | A broadcast was replaced while the station controller was unattended. | `underground_radio_station_start` |
| 41 | Missing Courier Ledger | case | A courier vanished between two depots; the handoff ledger names the final receiving clerk. | `missing_courier_ledger_start` |
| 42 | Counterfeit Payroll Trail | case | Payroll adjustments match a vendor account that should have been closed. | `counterfeit_payroll_trail_start` |
| 43 | Stolen Prototype Logs | case | The prototype bench was used after the inventory team sealed the laboratory. | `stolen_prototype_logs_start` |
| 44 | Altered Water Reports | case | The signed laboratory results differ from the values on the public dashboard. | `altered_water_reports_start` |
| 45 | Ghost Shipping Orders | case | Containers were billed to an address that no customer account owns. | `ghost_shipping_orders_start` |
| 46 | Erased Visitor Records | case | The front desk and access controller disagree about an after-hours visitor. | `erased_visitor_records_start` |
| 47 | Duplicate Insurance Claims | case | Two claims reference the same inspection photo and different incident dates. | `duplicate_insurance_claims_start` |
| 48 | Black Market Exam Papers | case | Exam custody records show an unauthorized print request before release. | `black_market_exam_papers_start` |
| 49 | Forged Safety Certificates | case | A certificate was signed by an account whose authority had expired. | `forged_safety_certificates_start` |
| 50 | Diverted Relief Supplies | case | Relief parcels were marked delivered before reaching the regional depot. | `diverted_relief_supplies_start` |
| 51 | Vanishing Art Inventory | case | A gallery's private inventory contains a replacement with a copied catalog number. | `vanishing_art_inventory_start` |
| 52 | Tampered Election Kiosks | case | A fictional city kiosk reports maintenance performed by a retired service account. | `tampered_election_kiosks_start` |
| 53 | Leaked Patient Referrals | case | An anonymized referral index was exported outside its scheduled reporting window. | `leaked_patient_referrals_start` |
| 54 | Corrupted Sensor Calibration | case | Sensor measurements are plausible but their calibration signatures do not agree. | `corrupted_sensor_calibration_start` |
| 55 | Unauthorized Night Shift | case | Three badge holders entered during a shift that staffing records say never existed. | `unauthorized_night_shift_start` |
| 56 | Hidden Warehouse Tenant | case | A sealed storage room receives power under a nonexistent lease. | `hidden_warehouse_tenant_start` |
| 57 | Swapped Construction Bids | case | The submitted bid differs from the revision kept by the review committee. | `swapped_construction_bids_start` |
| 58 | Phantom Subscription Charges | case | Dormant accounts are charged for a service absent from the public price list. | `phantom_subscription_charges_start` |
| 59 | Deleted Research Attribution | case | A dataset's attribution changed between its review and publication. | `deleted_research_attribution_start` |
| 60 | Unlisted Server Rack | case | Power readings reveal an occupied rack missing from the inventory. | `unlisted_server_rack_start` |
| 61 | Known Host Inventory | utility | List discovered hosts and their live status. | `known_host_inventory_open` |
| 62 | Owned Machine Inventory | utility | List machines already under the player's control. | `owned_machine_inventory_open` |
| 63 | Service Exposure Map | utility | Inspect running services on discovered machines. | `service_exposure_map_open` |
| 64 | Custom Vulnerability Ledger | utility | List plugin-defined weaknesses on discovered devices. | `custom_vulnerability_ledger_open` |
| 65 | Network Route Atlas | utility | Show gateway relationships and internal addresses. | `network_route_atlas_open` |
| 66 | Device Type Catalog | utility | List registered plugin device types and firmware. | `device_type_catalog_open` |
| 67 | Quest Progress Board | utility | Summarize independent plugin quests and rewards. | `quest_progress_board_open` |
| 68 | Mail Triage Desk | utility | Summarize unread mail on the player's computer. | `mail_triage_desk_open` |
| 69 | Hardware Capacity Report | utility | Compare memory and storage on discovered machines. | `hardware_capacity_report_open` |
| 70 | Budget Planning Desk | utility | Show cash, loan balance and available upgrade prices. | `budget_planning_desk_open` |
| 71 | Investigation Status Desk | utility | Display the current in-game investigation state. | `investigation_status_desk_open` |
| 72 | Incident Log Viewer | utility | Review recent logs on owned machines. | `incident_log_viewer_open` |
| 73 | Organization Directory | utility | List discovered organizations and their gateways. | `organization_directory_open` |
| 74 | Session Notes Exporter | utility | Export player notes into an in-game report. | `session_notes_exporter_open` |
| 75 | Upgrade Inventory Desk | utility | List purchased upgrades and registered shop items. | `upgrade_inventory_desk_open` |
| 76 | Pocket Memory Upgrade | economy | Add 256 MB of memory for lightweight terminal work. | `pocket_memory_upgrade_status` |
| 77 | Developer Memory Upgrade | economy | Add 1 GB of memory for script development. | `developer_memory_upgrade_status` |
| 78 | Workstation Memory Upgrade | economy | Add 2 GB of memory for parallel game workloads. | `workstation_memory_upgrade_status` |
| 79 | Analyst Memory Upgrade | economy | Add 4 GB of memory for a larger virtual workstation. | `analyst_memory_upgrade_status` |
| 80 | Pocket Storage Upgrade | economy | Add 2 GB of virtual disk capacity. | `pocket_storage_upgrade_status` |
| 81 | Archive Storage Upgrade | economy | Add 8 GB of virtual disk capacity for evidence. | `archive_storage_upgrade_status` |
| 82 | Research Storage Upgrade | economy | Add 16 GB of virtual disk capacity for research. | `research_storage_upgrade_status` |
| 83 | Cold Storage Upgrade | economy | Add 32 GB of virtual disk capacity for long-term archives. | `cold_storage_upgrade_status` |
| 84 | Host Identity Tool | economy | Report the current host, address and access level. | `host_identity_tool_status` |
| 85 | Route Trace Tool | economy | Report the current connection chain and proxy exit. | `route_trace_tool_status` |
| 86 | Cashflow Summary Tool | economy | Report available funds and outstanding debt. | `cashflow_summary_tool_status` |
| 87 | Mission Checklist Tool | economy | Print the active mission checklist. | `mission_checklist_tool_status` |
| 88 | Process Survey Tool | economy | Print process ownership, CPU use and memory use. | `process_survey_tool_status` |
| 89 | Directory Audit Tool | economy | Inspect file ownership and permissions in the current directory. | `directory_audit_tool_status` |
| 90 | Heat Watch Tool | economy | Report suspicion and investigation progress. | `heat_watch_tool_status` |
| 91 | Storm Relay Outage | event | A coastal relay loses its uplink as a simulated storm approaches. | `storm_relay_outage_start` |
| 92 | Rolling Backup Leak | event | A rotating backup exposes an obsolete manifest before a new snapshot replaces it. | `rolling_backup_leak_start` |
| 93 | Expired Badge Incident | event | A temporary badge remains active after its shift expires. | `expired_badge_incident_start` |
| 94 | Factory Shift Handover | event | An unattended control desk receives the next shift's maintenance handover. | `factory_shift_handover_start` |
| 95 | Radio Interference Burst | event | A radio controller reports a short interference burst and a failed relay test. | `radio_interference_burst_start` |
| 96 | Cold Chain Alarm | event | A simulated refrigerator alarm appears while its monitoring endpoint goes silent. | `cold_chain_alarm_start` |
| 97 | Parcel Queue Surge | event | A parcel router accumulates delayed pickups during an intake surge. | `parcel_queue_surge_start` |
| 98 | Battery Reserve Warning | event | A backup battery controller reports a declining reserve during a grid drill. | `battery_reserve_warning_start` |
| 99 | Audit Window Countdown | event | A temporary audit service is available during a short scheduled inspection. | `audit_window_countdown_start` |
| 100 | Maintenance Credential Rollover | event | A service gateway rotates its maintenance credentials at the end of a shift. | `maintenance_credential_rollover_start` |

## Validation

The repository's `core/tests/community_plugins.rs` exercises every plugin independently and all 100 together on the real game engine. It checks commands and UI actions, custom vulnerabilities, mission completion, single rewards, purchases, market scripts, event phases, persistence and address conflicts.

Check plugins sequentially: `cargo test --locked -p blackwire-core --test community_plugins every_plugin -- --test-threads=1`.

The combined test is ignored by default and runs only on a GitHub runner. Content plugins attach handlers to their own vulnerabilities, so an unrelated hack does not run all 70 content handlers.
