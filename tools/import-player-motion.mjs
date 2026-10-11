import fs from 'node:fs';
import path from 'node:path';
import {decodeIdle} from './import-idles.mjs';
const [base,out,gamma='D:/GAMMA',profile='G.A.M.M.A',anomaly='D:/Anomaly']=process.argv.slice(2);
if(!base||!out)throw Error('Usage: import-player-motion.mjs <extracted Anomaly> <external web directory> [GAMMA] [profile] [Anomaly]');
const repo=path.resolve(import.meta.dirname,'..').toLowerCase(),destination=path.resolve(out).toLowerCase();
if(destination===repo||destination.startsWith(repo+path.sep))throw Error('Game resources must remain outside the repository');
const relative='meshes/actors/stalker_animation.omf',list=path.join(gamma,'profiles',profile,'modlist.txt');
const candidates=fs.existsSync(list)?fs.readFileSync(list,'utf8').split(/\r?\n/).filter(s=>s.startsWith('+')).map(s=>path.join(gamma,'mods',s.slice(1),'gamedata',relative)):[];
candidates.push(path.join(anomaly,'gamedata',relative),path.join(base,relative));
const source=candidates.find(p=>fs.existsSync(p));if(!source)throw Error('Missing stalker animations');
const data=fs.readFileSync(source),clips={};
for(const [state,name] of Object.entries({idle:'norm_idle_1',walk:'norm_walk_fwd_1',run:'norm_run_fwd_1'})){
 const clip=decodeIdle(data,name);
 // Server movement owns travel. Keep local root sway, remove net loop displacement.
 const root=clip.tracks.find(t=>t.bone==='bip01');
 if(root&&root.positions.length>3){for(const axis of [0,2]){const start=root.positions[axis],end=root.positions.at(-3+axis),frames=root.positions.length/3;for(let i=0;i<frames;i++)root.positions[i*3+axis]-=(end-start)*i/(frames-1);}}
 clips[state]=clip;
}
fs.writeFileSync(path.join(out,'player-motion.json'),JSON.stringify(clips));console.log('Player motion source: '+source);console.log(Object.fromEntries(Object.entries(clips).map(([k,c])=>[k,{frames:c.frames,fps:c.fps}])));
