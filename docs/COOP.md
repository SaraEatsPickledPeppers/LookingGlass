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

The internet transport is being evaluated. Cloudflare Quick Tunnels support temporary
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

The final campaign matrix must include all 39 visits, both trigger orders, a player
dying during traversal, both dying, checkpoint reload, a guest falling behind, and a
disconnect around a transition. The network test harness should inject delay/jitter
and exercise slow readers rather than assuming loopback performance represents the internet.

## Current evidence

- `coop` exists from current public main.
- `src/coop/rules.rs` implements policy decisions and scenario tests. These are
  not yet connected to the viewer or enemy controllers. All six initial policy
  scenarios pass: countdown, simultaneous death, saved/cinematic countdown,
  visible targeting, scene ownership/reunion and the boarding barrier.
- Host/Join, the two-process launcher and full-campaign integration are not yet implemented.
