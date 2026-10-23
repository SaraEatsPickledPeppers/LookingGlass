# Co-op campaign verification matrix

Source: `campaign::route()` in `src/campaign.rs`. Return visits are separate requirements.
No visit has passed co-op verification yet. Existing single-player route tests do not
supply evidence for two-player play. Record a build identity and reproducible evidence
before changing a row to passed.

Each visit requires both host/guest roles and both first-arrival orders, coherent enemies
and individual consumables, single-player cutscenes with safe reunion, shared-transport
boarding, a traversal death followed by timed spectating/respawn, a simultaneous wipe
and checkpoint reload, a trailing guest, and disconnect/rejoin near a transition.
Where a feature is absent, record why it is not applicable instead of silently omitting it.
The finale must verify completion for both players in place of an outgoing transition.

Repeat traversal and combat with latency/jitter. Keep separate single-player/save
regression evidence. The leaves, swimming, moving supports and scripted rides need
rendered two-process playtests as well as headless checks.

Entrance-only evidence: `--coop-campaign-check` passes all 39 visits for paired
placement, two seconds of arrival physics, shared-world snapshot restoration and
entrance wipe placement. Both underwater death roles also pass at the temple and
first garden entrance. This is preparation for the traversal checks below, not
completion of a campaign row. Evidence: `private/coop-progression-entrances4.log`.

Persistence-only evidence: the public two-process fixture
`private/coop/travel-test-0abfe07a7db0413db758682defa42cd9/` passes travel/rejoin,
save/close/restart, exact shared checkpoint/team-wipe restoration and backup
recovery with distinct player resources. It uses staged school transitions;
quest-gated exits and complete level traversal remain unverified.

| # | Map | Entrance | Co-op evidence |
| ---: | --- | --- | --- |
| 1 | `gvillage` | `Initial` | Not run |
| 2 | `pandemonium` | `Initial` | Not run |
| 3 | `fortress1` | `Initial` | Not run |
| 4 | `fortress2` | `Initial` | Not run |
| 5 | `fortress1` | `fortress1_start2` | Not run |
| 6 | `skool1` | `Initial` | Not run |
| 7 | `skool2` | `Initial` | Not run |
| 8 | `skool1` | `skool1_start2` | Not run |
| 9 | `potears1` | `Initial` | Not run |
| 10 | `potears2` | `Initial` | Not run |
| 11 | `potears3` | `Initial` | Not run |
| 12 | `utemple` | `Initial` | Not run |
| 13 | `garden1` | `Initial` | Not run |
| 14 | `garden2` | `Initial` | Not run |
| 15 | `garden3` | `Initial` | Not run |
| 16 | `garden4` | `Initial` | Not run |
| 17 | `centipede1` | `Initial` | Not run |
| 18 | `centipede2` | `Initial` | Not run |
| 19 | `wforest` | `Initial` | Not run |
| 20 | `wchess1` | `Initial` | Not run |
| 21 | `wchess2` | `Initial` | Not run |
| 22 | `rchess1` | `Initial` | Not run |
| 23 | `funhouse` | `Initial` | Not run |
| 24 | `hatter1` | `Initial` | Not run |
| 25 | `hatter2` | `Initial` | Not run |
| 26 | `jlair1` | `Initial` | Not run |
| 27 | `jlair2` | `Initial` | Not run |
| 28 | `wforest` | `wforest_start2` | Not run |
| 29 | `hedge1` | `Initial` | Not run |
| 30 | `tower1` | `Initial` | Not run |
| 31 | `hedge2` | `Initial` | Not run |
| 32 | `tower2` | `Initial` | Not run |
| 33 | `hedge3` | `Initial` | Not run |
| 34 | `tower3` | `Initial` | Not run |
| 35 | `grounds1` | `Initial` | Not run |
| 36 | `grounds2` | `Initial` | Not run |
| 37 | `facade` | `Initial` | Not run |
| 38 | `keep` | `Initial` | Not run |
| 39 | `qlair` | `Initial` | Not run |
