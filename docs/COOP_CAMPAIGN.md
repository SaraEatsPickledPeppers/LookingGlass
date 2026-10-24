# Co-op campaign verification matrix

Source: `campaign::route()` in `src/campaign.rs`. Return visits are separate requirements.
The village, Pandemonium, first Fortress and Beyond visits have the complete combined co-op evidence below.
Existing single-player route tests do not supply evidence for two-player play.
Record build identities and reproducible evidence before changing a row to passed.

Each visit requires both host/guest roles and both first-arrival orders, coherent enemies
and individual consumables, single-player cutscenes with safe reunion, shared-transport
boarding, a real gameplay death followed by timed spectating/respawn, a simultaneous wipe
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
Both final views were inspected and all owned processes/helpers closed.
The remaining enemy/consumable requirements now pass on build
`fe1f5fd905c8262e13dc3bae6e86dfef7a57752504c84d2575e52c1d87bd7a0b`, in
`private/coop/village-combat-44b591c1f47d453aa0afd3b9514fb7df/` (public internet,
40 ms delay + 20 ms jitter per sender). Its sole live guard selects nearer Blue,
then nearer Red, with both visible. Each checked hit removes 12 Sanity from the
selected player and leaves the distant partner unchanged. After defeating it
with ordinary Blade attacks, both follow the normal ledges to the placed health
shard: Blue heals 88 to 100 while Red stays at 88 and retains her pickup; Red then
heals to 100 without changing Blue's resources. Both clients observe separate
collection. All 3,986 comparable sent/restored player, lives, checkpoint and
encounter states match; 146 coincident full local reports also agree. Both attack
views, the guest's still-available pickup, and the final healed view were inspected.
Both clients/helpers close normally and the invite is removed. Combined with the
earlier full-route/recovery/rejoin evidence, this completes the village row. The
latest runtime changes only add opt-in test observations; they change neither
single-player behavior nor the previously verified scene/recovery mechanics.

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
Both clients' retained histories also verify individual consumable rewards in the
Blue-first rejoin run and the Red-first route above. `tools/check_coop_rewards.py`
requires separate collection and healing, unchanged partner resources, and an
uncollected partner copy remaining in that player's loot ledger. Both clients
observe the same reward IDs. In the Blue-first run, the first guard's drop heals
Blue from 93.667 to 100, then Red from 84.533 to 99.533; collecting either copy
does not change the other player's resources. The later ranged guard's drop also
passes. Evidence: `private/coop-pandemonium-personal-rewards1.json` and
`private/coop-pandemonium-personal-rewards-red1.json`, on the same `ec4ccf71` build.
The first club guard now also passes a focused public-internet encounter check
on build `fe1f5fd905c8262e13dc3bae6e86dfef7a57752504c84d2575e52c1d87bd7a0b`,
in `private/coop/pandemonium-combat-ac557b2cbaa240d4a61b04df68220399/`.
Both Alices are visible; the guard selects and damages nearer Red, then nearer
Blue after their ordinary movement swaps positions. The distant partner's health
stays unchanged during each checked hit. After the Blade defeats the guard,
Blue heals 57.733 to 72.733 while Red retains her copy and resources; Red then
heals 87.667 to 100 without changing Blue's resources. Both clients independently
observe these grants. All 2,438 comparable sent/restored player, lives, checkpoint
and encounter states match; six coincident full local reports match as well.
Both target views and the guest pickup view were inspected, and both clients exit
normally. The normal entrance rope/cart are included, but later guards and the
exit are outside this focused fixture. The later full-route repeats below complete
the encounter comparison and the Pandemonium row.

Both full public routes now pass with the new encounter/player comparison on the
same `fe1f5fd9` build, again with 40 ms delay and 20 ms jitter:

- Blue leads: `private/coop/pandemonium-test-94511445d6bc41e9a4e92a1a50ec4f18/`.
  All four scenes belong to Blue, with reunions at ticks 2,701/14,409/34,878.
  A fresh guest rejoins near the exit (ticks 38,251/38,334), retaining both players'
  resources and completed scenes. All 5,997 comparable sent/restored states and
  31 coincident local reports match. Both reach Fortress alive, at 46.667/25.134 Sanity.
