import * as THREE from './vendor/three.module.js';
// OGF bind rotations use X-Ray's setXYZi convention, followed by a Z reflection.
function bindMatrix(b){
 const [x,y,z]=b.rotation,h=-y,p=-x,t=-z,sh=Math.sin(h),ch=Math.cos(h),sp=Math.sin(p),cp=Math.cos(p),sb=Math.sin(t),cb=Math.cos(t);
 const a=[ch*cb-sp*sh*sb,-cp*sb,sp*ch*sb+sh*cb,0,sp*sh*cb+ch*sb,cp*cb,sh*sb-sp*ch*cb,0,-cp*sh,sp,cp*ch,0,...b.position,1];
 for(let i=0;i<16;i++)if((i%4===2)!==(Math.floor(i/4)===2))a[i]*=-1;
 return new THREE.Matrix4().fromArray(a);
}
export async function loadCharacters(resources,texture){
 const source=JSON.parse(new TextDecoder().decode(await resources.get('characters.json'))),templates={};
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
  // Neutral and bandit/mercenary OGF skeletons have opposite bind headings.
  // Normalize each to the gameplay convention: forward is -Z.
  // Rotate the visual after binding, leaving gameplay heading and labels unchanged.
  model.rotation.y=['bandit','mercenary'].includes(name)?0:Math.PI;
  root.userData.bones=bones;root.userData.rest=bones.map(b=>b.quaternion.clone());
  return root;
 };
}
export function animateCharacter(root,time,moving,active,sitting=false){
 const {bones,rest}=root.userData;root.children[0].position.y=sitting?-.5:0;
 bones.forEach((bone,i)=>{
  bone.quaternion.copy(rest[i]);const name=bone.name.toLowerCase();
  const side=/(_l_|left)/.test(name)?1:-1;
  if(sitting&&/thigh/.test(name))bone.rotateX(-1.3);
  if(sitting&&/calf/.test(name))bone.rotateX(1.3);
  if(/upperarm/.test(name))bone.rotateZ(-side*.95);
  if(/thigh|upperarm/.test(name)&&moving)bone.rotateX(Math.sin(time*.014+(side>0?0:Math.PI))*.4);
  if(/forearm/.test(name))bone.rotateX(active?-.9:-.2);
 });
}
