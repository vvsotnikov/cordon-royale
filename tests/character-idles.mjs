// Optional integration check against locally imported resources; no game data is committed.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import * as THREE from '../public/vendor/three.module.js';
import {loadCharacters,animateCharacter} from '../public/characters.js';
const root=process.argv[2];if(!root)throw Error('Pass the external web asset directory');
const idles=JSON.parse(fs.readFileSync(path.join(root,'idle-clips.json')));
const make=await loadCharacters({get:async n=>fs.readFileSync(path.join(root,n))},async()=>new THREE.Texture());
for(const [name,clip] of Object.entries(idles)){
 const character=make(name),snapshot=()=>character.userData.bones.flatMap(b=>[...b.position.toArray(),...b.quaternion.toArray()]);
 animateCharacter(character,0,false,false);const first=snapshot();
 animateCharacter(character,clip.frames/clip.fps*430,false,false);assert.notDeepEqual(snapshot(),first,name+' must animate');
 for(let frame=0;frame<clip.frames;frame++){
  animateCharacter(character,frame/clip.fps*1000,false,false);character.updateMatrixWorld(true);
  const points=character.userData.bones.map(b=>b.getWorldPosition(new THREE.Vector3()));
  assert(points.every(p=>p.toArray().every(Number.isFinite)),name+' finite transforms');
  const bounds=new THREE.Box3().setFromPoints(points),size=bounds.getSize(new THREE.Vector3());
  assert(size.x<4&&size.y<3&&size.z<4&&bounds.min.y>-.5,name+' intact skeleton');
 }
 animateCharacter(character,0,false,false);character.updateMatrixWorld(true);
 const pos=n=>character.userData.bones.find(b=>b.name===n).getWorldPosition(new THREE.Vector3());
 const forward=name==='dog'||name==='bloodsucker'?pos('bip01_head').sub(pos('bip01_pelvis')):pos('bip01_l_toe0').sub(pos('bip01_l_foot'));
 assert(forward.z<0,name+' faces forward');
 animateCharacter(character,clip.frames/clip.fps*1000,false,false);
 assert(snapshot().every((v,i)=>Math.abs(v-first[i])<1e-5),name+' loops continuously');
 console.log(name+': animated, bounded, forward-facing, loops');
}