- Red leads: `private/coop/pandemonium-test-c3449c00e8bb41d0b6c7f39bb4729a98/`.
  All four scenes belong to Red, with reunions at 2,756/14,621/34,994. All 6,362
  comparable sent/restored states and 65 coincident local reports match. Both reach
  Fortress alive, at 30.001/45.201 Sanity.

Both retain personal Cards and independently collected healing rewards. The full
encounter records include all seven authored guards, six normal combat deaths,
and projectile attacks from both encountered Diamond guards. The distant return
guard stays alive, as on the ordinary route. Neither run uses respawns or wipes.
Rope, boarding-wait and initial Fortress cinematic views were inspected; those
arrival views do not verify the Fortress visit. Both client pairs and helpers
exit normally, and both invites are removed. Coverage summaries are retained as
`encounter-coverage.json` inside each fixture.

The first Fortress visit now has complete combined evidence (details and failed
development attempts are in `docs/COOP.md`): native paired split-room carry and
single-player route/cinematic regressions on `bac55928`; Blue-leading full public
route with fresh guest rejoin (`fortress-test-cc6abe54e07e4f2fac11ff3621b1c75d`)
and Red-leading full route (`fortress-test-3cf5044c3af64e3e97ed3ef09d70f751`)
on `fda92609`; focused Boojum targeting/personal healing
(`fortress-combat-e4a4260346de427c84f93e82e338f187`) and both combat-death
roles plus shared checkpoint restore (`fortress-recovery-6388fe74d6414e87a48f45ee92ceda44`)
on `2271c7db`. All public runs use 40 ms delay + 20 ms jitter and compare actual
sent/restored player/enemy state. The newer builds add opt-in diagnostics and
ordinary-input test controls; gameplay is unchanged from the paired-support build.
Its entry scene starts automatically with Blue as owner even when Red leads the
subsequent route. The split room is an environmental trap; passenger boarding is
not applicable. The later return visit and full Beyond the Wall traversal remain unverified.

Beyond the Wall now has native adapter evidence on build
`c17d9c2b1eedbd621fd56923648b8f38ec4a58bfd1ba1bae5ef39aac4c0e53c8`
(`private/coop-beyond-adapter-check5.log`). Staged checks on the actual map cover
paired Rage-lift carry at 30/60/144 Hz, either rider blocking both proposed carries,
rolling-bridge boarding, the later elevator's saved boarding wait without freezing
other machinery, survivor-only use, and either player's Rage pickup/scene/reunion
and musical-lever operation. Each player retains an individual Rage pickup.
These are focused fixtures, not entrance-to-exit routes or rendered/network proof.
The subsequent sound fixture on build `0944f214` adds public two-process evidence
for Red's musical demonstration, both reset-lever operators, identical numbered
notes, actual mixer recordings and a fresh guest rejoin without old-note replay
(`private/coop/beyond-audio-1dbe4744e4984a3394b41ce156a4aead/`). It compares 675
sent/restored states with 40 ms delay plus 20 ms jitter per sender. The start is
staged outside the musical trigger. Complete traversal/combat, death/wipe and
rejoin-to-exit remain open, so the completed count stays three of 39.
The final `10aa2112` build also passes the same musical fixture with Blue first
(`private/coop/beyond-audio-d97405d188c34927aa4e24529f05f29f/`) and a separate
unskipped six-line village opening with both clients' actual audio output
(`private/coop/village-voice-0e430b62bac8415a8ecf762320d21940/`). These focused
checks add sound evidence without granting another full campaign row.

