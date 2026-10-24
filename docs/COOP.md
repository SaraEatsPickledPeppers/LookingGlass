# Two-player co-op design

Development branch: `coop`, based on public main `0220970`.
This is work in progress. A policy test or a connected pair of windows does not
establish campaign support. Single-player behavior and saves must remain supported.

## Agreed behavior

- Two separate players: blue Alice and the same model with red clothing. Recolor
  clothing at runtime from user-supplied textures; do not distribute edited game assets.
- Enemies choose the nearest living player they can see, respecting their sight
  rules, occlusion and invisibility. Resolve ties consistently on the host.
- The first player to reach a scene triggers it. Original cutscenes show one
  Alice. At the end both players reunite at safe positions near the authored end.
- Lifts and other shared transports that fit both wait until both living players
  are inside their boarding areas. One-person scripted rides need a separate policy
  and must never strand the other player.
- Health, Will, consumables and pickup collection are tracked per player. One
  player's consumable pickup does not remove the other player's entitlement.
- Death starts ten seconds of spectating the survivor. Respawn near that survivor
  with low health. Initial tuning is 25/100 Sanity. If no safe position exists,
  keep spectating and retry; never spawn inside geometry or above a lethal drop.
- If both die, restore an earlier shared checkpoint and place both players safely.
- The main menu, including ESC, offers Co-op with Host and Join. Host makes a link;
  Join accepts that link. Internet joining belongs in the first playable build.
- Two processes on one PC must use independent settings, saves, inputs and cameras,
  communicating through the same network implementation used between PCs.
- Every campaign visit must be playable, including leaves, swimming and transports.

Implementation defaults to validate in playtests: no friendly fire; save ownership
on the host; world progression shared; personal menus do not pause the partner;
respawn timers suspend during cinematic control. Story items and their world effects
must be separated from consumable inventories so quest progress cannot deadlock.

## Architecture

The host owns enemies, damage, pickups, puzzles, scenes, movement validation,
checkpoint restoration and level transitions. A guest sends bounded player inputs;
it cannot upload arbitrary save state or decide damage. Both processes render their
own camera, HUD and character presentation. The world continues when a window loses
focus; the unfocused player's controls become neutral.

Existing fixed-step movement, stable per-visit entity IDs and save snapshots provide
building blocks. Existing map controllers take one player and need explicit two-player
adapters. Merely running two independent worlds and exchanging positions is insufficient.
Host state snapshots and ordered events must reconcile the guest's world and predictions.

Network setup must report progress, handle cancellation, reject expired/malformed links,
limit connection/message sizes, verify compatible builds/data, and clean up owned helper
processes when hosting ends. Invite tokens are session secrets and must not be committed
or printed in public logs. Input must time out to neutral after a disconnect.

