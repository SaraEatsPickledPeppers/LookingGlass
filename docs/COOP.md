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
- `src/coop/rules.rs` implements policy decisions and scenario tests. These are
  not yet connected to the viewer or enemy controllers. All six initial policy
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
- Main/ESC Host/Join, shared gameplay and full-campaign integration remain unfinished.
  Current probes verify networking, not two playable Alices.

The next session layer (`protocol.rs`, `session.rs`, `hosting.rs`) checks exact source
and dependency identity plus the mounted game-data fingerprint. Both roles must
acknowledge loading a visit before guest controls become active. Every reload or
transition has a new epoch: old controls cannot leak into the next visit. Held
movement/attacks become neutral after 500 ms without fresh input, while cumulative
jump/use counts preserve short presses across a busy send queue. An incompatible
guest releases the slot so the same host can accept a correct build. Host setup,
helper download and cancellation run off the UI thread.

These APIs still need viewer/menu integration; their tests do not establish that
gameplay already uses them. The source identity is a compatibility check, not an
attestation that a peer is trustworthy. Guest messages cannot upload world state.

The session changes passed 689 Rust tests (one unrelated ignored test), including
real-socket scenarios for readiness, a transition with input still in flight,
incompatible data followed by a successful rejoin, disconnection and loading timeout.
The shared background Host setup also passed separate-process loopback and public
internet probes with the delay/jitter above. The public probe waits for helper
shutdown before exiting. The game quit path still needs this integration.
After tightening cancellation and the forged-message regression, all 21 focused
co-op tests passed again. This final focused run includes the latest source changes.

Overall estimate at this milestone: **12%**. Credited points are 8/10 for foundation
and diagnostics and 4/15 for internet session infrastructure. The other four areas
remain 0 credited points until their behavior is integrated and verified. This
deliberately gives no campaign or gameplay credit to standalone policy tests.

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