Normal-entrance network traversal now reaches arch step 11 on build
`496972c89d1bfb2d2f9b3518f8f90e929ebb6e8ef62e09df31fb95650c5d57da`
in `private/coop/beyond-route-aa5a24b4065b48d7bb126d15146b0f9b/`.
Both independent clients cross the flipping corridor, solve the musical puzzle
with alternating operators, cross all nine rolling-walkway stages and take the
two authored portals using ordinary inputs. All three scenes have Blue as owner;
both players reach the arches at 100 Sanity, with no respawns. There are 7,143
matching actual sent/restored states under 40 ms delay plus 20 ms jitter.
The run stops on the next stair command: its test controller required less than
90 units of vertical separation between steps that remain 102.4 units apart.
This is partial traversal evidence, not an exit, rejoin or recovery pass.
The focused delayed public stair probe subsequently passes on build
`7eec625b6f796e7878748a243126ca79df900c889b7cc00a9f4e61e02f915424`:
`private/coop/beyond-arches-probe-0387775c95634254acd868903865bf8f/`.
Both players climb the moving steps, make the running jump to the upper door
and take the authored portal, with 1,027 matching sent/restored states and both
rendered upper-door views inspected. Its initial placement is staged outside
the rising-room trigger, so it adds no normal-entrance/full-row credit.
On that same build, a focused public corridor probe passes all flipping-platform
gaps and the authored portal in
`private/coop/beyond-corridor-probe-f26639b408ba470b82d9b3a6498ea8e7/`, with
1,709 matching states and both rendered 100-Sanity portal views inspected.
Its initial placement is staged before the corridor trigger. Subsequent full
repeats have not reached the exit: one stops at lever range, one at a Boojum
knockback on the spiral stairs, and one at a later corridor navigation gap.
The harness now uses precise lever approaches, broader combat waits and explicit
corridor jumps. The subsequent Blue-leading run reaches the actual Fortress
return in `private/coop/beyond-route-3ea9b2df6c034cffb7c1b83a3df0ccc9/` on
`7eec625b`: all three Blue-owned scenes, frozen partners, safe reunions, puzzle,
walkway, final door and 7,722 matching sent/restored states pass without respawns.
The original harness fails after successful travel because its reconnect check
still expects Beyond state. The retained route passes the same extracted route
validator (`audited-route-result.json`); the separate post-transition reconnect
audit also passes. The fixture now reconnects before the outgoing trigger.

Combat and individual earned healing pass on that build under 40/20 ms public
delay/jitter in `private/coop/beyond-combat-04edc0990c244e69995a04acf6a9d34f/`.
The guard selects nearer visible Red and then Blue, damages only each target,
and both collect separate healing copies. All 505 compared sent/restored states
and nine coincident full enemy/player reports match; target/pickup views were
inspected. Both real combat deaths and the shared earlier-checkpoint wipe pass
in `private/coop/beyond-recovery-6ca7a13ffb634602bc613cff7d893097/`: each role
spectates for exactly 1,200 ticks and returns at 25 Sanity, 48 units from the
survivor, with inventory preserved. The wipe restores personal resources,
placement, enemies and machinery. All 1,943 compared sent/restored states and
89 coincident full enemy/player reports match; recovery views were inspected.
The corrected Red-leading full repeat passes on the same build/settings in
`private/coop/beyond-route-26dba8d47c88465abb0ab7cc4a1eb6bd/`: all three Red-owned
scenes, safe reunions, no respawns, 7,885 matching sent/restored states and a
fresh guest reconnect before the exit. Both arrive at `fortress1_start2` at
100 Sanity. Guest stair/upper-door/return views were inspected, including live
Anode observation. Blue's obscured return capture exposed a missing camera
collider on the visible return portal. Build `cf86b004` corrects that camera
collision; the focused two-client repeat `fortress-return-camera-579dd23381c84f05aa0c6937d36aec74`
has clear stationary entrance views. Both native body positions also pass at
30/60/144 Hz, alongside the original Fortress cinematic/save regressions.

