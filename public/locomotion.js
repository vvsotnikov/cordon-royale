// Animation speed follows authoritative movement, not the renderer catching up
// with a stationary snapshot between network packets.
export class MovementSample {
 constructor(){this.time=null;this.position=null;this.speed=0;}
 update(position,time){
  if(this.time!==null&&time>this.time){const distance=Math.hypot(position[0]-this.position[0],position[2]-this.position[2]);this.speed=distance>3?0:Math.min(8,distance/(time-this.time));}
  if(this.time===null||time>this.time){this.time=time;this.position=position.slice();}
  return this.speed;
 }
}
export function locomotionState(speed,previous){
 if(previous==='idle'||!previous)return speed<.3?'idle':speed>6.2?'run':'walk';
 if(speed<.15)return 'idle';
 return previous==='run'?(speed<5.4?'walk':'run'):(speed>6.2?'run':'walk');
}
