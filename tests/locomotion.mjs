import assert from 'node:assert/strict';
import {MovementSample,locomotionState} from '../public/locomotion.js';
for(const fps of [30,60,144])for(const interval of [.05,.075,.1,.15]){
 const sample=new MovementSample();let state='idle',switches=0;
 for(let frame=0;frame<fps*10;frame++){
  const tick=Math.floor(frame/fps/interval),time=tick*interval;
  const speed=sample.update([time*7.4,0,0],time),next=locomotionState(speed,state);
  if(state!==next)switches++;state=next;
 }
 assert.equal(state,'run');assert.equal(switches,1,'steady sprint must not restart between snapshots');
 const t=sample.time;sample.update(sample.position,t+.05);assert.equal(locomotionState(sample.speed,state),'idle','actual stop ends running');
}
let state='run';for(const speed of [5.9,6.1,5.8,6.2,7.4]){state=locomotionState(speed,state);assert.equal(state,'run');}
assert.equal(locomotionState(4.8,'run'),'walk');
const sample=new MovementSample();sample.update([0,0,0],0);assert.equal(sample.update([20,0,0],.05),0,'teleport is not running');
console.log('PASS snapshot cadence, run hysteresis, stop, walk and teleport');