Focused public transport checks pass Rage boarding, ownership, a single scene,
paired 48-unit descent and individual power copies in both orders on `7eec625b`
(`beyond-rage-ffb2d9ac54324e4ca8b926673385d316`, Red; `beyond-rage-fdfee62536404a0d97d6c2633a33f0e6`, Blue).
They walk from the entrance and compare 956/965 sent/restored states. Their
middle-scene captures exposed a hidden owner body; `cf86b004` now draws its
authoritative Rage pose instead of hiding every gameplay body during a scene.
Rolling-bridge waiting/boarding passes under 40/20 ms public delay/jitter with
Red first (`beyond-bridge-5cd7763586a34c51919d81c6feec49c6`, `7eec625b`, 156 states)
and Blue first (`beyond-bridge-07f7c0ef71a64c2aa9a796aeade41fbb`, `cf86b004`, 170).
Those bridge probes stage the initial approach; both waiting/boarding views
were inspected. Final `cf86b004` Rage repeats pass with Red first in
`beyond-rage-7d58e582d71842dfaae8f083f4998a27` (936 matching states) and Blue first
in `beyond-rage-ca85ba3508fb4818846294edf437d0a2` (960). Both clients' middle-scene
views were inspected in each run: the owning Alice is visible with the correct
clothing and transformation pose, and the partner is hidden. The final build
passes 745 Rust tests, debug/release builds, seven source-package tests and
the Fortress cinematic plus native Fortress/Beyond co-op regressions.
Cycling-elevator public checks now pass with both first riders on `cf86b004`,
under the same 40/20 ms delay/jitter: `beyond-elevator-d57b738905834aa6a1ab2d1c730c3c59`
(Red, 811 matching states) and `beyond-elevator-da651fab20844150a026e2a640f1ebc2`
(Blue, 775). These staged upper-return approaches use the authored activation
trigger, then ordinary descent, boarding, riding and running dismount jumps.
They verify boarding wait, paired rise, dismount wait and empty return clock;
waiting/dismount/final views were inspected. Normal's actual outer landing is
used; the short side bridges exist only on Easy.

A further late-Rage pickup probe exposed a phase-reset freeze on `cf86b004`
(`beyond-late-rage-e16f3f79bbe141f387b6d401f0030397`). `ef3db23e` now preserves a
cycle already started before the pickup, using the existing saved timestamps.
Both public repeats pass: `beyond-late-rage-510a52db30ac4a858ec444cd09c10e84` (Red,
932 matching states) and `beyond-late-rage-0dedcf3a803a4016babffde8b517a11c` (Blue,
927). Scene-owner and paired reunion views were inspected; the original scene
advances, the hidden partner stays still and both resume supported on the lift.
Native watch/skip/pause/save/restore cases pass for both owners, alongside
original Rage/legacy/difficulty and Beyond world/cinematic regressions, all 746
Rust tests (one ignored), debug/release builds and seven source-package tests.
Logs and full identities are in `COOP.md`. Combined with the full traversal,
combat, resource, death/wipe and reconnect evidence above, Beyond's row passes.

| # | Map | Entrance | Co-op evidence |
| ---: | --- | --- | --- |
| 1 | `gvillage` | `Initial` | Passed (combined evidence above): both arrival orders, all scenes/reunions, nearest-visible guard damage in both roles, individual health/Blade pickups, matching encounter/player states, both death/spectator/respawn roles, saved-checkpoint wipe, trailing partner and fresh guest rejoin-to-exit; shared boarding N/A |
| 2 | `pandemonium` | `Initial` | Passed (combined evidence above): native and delayed public routes in both orders, rope/cart, four scenes/three reunions, personal Cards/healing rewards, nearest-visible guard damage in both roles, full-route encounter/player comparison, both hazard death/respawn roles, saved-checkpoint wipe, trailing partner and fresh guest rejoin-to-exit |
| 3 | `fortress1` | `Initial` | Passed (combined evidence above): both delayed public route orders, original scene/reunion, independent portals, paired split supports, upper jumps and exit, fresh guest rejoin, nearest-visible Boojum selection/damage in both roles, separate healing, coherent enemies, both combat death/spectator/respawn roles and saved-checkpoint wipe; shared boarding N/A |
| 4 | `fortress2` | `Initial` | Passed (combined evidence above): full traversal in both orders (Blue audited, Red clean exit), fresh guest rejoin before exit; both Rage, bridge and cycling-elevator boarding orders; late Rage on the cycling lift; notes/reset/rejoin with audio; nearest-visible guard damage and separate healing; both combat death/spectator/respawn roles and saved-checkpoint wipe; clear return camera and visible Rage owner |
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
