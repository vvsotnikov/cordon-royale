import * as THREE from './vendor/three.module.js';
import {locomotionState} from './locomotion.js';
// OGF bind rotations use X-Ray's setXYZi convention, followed by a Z reflection.
function bindMatrix(b){
 const [x,y,z]=b.rotation,h=-y,p=-x,t=-z,sh=Math.sin(h),ch=Math.cos(h),sp=Math.sin(p),cp=Math.cos(p),sb=Math.sin(t),cb=Math.cos(t);
 const a=[ch*cb-sp*sh*sb,-cp*sb,sp*ch*sb+sh*cb,0,sp*sh*cb+ch*sb,cp*cb,sh*sb-sp*ch*cb,0,-cp*sh,sp,cp*ch,0,...b.position,1];
 for(let i=0;i<16;i++)if((i%4===2)!==(Math.floor(i/4)===2))a[i]*=-1;
 return new THREE.Matrix4().fromArray(a);
}
function makeClip(idle,bones){
   const tracks=[];
   for(const track of idle.tracks){
    const bone=bones.find(b=>b.name===track.bone);if(!bone)throw Error('Missing idle bone '+track.bone);
    for(const [property,values,size,Type] of [['quaternion',track.rotations,4,THREE.QuaternionKeyframeTrack],['position',track.positions,3,THREE.VectorKeyframeTrack]]){
     const count=values.length/size,times=Array.from({length:count},(_,i)=>i/idle.fps),loopValues=values.slice();
     if(count>1){times.push(idle.frames/idle.fps);loopValues.push(...values.slice(0,size));}
     tracks.push(new Type(bone.uuid+'.'+property,times,loopValues));
    }
   }

 return new THREE.AnimationClip(idle.name,idle.frames/idle.fps,tracks);
}
export async function loadCharacters(resources,texture){
 const source=JSON.parse(new TextDecoder().decode(await resources.get('characters.json'))),templates={},idles=JSON.parse(new TextDecoder().decode(await resources.get('idle-clips.json'))),playerMotion=JSON.parse(new TextDecoder().decode(await resources.get('player-motion.json')));
 for(const [name,model] of Object.entries(source)){
  const parts=[];
  for(const m of model.meshes){
   const g=new THREE.BufferGeometry();g.setAttribute('position',new THREE.Float32BufferAttribute(m.positions,3));g.setAttribute('uv',new THREE.Float32BufferAttribute(m.uv,2));g.setAttribute('skinIndex',new THREE.Uint16BufferAttribute(m.joints,4));g.setAttribute('skinWeight',new THREE.Float32BufferAttribute(m.weights,4));g.setIndex(m.indices);g.computeVertexNormals();
   parts.push({geometry:g,material:new THREE.MeshStandardMaterial({map:await texture(m.texture),roughness:.9,side:THREE.DoubleSide})});
  }
  templates[name]={bones:model.bones,parts};
 }
 return name=>{
  const template=templates[name],root=new THREE.Group(),model=new THREE.Group(),bones=template.bones.map(b=>{const bone=new THREE.Bone();bone.name=b.name;bindMatrix(b).decompose(bone.position,bone.quaternion,bone.scale);return bone;});
  root.add(model);
  template.bones.forEach((b,i)=>{const parent=template.bones.findIndex(p=>p.name===b.parent);(parent<0?model:bones[parent]).add(bones[i]);});
  root.updateMatrixWorld(true);const skeleton=new THREE.Skeleton(bones);
  for(const part of template.parts){const mesh=new THREE.SkinnedMesh(part.geometry,part.material);model.add(mesh);mesh.bind(skeleton);mesh.frustumCulled=false;}
  // OMF enemy poses already face -Z after conversion; the procedural player
  // uses its opposite-facing bind pose. Apply heading after binding.
  model.rotation.y=idles[name]?0:Math.PI;
  if(idles[name]){
   const idle=idles[name];
   const mixer=new THREE.AnimationMixer(model);mixer.clipAction(makeClip(idle,bones)).play();root.userData.idle=mixer;
  }
  if(name==='stalker'){
   const mixer=new THREE.AnimationMixer(model),actions=Object.fromEntries(Object.entries(playerMotion).map(([state,clip])=>[state,mixer.clipAction(makeClip(clip,bones))]));
   root.userData.motion={mixer,actions,state:null,lastTime:null};
  }
  root.userData.restPositions=bones.map(b=>b.position.clone());root.userData.bones=bones;root.userData.rest=bones.map(b=>b.quaternion.clone());
  return root;
 };
}
export function animateCharacter(root,time,moving,active,sitting=false){
 if(root.userData.idle){root.userData.idle.setTime(time/1000);return;}
 const {bones,rest,motion}=root.userData,model=root.children[0];model.position.y=sitting?-.5:0;
 if(motion){
  const dt=motion.lastTime===null?0:Math.min(.1,Math.max(0,(time-motion.lastTime)/1000));motion.lastTime=time;
  if(!sitting){
   const speed=typeof moving==='number'?moving:(moving?4.8:0),state=locomotionState(speed,motion.state);
   model.rotation.y=0;
   if(state!==motion.state){const previous=motion.actions[motion.state],next=motion.actions[state];const phase=previous&&motion.state!=='idle'&&state!=='idle'?previous.time/previous.getClip().duration:0;
    next.reset().setEffectiveWeight(1).play();next.time=phase*next.getClip().duration;if(previous)previous.crossFadeTo(next,.18,false);motion.state=state;}
   motion.mixer.update(dt);return;
  }
  motion.mixer.stopAllAction();motion.state=null;model.rotation.y=Math.PI;
 }

 bones.forEach((bone,i)=>{
  bone.position.copy(root.userData.restPositions[i]);bone.quaternion.copy(rest[i]);const name=bone.name.toLowerCase();
  const side=/(_l_|left)/.test(name)?1:-1;
  if(sitting&&/thigh/.test(name))bone.rotateX(-1.3);
  if(sitting&&/calf/.test(name))bone.rotateX(1.3);
  if(/upperarm/.test(name))bone.rotateZ(-side*.95);
  if(/thigh|upperarm/.test(name)&&moving)bone.rotateX(Math.sin(time*.014+(side>0?0:Math.PI))*.4);
  if(/forearm/.test(name))bone.rotateX(active?-.9:-.2);
 });
}