The development internet transport uses Cloudflare Quick Tunnels, which support temporary
public URLs without an account, but are explicitly a development service with no uptime
guarantee; this cannot be silently treated as a production service commitment.
See [Quick Tunnels](https://developers.cloudflare.com/tunnel/get-started/quick-tunnels/)
and [WebSockets](https://developers.cloudflare.com/network/websockets/).

## Delivery and verification weighting

Overall percentages are engineering estimates against these fixed weights, not elapsed
time or a count of files. Credit requires working behavior and relevant evidence.

| Area | Weight | Required evidence |
| --- | ---: | --- |
| Foundation and one-PC test setup | 10% | Branch, agreed rules, independent two-process harness, useful diagnostics |
| Internet Host/Join | 15% | Main/ESC menu, real public invite connection, cancellation, disconnect/reconnect, bounded protocol |
| Two players and movement | 15% | Blue/red clothing, separate cameras/input, synchronized walking/jumping/climbing/swimming and moving supports |
| Combat, personal resources and death | 20% | Nearest-visible AI, coherent damage, individual pickups, timed spectating/respawn, two-player wipe/checkpoint |
| Scenes, transport and progression | 20% | First arrival, single-Alice scenes, safe reunion, boarding barriers, shared saves and level changes |
| Full-campaign verification | 20% | Every visit exercised by both roles, leaves/swimming regression, latency/disconnect tests, single-player regressions |

The [campaign matrix](COOP_CAMPAIGN.md) includes all 39 visits. Each requires both trigger orders, a player
dying during traversal, both dying, checkpoint reload, a guest falling behind, and a
disconnect around a transition. The network test harness should inject delay/jitter
and exercise slow readers rather than assuming loopback performance represents the internet.

## Current evidence

- `coop` exists from current public main.
- `src/coop/rules.rs` implements policy decisions and scenario tests. The death
  policy now drives the movement slice below; scene, boarding and targeting
  policies still need campaign integration. All six initial policy
  scenarios pass: countdown, simultaneous death, saved/cinematic countdown,
  visible targeting, scene ownership/reunion and the boarding barrier.
- Authenticated, bounded WebSocket transport runs on background threads, with TLS
  on public invites, one guest slot, reconnect generations and delay/jitter injection.
  Slow readers disconnect without growing an unbounded queue. Invite secrets stay
  in link fragments and authorization headers, never normal logs or command lines.
- `--profile <folder>` isolates settings and saves. The two-process network probe
  uses the same profile resolution, transport and public tunnel as the future UI.
- On 2026-10-23, the current transport passed 681 Rust tests (one unrelated ignored
  test), a two-process loopback run at 60 ms delay + 30 ms jitter per sender, and
  a two-process public internet run at 40 ms + 20 ms per sender. Both exchanged
  16 ordered messages, verified independent profiles/PIDs and observed clean guest
  shutdown. The internet guest joined immediately after invite publication.
- New tunnel hostnames initially failed DNS resolution when published too early.
  Hosting now waits for tunnel registration and a 12-second propagation grace.
  This passed repeated public tests; it is not a universal DNS availability guarantee.
- Main/ESC Host/Join now opens the development movement/combat slice. Full-campaign
  integration remains unfinished. These transport probes verify
  networking; the later rendered checks exercise two independently controlled Alices.

The next session layer (`protocol.rs`, `session.rs`, `hosting.rs`) checks exact source
and dependency identity plus the mounted game-data fingerprint. Both roles must
acknowledge loading a visit before guest controls become active. Every reload or
transition has a new epoch: old controls cannot leak into the next visit. Held
movement/attacks become neutral after 500 ms without fresh input, while cumulative
jump/use counts preserve short presses across a busy send queue. An incompatible
guest releases the slot so the same host can accept a correct build. Host setup,
helper download and cancellation run off the UI thread.

The development movement viewer and main/ESC co-op menus now use these APIs;
the campaign still needs integration. The source identity is a compatibility check, not an
attestation that a peer is trustworthy. Guest messages cannot upload world state.

The session changes passed 689 Rust tests (one unrelated ignored test), including
real-socket scenarios for readiness, a transition with input still in flight,
incompatible data followed by a successful rejoin, disconnection and loading timeout.
The shared background Host setup also passed separate-process loopback and public
internet probes with the delay/jitter above. The public probe and development
movement viewer both wait for helper shutdown before exiting.
After tightening cancellation and the forged-message regression, all 21 focused
co-op tests passed again. This final focused run includes the latest source changes.

Overall estimate after completing the village, Pandemonium, both Fortress visits and Beyond verification rows: **about 42%**. Credited
points are 8/10 for foundation/diagnostics, 7/15 for internet Host/Join,
9/15 for two-player movement/presentation, paired school supports, dry/swimming arrival placement, complete delayed village/Pandemonium traversal in separate clients and rendered independent rope controls, and 7/20 for native targeting, host-owned
weapons/damage, personal resources, timed respawns and full saved-state team wipes,
plus 8/20 for shared world replication, ordered contacts, the verified theatre
ownership/reunion slice, school boarding barriers, visit history and restart/backup
restoration of shared checkpoints, and native traversal of all six village scenes
and all four Pandemonium scenes with either player arriving first (also verified through public internet clients).
Full-campaign verification now earns 100/39 points (about 2.56/20), for the village,
Pandemonium, both Fortress visits and Beyond's complete combined evidence. Total
credited points are about 41.56, rounded to 42%. Standalone policy tests receive
no campaign credit; 34 visits remain open.

Pandemonium now uses each replicated player's own rope grip in co-op, while the
single-player controller and saved grip remain unchanged. The host advances its
shared machinery once and includes both living bodies when checking quest-gate
obstructions. Either player can collect the shared Gnome key. The authored
minecart trigger waits for both living players in its boarding volume, retains the
first requester across a snapshot restore, then runs its original one-Alice ride
and reunites the pair at the far landing. Scene takeover releases old rope grips.

`--coop-pandemonium-check` passes staged real-geometry checks for both first-player
orders: simultaneous grabs on one rope, opposing climbs, wire roundtrip of both
grips, personal release, a two-second boarding wait, requester retention, one
complete minecart ride, safe reunion and the partner collecting the key. Evidence:
`private/coop-pandemonium-adapters1.log`, build
`d5443db066562bb26284db40f43d3e55a91805ade8aac6f787de61f0dea6e5aa`.
These first checks use the real host arena with explicitly staged starting positions;
they do not establish a full route, rendered rope fidelity or separate-client play.

The original watched and skipped single-player Pandemonium routes, state/save
checks and the school/village co-op world regressions also pass. The watched route
finishes in 26,139 ticks with nine jumps, 13 throws, one portal and 47.334 Sanity.
Evidence: `private/coop-pandemonium-singleplayer1.log`,
`private/coop-pandemonium-singleplayer-skip1.log`,
`private/coop-pandemonium-singleplayer-state1.log` and
`private/coop-pandemonium-world1.log`. All 734 Rust tests pass (one pre-existing
ignored test). The newer route and presentation evidence follows below.

The co-op renderer now deforms each rope through the players' animated supporting
hands. Its loose tail follows the physical grip, so a hand-placement animation
cannot inject velocity into the strand. Dynamic mesh indices follow the added
hand rings. The two players retain independent checked pendulums; this is not a
coupled two-person load solver. Unit coverage checks player-order independence,
exact hand contacts, finite geometry, pause, either player's release and the
invariance of tail physics under animated hand movement. Attachments are ordered
by body height and samples follow that path, so crossing radial lengths while
swinging cannot place the tail above the lower player. Single-player continues
using its original physical grip and rope shape.

`--coop-pandemonium-route-check` now passes from the normal entrance to the
authored Fortress exit with both first-arrival orders, using ordinary inputs and
no position/resource/world writes. Both runs take 33,993 ticks, retain a personal
Cards pickup each and finish at 60.934 Sanity each without a death or checkpoint
rollback. Each first player owns all four scenes (warning, minecart, Gnome return,
airship), with three safe paired reunions before the outgoing transition. The
original return trigger previously only set a progression flag: it now starts the
existing return cinematic, which the single-player route also explicitly checks.
This result is a single-process two-body simulation, not a network traversal.

Evidence: `private/coop/pandemonium-route6/coop-pandemonium/` and
`private/coop-pandemonium-route6.log`, build
`a9b50ed49c886e859e6a0a9cbcaec0d833fd08d799a4e1f74527fa0626da844b`.
The staged adapters repeat successfully on that build in
`private/coop-pandemonium-rope-native8.log`, with rope/cart captures under
`private/coop/pandemonium-rope8/coop-pandemonium/`. The shared-rope and return-reunion
captures were inspected. The accelerated checks advance the tail every simulation
tick rather than only when drawing an occasional frame.

Earlier failed routes remain under `private/coop/pandemonium-route1` through
`pandemonium-route4`: respectively an incorrect height goal on the sloping return
street, excessive waiting on damaging ground, the disconnected return cutscene,
and a portal check that did not allow time for the restored scene. The driver now
follows horizontal goals on that street, moves both players through the hazard
together and waits for the first portal arrival's scene/reunion. These fixes do
not change movement speed, level damage or the requested first-arrival policy.

All 736 Rust tests pass (one pre-existing ignored). The watched/skipped
single-player routes, cinematic pause/save/skip checks, machinery state checks
and school/village co-op world regressions pass on the preceding
`b5b78ca663f3e3c6fa94aa526cdf5479b95b2a09a1cbbf89cc39eea1ba005b3d` build, in
`private/coop-pandemonium-sp7-{route,skip,cinematic,state,world}.log`. The watched
route takes 26,897 ticks and finishes at 47.334 Sanity; the skipped route takes
25,529 ticks and finishes at 32.334 Sanity. Both retain nine jumps, 13 throws and
one portal. The native single-player machinery/cast/rope render checks also pass
in `private/coop-pandemonium-sp-render7.log`; the rope ascent capture was inspected.
The final co-op native repeat used the authorized PC fallback while Anode was
busy; the preceding full native render checks ran in Anode. All error logs are
empty and all owned game processes closed. The final change only orders multiple
co-op grip attachments; the single-player one-grip path is unchanged. The later
public separate-client traversal, recovery and rejoin evidence follows below.

Run the staged check in Anode after building:

```text
looking-glass.exe --data <base-folder> --coop-pandemonium-check --no-audio --profile private/coop/pandemonium-adapters
looking-glass.exe --data <base-folder> --coop-pandemonium-route-check --no-audio --profile private/coop/pandemonium-route
```

The first public two-process Pandemonium attempt exposed a delayed rope-use race:
Red's ordinary airborne physics caught the rope before her grab press reached the
host, so the old toggle interpreted that press as a release. She fell to damaging
ground and died; this is a failed traversal, retained in
`private/coop/pandemonium-test-6254d8209f784c2ab361f596a8e0b850/` on build
`5c1a8e34ba15f14de44cc192876da422cab3ee147bad0851bbf2b7081fcb9b5f`.

Protocol revision 4 separates cumulative grab and release edges according to the
local player's observed grip and nearby rope. The host consumes each edge once and
only requests the matching ordinary rope action: a late grab cannot release an
existing grip, and a late release cannot attach a detached player. Edge counts
remain bounded and expire under the input watchdog. Dialogue, doors, movement
locks and single-player controls keep their existing paths. The regression test
reproduces automatic attachment followed by a delayed encoded grab, then verifies
an explicit release, duplicate suppression, timeout and counter bounds. All 740
Rust tests pass (one pre-existing ignored) in
`private/coop-pandemonium-network-rope-tests2.log`.

The independent-client driver now supports rope controls, horizontal goals and
local map markers using ordinary inputs. The Pandemonium harness checks shared
rope descent, minecart boarding, all four scene owners/reunions, personal Cards
and the authored Fortress transition. Run it in either first-arrival order:

```text
python tools/test_coop_pandemonium.py --data <base-folder> --first blue --delay-ms 40 --jitter-ms 20
python tools/test_coop_pandemonium.py --data <base-folder> --first red --delay-ms 40 --jitter-ms 20
```

The Blue-first repeat passes through the public invite connection with 40 ms delay
and 20 ms jitter per sender on build
`ec4ccf71f5a634f1c8c03ccbc6924fa3492e9fac1421dd813cd4937c06118a48`.
Both clients agree on all four Blue-owned scenes and three reunion ticks, retain
their actual personal Cards across the Fortress transition, and match complete
player state at 207 identical authoritative ticks. There are no deaths, respawns
or checkpoint resets; Blue arrives with 27.134 Sanity and Red with 21.135.
Shared-rope, landing, boarding-wait and initial Fortress cinematic captures were
inspected. The last capture is during Fortress's arrival sequence; it does not
verify that next visit. Both client processes exit normally and the private invite
is removed. Evidence: `private/coop/pandemonium-test-25726e42ca974a459cf70cd356606c0e/`,
`private/coop-pandemonium-network-blue2.log`. This run used the authorized local
desktop fallback while Anode was busy. Later recovery/rejoin results follow below;
consumable, first-guard and full-route encounter comparison checks now follow in
the encounter section and complete the Pandemonium verification row.

The Red-first repeat also passes on the same build and delay/jitter settings,
running in Anode. Both clients observe all four Red-owned scenes, three reunions,
the delayed cart departure and personal Cards carried into Fortress. They agree
on 89 matching authoritative ticks; Blue arrives with 39.867 Sanity and Red with
23.468, with no deaths, respawns or checkpoint resets. Both rope views, Red's
boarding-wait notice and the initial Fortress cinematic view were inspected.
Both game clients and the test job exit normally (job exit 0), and the invite is
removed. Evidence: `private/coop/pandemonium-test-a2cf6558bee64719af5fda4834594ca6/`,
`private/coop-pandemonium-network-red1.log`. Python compilation and all 19 existing
setup/source-package checks pass. No campaign row is promoted to passed by these
clean traversals alone.

Pandemonium now has separate ordinary-input recovery and fresh-process rejoin
checks. The recovery test follows the real entrance rope to the warning scene,
then sends each player onto the damaging street while the survivor walks on the
supported landing. After both individual recoveries, it sends both low-health
players into the hazard and compares the result with the game's earlier automatic
checkpoint. It never writes health, positions, inventories or shared world state.
Placement is checked on each client's first restored frame, required within one
simulation second, allowing normal horizontal movement during the recorded host
ticks since restoration; resources
and the selected quest/scene completion fields compare exactly. The rejoin check
closes the guest process before the airship exit, starts a new one using the same
invite/profile, compares both players' resources and completed scenes, then
continues through the authored transition.

```text
python tools/test_coop_pandemonium_recovery.py --data <base-folder> --first red --delay-ms 40 --jitter-ms 20
python tools/test_coop_pandemonium.py --data <base-folder> --first blue --rejoin --delay-ms 40 --jitter-ms 20
```

The first recovery attempt verified both ten-second respawns but its shared-wipe
approach snagged scenery. The second attempted approach also snagged scenery;
the test now uses the ordinary route planner to reach a point farther into the
damaging street. The third and fourth runs reached the wipe but a fixed 20-unit
placement assertion did not account for movement before the reports arrived.
The verification now uses the first restored samples and bounds horizontal drift
by the normal 320-unit/second maximum and the exact elapsed host ticks; the
vertical platform tolerance remains 20 units. These diagnostic runs are retained
under `private/coop/`: `pandemonium-recovery-f763a479554d4031b5e41e2a707fc4a7`,
`pandemonium-recovery-13bddb5bb8074e8cb0d65cb96d74041e`,
`pandemonium-recovery-1d4418c09da24a8fbe93c9f757a0a9ee` and
`pandemonium-recovery-3f1b82a621884511bf09cd1daaa1e2c1`.

The fifth recovery run passes on build
`ec4ccf71f5a634f1c8c03ccbc6924fa3492e9fac1421dd813cd4937c06118a48`, over the
public invite with 40 ms delay and 20 ms jitter per sender. Red dies at tick 6,054
and revives at 7,254; Blue dies at 8,962 and revives at 10,162. Both clients agree
on each exact ten-second interval, each 25-Sanity revival 48 units from the
survivor, both moving spectator cameras, retained inventory and the survivor's
unchanged health. The team wipe at tick 11,170 restores the actual automatic
checkpoint saved at tick 1, including full personal resources and the earlier
quest/scene flags. The first host and guest placement samples, respectively 4 and
12 ticks after restoration, have zero horizontal displacement from the saved
positions. Both spectator views and both restored-checkpoint views were inspected.
Both clients and the Anode job exit normally; the invite is removed. Evidence:
`private/coop/pandemonium-recovery-1127245225934d65ac70d7f9af173184/`,
`private/coop-pandemonium-recovery-internet5.log`. This is the entrance-side
hazard/recovery scenario; it ends at the restored checkpoint without an outgoing
transition. The traversal and rejoin tests exercise the later route separately.

The fresh-guest rejoin run also passes on that build and public delay/jitter
settings. Blue leads the full route; Red closes her original process just before
the airship and joins again in a new process. The before/after reports preserve
both players' Sanity, Will, owned/collected items, equipped weapon, school items,
staff components and difficulty, as well as the minecart, key and completed-scene
flags. Both continue into Fortress with their personal Cards (Blue 45.734 / Red
15.668 Sanity), without deaths, respawns or checkpoint resets. All four owners,
three reunions and 126 matching authoritative ticks pass. The final initial
Fortress cinematic views were inspected; both clients and the Anode job exit
normally, and the invite is removed. Evidence:
`private/coop/pandemonium-test-b5965aedc3274cba8065c2c111d4421f/`,
`private/coop-pandemonium-rejoin-internet1.log`. The game source is unchanged from
the preceding 740-test build; this checkpoint extends the independent-client
harnesses and records their evidence. Later focused consumable/guard checks are
recorded below, followed by both successful full-route encounter comparisons.

The native enemy combat foundation now supplies two live targets to each enemy
and keeps damage, knockback and Will drain separate for Blue and Red. Closest
visible selection respects cover, death, invisibility, card-guard sight cones and
club-guard home limits. An attack retains its valid victim while it finishes;
ordinary last-seen pursuit continues when both players take cover. Projectiles
can hit either living body regardless of the enemy's current aim. Clockwork steam
uses separate hit bits so both players can be hit once by the same attack.
The live roster is omitted from saved NPC state, preserving the single-player
save representation. Feedback aggregation preserves both players' results.

The combat-foundation revision passed **703 tests, one unrelated ignored test**.
`Npcs::update_players` advances a cast once and lets either living player reach its
ambush triggers. The current viewer integration adds host-owned weapon simulation,
guest NPC/weapon replication and individual pickup presentation, as described below.
Two summoned demons, boss/controller adapters and shared world checkpoint restoration
remain unfinished. No campaign matrix row is credited by these tests.

The headless `--combat-check` also passes with user-supplied school geometry and
the actual club-guard animation timings. At each of 30, 60 and 144 Hz, the closer
Red Alice receives 60 damage during the first 12-second phase and Blue receives
none. After a guard-state serialization roundtrip and Red's death, the survivor
Blue receives 84 damage in the next phase and Red receives none. Final position
and damage agree across all three frame rates. The existing single-player pursuit,
Blade contact and defeat checks pass alongside this. Reproduce with:

```text
cargo test --locked
cargo build --locked
target/debug/looking-glass.exe --combat-check --data <original-base-folder>
```

### Integration constraints found in the existing engine

- `viewer.rs` owns one player, inventory, character, camera and save record. Add
  explicit per-player state while keeping world time and progression on the host.
- `Menu::run` currently waits in a modal loop. Co-op needs a per-frame menu update
  so one player's settings/menu and window focus cannot freeze the other's world.
- Level controllers advance shared machinery and enemies through single-player
  hooks. Do not call a shared controller twice per tick to process two players;
  split world advancement from per-player contacts and rider displacement.
- Enemy selection must be per enemy, not one globally selected player for the
  whole level. Route damage/impulses to the player that actually received the hit.
- Alice's blouse/leg/inner-mouth regions share a texture. Runtime red clothing
  needs a blue-fabric mask on known clothing textures, preserving skin, apron,
  mouth, hair, and power-up appearances. Never distribute altered original assets.

### Shared world contact adapter

`interaction::coop` separates two players' trigger occupancy/cooldowns from the
shared event runtime. Both living players can receive hazard damage or use a
teleporter independently. Simultaneous contacts with a shared trigger fire it once,
with arrivals ordered by swept contact time, then stable identities for exact ties. Each
activation retains its player owner for scene/exit handling. Shared delayed events
advance once even when both players have contacts. The existing event gates and
one-shot rule history remain authoritative.

Door use shares the single-player implementation. Simultaneous use of the same
paired leaves toggles them once, and the door's swept arc checks both living
bodies. Personal contact history can be serialized beside the interaction snapshot
and cleared for one respawn without rearming shared one-shot events. The original
single-player save representation is unchanged.

The rendered co-op viewer now owns this shared world on the host and replicates
its event history, contacts and story state. Controller/mover adapters and complete
campaign progression still need integration. No campaign-matrix credit is assigned
merely for adding this interface. Its actual-map
check is `--coop-world-check --data <original-base-folder>`; `--world-check` remains
the single-player regression.

The interaction refactor passed 717 Rust tests (one unrelated ignored test).
Eight new cases cover staggered and simultaneous hazard contacts, independent
teleports, both first-arrival orders, competing exits in one update, repeatable
trigger reentry, delayed world time, contact snapshot validation/restoration and
simultaneous door use with either body blocking its swept arc. These are engine
checks and do not establish a completed two-player campaign visit.

The actual-school check passed with either Blue or Red opening the door at 30,
60 and 144 Hz. Both bodies first walk against the closed door for long enough to
stop, remain blocked for another second, then cross after one use. The authored
fatal-fall volume damages both independently; a shared snapshot/contact roundtrip
and resetting Blue's contacts leave Red's cooldown intact. Evidence is in
`private/coop-world-native.log`.

All 39 initial world snapshot hashes, event signatures and byte lengths match the
pre-refactor executable. This was compared independently from the `HASH` records
against `private/coop-world-before-registry.json`: the existing registry comparator
intentionally skips registered controllers, so its summary alone is insufficient
for this claim. Evidence is `private/coop-world-after-registry.log`. These are
single-player initialization/save-format checks, not co-op campaign traversal.

The broad single-player world check exposed an older driver assumption: its steam
floor route allowed two seconds to walk more than 368 units at 104 units/second.
Both the saved pre-refactor executable and this revision stop at the same position
(`y=2637.4326`). The driver now derives its duration from the current walk speed;
its required endpoint, support and collision assertions are unchanged. Failed
pre-fix logs remain in `private/coop-world-before-single.log` and
`private/coop-world-single.log`.
After that driver correction, `--world-check` passes all three school-door rates,
all eight floor supports, both visits' seam crossings at 30/60/144 Hz, and fatal-fall
reset. Each seam route ends at `y=2845.4404`; evidence is
`private/coop-world-single-final.log`. Seven source-package tests also pass.

Live contact dispatch now resolves personal damage/teleport immediately, ending
that body's old sweep before it can consume a later scene or exit. The partner's
contacts remain eligible, and teleport cooldowns survive relocation.

### Live world and school theatre integration

`coop::stage` owns shared interactions, contacts, story, hints and the active scene
owner. Scene ticks advance the original single-Alice controller exactly once,
freeze the partner and hold respawn timers. Both windows use the replicated
original camera, dialogue and level art. Completion finds a supported nearby
position outside hazard volumes before returning control to both players.
Relocations cut camera/body interpolation without being counted as death respawns.
An entrance wipe restores shared world/story/contact state before checking body
placements, together with native NPC and character state.

`--coop-world-check` now also runs the actual theatre for both first-arrival roles:
5,592 fixed ticks each, frozen waiting partner, camera/mover snapshot agreement,
48-unit safe reunion and full shared-state reset. The check passes scene state
through the same compressed host message envelope used on the network.
Single-player `--world-check` and `--school-check` pass; all 39 initial visit
records match the retained pre-refactor baseline independently of the registry's
comparison exemptions. These checks do not establish a completed campaign visit.

The first rendered attempt exposed an envelope problem: the old internally tagged
message buffered integer entity-map keys in a form that could not be restored.
Host messages now use an external tag, with a regression covering real interaction
snapshots. Protocol revision 2 and the exact build identity prevent mixed clients.

The intermittent initial-join failure also reproduced against the retained older
executable: a valid HTTP upgrade delayed by 50, 200 or 500 ms was aborted every time.
Windows accepted sockets inherit the listener's properties
([Microsoft documentation](https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-accept)); the bounded
synchronous upgrade now explicitly selects blocking mode before authentication,
then returns to nonblocking exchange. The regression forces the inherited mode
even on other platforms. `tools/test_coop_handshake.py` exercises the production
listener with delayed bytes without using game assets. Failed pre-fix evidence:
`private/coop/handshake-test-a241a70853194f21845271a9abdedc0e`.

`tools/test_coop_scene.py --data <original-base-folder> --first blue` starts two
independent rendered clients near the theatre. The chosen player walks into the
authored trigger; neither driver starts a scene directly. Use `--first red` for
the other arrival order, `--local` for loopback, and `--delay-ms` / `--jitter-ms`
for network conditions. It verifies scene ownership, partner immobilization,
camera movement, matching authoritative reunion ticks and nearby living bodies,
and captures the scene and reunion. This is a scene fixture, not a traversal test.

Rendered checks on 2026-10-23 passed with Blue first on loopback and Red first over
the public TLS tunnel with 40 ms delay plus 0–20 ms jitter per sender. Both windows
observed the same owner and reunion tick, and a 48-unit separation. Captures were
inspected; the final presentation uses the original scene field of view/fades and
suppresses the ordinary gameplay HUD. Evidence under `private/coop/`:
`scene-test-538e7bf8119543b88d43e770ed292147` (Blue, before final presentation tuning),
`scene-test-459a4e14b3724b5ba9ebe5769871a9b9` (Red, final presentation).
The final production-listener regression accepts upgrades at all three delays:
`handshake-test-1449f00e111d4c608db6c5e3dd843f94`.

At the theatre checkpoint, the full Rust suite passed 723 tests (one unrelated ignored test), including
the real entity-map envelope and delayed-upgrade regressions. Source-package (7),
portable setup (6) and Windows setup (6) tests also pass. Final headless evidence:
`private/coop-stage-verified-tests.log` and `private/coop-stage-verified-native.log`.
A subsequent public recovery attempt did not reach gameplay because the guest
could not resolve its newly created tunnel address; retain
`private/coop/play-test-72524584443844d1b1d5ad372ddd5d9c`. The current DNS grace
period does not guarantee immediate invite availability. That remains an internet
hosting reliability limitation; the successful theatre run does not erase it.
The local recovery regression with the same 40/20 ms conditions passed: Red died
at tick 2,077, returned at tick 3,277 with 25 health and 48-unit separation, then
both clients restored the checkpoint at tick 4,261. Shared world restoration did
not change the ten-second timer or spectator/recovery camera behavior. Evidence:
`private/coop/play-test-d49bd2e45c5d402a9256785059bbf298`.
The final local Blade-combat regression also passes with the shared world enabled:
both players attack, both clients see the same defeated guard, Blue remains at
100 health and Red reaches 64. Evidence:
`private/coop/play-test-46f596d015874301b65afbe05631577b`.
All owned game and tunnel processes were gone after testing.

The level-change/load-barrier path is connected and carries each player's own
resources, inventory and death countdown. Full round-trip campaign state caching,
shared saves, controller combat, moving supports beyond the school adapter below, co-op audio,
and live transition/disconnect coverage remain unfinished. Free-play controller
updates outside the school remain pending; one theatre passing cannot
stand in for those systems or for the 39-visit campaign matrix.

## Shared school lifts

The host now advances the school machinery once per world step. A moving support
plans both living riders' positions before committing either body, its own collider
or its motion clock. An obstruction above either rider, or a partner below a
descending support, stops the movement without a partial carry. Jumping bodies
are not dragged along. The single-player path uses the same helper with one body.

The spiral and ordinary elevator wait for both living players before activation
and before leaving either endpoint with a passenger. Empty return trips remain
available. The observatory lift's one-shot contact also waits for both living
players. Pending requests preserve the first living requester across compressed
world/contact snapshots; a dead requester yields to the survivor. Waiting does
not consume the authored trigger or its cooldown. A notice explains the wait.
The observatory check stages an already-open gate; its complete quest/cutscene
route is not yet verified in co-op.

`--coop-world-check` now covers all three boarding contacts in both arrival orders,
with both alive, the waiting partner dead, and the original requester dead. It also
checks the two ordinary lifts through complete cycles at 30/60/144 Hz, keeping
both bodies clear before and after physics, preserving rider offsets, restoring
mid-trip state, holding at both endpoints, returning empty and carrying a sole
living rider. Obstructions over either rider stop both bodies and the lift clock;
removing the obstruction resumes travel. The integrated host world loop passes
both ordinary lifts with Blue first and Red first. Theatre/reunion/wipe checks
still pass. Evidence: `private/coop-lifts-native4.log`.

The first fixture used an outside spiral position that encounters the landing
lip near the top. The safety check correctly stopped both riders; the unobstructed
full-cycle fixture uses the inside position. Earlier fixture setup errors used a
starting point above the elevator deck and an incorrect assumed body height for
the temporary ceiling. Their failed logs remain private; they are not counted as
passing runs.

The full Rust suite passes 726 tests (one unrelated ignored test), including three
atomic carry regressions. Single-player school state/movers, shelf footing at
30/60/144 Hz and continuous school traversal pass. All 39 fresh visit records are
exactly equal to the frozen pre-refactor baseline, including registered controllers.
Evidence: `private/coop-lifts-tests.log`, `private/coop-lifts-school1.log`,
`private/coop-lifts-footing1.log`, `private/coop-lifts-school-route1.log`, and
`private/coop-lifts-registry.json` compared with `private/coop-world-before-registry.json`.

`tools/test_coop_lift.py --data <original-base-folder> --lift elevator --local
--delay-ms 40 --jitter-ms 20` runs two independent rendered clients on an actual
lift. It uses the ordinary Host/Join and contact/movement pipeline, with staged
starting positions and no injected world updates. Wait for the full-cycle check,
capture each window with F12, then close both windows. Omit `--local` for public
TLS; `--lift spiral` selects the rotating lift. This fixture verifies the ride;
arrival-order and obstruction coverage comes from the native checks above.

The delayed loopback elevator check passed on 2026-10-23: both riders reached
696.03125 and returned, with aligned body/mover snapshots in both clients. Both
captures were inspected and both processes exited successfully. Evidence:
`private/coop/lift-test-e98f064766774ad08854c7ba79f3192e`.
The rotating lift also passed through the public TLS tunnel with 40 ms delay plus
0–20 ms jitter per sender. Both riders reached 64.03125, completed the return and
stayed aligned with the shared support in both windows. Captures were inspected
and both clients exited successfully; no game or tunnel helper remained. Evidence:
`private/coop/lift-test-3e9dd3b1787b413d8767094b6d70fb16`. This successful connection
does not remove the previously recorded fresh-hostname DNS limitation.
Every campaign-matrix row remains **Not run**.

## Shared visit history and arrival placement

The host now retains each departed visit's shared controllers, story/hint state and
native NPC state for the lifetime of the session. Canonical first entrances share
one cache key; authored return visits remain separate. Re-entering restores that
world before placing the pair, resets their old contact sweeps, and makes the
restored world the new entrance checkpoint. Individual resources and pickup
entitlements travel with each player's roster entry, never with an old world copy.
Destination geometry, restored collision, arrival positions and art are prepared
before committing the local visit history. A load error still ends the co-op session;
there is no persistent shared save/resume UI or recoverable failed-transition UI yet.

Protocol revision 3 adds an ordered guest loading acknowledgement. The host keeps
draining old controls until it receives that acknowledgement, then begins its
synchronous asset load. The queue stays bounded and retains its ordinary stall
deadline; loading does not require increasing either. This addresses the public
travel run in `travel-test-7f597a3d82be4a71a8d81999563481e8`, where late controls
filled the host queue during asset loading and disconnected the guest.

Partner placement accepts clear swimming positions in the survivor's water volume.
Dry placement checks a complete footprint against the local floor plane, including
slopes. Entrance placement can search up to 256 units below an authored elevated
start; ordinary revival remains limited to 32 units down. Walls, narrow ledges,
harmful liquids and hazardous destination triggers remain rejected. These changes
are confined to co-op placement; single-player recovery is unchanged.

`--coop-campaign-check` checks all 39 authored entrances for two clear bodies,
two seconds of collision/swimming physics, world snapshot roundtrip and an entrance
wipe. It deliberately does not advance the level controllers or establish traversal.
All 39 pass (`private/coop-progression-entrances4.log`). The underwater temple and
first garden entrance also pass both death roles: ten seconds of following the
living swimmer, then revival at the same depth with 25 health and fresh air. This
uses real entrance geometry and ordinary movement/death policy, without enemy AI.
The shared-world suite still passes its school lifts, theatre reunion and
revisit-checkpoint rollback (`private/coop-progression-world2.log`).

`tools/test_coop_travel.py --data <base> --reconnect --delay-ms 40 --jitter-ms 20`
uses two rendered clients and a public invite (add `--local` for loopback). Its
explicit `--coop-travel-check` driver opens the first school door with ordinary input,
requests a normal load barrier to the return entrance, disconnects/rejoins the guest,
then requests the first entrance again. It checks exact world/NPC restoration,
separate visit histories, personal resources, paired relocations and clean exits.
The destination requests are staged; this does not exercise the quest-gated exits.

On 2026-10-23, the public-invite run `travel-test-9b33f710636c4e03a1a54e81c9830d98`
passed all gameplay assertions at 40 ms delay + 20 ms jitter per sender, including
guest replacement, three visit epochs, exact cached world/NPC restoration and
preserved inventories. Both final views were inspected on the user's desktop after
Anode became occupied. The run's `ready.json` records those gameplay checks. The
manual capture/close stage timed out because the foreground automation's F12 did
not produce game captures; therefore this is not a completely green harness run
or proof of clean voluntary exits. Its cleanup removed the owned clients, tunnel
helper and invite. The earlier loopback run `travel-test-9ec9dd5f7d1c43048b14e2d791f76183`
also passed gameplay assertions but exceeded its manual inspection deadline.
The subsequent public run `travel-test-b41a73f3337d46a2ada6b94db1d03b3a`
passed the entire harness on Anode at the same delay/jitter, including guest
replacement, automatic captures, clean exits and helper/invite cleanup. Both
final captures were inspected. The fixture captures after five seconds in the
returned visit, then closes the guest and host itself; it requires no desktop
input. The full Rust suite at the travel milestone passed 730 tests, with one pre-existing
ignored test (`private/coop-progression-tests3.log`), including the new slow-load
barrier regression. No co-op release or full campaign playthrough is claimed.

## Shared co-op checkpoints

The host owns a separate `coop-checkpoint.json` in its settings/profile folder.
It contains both bodies and individual inventories, character actions/projectiles,
the shared stage and NPC state, and cached first/return visits. The host's ESC
menu can save while both players are alive and outside a cutscene/transition.
Normal sessions also save at the first freely controlled moment of each chapter;
leaving co-op does not silently replace a manual checkpoint. Explicit development
fixtures do not autosave into the user's checkpoint.

Continue co-op restores the saved chapter even if another chapter is selected in
the main menu. Its network epoch starts fresh and a new invite is issued. Red's
body/resources come from the host; the guest does not need a save file. A team
wipe restores the same world, NPCs, character actions and two bodies as the saved
checkpoint, including after restarting both clients. Individual ten-second
respawns still use nearby placement rather than rolling back the world.

The versioned co-op format is separate from single-player format 12. It checks
the game-data fingerprint, checksum, campaign/difficulty, living roster and
structural limits, with a 32 MiB file bound. Scene/character/actor state and body
placement are validated while constructing the replacement scene. A valid old
primary becomes `.json.bak` before atomic replacement; a damaged primary never
replaces that backup. Loading can recover the backup and reports this to the
host. Saving changes the in-memory wipe checkpoint only after the disk commit
succeeds. Invites, tokens, source assets and connection identities are not saved.

`tools/test_coop_checkpoint.py --data <original-base-folder>` runs on Anode with
two independent profiles, a real public invite and 40 ms delay/20 ms jitter per
sender (`--local` selects loopback). It combines the staged three-visit travel
fixture with distinct seeded health/Will values, closes both clients, starts a
new host/guest pair from the save, and forces a team wipe. It then repeats from
a deliberately damaged primary with a valid backup. Exact-state assertions run
before resumed simulation and immediately after the wipe; network snapshots,
clean exits, file preservation and separate single-player sentinels are checked
by the harness. This is persistence evidence, not a completed campaign visit.

On 2026-10-23, the complete public-invite run
`travel-test-0abfe07a7db0413db758682defa42cd9` passed all three phases: three-visit
travel/rejoin/save, fresh-process primary resume/wipe, and fresh-process backup
resume/wipe. The school door remained opened, two cached visits survived, both
clients reported health 83/47, and the native assertions restored the complete
saved players/resources/character/world/NPC state exactly. Dead-team save attempts
were rejected; primary/backup and single-player sentinels remained unchanged.
Both final views were inspected, showing Blue and Red correctly with the saved
health. All clients exited cleanly. Evidence is the folder's `result.json`,
`resume-result.json`, `backup-result.json`, histories, native logs and captures.
The full Rust suite passes 733 tests with one pre-existing ignored test
(`private/coop-checkpoint-tests.log`); the debug build passes
(`private/coop-checkpoint-build.log`). No campaign-matrix row is promoted by this fixture.

A native Anode menu pass then opened Co-op from the single-player village ESC
menu, selected Host → Continue co-op, and restored the saved school with health
83/47 and both cached visits. Clicking Save checkpoint displayed success and
wrote the fresh session epoch to the co-op file; the single-player quick-save
sentinel stayed unchanged. Leave co-op returned to the host chooser and the app
exited with code 0. The same evidence folder contains `menu-result.json`,
`host/coop-continue-menu.png` and `host/coop-menu.png`. No owned game windows or
tunnel helpers remained afterward.

## Windowed movement and combat slice (development only)

The main menu and single-player ESC menu now have a **Co-op** button (keyboard
shortcut C), with **Host** and **Join**. Host creates an internet invite; Copy
invite puts the complete link on the clipboard. Join accepts that link with
Ctrl+V or Paste link. Invalid links stay in the form with an explanation.
Hosting/connecting can be cancelled. Leaving or a connection error returns to
the co-op screen. A suspended single-player session stays in memory and can be
resumed afterward. Host starts at the chapter entrance when there is no co-op
checkpoint. With a checkpoint, it offers **Continue co-op** or **Start chapter**.
Continue restores both players and the shared world and creates a new invite;
the guest joins normally and receives the host's saved Red Alice.

The connected ESC panel has Resume, Save checkpoint (host), Copy invite (host),
and Leave co-op. It is
drawn once per frame, so it does not pause simulation or network updates. An
incompatible guest releases its slot while the host keeps the invite available.
`tools/run_coop_pair.ps1 -Data <original-base-folder> -Menu` opens two independent
main menus to exercise this flow using the public internet connection.

On 2026-10-23, native Anode tests exercised Host from the main menu and Join
from a paused single-player ESC menu, using the copied public invite. The guest
left and rejoined using the retained link after moving. With the host's menu
open, Red moved 105.91 units while Blue stayed still. Ending the host session
returned the guest to the Join screen. Both game processes exited with code 0;
no owned game windows or tunnel helper remained.

The single-player quick save stayed byte-for-byte unchanged during co-op.
After returning to the original school chapter, a newly written quick save
confirmed the same position, health and inventory. This is evidence for session
isolation, not for restoring a shared co-op campaign checkpoint.

The first menu pass also checked invalid links, cancelling Host, and rejection
of an older build followed by a valid join. It exposed two problems, both fixed
and retested: accumulated gameplay text could alter the retained invite, and a
Windows reader holding the optional diagnostic file could end gameplay. Text
input is now cleared at the gameplay/menu boundary; diagnostic writes are best
effort and the live report is removed on exit. The final public run continued
through an explicit three-second read-only lock of that diagnostic file.
Error messages now have their own readable panel.

The menu integration passed 696 Rust tests with one unrelated ignored test.
After the fixes above, all 28 focused co-op tests and the native public menu flow
passed. The automated two-window loopback movement check also passed with the
new menu and diagnostic cleanup in place. Original single-player save files outside the isolated test profile were
not used or changed.

`--coop-host <private-invite-file>` and `--coop-join <private-invite-file>` now open
two actual game clients. Hosting uses the public internet tunnel by default;
`--coop-local` after the host argument selects loopback for local diagnostics.
Both still require the normal `--data`, `--profile`, and optional `--map` arguments.
F8 copies the host invite. ESC opens a small personal overlay; Q leaves the session.
The world continues when a window loses focus and its controls become neutral.

The host advances two independent movement bodies at 120 Hz and sends validated
snapshots at up to 20 Hz. The guest renders the received state; it cannot decide
positions or health. Each client follows its own player, with camera collision and
guest-side visual smoothing. Alice's blue-fabric pixels in the known clothing
textures are recolored for player two at load time, including packed FTX textures.

`world.rs` integrates individual health, falling/liquid damage, the 10-second
spectating policy, safe nearby respawn at 25 health, and a two-body entrance checkpoint
for this slice. Its tests exercise movement independence, death/respawn/wipe and
refusing a spawn over a drop. This is not yet the campaign checkpoint system.

Run `python tools/test_coop_play.py --data <original-base-folder>` in a desktop
session (prefer Anode). It creates independent profiles and two rendered clients,
uses scripted controls through the same session and simulation as normal controls,
checks that both bodies moved and both clients agree, verifies separate cameras,
captures both views and closes its processes/helpers. Add `--local` for loopback.
The test never writes campaign saves or distributes original assets. Its scripted
controls are enabled only by the explicit development flag `--coop-drive`.

Add `--death --death-first red` or `--death --death-first blue` for the staged
guard recovery check. Ordinary movement makes the chosen Alice the nearest target;
the real guard supplies all damage. The survivor moves during the countdown, then
the pair continues until a wipe. The harness records both clients' death clocks,
spawn ticks, camera subjects and native guard state, and checks the ten-second
delay, 25-health nearby respawn, moving spectator camera and shared entrance reset.
Use `--delay-ms 40 --jitter-ms 20` to add outbound network delay per sender.
Both rendered recovery scenarios still cover only the school encounter.

Snapshots identify each new body by its host spawn tick. Guest smoothing and
camera collision history reset when the watched player changes or respawns,
including short-distance respawns and checkpoint resets. Successful placement,
health and the living state are published together. Co-op draws its own spectator
countdown without the single-player retry panel; single-player keeps that panel.

On 2026-10-23, a rendered local recovery run killed Blue first at host tick 2006
and respawned her at tick 3206 with 25 health, 48 units from Red. Blue's camera
followed Red moving during the countdown and switched back at recovery. Both
clients reported the same respawn and the same entrance reset at tick 4208,
including restored guard health, position and action state. Captures confirmed the
spectator view no longer contains the single-player retry panel. Evidence:
`private/coop/play-test-26fd3e2ac8944ab2b33ed7227fdb0d27`.

The public internet recovery check added 40 ms delay and 0-20 ms jitter per sender.
Red died at tick 2077 and respawned at tick 3277 with 25 health, 48 units from Blue.
Her camera followed Blue moving during the countdown. Both clients agreed on the
entrance reset at tick 4261 and the guest acknowledged 1098 inputs. Both processes
exited successfully and the owned tunnel stopped. Evidence:
`private/coop/play-test-55aa3fe401e24789a3fd6f1f5a9268dd`.
The reverse order also passed over the public connection with the same added
delay/jitter: Blue died at tick 2004, respawned at tick 3204 with 25 health and
48-unit separation, and both clients restored the entrance at tick 4213. This
also exercises delayed guest movement while the host spectates. Evidence:
`private/coop/play-test-65f645a093ca4b0b98770c097ceff993`. No game windows or owned
tunnel helper remained after the tests.
The recovery revision passed 709 Rust tests (one unrelated ignored test), plus the
19 source-package and portable/Windows setup tests. The reset tested here restores
the entrance bodies and native guard, not the full campaign world or a saved puzzle.

On 2026-10-23 the first loopback rendered test passed: blue moved 207.14 units and
red 112.97 units (limited by the wall), both clients agreed within one unit, and
the guest acknowledged 256 control messages. The full suite passed 694 tests with
one unrelated test ignored. That first capture exposed packed-texture recoloring
and overlay spacing errors, which were corrected and visually checked in later runs.

The public internet movement run subsequently passed, including independent
cameras, matching authoritative positions, acknowledged guest input, captures and
clean process/helper shutdown. The captures show blue and red clothing together.
An earlier public run disconnected during play. The transport now tolerates bounded
bursts between render frames by pausing reads rather than disconnecting immediately;
the new burst regression and all 27 focused co-op tests pass, including the existing
persistent slow-reader limit. Keep the failed run in the private evidence rather
than treating the first internet attempt as a pass.

Real Anode keyboard/focus testing also passed: Red moved 107.47 units while Blue
stayed still, then Blue moved 104.00 units while Red stayed still. Both clients left
through ESC then Q; the interactive launcher exited successfully with no game
windows left behind. `tools/run_coop_pair.ps1 -Data <original-base-folder>` launches
that interactive pair; add `-Local` for loopback. It creates independent profiles,
waits for the invite, starts the guest and retains private diagnostics.

The final local rendered check passed after the sleeve recoloring and close-camera
visibility fixes. Both clients agreed on positions; Blue moved 208.05 units and Red
112.97 units. The inspected capture shows red sleeves/dress, blue player one,
unchanged skin/apron/hair/boots, and separate readable overlays. Both processes
exited successfully and no game windows remained. This final local run validates
the visual changes after the successful public internet run above.

The native combat pipeline now advances both real weapon implementations and a
single enemy cast on the host. Guest controls can select owned toys and attack;
cumulative button counters preserve short clicks across network send intervals.
Damage, Will, map pickups and enemy essence drops are personal. Each window draws
its own HUD and remaining pickups, and receives the same enemy and weapon state.
Death decisions run once after both players' combat damage resolves. A wipe resets
this slice's entrance bodies, native NPCs and character state; this remains separate
from a complete campaign checkpoint.

Combat snapshots include full animation poses, compressed with bounded zlib packets.
Both compressed and decoded size are capped at 8 MiB; truncated, oversized and trailing
data are rejected. Incoming NPC state must match the loaded cast; model indices stay
local. Character restoration currently uses the save validation/presentation path,
so smoother guest animation and effect continuity still need work.

Add `--combat` to `tools/test_coop_play.py` to exercise a staged school guard using
both players' actual Blade attacks. Only the host receives `--combat-preview`; the
visit handshake tells the guest to load the matching encounter. The harness checks
personal damage during the fight (essence can heal it afterward), attacks by both
players, shared defeat, matching positions, separate initial cameras and clean exits.
For an interactive staged fight, add `-Combat` to `tools/run_coop_pair.ps1` with
the default school map and without `-Menu`.

On 2026-10-23, the rendered staged fight passed both loopback and public internet
tests using two independent processes/profiles and compressed combat snapshots.
Blue remained at 100 health, Red reached 64, both performed two real Blade attacks,
and both clients saw the same guard defeated. Public guest input reached sequence
411; final body positions matched within one unit. Both processes exited successfully
and the owned helper stopped. Captures were visually inspected. Private evidence:
`play-test-b1505039eeb04cfea0d87a16c5c75d09` (local) and
`play-test-ebe842687f2340568aee5b2d0941cc30` (internet), under `private/coop/`.

Earlier unsuccessful runs are retained too: one fighter could not reach the guard,
then both fighters interrupted it before it dealt damage. The final driver waits
for the guard's approach before attacking. The camera assertion also now accounts
for the school back wall retracting two separate camera arms to nearby origins.
These test corrections are not evidence of a change to normal enemy difficulty.

The full Rust suite passed 708 tests (one unrelated ignored test) at both default
concurrency and eight workers. Two earlier default-concurrency runs timed out while
waiting for an initial transport connection; focused and subsequent full runs passed.
Caller locations and connection-state failures are now reported for diagnosis. The
intermittent timeout's cause is not established; retain its private logs rather than
claiming those earlier runs fixed it. The later delayed-upgrade reproduction and
socket-mode fix are recorded in the live-world section above.
Single-player and two-player headless school combat checks
also pass at 30, 60 and 144 Hz.

After combat integration, the ordinary local movement regression also passed:
Blue moved 208.07 units and Red 112.97, with matching body positions and independent
cameras. Evidence is `play-test-98a3ab7d87454f7bb2cba07530c75ec1`. All owned game
processes and the tunnel helper were gone after these checks. Source packaging
tests (7), portable setup tests (6) and Windows setup tests (6) passed as well.

Still missing from this slice: two-demon targeting, co-op audio, remaining scripted enemy/world
controllers, moving supports beyond the school/village adapters, remaining cutscenes, full campaign progression and checkpoint validation,
prediction/latency tuning and campaign validation. Do not use the movement or staged
combat test as evidence of completing any campaign visit.

## Village integration and traversal

The host now advances trigger-owned encounters with the same two-player target
roster as placed NPCs. Their activation, damage, defeat and personal loot join the
shared simulation. Explicit conversations, the actual hallway Blade pickup scene
and shootable world switches enter the same scene-ownership path. Village machinery
advances once for both bodies using the common atomic carry function. The village's
derived hidden/relocated cast is rebuilt after snapshots and checkpoint restoration,
including on the guest; guests do not advance the story or enemy clocks.

An earlier `--coop-village-check` ran the production native arena from an unarmed
village entrance to its authored Pandemonium exit, with each first-arrival order.
Both collected their own actual Blade pickup and completed the six scenes. It also
counted two voice-only lines as cinematic handoffs; the correction and current
six-reunion results are recorded below. Those runs took 22,512 fixed ticks, ending with
the leader at 70 Sanity and the partner at 100, and performs no deaths, checkpoint
resets, test resource grants or test teleports. Only production scenes relocate players.
This is a single-process, two-body traversal, not a separate-process network test
or, by itself, a passed campaign-matrix row. Later network/death/rejoin and combat
evidence completes that row. That earlier native repeat passed with build identity
`06ad4947fcada8789a50aa239b059306b4453fbd2328cf4459ff1f041dccd5c5` in
`private/coop/village-route-2/coop-village/`, including per-reunion captures.
Blue's third/fifth handoffs and Red's climb handoff were visually inspected.
The initial run remains in `private/coop/village-route-1/coop-village/`.
All 734 Rust tests pass, with one pre-existing ignored test
(`private/coop-village-tests3.log`).

The same build also passes the public two-process checkpoint regression with
40 ms delay and 20 ms jitter per sender: travel/rejoin, exact restart/team-wipe
restoration and corrupt-primary backup recovery. Both final views were inspected;
all clients/helpers exited. Evidence:
`private/coop/travel-test-2e029f8371424a16884acb1619a3bbd0/` and
`private/coop-village-checkpoint.log`. This fixture still uses staged school travel,
not a networked village playthrough.

The new `--coop-world-check` village fixture compares 960 shared machinery ticks
against single-player poses and verifies paused clocks and serialized continuation.
It stages both bodies at a clear entrance; it does not claim riding every mechanism.
The original roof/sawmill rider probes found no clear standing positions inside the
map's collision boundaries and are retained in `private/coop-village-world1.log`
through `world3.log`, not counted as successful ride tests. Atomic rider and blocker
behavior remains covered by `moving::tests` and the authored school lift fixtures.
The corrected world fixture passes in `private/coop-village-world4.log`.

The single-player village route remains exactly 17,790 ticks, six jumps, two throws,
zero combat damage and 100 Sanity, matching the frozen pre-change executable. The
single-player machinery and six-scene cinematic checks also pass. Evidence:
`private/coop-village-before-*.log` and `private/coop-village-after-*.log`.

Subsequent separate-client traversal exposed an incorrect distinction in the shared
story clock: the two voice-only Cheshire lines froze an airborne player and caused
an extra reunion. Single-player allows exploration during those lines. Co-op now
advances ordinary speech alongside movement, pauses attacks/enemies while listening,
and reserves single-Alice control and reunion for controller-owned cutscenes. Either
living player can advance ordinary speech; that E press is consumed by dialogue.
Actual cutscene completion also clears grounded running momentum. Automatic saves
continue waiting for speech to finish before attempting a checkpoint.

The corrected native traversal passes both first-arrival orders in 22,297 ticks,
with six cinematic reunions, 35 ticks of observed movement during speech, both
actual Blade pickups and 100 Sanity for both players. Evidence:
`private/coop/village-route-4/coop-village/`, build
`0eb413f6731105a82780dc10d716c793fc04b3d7c3931d11391cbf6e9de9c2ee`.
Red's final cinematic reunion was visually inspected. The school/world fixture
passes in `private/coop-network-speech-world.log`; the unchanged 17,790-tick
single-player route passes in `private/coop-network-speech-singleplayer.log`.
All 734 Rust tests pass (one pre-existing ignored), recorded in
`private/coop-network-route-tests4.log`.

Failed development runs remain in `private/coop-network-village-local1.log` through
`local5.log`. The first attempts exposed driver timing/input differences; later ones
exposed the speech freeze above. The fifth run reached Pandemonium with both clients,
but failed its speech-movement evidence assertion because the driver skipped the
short lines between diagnostic samples. It is not recorded as a passed network test.
The driver now lets ordinary speech play while continuing movement.

The first public Red-leading run (`private/coop-network-village-internet-red1.log`)
also found an interactive client bug: the guest fed an unused local input latch,
which could reject the ninth accumulated button edge. Only the host samples that
latch. Guests now send their cumulative actions directly through the session,
without also filling a second queue that is never consumed. The host's bounded
input validation remains in place.
The repeat in `private/coop-network-village-internet-red2.log` ended on the
driver's approach ramp before reaching the earlier failure; it does not verify
the input fix. The harness now walks that approach through positions recorded by
the single-player route. Full delayed traversal is still pending. The current
build `028486987c95dd8d2ef21a4a148622aa8e9644aeb51fa2bfb7a2ba3f7113f210`
passes all 734 Rust tests (one ignored) in `private/coop-network-route-tests6.log`.

Later public runs separated test-driver failures from game failures. Guest plans
must time their input segments on the local clock; using received simulation ticks
turned network jitter into uneven key holds. Comparing a just-sent segment with
a delayed position also restarted valid plans and duplicated movement. The driver
now waits for its guest sequence to finish and the body to settle before replanning.
The host still checks each segment against its directly observed simulation.
These changes apply only to the explicitly enabled route driver.

The fifth public Red-leading run reached Pandemonium with both players at 100 Sanity,
but failed final verification: scene labels were sampled before the host step while
ownership and bodies were recorded after it. A completed scene could therefore appear
with no owner. Diagnostics now observe all those fields after simulation, record
their frame identity, and the harness asserts that identity matches. The earlier
failed runs remain evidence of development findings, not successful full tests.
The updated build passes 734 Rust tests (one ignored) in
`private/coop-network-route-tests8.log`.

The sixth public run verified consistent diagnostic clocks and correct owners for
the first five scenes, then missed the late climb. Its single long input plan kept
steering after the first missed landing. The harness now follows separate goals at
the actual intermediate landings before continuing the climb, with the same public
transport and delay/jitter settings. The failed run is retained in
`private/coop/village-test-a1044142d0b04cdeb70771de3b6ebe64/`.
The seventh run cleared those landings with both players and recorded all six
correct scene owners, then missed the final descent. The harness now also walks
the supported approach to that drop before planning the landing. Evidence:
`private/coop/village-test-e5eb38fdf0b24efb869e3c71abd6936a/`.

The public Red-leading repeat now passes the full traversal with 40 ms injected
delay and 20 ms jitter per sender. Both independent clients reach Pandemonium at
100 Sanity. Each observes all six correctly owned scenes and six reunions, both
actual personal Blade pickups and 50 moving-speech sample pairs without a voice-only
warp. All 84 matching authoritative ticks agree. Final captures from both clients
were visually inspected; both processes exit cleanly and the tunnel helper stops.
Evidence: `private/coop/village-test-eb20704fbc3547b1ab7bf766699c54a8/`, build
`5cae1ad45d01ad0b9a6f04a8f9a209c9bbff3ff804697cde94dbb3fefe2f5114`, reproduced by
`tools/test_coop_village.py --first red --delay-ms 40 --jitter-ms 20 --data <base-folder>`.
The Blue-leading repeat passes on the same build and network settings in
`private/coop/village-test-064657842c4d466cb6e551e24be6bb98/`: both players finish
at 100 Sanity, all six scene owners/reunions and personal Blades pass, 81 matching
authoritative ticks agree, and ordinary speech includes 49 host/51 guest moving
sample pairs without a warp. Both processes and the helper exit cleanly. Thus both
network arrival orders are verified. At that checkpoint the death/rejoin matrix was
unfinished; the later recovery and combat evidence completes the row. The 19 synthetic
packaging/setup checks also pass.

Run the native check in Anode (it needs a graphics context), with an isolated profile:

```text
looking-glass.exe --data <base-folder> --coop-village-check --no-audio --profile private/coop/village-route
```

The separate-client village harness is `tools/test_coop_village.py`. Run it in
Anode after building, with `--first blue` and `--first red` in separate runs:

```text
python tools/test_coop_village.py --data <base-folder> --local --delay-ms 0 --jitter-ms 0 --first blue
python tools/test_coop_village.py --data <base-folder> --first red --delay-ms 40 --jitter-ms 20
```

Omit `--local` for the public invite path. Each process has its own profile and
receives only local navigation goals through `--coop-route-driver <command-file>`.
Its planner runs against its own observed world; movement, jumps, attacks and
dialogue inputs still use the normal client/session path. Commands expire after
three seconds without an updated local file. The driver never writes player
positions, inventory, health or shared world state. It is explicitly enabled for
testing and is absent from ordinary Host/Join play.

The harness retains scene captures, route plans and authoritative frame histories
under `private/coop/village-test-*/`, then removes the invite and closes only its
own processes. It rejects a death, checkpoint rollback, blocked body, missed scene,
wrong scene owner, missing personal Blade, divergent matching network frame or
missing authored transition. A clean traversal alone still does not complete the
village death/disconnect/checkpoint matrix.

`--rejoin` on the village harness closes the guest near the exit, starts a fresh
guest process using the retained invite, checks both players' health/inventories,
placements and completed scenes, then continues through the authored exit. Its
before/after reports and separate guest logs stay in the private fixture directory.

`tools/test_coop_village_recovery.py` exercises actual falls after both players
collect the Blade. It checks each dead player's camera while the survivor walks,
the exact 1,200-tick countdown, nearby 25-health revival and retained personal
pickups. It then sends both players over the drop and checks shared restoration
against the game's earlier saved checkpoint. This fixture ends at that restored
checkpoint; outgoing traversal and rejoining are checked separately.
Route-driver profiles now use the same automatic checkpoint conditions as ordinary
play. The older staged movement/combat fixtures still opt out of automatic saves.

```text
python tools/test_coop_village.py --data <base-folder> --first blue --rejoin --delay-ms 40 --jitter-ms 20
python tools/test_coop_village_recovery.py --data <base-folder> --first red --delay-ms 40 --jitter-ms 20
```

The first recovery attempt reached the late scene but its direct diagonal movement
hit a barrier before the drop. It timed out without a death and supplies no recovery
evidence (`private/coop/village-test-fc87e73fd03b436caa14dc5ece1de800/`). The revised
test follows the supported approach before walking off the unrailed edge.
The second attempt observed Red's exact 1,200-tick recovery, but the survivor's
8.55-unit walk was shorter than the camera-motion assertion required. The survivor
now walks farther along the supported ramp. The third attempt passed both individual
recoveries and observed a team wipe; its final resource assertion incorrectly
treated the empty current-visit loot ledger, created by the first resumed ordinary
tick, as a changed inventory. The comparison now models only that empty ledger
creation and still compares every resource, award and drop. These failed fixtures
are retained as `village-test-f6163c27d91f45fbb9b41f7b21436584` and
`village-test-3e4d85e9f83643cd99cf26bb6cd4585e` under `private/coop/`.

The fourth recovery run passes over the public invite path with 40 ms delay and
20 ms jitter per sender. Red dies at tick 30,757 and revives at 31,957; Blue dies
at 34,524 and revives at 35,724. Both clients agree on those exact ten-second
intervals, both moving spectator cameras, each 25-Sanity revival within 50 units
of the survivor, retained personal Blades and the survivor's unchanged health.
The later team wipe at tick 37,338 restores the actual automatic checkpoint saved
at tick 2,916: both original resources/placements and the earlier scene/hatch state.
Both restored-checkpoint views were inspected and both clients exit normally.
Evidence: `private/coop/village-test-db59178229354bdbb2011361cf51ca9d/`, build
`f6304f3344c2f6d0340da41879fb00c4c7d37ada0076df301d49ae75a49f331d`.

The first fresh-process village rejoin run preserves both players and completed
scenes, but its guest misses the final descent afterward. It is not a complete
traversal pass (`private/coop/village-test-08f40261814744a28eac815202f0d736/`).
The test now follows the supported upper ledge farther south before crossing
to the lower landing, avoiding a long diagonal drop that grazed its north edge.
The repeat passes the complete Blue-leading traversal, fresh guest rejoin and
authored transition into Pandemonium with the same public delay/jitter settings.
It retains both Blades and completed scenes, all six scene owners/reunions, and
matches both players' authoritative state at 63 sampled ticks. Blue finishes at
100 Sanity and Red at 92; there are no deaths, respawns or checkpoint rollbacks.
Both final views were inspected; both clients and the tunnel helper exit cleanly.
The retained before/after rejoin reports also agree on Will, equipped weapon,
school items, staff components and difficulty for both players in both clients.
Evidence: `private/coop/village-test-49531ea8bd3f460da591f2be6a8c4465/`, on the
same `f6304f3344c2f6d0340da41879fb00c4c7d37ada0076df301d49ae75a49f331d` build.
The source change and harnesses also pass 734 Rust tests (one pre-existing ignored),
19 packaging/setup checks and Python compilation. The later focused checks below
complete the village's consumable/enemy evidence; 37 visits still prevent claiming
full campaign support.

### Public encounter and personal-consumable checks

`tools/test_coop_village_combat.py` and `tools/test_coop_pandemonium_combat.py` use
normal starts, independent processes/profiles and ordinary walking/weapon inputs.
They approach a real guard with attacks disabled, verify that it selects/damages
the nearer visible Alice, swap the players and repeat, defeat it with the Blade,
then observe each player's separate healing pickup. They default to the public
invite connection with 40 ms delay and 20 ms jitter per sender. For example:

```powershell
python tools/test_coop_village_combat.py --data C:\Alice\base --first blue
python tools/test_coop_pandemonium_combat.py --data C:\Alice\base --first red
```

Both pass in Anode on build
`fe1f5fd905c8262e13dc3bae6e86dfef7a57752504c84d2575e52c1d87bd7a0b`:

- Village: `private/coop/village-combat-44b591c1f47d453aa0afd3b9514fb7df/`.
  Both checked guard hits deal 12 only to the selected player; the placed shard
  independently heals each Alice from 88 to 100. All 3,986 comparable sent/restored
  states and 146 coincident full reports agree. Both attack views, the guest's
  retained pickup after Blue collects, and the final healed view were inspected.
- Pandemonium: `private/coop/pandemonium-combat-ac557b2cbaa240d4a61b04df68220399/`.
  The normal rope/cart lead to the first club guard; Red and then Blue are selected
  and take the checked hits while the distant partner remains unchanged. Its drop
  heals Blue 57.733 to 72.733 and then Red 87.667 to 100 independently. All 2,438
  comparable sent/restored states and six coincident full reports agree. Both attack
  views and the guest pickup view were inspected. Later enemies/the exit are outside
  this focused fixture; subsequent full-route repeats complete that coverage.

The opt-in route driver records the guard's host-selected target, distances and
line of sight. Guests do not run AI to produce a second target choice. A bounded
64-entry SHA-256 history records players, lives, checkpoint generation and
`Encounters` state at actual host send/guest restore points. The harness compares
these exact ticks regardless of the two local report clocks; this does not claim
coverage of other NPC/controller fields. It also compares full encounter/player
records when local report ticks coincide. These observations never enter the
input protocol or alter game state. Regular clients without the driver keep no
such history. All 740 Rust tests pass (one pre-existing ignored), and the normal
executable builds successfully.

`tools/check_coop_rewards.py` independently checks both clients' retained histories
for separated collection and healing, unchanged partner resources and retained
personal enemy drops. Explicit `--map-pickup` IDs also check placed consumables.
It verifies the earlier Blue-first and Red-first Pandemonium full-route histories
on build `ec4ccf71f5a634f1c8c03ccbc6924fa3492e9fac1421dd813cd4937c06118a48`:
`private/coop-pandemonium-personal-rewards1.json` and
`private/coop-pandemonium-personal-rewards-red1.json`. Negative checks reject absent
healing, shared removal of enemy drops, and missing guest observations.

Development trials are retained as failures: the first village approach stopped
outside melee range (`village-combat-8271af180b1347ad96aa67325be8c4bc`); its next
drop chase entered the damaging street (`village-combat-fa63edb33a6f497b9619dbb479222605`).
The successful test uses the supported route's placed shard. The first Pandemonium
trial completed its gameplay checks but had only three coincident local report
ticks (`pandemonium-combat-d99e90f584e24341aa3486dabf794c3d`), motivating the bounded
send/restore observations. No health, inventory or enemy behavior was changed to
make these tests pass. Successful clients/helpers close normally and remove invites.

The full Pandemonium route then passes again in both arrival orders on `fe1f5fd9`,
with the same public delay/jitter settings. Blue's run also leaves/rejoins a fresh
guest before the final airship. All seven encounter actors are included in the
comparison, including six combat deaths and both encountered Diamond guards'
projectiles. Blue's run matches 5,997 sent/restored states and 31 local reports;
Red's matches 6,362 and 65. All four scene owners, three safe reunions, delayed
cart boarding, shared rope, retained personal Cards and independently collected
healing rewards pass. Neither run has a death, respawn or wipe. Both arrive in
Fortress alive (Blue-led 46.667/25.134 Sanity; Red-led 30.001/45.201).
Evidence: `private/coop/pandemonium-test-94511445d6bc41e9a4e92a1a50ec4f18/` and
`private/coop/pandemonium-test-c3449c00e8bb41d0b6c7f39bb4729a98/`; each retains an
`encounter-coverage.json` summary. Rope, boarding-wait and initial Fortress
cinematic views were inspected. Both clients/helpers exit and invites are removed.
Combined with the earlier recovery/wipe evidence, Pandemonium is the second passed
campaign row. The initial Fortress arrival images do not verify that next visit.

## Fortress moving-room preparation

The first Fortress controller now advances its moving room once for all living
co-op bodies. It calculates every rider's proposed position before committing;
an obstruction affecting either player holds both riders and the shared room
clock until it clears. Original single-Alice scenes still use their original
controller path. Moving trigger volumes are refreshed with the same poses as in
single player. The split room is a first-arrival environmental trap, not a shared
passenger lift, so its authored trigger does not gain an all-aboard wait.

`--coop-fortress-check` stages two bodies on the real `split_middle` geometry.
On build `bac55928de0966ab179553addb99d0c14888d14eab63f352fb34c906564d374f`,
30/60/144 Hz checks carry both 220 units, preserve pause and serialized restore,
and exactly match one-player carry. Separate obstructions of either rider stop
both and resume after removal. Evidence: `private/coop-fortress-support-native1.log`.
This is a staged support check, not a complete two-client campaign traversal.

Both single-player Fortress routes match the previous `fe1f5fd9` build exactly:
arrival takes 16,020 ticks, 10 jumps, one Blade throw and 22 Cards; return takes
5,343 ticks, three jumps, one Blade throw and 41 Cards. Both finish at 100 Sanity
with no combat damage, flight or recovery. Watched/skipped cinematic outcomes,
pause, mid-scene restore, legacy saves and the one-shot ambush also pass.
Evidence: `private/coop-fortress-sp-comparison1.json`,
`private/coop-fortress-sp-route1.log`, `private/coop-fortress-sp-cinema1.log`.
All 740 Rust tests pass (one pre-existing ignored), and the normal build succeeds.
Full network routes, map-specific recovery, and the later return visit remain
separate requirements; the first Blue-leading network result follows below.
The campaign count remains two of 39 visits.

The separate-client first-visit probe is under development:

```text
python tools/test_coop_fortress.py --data C:\Alice\base --first blue --rejoin
python tools/test_coop_fortress.py --data C:\Alice\base --first red
```

It uses the public invite with 40 ms delay and 20 ms jitter per sender by default.
Each process receives ordinary local goals and controls; the harness reads the
Fortress scene/state, requires independent portal contacts and the completed
split, compares authoritative player/enemy snapshots, and checks optional fresh
guest rejoin before the authored exit. The opt-in driver can now send one ordinary
jump per command and a run modifier without inventing a grounded destination for
an airborne doorway. Neither control changes the network input protocol. Its
legacy-command/rope regressions pass and the normal build succeeds on
`fda9260961ca3342961c3a50c0d71e3ff8a23d820466e92efa3c009cff63cff2`.

Trials 1–8 are failures, not campaign credit. Long timed paths missed landings,
the guest overshot an upper ledge, and walking a gap fell below its sloping landing.
The route now uses supported waypoints, a landing farther onto the upper platform,
and received-position steering for the gap jump. The failed trial summary is
`private/coop-fortress-development-summary.json`; logs are
`private/coop-fortress-internet-blue1.log` through `blue8.log`.
Trials 3 and 4 were stopped after the failure was observed and retain final
observations/captures instead of full histories; the others retain full histories.
All eight have their owned clients/helpers stopped and invite files removed.
Creating `stop-test` inside a running Fortress probe's printed evidence folder
now stops it through its normal cleanup and preserves its partial history.

Trial 7 reached the last raised section with both players at 100/92 Sanity, but
its next navigation search timed out. Its partial checks match 5,114 sent/restored
player/enemy states and 87 full reports including Fortress state. Both clients
observe the Blue-owned entry scene, the reunion at tick 10,020, independent portal
contacts and a completed split. Both sloping-walkway views were inspected.
Evidence: `private/coop/fortress-test-c85881127e254ef2b5969aa41136dd02/partial-checks.json`.
Trial 8's direct jump landed both bodies, but continued exact-marker steering ran
the guest back off the edge. The controller now releases input after a real jump
lands near the destination at the right height, then verifies the body remains
grounded after the delayed inputs settle. These partial results do not verify an
exit, rejoin, recovery matrix or the later return visit.

The Blue-leading full route with fresh guest rejoin now passes on `fda92609`, in
`private/coop/fortress-test-cc6abe54e07e4f2fac11ff3621b1c75d/` (public internet,
40 ms delay + 20 ms jitter per sender). Both clients observe the original entry
scene owned by Blue (the tied automatic arrival), one safe reunion at tick 10,020,
their independent portal contacts, the completed split and the authored exit to
`fortress2` epoch 2. All 3,740 comparable sent/restored player/enemy states match;
50 coincident full reports also match, including Fortress controller state.
The guest exits and rejoins in a fresh process near the final doorway (ticks
24,817/24,901), preserving both players' resources, placements and completed
scene/machinery progress. Both finish alive at 85/80 Sanity with their Blade and
Cards, no respawn and no wipe. The sloping-walkway view and both final views were
inspected. Both clients and the helper close normally and the invite is removed.
The Red-leading route also passes on the same build and delay/jitter settings:
`private/coop/fortress-test-3cf5044c3af64e3e97ed3ef09d70f751/`.
It observes the same original arrival/reunion, independent portal contacts,
completed split and authored exit. All 3,870 comparable sent/restored states and
93 coincident full reports agree. Both arrive alive at 100 Sanity with their
weapons, without a respawn or wipe. The Red sloping-walkway and final arrival
views were inspected; clients/helper exit normally and the invite is removed.
These routes verify both first-visit orders and Blue-leading rejoin. Targeting,
consumables and recovery are checked separately below; the next level and later
Fortress return visit remain separate requirements.

The focused Fortress checks use the same independent profiles, public invite and
ordinary input driver:

```text
python tools/test_coop_fortress_combat.py --data C:\Alice\base --first red
python tools/test_coop_fortress_recovery.py --data C:\Alice\base --first red
```

The opt-in report now includes Boojum selection, visibility, distance and stable
encounter IDs (the two room Boojums have no authored names). It changes neither
normal gameplay nor the network protocol. The combat probe swaps the nearest
player, takes real scream damage, defeats both active Boojums with the Blade,
then checks an available personal healing drop. The recovery probe leaves the
survivor in the courtyard while the other enters the Boojum room, checks both
ten-second spectator/respawn roles, then sends both low-health players into the
fight to verify their actual automatic checkpoint. These checks supply no health,
position, inventory or shared-world writes.

The focused combat probe passes on build
`2271c7db46267916aa0d866bcc99db4b4d94fce2516014274027895145d58ab8` in
`private/coop/fortress-combat-e4a4260346de427c84f93e82e338f187/` (public internet,
40 ms delay + 20 ms jitter). The observed Boojum selects nearer Red, then Blue,
with both visible. The checked damage changes Red 100 to 93, then Blue 100 to 98,
leaving the farther partner unchanged at each check. Both active Boojums are
defeated by ordinary Blade attacks. The same reward heals Blue 87 to 100 while
Red stays at 85 and retains her copy, then heals Red to 100 without changing
Blue. An independent history check confirms separate healing/collection in both
clients. All 2,442 comparable sent/restored states and 78 coincident full enemy/
player reports match. The first two probe attempts are retained as failures:
the first retreated under a lintel that blocked throws; the second approached a
farther health drop and collected the nearer one on the way, becoming too healthy
to collect the chosen drop. Neither contributes a campaign pass.
The Blue damage view, both-player pickup view and final healed Red view were
inspected. Both clients and the helper exited normally; the invite was removed.

Fortress recovery uses inventory at the observed death as its preservation
reference. A real pickup during the preceding fight is allowed; respawning must
retain it. The earlier village and Pandemonium recovery histories still pass
that comparison for both roles in both clients (`private/coop-fortress-recovery-inventory-audit.json`).
The courtyard survivor route uses the ordinary jump planner across its gap.
The first two Fortress recovery attempts are retained as failures: one used the
pre-fight inventory reference, and the other left the survivor hanging from a
ledge after a fall-reset portal. The latter correctly deferred respawning until
a safe platform was available; it does not supply a successful timed recovery.

The complete recovery probe passes on the same `2271c7db` build and public
40 ms + 20 ms connection in
`private/coop/fortress-recovery-6388fe74d6414e87a48f45ee92ceda44/`.
Red dies at tick 15,887 and respawns at 17,087; Blue dies at 19,780 and returns at
20,980. Both have exactly 25 Sanity, stand 48 units from the survivor on clear
ground, retain their death-time inventory and regain their own camera after
following the moving partner. The survivor takes no damage during either check.
Both then die in the actual Boojum room. Tick 24,430 restores the automatic
checkpoint from tick 10,021, including personal resources, earlier placements,
scene/machinery state and the Boojums' original positions with no scream waves.
The first restored reports arrive 22/17 ticks after restoration; movement stays
within the bound derived from those ticks (Blue drifts 15.85/11.52 units, Red 0).
All 3,662 comparable sent/restored states and 88 coincident enemy/player reports
match. The spectator view, revived pair and both restored views were inspected;
both clients/helper close normally and the invite is removed.

Combined with the two full routes, fresh guest rejoin, native paired supports and
single-player regressions above, the first Fortress visit is now passed in the
campaign matrix. Its splitting room is a first-arrival environmental trap, not
an activated passenger lift; a shared boarding barrier is not applicable here.
The opt-in diagnostic build and both route-driver regression tests pass in
`private/coop-fortress-combat-build1.log` and
`private/coop-fortress-combat-driver-tests1.log`. Normal gameplay is unchanged by
the additional read-only Boojum report.

## Beyond the Wall co-op adapters

The shared Beyond controller now advances once per host tick and checks both
living bodies before committing a moving support. Either player can operate the
musical levers; the actual final operator owns the original single-Alice scene,
including when the partner also tries a lever on that tick. Note replay outputs
advance once for the world. Sound playback was added in the follow-up below.

The rolling bridge waits for both living players to reach the stable tiles before
rolling the rear tile away. The Rage pickup waits for both to board the lift,
starts its original transformation scene for the collector, and reunites the
partner on the lift before descent. The other player's copy remains available
and an ordinary jump from the lowered lift can collect it without replaying the
scene. The later cycling elevator waits while only part of the living team is
aboard, including during disembarkation, and cycles normally when empty. Its
saved boarding delay affects only the lift; other machinery keeps running. A
sole survivor does not wait for a dead partner.

Build `c17d9c2b1eedbd621fd56923648b8f38ec4a58bfd1ba1bae5ef39aac4c0e53c8`
passes `--coop-beyond-check` on the authored map geometry. It checks paired
48-unit descent at 30/60/144 Hz, single-player clock/carry parity, pause and JSON
restore, either obstructed rider stopping both carries and resuming after the
obstruction is removed, both boarding orders, saved waits and invalid-delay
rejection, both scene owners, frozen partners, lift reunion and separate pickups.
The native fixture uses staged placements and does not run combat or a complete
route. Evidence: `private/coop-beyond-adapter-build5.log` and
`private/coop-beyond-adapter-check5.log`.

Single-player machinery/cinematics and the power-up suite (including watched,
skipped and legacy-save Rage scenes) pass in `private/coop-beyond-sp-state5.log`
and `private/coop-beyond-sp-items5.log`. The baseline power-up run also passed.
The full single-player route already failed at arch step 12 on the unchanged
`2271c7db` executable. The final build reproduces its entire 192-line output
exactly, including that failure; this is not a route pass. The comparison is
recorded in `private/coop-beyond-sp-comparison5.json`, with both original logs.
Existing co-op school/world checks, the Fortress paired supports and all 39
entrance-only checks pass in `private/coop-beyond-world5.log`,
`private/coop-beyond-fortress5.log` and `private/coop-beyond-entrances5.log`.
The complete Rust suite passes: 740 passed, no failures, one existing ignored
test (`private/coop-beyond-adapter-tests5.log`).

This adapter milestone added no completed campaign row. Full Beyond traversal,
combat and recovery remain open; the focused sound/rejoin evidence follows below.
The overall estimate remains about 41%, with three of 39 visits fully verified.

## Co-op sound and dialogue

The host now publishes a bounded journal of numbered interaction, movement,
weapon, enemy and cinematic sound cues. Each client consumes new cues once,
even when snapshots repeat or intermediate snapshots are skipped. Old cues are
not replayed on joining, travel or checkpoint rollback. Continuous music and
loops are reconstructed from shared state; original story voices follow the
host's line and elapsed position, including cinematic speech holds. Each client
uses its own camera for spatial listening. The single-player/menu mixer is
paused while the co-op viewer owns playback.

The first delayed public sound fixture passed on build
`0944f2141e72b2011c2a77d06e90d71289b5dcd5ed3e1488cb4fe5b44369a9ba`:
`private/coop/beyond-audio-1dbe4744e4984a3394b41ce156a4aead/`.
It stages only the initial feet outside the musical demonstration trigger, then
uses ordinary movement and E inputs through two real clients. Red triggers the
original demonstration; Blue and Red each operate the reset lever. Both clients
receive exactly the same three numbered notes in each sequence. A fresh guest
process rejoins without replaying the completed demonstration, with resources
and puzzle progress intact. The fixture compares 675 actual sent/restored states
at 40 ms delay plus 20 ms jitter per sender and captures both rendered views.

All three app-only WAV recordings contain nonzero output. Cross-correlation
against locally read original note recordings also identifies all three notes
in order in the initial guest's recording (approximately 3.65, 4.73 and 5.75
seconds). This corroborates real mixer output beyond cue metadata; it is not a
subjective listening or full sound-fidelity assessment. Original assets and
recordings remain private. Anode's session has no isolated audio endpoint, so
this fixture used the authorized main desktop; its owned windows were closed.

The four focused journal/voice-cursor tests pass, including collapsed snapshots
at 30/60/144 Hz, join/travel/wipe/expiry boundaries, bounded bursts, malformed
wire fields and dialogue cursor changes. The complete suite on this build
passes with 744 tests, no failures and one existing ignored test. Native
Beyond co-op checks additionally verify the three-note reset order for either
operator. Single-player Beyond state/cinematics and existing co-op world/scene
checks pass. Evidence: `private/coop-audio-tests1.log`,
`private/coop-audio-all-tests1.log`, `private/coop-audio-beyond-native1.log`,
`private/coop-audio-sp-beyond1.log` and `private/coop-audio-world1.log`.
The original audio asset/clock check also passes: 336 referenced files decoded,
38 music cues and 900 ambient emitters; two original missing references are
reported (`alice/idle_move2.wav`, `icewand/icw_idle.wav`). The live musical
fixture reports no missing cues (`private/coop-audio-assets1.log`).

Final build `10aa2112129158e4810cf5856895fd6d32b2eac586afa20a29104acc3f293fc4`
also passes the full 744-test suite and native Beyond checks
(`private/coop-audio-all-tests2.log`, `private/coop-audio-beyond-native2.log`).
This follow-up corrects camera-directed stereo listening and clears reaction
gating/swim ambience on world restore. Its unskipped village opening test passes
in `private/coop/village-voice-0e430b62bac8415a8ecf762320d21940/`: both public
clients observe all six original lines in the same order, one voice start per
line, elapsed positions matching the host story, and held dialogue during the
original camera choreography. It compares 1,136 sent/restored states under the
same delay/jitter. Both 30-second app recordings contain the first Rabbit voice:
correlation with the locally read original is above 0.96. The recordings stop at
the capture limit; the full six-line sequence is verified by runtime observations
and playback logs, not a complete six-line acoustic comparison. The driver now
supports watching dialogue without generating its usual advance presses.

The final build also passes the musical fixture with Blue arriving first,
both reset operators and a fresh guest process:
`private/coop/beyond-audio-d97405d188c34927aa4e24529f05f29f/`
(`private/coop-audio-internet3.log`). It again verifies identical numbered notes,
no historical replay, nonzero output without missing cues, independent app
recordings and matching sent/restored state. Both runs close their owned windows
and remove their invite files. The initial rejected launch (audio capture lacked
the required frame limit) is retained in `private/coop-audio-internet1.log`;
the harness now supplies that limit before launch.

Reproduce the musical fixture with:

```text
python tools/test_coop_audio.py --data <base-folder> --first red
python tools/test_coop_voice.py --data <base-folder>
```

This is a staged musical demonstration/reset/rejoin check, not a full Beyond
route. Remote pain/pickup cue fidelity, menu-to-game audio handoff, rendered Rage
presentation and the remaining campaign traversal/combat/recovery checks still
need verification. No additional campaign visit or percentage point is credited.

## Beyond normal-entrance network route

`tools/test_coop_beyond.py` starts two independent profiles at the normal Beyond
entrance and uses ordinary movement, jumping, attacks and lever inputs. It reads
replicated machinery clocks to time the flipping corridor and moving stairs.
It rejects deaths, checkpoint resets, obstructed bodies and unplanned recovery
portals rather than treating those shortcuts as clean traversal. Each private
run keeps the exact harness source, observations, final reports and captures.

The explicit local test driver now accepts a bounded input-delay estimate for
braking from delayed observations and a moving-step target. These affect only
the driver's ordinary inputs; they do not change player physics, received state
or the host's authority. Other route fixtures keep their existing zero-delay
steering unless they opt in.

The fourth public run, build `496972c8`, reaches arch step 11 from the entrance
with 40 ms delay plus 20 ms jitter per sender. It crosses the flipping corridor,
solves the puzzle with alternating players, completes the rolling walkway and
observes all three Blue-owned scenes. Both players reach the arches with 100
Sanity; the minimum along the route is 96 for Blue and 100 for Red. No player
respawns, and 7,143 actual sent/restored network states agree. Evidence and both
rendered views are retained in
`private/coop/beyond-route-aa5a24b4065b48d7bb126d15146b0f9b/`.
The run stops at the next stair because the driver required a vertical gap below
90 units between steps whose gap is always 102.4. This is not a completed route.

Earlier failed attempts are retained: the delayed guest overshot a jump landing
(`1a1cbed4`), simultaneous automatic door presses prevented passage (`888da19b`),
and delayed walking overshot a platform edge into a recovery portal (`217a1088`).
The harness now anticipates braking, sends the leader through that narrow door
first, and aims farther inside the platform. The moving-step driver uses the
native route's 110-unit launch bound and descending upper-wave timing. The upper
exit uses the native route's running approach, and the final door waits for the
shuffle to settle before being approached. These test adjustments still require
the resulting full route to pass.

Two further repeats stopped at the gap after the third flipping platform. In
`beyond-route-d12f8fe4562c400fb1d66815c43a99a8`, the guest's static plan treated a
passing platform as permanent floor and never jumped. In
`beyond-route-f11fb0abdca14a369a9affd8dba05144`, an outer-harness run-up lasted
too long because it waited on periodic network reports. Both recovery portals
were rejected. The driver now times bounded run-ups alongside local input
sampling: host simulation ticks and the guest's local clock. A regression test
checks that stalled guest snapshots cannot lengthen the run-up. The corridor
gap now uses that ordinary running jump instead of a static-floor plan.

Reproduce with a debug build and locally installed game data:

```text
python tools/test_coop_beyond.py --data <base-folder> --first blue --rejoin
python tools/test_coop_beyond.py --data <base-folder> --first red --rejoin
```

For diagnosis, `--arches-only` stages the initial position at a verified point
outside the rising-room trigger and then supplies ordinary inputs through the
stairs. Its evidence folder and result explicitly identify it as a staged probe;
it cannot complete a campaign row or establish normal-entrance traversal.
`--corridor-only` similarly starts just before the corridor trigger. The two
probe flags are mutually exclusive.
That probe passes on build
`7eec625b6f796e7878748a243126ca79df900c889b7cc00a9f4e61e02f915424` in
`private/coop/beyond-arches-probe-0387775c95634254acd868903865bf8f/`:
both climb the steps, complete the upper running jump and use the authored
portal. There are 1,027 matching sent/restored states under the default delay
and jitter. Both rendered upper-door views were inspected. The first probe
(`20cac92e`) was rejected at startup because its proposed spawn was obstructed;
the successful probe uses a clear point recorded in the normal traversal.

The same build completes the whole corridor from the normal entrance in
`private/coop/beyond-route-3ccb48bd81f34998802d1600e37f7f42/`, with 4,516
matching sent/restored states and no respawns. That repeat stops before the
first musical note: the navigation landing radius left the operator outside
lever reach. The harness now finishes each lever approach with precise ordinary
walking before its single E press.
The next repeat (`beyond-route-4fd969389a574d3997680914aa974503`) clears the
corridor and all three levers, then stops when a Boojum's scream knocks Blue
off the spiral stairs into a recovery portal. The test had treated a nearby
enemy hidden by the stairs as safe. It now fights from the upper landing before
descending, allows nearby Boojums time to approach, and waits for existing
scream projectiles and knockback to settle before departing a combat stop.
The following full-route repeat (`beyond-route-d1833f4c4eca4f698d5a499781c15b5b`)
stops earlier, at another static-planner misstep toward `(2980,3456,-64)`.
The remaining corridor gaps now use explicit ordinary jumps too. The focused
corridor probe passes on the same `7eec625b` build in
`private/coop/beyond-corridor-probe-f26639b408ba470b82d9b3a6498ea8e7/`:
both traverse all flipping-platform gaps and take the authored portal, with
1,709 matching sent/restored states under 40/20 ms delay/jitter. Both rendered
portal views show 100 Sanity and were inspected. This initial placement is
staged. The subsequent normal-entrance traversal below covers the revised
combat stops and final shuffled-door exit.

The Blue-leading normal-entrance run on the same build reaches the authored
`fortress1_start2` return with both players alive in
`private/coop/beyond-route-3ea9b2df6c034cffb7c1b83a3df0ccc9/`.
It has 7,722 matching sent/restored states, all three Blue-owned route scenes,
frozen partners and safe reunions, a solved musical puzzle, walkway stage nine,
the correct final door and no respawns. Minimum Sanity is Blue 96 / Red 98;
both arrive in Fortress at 100. The original harness exits with an error after
the actual traversal: it scheduled its Beyond reconnect check after crossing
the outgoing trigger, then looked for Beyond state in the Fortress visit.
The retained observations pass the extracted `verify_route()` validator in
`audited-route-result.json`; `audited-post-transition-rejoin.json` independently
checks the actual fresh guest's preserved Fortress placement and resources.
This is audited traversal evidence, not a claim that the original harness
exited successfully. The fixture now reconnects at the safe pre-exit landing.

The corrected Red-leading full route exits successfully in
`private/coop/beyond-route-26dba8d47c88465abb0ab7cc4a1eb6bd/`, on the same build
and 40/20 ms public connection. All three scenes belong to Red, all reunions
and traversal invariants pass, and 7,885 sent/restored states match. A second
guest process joins at the safe pre-exit landing with personal resources,
placements and completed puzzles intact. Both reach `fortress1_start2` at
100 Sanity. The rendered guest stair/upper-door/return views were inspected,
including a live Anode observation. The host return capture is orange and
obscures the scene despite valid placement; the route assertions alone did not
detect it. The scoped camera correction and rendered repeat are recorded below.

The separate public combat check passes on `7eec625b` in
`private/coop/beyond-combat-04edc0990c244e69995a04acf6a9d34f/`.
The hallway guard chooses nearer visible Red, then nearer visible Blue. Each
checked hit removes 12 Sanity from its target without hurting the other player.
Both collect their own earned healing drop after ordinary Blade combat: Blue
heals 76 to 91 while Red stays at 88; Red then heals to 100 while Blue stays at
91. There are 505 matching actual sent/restored states and nine coincident full
enemy/player reports. Both target views and the final personal pickup view
were inspected. Reproduce with `tools/test_coop_beyond_combat.py --data <base>`.

Both combat deaths and the shared wipe pass on the same build/settings in
`private/coop/beyond-recovery-6ca7a13ffb634602bc613cff7d893097/`.
Red dies at tick 3,660 and returns at 4,860; Blue dies at 8,492 and returns at
9,692. Each spectates the moving survivor for exactly 1,200 ticks, returns at
25 Sanity about 48 units away, and preserves inventory. A subsequent team wipe
at tick 13,015 restores the actual earlier checkpoint's personal resources,
placement, enemy state and Beyond machinery. There are 1,943 matching
sent/restored states and 89 coincident full enemy/player reports. Spectator,
respawn and checkpoint views were inspected. Both fixtures use normal-entrance
movement and actual enemy damage; neither injects health or world state.
Reproduce recovery with `tools/test_coop_beyond_recovery.py --data <base>`.

The focused passenger probes in `tools/test_coop_beyond_transports.py` use the
same public connection and delay/jitter. `--probe rage` walks from the normal
entrance; `--probe bridge` explicitly stages only the initial bridge approach.
Both accept `--first blue` or `--first red`, and neither claims a full route.
On `7eec625b`, Rage passes with Red first in
`private/coop/beyond-rage-ffb2d9ac54324e4ca8b926673385d316/` (956 matching states)
and Blue first in `private/coop/beyond-rage-fdfee62536404a0d97d6c2633a33f0e6/`
(965). The first player waits for the partner, owns the original scene, then
both descend 48 units together. The other player collects an independent Rage
copy with an ordinary jump; the scene runs once. These state assertions pass,
but inspecting the middle of the scene revealed that its owning Alice was
hidden: the generic co-op cinematic drawing rule hid both gameplay bodies.
The renderer now draws the owner's existing authoritative Rage pose while
keeping the other player hidden. Other cinematic actor drawing is unchanged.
The corrected `cf86b004` public repeats pass with Red first in
`private/coop/beyond-rage-7d58e582d71842dfaae8f083f4998a27/` (936 matching states)
and Blue first in `private/coop/beyond-rage-ca85ba3508fb4818846294edf437d0a2/`
(960). Both clients' middle-scene views were inspected in each run: exactly the
owning Alice is visible in the original transformation pose, with the correct
clothing. All boarding, frozen-partner, descent and independent-copy assertions
also pass on this final build.

The bridge waits for a trailing player, shows the boarding notice, and then
advances once with both safely aboard in both orders. Red's run is
`private/coop/beyond-bridge-5cd7763586a34c51919d81c6feec49c6/` on `7eec625b`
(156 matching states). Blue's run is
`private/coop/beyond-bridge-07f7c0ef71a64c2aa9a796aeade41fbb/` on
`cf86b00492db900ea0f523d75c5bde56c763f11e7890fb1ef74775f1a0c1ad24`
(170). Waiting/boarding views were inspected. Initial probe failures remain
retained: a broad navigation plan could not reach the Rage lift, a walking
approach stopped at its raised lip, and the bridge's combat wait could not
reach a distant enemy across the gap. The final probe uses ordinary approach
points/jumps and keeps real combat active. A Blue bridge setup also started
its partner already on a safe tile; both orders now place the trailing role
on the same rear approach before checking the barrier.

The `cf86b004` build also fixes the obscured Fortress return view. Blue's camera
was inside the visible `portal_back1` volume, which had no camera collider.
That visible portal now blocks cameras without becoming a player obstacle or
changing save state. An independent real-geometry regression checks both
entrance positions at 30/60/144 Hz in `--fortress-cinematic-check`.
The original local two-client reproduction is
`private/coop/fortress-return-camera-0d47b1f15b4e4c0081c7f68b5679dddc/`;
the fixed repeat is `fortress-return-camera-579dd23381c84f05aa0c6937d36aec74/`.
Both fixed entrance views were inspected and are clear while stationary.
The existing watched/skipped/paused/resumed/legacy-save Fortress scene checks
and both native co-op Fortress/Beyond adapter suites also pass. Evidence:
`private/coop-fortress-camera-check7.log`, `private/coop-fortress-adapter-check7.log`
and `private/coop-beyond-adapter-check7.log`.

Final build `cf86b004` passes all 745 Rust tests (one existing ignored test),
debug/release builds and all seven source-package tests. The rendered clients
match the independently calculated source identity. Validation logs are
`private/coop-beyond-validation-tests7.log`, `private/coop-beyond-validation-build7.log`,
`private/coop-beyond-validation-release7.log`, `private/coop-beyond-source-tests7.log`
and `private/coop-beyond-source-identity7.json`. The source-only worktree audit
passes 765 files. No private captures, saves, game data or invite links are
included in the contribution.

Validation for this source build: 745 Rust tests pass with one existing ignored
test, both debug and release builds succeed, and all seven source-package tests
pass. The source identity matches the executable used by the passing stair
probe. Evidence: `private/coop-beyond-route-tests5.log`,
`private/coop-beyond-route-build5.log`, `private/coop-beyond-route-release5.log`,
`private/coop-beyond-source-tests4.log` and `private/coop-beyond-source-identity5.json`.

Internet joining and 40/20 ms delay/jitter are the defaults. Both full traversal
orders, combat/resources and death/wipe have the evidence above.

Cycling-elevator rendered checks now pass in both boarding orders on `cf86b004`:
`private/coop/beyond-elevator-d57b738905834aa6a1ab2d1c730c3c59/` (Red, 811 matching
sent/restored states) and `private/coop/beyond-elevator-da651fab20844150a026e2a640f1ebc2/`
(Blue, 775). `--probe elevator` stages the initial upper return landing inside
the authored activation trigger, then walks down, boards, rides and dismounts
through ordinary controls. It verifies two seconds of boarding and dismounting
wait in each client's own reports, continuing world time, paired ascent, safe
running jumps to the Normal-difficulty outer ledges and the empty lift's resumed
clock. Waiting/dismount/final views were inspected in both orders. Earlier
fixture failures retain a wall-bound approach, an incorrect Easy-only landing,
and a host-only observation deadline that ended before the delayed guest's full
wait. These probes grant transport evidence, not standalone full-route credit.

The additional `--probe late-rage` exposed a real freeze on `cf86b004` in
`private/coop/beyond-late-rage-e16f3f79bbe141f387b6d401f0030397/`: collecting the
box after cycling began changed the lift's phase, obstructed both bodies and
left the original scene at time zero. The stopped run retains those reports.
Build `ef3db23edd407fcac578bb0676a3af94c984bdd9882aab117864395ccbd73989` derives the
cycle's initial descent from whether Rage preceded the return trigger. A later
pickup cannot restart that descent or shift the cycle. Existing saved timestamps
suffice; no save field or format changes. The original early pickup is unchanged.
The phase regression, all 746 Rust tests (one existing ignored), and native
co-op checks pass. Both owners watch and skip the late scene with pause/save/
restore, frozen partners, clear bodies and a supported paired reunion. Original
Rage watch/skip/save/legacy cases on all difficulties and Beyond's other world/
cinematic regressions also pass. Logs: `private/coop-beyond-validation-tests8.log`,
`private/coop-beyond-adapter-check8.log`, `private/coop-beyond-items-check8.log`
and `private/coop-beyond-world-check8.log`.
Both corrected late-pickup public repeats pass under 40/20 ms delay/jitter:
`private/coop/beyond-late-rage-510a52db30ac4a858ec444cd09c10e84/` with Red first
(932 matching states), and `private/coop/beyond-late-rage-0dedcf3a803a4016babffde8b517a11c/`
with Blue first (927). Both check the advancing original scene, a frozen hidden
partner, one reunion and grounded bodies. The middle-scene owner and paired
reunion views were inspected in each run. The rendered clients match the source
identity in `private/coop-beyond-source-identity8.json`. Debug/release builds and
seven source-package tests pass (`private/coop-beyond-validation-build8.log`,
`private/coop-beyond-validation-release8.log`, `private/coop-beyond-source-tests8.log`).
Together with the earlier full routes, combat, recovery and rejoin evidence,
these checks complete Beyond's row: four of 39 visits are fully verified.
Overall weighted credit is about 41.05%, still reported as 41%.

The next visit's traversal now passes in both orders on the same `ef3db23e`
build. `tools/test_coop_fortress_return.py --data <base> --first red --rejoin`
starts at the actual `fortress1_start2` entrance and follows the upper wall with
ordinary navigation, Blade attacks and the running jump into the rocking school
window. `--first blue` repeats the other trigger order. Internet joining with
40/20 ms delay/jitter remains the default; neither run stages a mid-level start
or writes gameplay state. The shared harness now accepts an optional authored
entrance; existing first-visit fixtures retain their previous arguments.

Red's public run is `private/coop/fortress-return-0e72ab5bd10a471fbcf6fe4dc06c87fc/`
(1,047 matching sent/restored states); Blue's is
`private/coop/fortress-return-a66cad4ce4344e68b5ae5169cf15ad47/` (1,064).
Each verifies the proper owner of `Fortress1_Boojum_Attack`, a frozen partner,
one safe reunion, the activated ambush, no respawns/wipes, and a fresh guest
rejoin preserving resources/placement/completed scene state before the outgoing
trigger. Both players enter `skool1_start1` at 100 Sanity in both runs. Reunion,
window approach and both school entrance views were inspected. The scene's
automatic capture is its opening white fade, so a middle-reveal capture still
needs inspection. Earlier fixture attempts stopped while trying to throw the
Blade at a guard from behind the corner parapet; the passing route completes
the ordinary approach before waiting for that fight. Failed evidence remains
in `fortress-return-bcef832fb66d498e96ac997201cce2b1` and
`fortress-return-47dcb81c92b6468d99b36ea883a07a05`.

These are traversal/reconnect results. The return visit's dedicated nearest-
visible targeting, personal consumable and both death/respawn/wipe checks remain
open, together with the middle-reveal view. No fifth-row completion credit is
claimed. Seven source-package tests and Python syntax checks pass in
`private/coop-fortress-return-source-tests1.log`; the game source/build remains
the validated `ef3db23e` above.

The return visit's focused guard and personal-healing check now passes on that
same build in `private/coop/fortress-return-combat-86bb88db6fc24b00b5320f8c0f494b9c/`.
`tools/test_coop_fortress_return_combat.py --data <base>` walks from the authored
entrance with public 40/20 ms delay/jitter. With both players visible, guard
`end_spawn4` selects nearer Red, then nearer Blue. Each checked hit removes seven
Sanity only from the selected player. Ordinary Blade attacks defeat the guard;
Blue collects her earned shard and heals 72 to 97 while Red stays at 79 and
retains her copy. Red then heals 79 to 100 while Blue stays at 97. Both clients
observe those separate collections. All 578 comparable sent/restored states and
eight coincident player/enemy reports match. Both attack views and both healing
views were inspected. The shared combat/recovery helpers now accept an explicit
loot visit, so this return uses `fortress1$return`; first-visit defaults are
unchanged. No game source or save format changed.

`tools/test_coop_fortress_return_recovery.py --data <base>` also passes on the same
build and public delay/jitter in
`private/coop/fortress-return-recovery-15c11dd1a47044339884ce41fd8b686f/`.
Each Alice walks into the actual guard encounter without attacking while the
survivor stays on the upper wall. Red dies/respawns at ticks 3,029/4,229; Blue at
7,055/8,255. Both clients agree on exactly 1,200 ticks, 25-Sanity recovery,
moving spectator cameras, restored personal cameras, retained inventory and
unchanged survivor health. Separation is 49.19 and 48.89 units respectively.
Their subsequent shared wipe at tick 10,518 restores the automatic checkpoint
from tick 1: exact resources, earlier Fortress state, enemy health/activation/
positions/state, and both starting placements within ordinary movement bounds.
All 1,538 comparable sent/restored states and 31 coincident player/enemy reports
match. Both spectator views and both restored checkpoint views were inspected.
Both recovered-player views were also inspected. The combat and recovery
clients/helpers close normally and remove their invites.

The route fixture now explicitly captures the reveal after its original white
fade. The Red-first repeat passes in
`private/coop/fortress-return-53bf1343634e43e19a655a6ffd592774/`, with 1,042 matching
sent/restored states, one reunion at tick 2,911 and fresh guest rejoin before the
school exit. Its middle captures show the original Boojum cutaway in both clients,
with no extra Alice in the shot; both final school views were also inspected.
The reported scene clocks after capture are 2.33 seconds in the host and 2.05 in
the delayed guest. Both finish at 100 Sanity. The original opening-fade captures
remain in the evidence, and the new capture does not change game state.

The Blue-first repeat also passes in
`private/coop/fortress-return-e744240e838d43b6b1776eea5d0c2ede/`: 1,036 matching
sent/restored states, the Blue-owned reveal, reunion at tick 2,906 and a fresh
guest rejoin before the exit. Both middle-reveal views and both final school
views were inspected; scene clocks after capture are 2.21/2.02 seconds. Both
arrive at 100 Sanity without respawns or a wipe. These two completed routes use
fixture SHA-256 `24aac8cc240f3ff41d2ba5423aa3e155f852c9d384470dec16d7ec5878e81d16`.

There is no activated passenger lift or ride on the return route. Fortress's
split-room trigger is first-visit-only (`Fortress::gate`), while the rocking
school is ambient machinery (`Fortress::advance_machinery`), so shared boarding
is not applicable here. Combined traversal, visual, combat, recovery and rejoin
evidence now completes the fifth campaign row. Overall weighted progress is
41.56%, reported as about 42%; 34 visits remain open. The game remains the frozen
`ef3db23e` build with its existing 746 passing Rust tests (one ignored), release
build and native/single-player checks recorded above. This checkpoint adds only
Python fixtures and evidence documentation. Seven source-package tests, Python
syntax checks and the source-boundary audits cover the authored additions.

## First school verification in progress

The first school visit has partial public two-process evidence on build
`4fd9c6cca856cc455099b23286834117ecb693144726781a33103c34bb4fab00`.
The only runtime addition is opt-in reporting of school scene names. The new
`tools/test_coop_school.py` fixture uses ordinary movement and combat inputs from
the normal entrance. Its retained Red-first run reaches both personal Mallets,
four correctly owned scenes with safe reunions, both lifts (including delayed
boarding), and the four-book bridge. All 14,137 comparable network states and
117 coincident full school/player/encounter reports match. Both waiting and
completed lift views, and the middle shelf/book reveals, were inspected.

Evidence is in `private/coop/school-route-1bb1e30f303f4edc9abdeb31367d7952/`.
It remains a failed full-route test: the host's navigation search times out at
the first bridge book after the guest reaches it. The fixture now uses a direct
walk for that small step. No recipe, exit or rejoin success is inferred from the
partial run. Both complete arrival orders
remain outstanding; the total estimate stays at about **42%**, with **5/39**
campaign visits fully verified.

The separate recovery fixture now passes with the optimized executable and the
same public 40/20 ms delay/jitter in
`private/coop/school-recovery-84df4abea6c8431fbd0072b0a61ca189/`.
Both actual guard deaths lead to exactly 1,200 ticks of moving-partner spectating
and revival at 25 Sanity, 48 units from the survivor. The subsequent team wipe
restores the real earlier checkpoint's resources, school/enemy state and starting
positions within ordinary movement bounds. All 1,930 comparable network states
and eight coincident full school reports match. Both spectator, respawn and
checkpoint views were inspected. This proves recovery at the entrance; it does
not establish completion of the school route. Reproduce with
`tools/test_coop_school_recovery.py --data <base> --exe target/release/looking-glass.exe`.

The frozen build passes 746 Rust tests (one existing ignored test), debug and
release builds, the native co-op world check and the native school state/lift
check. The latter covers both lifts at 30/60/144 Hz, waiting/obstruction, book
state contacts and fresh/return restoration; it is not a full two-player route.

The optimized Red-first repeat in
`private/coop/school-route-44d72453baad4b1690465f308c26a03a/` subsequently crosses
all four books, plays the recipe scene and successfully reconnects a fresh guest.
Five owners/reunions, 10,233 network states and 294 full school reports agree.
The run still fails before the exit: the guest overshoots a lower shelf during
the timed descent and dies from the longer fall. The fixture now uses a fresh
theatre guard reward and a slower, observed edge descent. The Mallet approach
is split at the lower stair after another run missed the long planned jump;
the short approach reaches the platform in a focused probe. The essence near
the Mallet is return-only and is not made available on the first visit. The
original recipe cinematic contains one Alice in her original blue clothing,
including when Red is the trigger owner; that costume has not been recolored.

Focused combat and healing now pass on the same build/settings in
`private/coop/school-combat-2b97ba23cace4d9f8b9ee9202be946ea/`.
With both visible, the upper guard selects nearer Red, then nearer Blue; each
checked hit removes seven Sanity only from the selected player. The fresh
theatre-floor guard reward heals Blue 93 to 100 while Red stays at 58 with her
copy retained; Red then heals to 83 without changing Blue's resources. Both
clients observe the separate collections. All 1,662 comparable network states
and 28 full school reports match. Targeting and healing views were inspected;
the clients/helpers close and the invite is removed. Reproduce with
`tools/test_coop_school_combat.py --data <base> --exe target/release/looking-glass.exe`.
The updated full route takes that fresh reward, splits the Mallet approach at
the lower stair, and attempts a slow descent onto the lower library shelf.
Those changes still need a complete repeat. Overall progress remains 42%.

## Repeating a school section from a normal checkpoint

For diagnosis, the school route accepts `--checkpoint-stops balcony bridge descent`.
At either stop it holds both clients' movement inputs for up to three minutes.
Use the host's ESC menu to **Save checkpoint**, retain a byte-for-byte copy of
that profile's `coop-checkpoint.json`, and resume the menu. Create an empty
`continue-balcony`, `continue-bridge` or `continue-descent` file in the printed evidence folder to
continue the route. The option is off for unattended full-route tests.

The saved section can then be repeated with:

```text
python tools/test_coop_school_resume.py --data <base> --checkpoint <saved-file> --resume-point balcony --first red --rejoin
```

Use `--resume-point bridge` before crossing the books, or `--resume-point descent`
for a checkpoint taken after the recipe reunion.
The fixture copies the original save unchanged into a fresh host profile, uses
the game's normal resume path, and checks that both clients retain their
positions, personal resources and school progress. It records the checkpoint's
SHA-256 and source tick. It never edits health, positions or world state, and a
successful segment does not count as a full entrance-to-exit campaign pass.

## Reproducing transport checks

Build with `cargo build --locked`, then run:

```text
python tools/test_coop_network.py --delay-ms 60 --jitter-ms 30
python tools/test_coop_network.py --internet --delay-ms 40 --jitter-ms 20
```

The internet check downloads the official Windows x64 cloudflared 2026.10.2 helper
if absent, verifies its size (55,329,336 bytes) and SHA-256 before execution:
`12c7c887f8aaab779aef1dfa9b60673530a5adb88ecc55dd6df8f56e0b70f199`.
It creates only an owned temporary tunnel to the loopback co-op listener and stops
that helper at the end. It does not install a service or change firewall/router
settings. Each run writes private diagnostic evidence under `private/coop/` and
removes its invite file. These checks need no original game assets.

Quick Tunnels are for development/testing, not an uptime promise. A released build
needs a separately reviewed service choice and updated third-party notices for the
network dependencies/helper. No co-op release has been created.
