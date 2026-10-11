import assert from 'node:assert/strict';import fs from 'node:fs';import path from 'node:path';
import * as T from '../public/vendor/three.module.js';import {loadCharacters,animateCharacter} from '../public/characters.js';
const dir=process.argv[2];if(!dir)throw Error('Pass the external web asset directory');
const make=await loadCharacters({get:async n=>fs.readFileSync(path.join(dir,n))},async()=>null),player=make('stalker');let now=0;
for(const [state,speed] of [['idle',0],['walk',4.8],['run',7.4],['idle',0]]){
 let first;for(let i=0;i<120;i++){animateCharacter(player,now+=1000/60,speed,false);player.updateMatrixWorld(true);const points=player.userData.bones.map(b=>b.getWorldPosition(new T.Vector3()));assert(points.every(p=>p.toArray().every(Number.isFinite)));const size=new T.Box3().setFromPoints(points).getSize(new T.Vector3());assert(size.y>1&&size.y<2.5&&size.x<3&&size.z<3);if(i===30)first=points.flatMap(p=>p.toArray());if(i===90)assert.notDeepEqual(points.flatMap(p=>p.toArray()),first);}
 assert.equal(player.userData.motion.state,state);const bone=n=>player.userData.bones.find(b=>b.name===n).getWorldPosition(new T.Vector3());assert(bone('eye_left').sub(bone('bip01_head')).z<0,state+' faces forward');console.log(state+' animates and faces forward');
}
animateCharacter(player,now+=16,0,false,true);assert.equal(player.userData.motion.state,null);animateCharacter(player,now+=16,4.8,false,false);assert.equal(player.userData.motion.state,'walk');console.log('PASS sit interrupts motion; walking resumes');
