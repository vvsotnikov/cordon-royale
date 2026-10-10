# Cordon / Last Light

A browser first-person scavenging battle royale. One authoritative Rust room,
up to 12 human connections, five scavenger bots, searchable stashes, randomized
equipment, hitscan combat and a shrinking emission. Late arrivals spectate until
the next round. An empty room resets when the next player arrives.

The renderer uses **the actual Anomaly Cordon level**, not a reconstructed map.
The first arena is southern Cordon around the rookie village. X-Ray gameplay,
quests, shaders, AI and animations are not emulated; this is a separate prototype.
Characters and first-person weapons currently use simple original proxy models.

## Local resources — never committed

Install Rust stable and Node.js 22+. Use your own Anomaly installation. On Windows:

```powershell
pwsh -File tools/Import-Anomaly.ps1 -Anomaly D:/Anomaly -Gamma D:/GAMMA -Output D:/GameAssets/cordon-royale
$env:CORDON_ASSETS='D:/GameAssets/cordon-royale/web'
cargo run --release
```

Open http://localhost:8787 in a desktop WebGL2 browser. Importing uses Anomaly's
bundled converter, extracts Cordon and environment texture archives, then applies
enabled loose GAMMA texture overrides in MO2 priority order. It records original
paths in an external import manifest. It does not alter either game installation.
Packed mod archives and modified level geometry from mods are not overlaid yet.
The generated map is about 275 MB, so the first load can take a while.

`CORDON_ASSETS` must point outside this repository. Do not commit extracted files,
converted meshes, textures, sounds or captures. Anomaly/GAMMA resources retain
their original rights; this repository's MIT license applies to our code only.
Providing the assets over a public server is separate from publishing this repo.

## Multiplayer

All browsers connecting to the same process share one room. Start the server from
this repository (it serves `public/`). `BIND=0.0.0.0:8787` enables LAN access;
for internet access put the process behind an HTTPS reverse proxy supporting
WebSocket upgrades. Serve the frontend and `/ws` from the same origin. Do not
publish game assets without appropriate permission. There is no account system,
matchmaking, paid loot or persistent inventory. A restart clears the room.

WASD move; mouse look/fire; Shift sprint; hold E to search; R reload; H heal;
Escape releases the pointer. Search rewards are earned in-game. Gunfire only
damages players after the preparation phase. Eliminated players spectate until
the next round. Desktop keyboard/mouse required.

## Architecture and validation

The server accepts bounded movement/aim inputs, never positions, damage or loot
claims. It owns 20 Hz movement, collision against imported CFORM triangles,
inventory, line-of-sight shots, stash ownership and round transitions. The client
interpolates snapshots and renders converted X-Ray visual buffers with Three.js.
This prototype does not provide lag compensation, jumping, robust capsule physics,
competitive anticheat or production denial-of-service protection.

```powershell
cargo fmt --check
cargo test
cargo test cordon_stairs -- --ignored # requires imported local Cordon assets
node --check public/game.js
node tests/room.mjs # with a fresh server running on localhost:8787
```

No GitHub build workflows. Run checks locally before committing. Three.js is
vendored under its MIT license in `public/vendor/THREE-LICENSE.txt`.

In embedded browsers that reject pointer capture, aim by holding the right mouse
button and dragging; left click still fires. Chrome/Edge support normal mouse look.
The importer currently uses base color textures without X-Ray terrain blending,
lightmaps, grass details or atmospheric shaders, so lighting and ground appearance
are simplified. Spawn and stash heights are tuned for southern Cordon.
