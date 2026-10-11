import assert from 'node:assert/strict';import fs from 'node:fs';import path from 'node:path';
import * as T from '../public/vendor/three.module.js';import {loadCharacters,animateCharacter} from '../public/characters.js';
const dir=process.argv[2];if(!dir)throw Error('Pass the external web asset directory');
const make=await loadCharacters({get:async n=>fs.readFileSync(path.join(dir,n))},async()=>null),player=make('stalker');let now=0;
for(const [state,speed] of [['idle',0],['walk',4.8],['run',7.4],['idle',0]]){
 let first;for(let i=0;i<120;i++){animateCharacter(player,now+=1000/60,speed,false);player.updateMatrixWorld(true);const points=player.userData.bones.map(b=>b.getWorldPosition(new T.Vector3()));assert(points.every(p=>p.toArray().every(Number.isFinite)));const size=new T.Box3().setFromPoints(points).getSize(new T.Vector3());assert(size.y>1&&size.y<2.5&&size.x<3&&size.z<3);if(i===30)first=points.flatMap(p=>p.toArray());if(i===90)assert.notDeepEqual(points.flatMap(p=>p.toArray()),first);}
 assert.equal(player.userData.motion.state,state);const bone=n=>player.userData.bones.find(b=>b.name===n).getWorldPosition(new T.Vector3());assert(bone('eye_left').sub(bone('bip01_head')).z<0,state+' faces forward');console.log(state+' animates and faces forward');
}
animateCharacter(player,now+=16,0,false,true);assert.equal(player.userData.motion.state,'sitDown');animateCharacter(player,now+=16,4.8,false,false);assert.equal(player.userData.motion.state,'walk');console.log('PASS sit interrupts motion; walking resumes');

// Small network-speed fluctuations must not reset the running action.
for(let i=0;i<40;i++)animateCharacter(player,now+=16,7.4,false);
const run=player.userData.motion.actions.run,before=run.time;
animateCharacter(player,now+=16,5.9,false);animateCharacter(player,now+=16,7.4,false);
assert.equal(player.userData.motion.state,'run');assert(Math.abs(run.time-((before+.032)%run.getClip().duration))<1e-5);
const phase=run.time/run.getClip().duration;animateCharacter(player,now,4.8,false);
assert(Math.abs(player.userData.motion.actions.walk.time/player.userData.motion.actions.walk.getClip().duration-phase)<1e-5,'walk/run transition preserves stride');
console.log('PASS running phase survives jitter and walk transitions');

for(let tier=0;tier<6;tier++){
 const actor=make('player_'+tier);
 for(let i=0;i<90;i++){
  animateCharacter(actor,i*16,7.4,false);actor.updateMatrixWorld(true);
  const meshes=[];actor.traverse(o=>{if(o.isSkinnedMesh){o.computeBoundingBox();meshes.push(o)}});
  for(const mesh of meshes){const size=mesh.boundingBox.getSize(new T.Vector3());assert(size.toArray().every(Number.isFinite));assert(size.y<3&&size.x<3&&size.z<3,'armor mesh must stay bound to animated skeleton');}
 }
}
console.log('PASS all six Ascend armor models animate without exploding geometry');

for(let tier=0;tier<6;tier++){
 const actor=make('player_'+tier);let time=0;
 for(let i=0;i<360;i++)animateCharacter(actor,time+=1000/60,0,false,true);
 assert.equal(actor.userData.motion.state,'sit');actor.updateMatrixWorld(true);
 const box=new T.Box3().setFromObject(actor,true);
 assert(box.min.y>-.2&&box.min.y<.2,'seated armor rests on ground');
 assert(box.max.y<1.5,'seated pose is lower than standing');
 const pose=actor.userData.bones.map(b=>b.quaternion.toArray());
 for(let i=0;i<45;i++)animateCharacter(actor,time+=1000/60,0,false,true);
 assert.notDeepEqual(actor.userData.bones.map(b=>b.quaternion.toArray()),pose,'seated idle moves');
 animateCharacter(actor,time+=1000/60,0,false,false);assert.equal(actor.userData.motion.state,'standUp');
 for(let i=0;i<90;i++)animateCharacter(actor,time+=1000/60,0,false,false);
 assert.equal(actor.userData.motion.state,'idle');
 animateCharacter(actor,time+=1000/60,0,false,true);
 animateCharacter(actor,time+=1000/60,7.4,false,false);assert.equal(actor.userData.motion.state,'run');
}
console.log('PASS all armor tiers sit on the ground, play seated idle, stand up and interrupt sitting to run');
