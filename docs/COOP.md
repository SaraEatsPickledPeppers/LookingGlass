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

Overall estimate after live shared-world and theatre checks: **28%** (previously 25%). Credited
points are 8/10 for foundation/diagnostics, 7/15 for internet Host/Join,
4/15 for two-player movement/presentation, and 6/20 for native targeting, host-owned
weapons/damage, personal resources and the entrance-checkpoint/respawn slice,
plus 3/20 for shared world replication, ordered contacts and the verified theatre
ownership/reunion slice. Full-campaign verification remains at zero. Standalone
policy tests receive no campaign credit; every campaign-matrix row remains Not run.

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

The final full Rust suite passes 723 tests (one unrelated ignored test), including
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
shared saves, controller combat, two-body moving supports/boarding, co-op audio,
and live transition/disconnect coverage remain unfinished. Free-play controller
updates beyond doors/contacts are not yet integrated; one theatre passing cannot
stand in for those systems or for the 39-visit campaign matrix.

## Windowed movement and combat slice (development only)

The main menu and single-player ESC menu now have a **Co-op** button (keyboard
shortcut C), with **Host** and **Join**. Host creates an internet invite; Copy
invite puts the complete link on the clipboard. Join accepts that link with
Ctrl+V or Paste link. Invalid links stay in the form with an explanation.
Hosting/connecting can be cancelled. Leaving or a connection error returns to
the co-op screen. A suspended single-player session stays in memory and can be
resumed afterward. Hosting currently begins a fresh movement slice at the chapter
entrance; campaign state transfer remains unfinished.

The connected ESC panel has Resume, Copy invite (host), and Leave co-op. It is
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

Still missing from this slice: two-demon targeting, co-op audio, scripted enemy/world
controllers, shared moving supports, cutscenes beyond the verified fixture, campaign progression/checkpoints,
prediction/latency tuning and campaign validation. Do not use the movement or staged
combat test as evidence of completing any campaign visit.

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
