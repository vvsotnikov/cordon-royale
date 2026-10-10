import assert from 'node:assert/strict';
import * as THREE from '../public/vendor/three.module.js';
import {FollowCamera} from '../public/follow-camera.js';

for (const fps of [30,60,144]) {
  const rig=new FollowCamera(),camera=new THREE.PerspectiveCamera(),actor=new THREE.Vector3();
  let previousY, largestStep=0;
  for(let frame=0;frame<fps*4;frame++) {
    const time=frame/fps, tick=Math.floor(time*20);
    // Network movement includes abrupt stair treads and a jump arc.
    const jump=time>2&&time<2.8 ? 1.4*Math.sin((time-2)/.8*Math.PI) : 0;
    actor.lerp(new THREE.Vector3(tick*.24,Math.floor(tick/4)*.2+jump,0),1-Math.exp(-18/fps));
    rig.target(actor,.7,.3,1/fps);rig.place(camera,6,1/fps);
    const forward=camera.getWorldDirection(new THREE.Vector3());
    assert(forward.distanceTo(rig.direction.clone().negate())<1e-6,'walking must not change camera pitch');
    if(previousY!==undefined)largestStep=Math.max(largestStep,Math.abs(camera.position.y-previousY));
    previousY=camera.position.y;
  }
  assert(largestStep<.3,'height steps must be smoothed');
  rig.place(camera,1,1/fps);assert(rig.distance<=1,'must not interpolate through a wall');
  rig.place(camera,6,1/fps);assert(rig.distance<1.5,'must not pop outward when an obstruction clears');
  rig.target(new THREE.Vector3(100,10,100),0,.3,1/fps);assert.equal(rig.focus.y,11.4,'teleports reset the pivot');
}
console.log('PASS stable walking/jumping pitch at 30/60/144 FPS, smooth stair height, wall clearance, gradual zoom recovery and teleport reset.');
