# Cordon / Evolution

A third-person browser progression game in the actual Anomaly Cordon map.
One Rust server hosts a shared camp with up to 12 players.

Power determines how far each click pushes the fight meter. Approach an enemy and
press E, then click to push toward victory while the opponent pushes back.
There is no fixed fight deadline. Retreat earns no reward. The server accepts
at most one attack per simulation tick; client-supplied damage is ignored.

Sit at the campfire (F) to gain power. Spend earned EXP on Ascend armor:

| Armor | EXP cost | Power per second |
|---|---:|---:|
| No armor | — | 1 |
| Leather jacket | 5 | 3 |
| Sunrise suit | 25 | 10 |
| SEVA suit | 100 | 35 |
| Exoskeleton | 500 | 100 |
| Nosorog exoskeleton | 2,000 | 300 |

Costs are for each successive upgrade. Ascending changes the character model
without resetting power, equipment or stage wins. New lootboxes contain weapons;
armor equipment from older saves retains its flat combat bonus independently of
Ascend armor, which controls appearance and training speed.

Each enemy is a stage. Defeating an enemy unlocks the next stage and its EXP
marker for this run. The dog's 1 EXP marker also requires a dog victory. Return
markers award 1/5/25/100/500 EXP; clearing all five enemies upgrades the last
marker to 2,500 EXP. Claiming returns to camp and clears current-run wins.
Lifetime trophies, power, equipment, Ascend armor and unspent EXP are retained.
Leaving and reconnecting preserves the current run. Old saves begin a fresh run.
The first victory over each enemy in a run has a 5% weapon lootbox chance.

## Run

Install Rust stable and Node.js 22+. Import resources from your own installation:

```powershell
pwsh -File tools/Import-Anomaly.ps1 -Anomaly D:/Anomaly -Gamma D:/GAMMA -Output D:/GameAssets/cordon-royale
$env:CORDON_ASSETS='D:/GameAssets/cordon-royale/web'
$env:BIND='0.0.0.0:8787' # LAN; omit for localhost only
cargo run --release
```

Open http://localhost:8787 or the host's LAN address. All clients of one server
share the same room. WASD moves, Shift sprints, right-drag or arrow keys orbit the
camera, Space jumps, E challenges the selected nearby enemy, I opens inventory,
F sits or stands at the fire, R claims nearby EXP, and B returns to camp.
Desktop keyboard/mouse required.

Profiles save every two seconds to `D:/GameAssets/cordon-royale/profiles.json`;
override with `CORDON_SAVES`. A random browser-local token identifies the profile.
Returning with the same browser and origin restores progress. Different devices,
localhost versus LAN addresses, or clearing site storage produce separate profiles.
Only one simultaneous connection per profile is allowed. This local prototype has
no account recovery or internet-grade authentication. A crash may lose the latest
two seconds of progress. Never publish profile files or imported resources.

## External resources

The importer extracts Cordon and environment textures, then applies enabled loose
GAMMA texture overrides in MO2 priority order. Packed mod archives and modified
level geometry are not overlaid. Collision honors the selected gamemtl.xr passable
flags: bushes are traversable; solid trunks, walls and ground remain collidable. It does not change either game installation.
Anomaly/GAMMA resources retain their original rights; the repository MIT license
applies only to our code. Three.js has its license in `public/vendor/THREE-LICENSE.txt`.
Stalkers, bandits, mercenaries, dogs, bloodsuckers and controllers use imported Anomaly
meshes, textures and skeletons, with simple procedural poses rather than original
animation clips. Equipment affects stats but does not yet swap the held weapon or
outfit model. Terrain combines its color map with the ground mask and repeating
grass, asphalt, earth and pebble textures. X-Ray AI and atmosphere shaders are not emulated.

Map, character models, and texture chunks are cached in IndexedDB, including on plain HTTP LAN origins.
Each load checks a small manifest and reuses unchanged chunks. Storage failures fall
back to downloads. The browser can evict cache data; scene creation still happens on
reload. After manually changing imported files, regenerate content-addressed chunks:

```powershell
node tools/prepare-cache.mjs D:/GameAssets/cordon-royale/web
```

The importer does this automatically. Generated manifests, binary chunks, saves and
captures belong outside the repository.

## Local validation

```powershell
cargo fmt --check
cargo test
cargo test cordon_ -- --ignored # requires imported Cordon assets
node --check public/game.js
node tests/resource-cache.mjs
node tests/dds.mjs
node tests/world-materials.mjs
node tests/follow-camera.mjs
node tests/locomotion.mjs
node tests/collision-materials.mjs
node tests/character-idles.mjs D:/GameAssets/cordon-royale/web
node tests/player-motion.mjs D:/GameAssets/cordon-royale/web
# Run a separate server with BIND=127.0.0.1:8788 and a temporary CORDON_SAVES path:
node tests/room.mjs
```

GitHub builds remain disabled. The server owns movement, campfire training, inventory, stage unlocks,
EXP claims and lootbox rewards. The client sends actions, never balances or item ownership.

Enemy idle poses are imported from local Anomaly OMF clips and loop on the OGF skeletons.
`tools/import-idles.mjs` converts them; `tools/prepare-cache.mjs` includes them in the
browser cache. Player idle, walk and run clips use the highest-priority enabled GAMMA
stalker animation override, falling back to local Anomaly resources. Run
`tools/import-player-motion.mjs` with the same base/output paths to refresh them, then
regenerate the cache. Sitting remains procedural; the original animation state machine
is not imported. Game animation data remains outside Git.
