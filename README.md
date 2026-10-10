# Cordon / Evolution

A third-person browser progression game in the actual Anomaly Cordon map.
One Rust server hosts a shared camp with up to 12 players.

Practice to build fieldcraft, equip weapons and armor to increase combat strength,
and challenge mutants or hostile stalkers. Encounters resolve after three seconds:
combat strength at least equal to the enemy's strength wins. This is a stat-based
progression game, not the previous aiming/shooting battle royale. Enemies award
wins and rubles; first victories against tougher enemies also award inventory loot.
Buy or equip items in the inventory. Promotions spend wins and reset fieldcraft,
keep inventory and rubles, and double future training gains. There are no paid items.

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
and B returns to camp. The Auto button repeats practice. Desktop keyboard/mouse required.

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
level geometry are not overlaid. It does not change either game installation.
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
cargo test cordon_stairs -- --ignored # requires imported Cordon assets
node --check public/game.js
node tests/resource-cache.mjs
node tests/follow-camera.mjs
# Run a separate server with BIND=127.0.0.1:8788 and a temporary CORDON_SAVES path:
node tests/room.mjs
```

GitHub builds remain disabled. The server owns movement, training rate, inventory,
rewards and promotions. The client sends actions, never balances or item ownership.
