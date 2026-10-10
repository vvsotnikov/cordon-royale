# Cordon / Evolution

A third-person browser progression game in the actual Anomaly Cordon map.
One Rust server hosts a shared camp with up to 12 players.

Power determines fight outcomes; EXP is a progression score only. Sit by the
campfire (F) to gain one base power per second. Moving or jumping stands up.
Weapons and armor add power while equipped. Approach an enemy and press E to
fight it; the server selects the nearest enemy within 4.5 metres. Encounters
resolve after three seconds, with power at least equal to the enemy winning.

Each enemy is a stage. The dog’s EXP marker is unlocked initially; defeating an
enemy unlocks the following stage and its marker. Walk to a marker and click
“Get X EXP” (or R) to collect 1/5/25/100/500 EXP and return to camp. Claims are
repeatable, require proximity, and preserve power and inventory. Each victory
has a 5% chance to award a lootbox. Open boxes in inventory, then equip the weapon
or armor. Boxes prefer unowned equipment; a completed collection yields duplicates.
There are no purchases, practice clicks or promotions. Existing saves keep power,
equipment and defeated enemies; EXP and unopened boxes start at zero.

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
node tests/follow-camera.mjs
node tests/collision-materials.mjs
node tests/character-idles.mjs D:/GameAssets/cordon-royale/web
# Run a separate server with BIND=127.0.0.1:8788 and a temporary CORDON_SAVES path:
node tests/room.mjs
```

GitHub builds remain disabled. The server owns movement, campfire training, inventory, stage unlocks,
EXP claims and lootbox rewards. The client sends actions, never balances or item ownership.

Enemy idle poses are imported from local Anomaly OMF clips and loop on the OGF skeletons.
`tools/import-idles.mjs` converts them; `tools/prepare-cache.mjs` includes them in the
browser cache. Player movement and combat are still procedural; this does not import
the original animation state machine. Game animation data remains outside Git.
