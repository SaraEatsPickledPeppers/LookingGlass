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
school quest-gated exits and complete level traversal remain unverified by that fixture.

Village partial traversal: `--coop-village-check` now reaches the authored exit
from the unarmed entrance with both native bodies and both first-arrival orders.
It verifies both real Blade pickups, all six scenes, six safe cinematic reunions,
continued movement during ordinary speech and no deaths/wipes. Evidence:
`private/coop/village-route-4/coop-village/`, build
`0eb413f6731105a82780dc10d716c793fc04b3d7c3931d11391cbf6e9de9c2ee`.
The former eight-reunion result also treated two voice-only lines as cutscenes;
that behavior has been corrected to match single-player exploration.
That result is a single-process simulation. A subsequent public two-process
Red-leading traversal also passes with 40 ms delay and 20 ms jitter per sender:
six correctly owned scenes/reunions, both personal Blades, 50 moving-speech sample
pairs in each client, 84 identical matching authoritative ticks, and both players
entering Pandemonium at 100 Sanity. Both final views were inspected and the clients
and helper exited cleanly. Evidence:
`private/coop/village-test-eb20704fbc3547b1ab7bf766699c54a8/`, build
`5cae1ad45d01ad0b9a6f04a8f9a209c9bbff3ff804697cde94dbb3fefe2f5114`.
The Blue-leading network repeat also passes on that build and those settings in
`private/coop/village-test-064657842c4d466cb6e551e24be6bb98/`: both finish at 100
Sanity, all six owners/reunions and personal Blades pass, 81 matching authoritative
ticks agree, and both clients observe movement during ordinary speech without a warp.
Both real fall/death roles and the shared saved-checkpoint wipe now pass in a
separate public run with the same delay/jitter. Each player spectates the moving
survivor for exactly 1,200 ticks, revives at 25 Sanity with their Blade preserved,
and regains their own camera. A subsequent team wipe restores the actual earlier
automatic checkpoint's resources, placements and scene progress. Both restored
views were inspected. Evidence:
`private/coop/village-test-db59178229354bdbb2011361cf51ca9d/`, build
`f6304f3344c2f6d0340da41879fb00c4c7d37ada0076df301d49ae75a49f331d`.
The village has ambient moving machinery and a shootable hatch, but no activated
shared passenger lift or ride (`Village::event` / `Village::advance_players`);
the shared-transport boarding requirement is not applicable to this visit.
Fresh-process guest rejoin followed by the authored exit now passes on the same
build in `private/coop/village-test-49531ea8bd3f460da591f2be6a8c4465/` (Blue leads,
40 ms delay + 20 ms jitter). Both personal Blades and completed scenes survive;
all six owners/reunions and 63 matching authoritative player-state samples pass.
Both enter Pandemonium alive (Blue 100 / Red 92 Sanity), without a respawn or wipe.
Both final views were inspected and all owned processes/helpers closed. Per-visit
consumable and enemy evidence remains open before this row can be marked passed.

Pandemonium partial traversal: the normal-start native two-body route now reaches
the authored Fortress exit with either player first. Both runs verify all four
scene owners, three safe reunions, personal Cards and no deaths/wipes; both finish
at 60.934 Sanity in 33,993 ticks. The Gnome return scene's original trigger now
starts its existing timeline. Shared-rope hand attachments and reunion captures
were inspected; staged grab/climb/release and minecart boarding checks also pass.
Evidence: `private/coop/pandemonium-route6/coop-pandemonium/`,
`private/coop-pandemonium-route6.log`, `private/coop-pandemonium-rope-native8.log`,
build `a9b50ed49c886e859e6a0a9cbcaec0d833fd08d799a4e1f74527fa0626da844b`.
Watched/skipped single-player routes, cinematic pause/save/skip, machinery state,
school/village world regressions and 736 Rust tests pass. This is still a
single-process route; the later separate-client evidence follows below.

The Blue-first public internet repeat passes with 40 ms delay and 20 ms jitter per
sender on build `ec4ccf71f5a634f1c8c03ccbc6924fa3492e9fac1421dd813cd4937c06118a48`.
Both players use the same entrance rope, the cart waits for its partner, all four
Blue-owned scenes and three reunions pass, both collect/retain personal Cards,
and 207 matching authoritative samples agree. Both arrive in Fortress alive
(Blue 27.134 / Red 21.135 Sanity) without respawns or checkpoint resets. Rope,
landing, boarding-wait and initial Fortress cinematic views were inspected; both
processes exit cleanly. Evidence:
`private/coop/pandemonium-test-25726e42ca974a459cf70cd356606c0e/`.
This caught and fixed a delayed grab that otherwise released an automatic rope
grip. Protocol revision 4 preserves grab/release intent, with bounds/watchdog and
actual-physics regression coverage; all 740 Rust tests pass.
The Red-first public repeat also passes on that build/settings in
`private/coop/pandemonium-test-a2cf6558bee64719af5fda4834594ca6/`, running in Anode:
four Red-owned scenes, three reunions, delayed boarding, both personal Cards and
89 matching authoritative ticks. Both arrive alive (Blue 39.867 / Red 23.468),
without respawns or checkpoint resets. Rope, boarding-wait and initial Fortress
cinematic views were inspected; both clients and the job exit normally.
Both entrance-side hazard deaths now pass over the same public connection and
delay/jitter settings on that build, in
`private/coop/pandemonium-recovery-1127245225934d65ac70d7f9af173184/`.
Red's death/respawn ticks are 6,054/7,254 and Blue's are 8,962/10,162: both clients
agree on exactly 1,200 ticks, moving spectator cameras, 25-Sanity recovery 48 units
from the survivor, retained inventory and the survivor's unchanged health. The
team wipe at tick 11,170 restores the automatic checkpoint from tick 1, with full
saved resources, initial placements and earlier quest/scene flags. Both first
restored samples have zero horizontal displacement from the saved placements.
Spectator/checkpoint views were inspected and both clients exit normally. This
fixture ends at the restored checkpoint.
Fresh-process guest rejoin near the airship followed by the authored exit also
passes on the same build/settings in
`private/coop/pandemonium-test-b5965aedc3274cba8065c2c111d4421f/` (Blue leads).
Both personal resources and completed cart/key/scene flags survive; all four
owners, three reunions and 126 matching authoritative ticks pass. Both retain
Cards and arrive alive (Blue 45.734 / Red 15.668 Sanity), with no respawn or wipe.
The initial Fortress cinematic views were inspected; both processes exit normally.
Per-visit consumable and enemy coverage remains open before this row can pass.

| # | Map | Entrance | Co-op evidence |
| ---: | --- | --- | --- |
| 1 | `gvillage` | `Initial` | Partial: both arrival orders, deaths/spectator/respawn in both roles, saved-checkpoint wipe and fresh guest rejoin-to-exit pass; consumable/enemy evidence pending; shared boarding N/A |
| 2 | `pandemonium` | `Initial` | Partial: native and delayed public routes in both orders, rope/cart, four scenes/three reunions, personal Cards, both hazard death/respawn roles, saved-checkpoint wipe and fresh guest rejoin-to-exit pass; consumable/enemy evidence pending |
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
