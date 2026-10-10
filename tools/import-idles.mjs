import fs from 'node:fs';
import path from 'node:path';
export function chunks(b){const m=new Map();for(let p=0;p<b.length;){if(p+8>b.length)throw Error('Truncated chunk');const id=b.readUInt32LE(p),n=b.readUInt32LE(p+4);if(id&0x80000000||p+8+n>b.length)throw Error('Unsupported chunk');m.set(id,b.subarray(p+8,p+8+n));p+=8+n;}return m;}
function reader(b){return {p:0,u16(){const v=b.readUInt16LE(this.p);this.p+=2;return v;},u32(){const v=b.readUInt32LE(this.p);this.p+=4;return v;},f(){const v=b.readFloatLE(this.p);this.p+=4;return v;},s(){const e=b.indexOf(0,this.p);if(e<0)throw Error('Unterminated name');const v=b.toString('utf8',this.p,e);this.p=e+1;return v;}};}
export function decodeIdle(data,name){
 const c=chunks(data),p=reader(c.get(15)),version=p.u16(),parts=p.u16(),bones=[];
 if(version<3||version>4)throw Error('Unsupported motion version');
 for(let i=0;i<parts;i++){p.s();const n=p.u16();for(let j=0;j<n;j++){const bone=p.s(),index=p.u32();bones[index]=bone;}}
 let speed=1;const definitions=p.u16();
 for(let i=0;i<definitions;i++){
  const clip=p.s();p.u32();p.u16();p.u16();const rate=p.f();p.f();p.f();p.f();if(clip===name)speed=rate;
  if(version>=4){const marks=p.u32();for(let j=0;j<marks;j++){const data=c.get(15);while(p.p<data.length&&data[p.p]!==10)p.p++;p.p++;const intervals=p.u32();p.p+=intervals*8;}}
 }
 if(!(speed>0))throw Error('Invalid playback rate');
 const motions=chunks(c.get(14));let selected;
 for(const [id,b] of motions){if(!id)continue;const r=reader(b);if(r.s()===name){selected={b,r};break;}}
 if(!selected)throw Error('Missing idle '+name);
 const {b,r}=selected,frames=r.u32(),tracks=[];
 for(let i=0;i<bones.length;i++){
  if(!bones[i])throw Error('Missing motion bone');
  const flags=b.readUInt8(r.p++),rotations=[],positions=[],constant=flags&2;
  if(!constant)r.p+=4;
  for(let f=0;f<(constant?1:frames);f++){
   // X-Ray's quaternion matrix is transposed relative to Three; then reflect Z.
   const q=[b.readInt16LE(r.p),b.readInt16LE(r.p+2),-b.readInt16LE(r.p+4),b.readInt16LE(r.p+6)];r.p+=8;
   const length=Math.hypot(...q);if(!length)throw Error('Zero quaternion');rotations.push(...q.map(v=>v/length));
  }
  if(flags&1){
   r.p+=4;const keys=[],wide=flags&4;
   for(let f=0;f<frames*3;f++){keys.push(wide?b.readInt16LE(r.p):b.readInt8(r.p));r.p+=wide?2:1;}
   const scale=[r.f(),r.f(),r.f()],base=[r.f(),r.f(),r.f()];
   for(let f=0;f<frames;f++)positions.push(...base.map((v,j)=>(v+keys[f*3+j]*scale[j])*(j===2?-1:1)));
  }else positions.push(r.f(),r.f(),-r.f());
  tracks.push({bone:bones[i],rotations,positions});
 }
 if(r.p!==b.length)throw Error('Unexpected motion layout');
 return {name,frames,fps:30*speed,tracks};
}
if(process.argv[1]&&path.resolve(process.argv[1])===path.resolve(import.meta.filename)){
 const [source,out]=process.argv.slice(2),repo=path.resolve(import.meta.dirname,'..');
 if(!source||!out)throw Error('Usage: import-idles.mjs <extracted Anomaly> <external web folder>');
 if(path.resolve(out).toLowerCase()===repo.toLowerCase()||path.resolve(out).toLowerCase().startsWith(repo.toLowerCase()+path.sep))throw Error('Assets must stay outside repository');
 const models={bandit:['actors/bandits_animation','norm_idle_1'],mercenary:['actors/stalker_animation','norm_idle_1'],dog:['monsters/dog_animation','stand_idle_0'],bloodsucker:['monsters/bloodsucker_animations','stand_idle_0'],controller:['monsters/controller_animation','stand_idle_0']};
 const output={};for(const [model,[file,clip]] of Object.entries(models))output[model]=decodeIdle(fs.readFileSync(path.join(source,'meshes',file+'.omf')),clip);
 fs.writeFileSync(path.join(out,'idle-clips.json'),JSON.stringify(output));console.log(Object.fromEntries(Object.entries(output).map(([k,v])=>[k,{frames:v.frames,bones:v.tracks.length}])));
}


