param([string]$Anomaly='D:/Anomaly',[string]$Output='D:/GameAssets/cordon-royale',[string]$Gamma='D:/GAMMA',[string]$Profile='G.A.M.M.A')
$ErrorActionPreference='Stop'
$repo=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..')).TrimEnd('\','/')
$destination=[IO.Path]::GetFullPath($Output).TrimEnd('\','/')
if($destination -eq $repo -or $destination.StartsWith($repo+'\',[StringComparison]::OrdinalIgnoreCase)){
    throw 'Imported game resources must be stored outside the code repository.'
}
$base=Join-Path $Output 'anomaly-base'
$web=Join-Path $Output 'web'
New-Item -ItemType Directory $base,$web -Force | Out-Null
& "$Anomaly/tools/converter.exe" -unpack -xdb "$Anomaly/db/levels/level_l01_escape.db0" -dir $base
if($LASTEXITCODE){throw 'Cordon extraction failed'}
& node "$PSScriptRoot/import-level.mjs" "$base/levels/l01_escape" $web
if($LASTEXITCODE){throw 'Geometry conversion failed'}
# Extract environment textures. Never copy source archives into the repository.
Get-ChildItem "$Anomaly/db/textures" -File | Where-Object {$_.Name -notmatch 'actors|weapons|ui|sky'} | ForEach-Object {
    & "$Anomaly/tools/converter.exe" -unpack -xdb $_.FullName -dir $base
    if($LASTEXITCODE){throw "Texture extraction failed: $($_.Name)"}
}
foreach($archive in @('meshes/meshes.db0','meshes/meshes_actors.db0','textures/textures_actors.db0','textures/textures_actors.db1')){
    & "$Anomaly/tools/converter.exe" -unpack -xdb "$Anomaly/db/$archive" -dir $base
    if($LASTEXITCODE){throw "Character extraction failed: $archive"}
}
& node "$PSScriptRoot/import-characters.mjs" $base $web
if($LASTEXITCODE){throw 'Character conversion failed'}
$sources=@("$base/textures","$Anomaly/gamedata/textures")
# MO2 modlist is highest priority first; reverse it so later copies win.
if(Test-Path "$Gamma/profiles/$Profile/modlist.txt"){
    $mods=@(Get-Content "$Gamma/profiles/$Profile/modlist.txt" | Where-Object { $_.StartsWith('+') })
    [array]::Reverse($mods)
    $sources+=@($mods | ForEach-Object {Join-Path "$Gamma/mods" ($_.Substring(1)+'/gamedata/textures')})
}
$missing=@();$manifest=@()
foreach($texture in Get-Content "$web/textures-needed.txt"){
    $relative=$texture.Replace('\','/')+'.dds';$found=$null
    foreach($root in $sources){if(Test-Path -LiteralPath "$root/$relative"){$found="$root/$relative"}}
    if(!$found -and (Test-Path -LiteralPath "$base/levels/l01_escape/$relative")){$found="$base/levels/l01_escape/$relative"}
    if($found){$dest=Join-Path "$web/textures" $relative;New-Item -ItemType Directory (Split-Path $dest) -Force | Out-Null;Copy-Item -LiteralPath $found -Destination $dest;$manifest+=@{texture=$relative;source=$found}}
    else{$missing+=$relative}
}
@{sources=$manifest;missing=$missing;profile=$Profile} | ConvertTo-Json -Depth 5 | Set-Content "$Output/import-manifest.json"
Write-Output "Imported $($manifest.Count) textures; $($missing.Count) unavailable. Local provenance: $Output/import-manifest.json"
& node "$PSScriptRoot/prepare-cache.mjs" $web
if($LASTEXITCODE){throw 'Browser cache preparation failed'}
