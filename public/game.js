import * as THREE from 'three';
import {DDSLoader} from './vendor/DDSLoader.js';
import {ResourceCache} from './resource-cache.js';
const $=id=>document.getElementById(id), canvas=$('view');
const renderer=new THREE.WebGLRenderer({canvas,antialias:true,powerPreference:'high-performance'});
renderer.setPixelRatio(Math.min(devicePixelRatio,1.5));renderer.setSize(innerWidth,innerHeight);renderer.outputColorSpace=THREE.SRGBColorSpace;renderer.setClearColor(0x8c9990);
const scene=new THREE.Scene();scene.fog=new THREE.FogExp2(0x8c9990,.0035);scene.add(new THREE.HemisphereLight(0xe6ecd5,0x57594b,2.1));
const sun=new THREE.DirectionalLight(0xffe3b0,1.8);sun.position.set(-70,130,-90);scene.add(sun);
const camera=new THREE.PerspectiveCamera(65,innerWidth/innerHeight,.05,260);camera.position.set(-185,4,165);camera.lookAt(-215,-17,122);
const dds=new DDSLoader(),materials=new Map(),actors=new Map(),enemies=[],staticMeshes=[];
const keys=new Set();let state,id,ws,playing=false,yaw=Math.PI,pitch=.3,dragging=false,holdTrain=false,auto=false,selected=0;
let last=performance.now(),inputAt=0,trainAt=0,lastEvent=null,noticeUntil=0,profileKey='',lastUi=0,cameraAt=0,cameraDistance=6;
const tracers=[];
let audio;const raycaster=new THREE.Raycaster(),desired=new THREE.Vector3(),focus=new THREE.Vector3();
let token;try{token=localStorage.getItem('cordon-profile');if(!/^[a-f0-9]{64}$/.test(token||'')){token=Array.from(crypto.getRandomValues(new Uint8Array(32)),v=>v.toString(16).padStart(2,'0')).join('');localStorage.setItem('cordon-profile',token)}$('name').value=localStorage.getItem('cordon-name')||'Stalker';}catch{token=Array.from(crypto.getRandomValues(new Uint8Array(32)),v=>v.toString(16).padStart(2,'0')).join('');}
function send(v){if(ws?.readyState===WebSocket.OPEN)ws.send(JSON.stringify(v));}
function notify(text){$('notification').textContent=text;noticeUntil=performance.now()+5000;}
function sound(win=false){if(!audio)return;const o=audio.createOscillator(),g=audio.createGain();o.type='sine';o.frequency.value=win?660:220;g.gain.setValueAtTime(.035,audio.currentTime);g.gain.exponentialRampToValueAtTime(.001,audio.currentTime+.18);o.connect(g).connect(audio.destination);o.start();o.stop(audio.currentTime+.18);}
function box(parent,size,pos,color){const m=new THREE.Mesh(new THREE.BoxGeometry(...size),new THREE.MeshStandardMaterial({color,roughness:.85}));m.position.set(...pos);parent.add(m);return m;}
function sphere(parent,r,pos,color){const m=new THREE.Mesh(new THREE.SphereGeometry(r,10,8),new THREE.MeshStandardMaterial({color,roughness:.8}));m.position.set(...pos);parent.add(m);return m;}
function stalker(color=0x52634c){
 const root=new THREE.Group(),body=new THREE.Group();root.add(body);
 const coat=box(body,[.65,.85,.4],[0,1.05,0],color);box(body,[.5,.3,.36],[0,.61,0],color);
 const legs=[box(body,[.24,.65,.28],[-.17,.32,0],0x28332a),box(body,[.24,.65,.28],[.17,.32,0],0x28332a)];
 box(body,[.5,.58,.27],[0,1.1,.31],0x534c33);sphere(body,.26,[0,1.63,0],color);box(body,[.29,.18,.16],[0,1.62,-.19],0xbaa17a);box(body,[.3,.08,.06],[0,1.64,-.29],0x202a23);
 const arms=[box(body,[.18,.65,.22],[-.44,1.02,0],color),box(body,[.18,.65,.22],[.44,1.02,0],color)];
 const gun=box(body,[.12,.14,.75],[.36,1.05,-.38],0x202922);
 root.userData={body,coat,legs,arms,gun};scene.add(root);return root;
}
function mutant(species){
 if(species==='dog'){
  const root=new THREE.Group();box(root,[.45,.48,1.05],[0,.57,0],0x77735c);sphere(root,.26,[0,.65,-.6],0x8d8972);box(root,[.28,.2,.36],[0,.53,-.8],0x5d6252);
  for(const x of [-.18,.18])for(const z of [-.35,.35])box(root,[.12,.43,.13],[x,.24,z],0x676b54);
  for(const x of [-.16,.16])box(root,[.08,.23,.12],[x,.93,-.56],0x5b6350);
  root.userData={body:root};scene.add(root);return root;
 }
 const root=stalker(species==='bandit'?0x44453e:species==='mercenary'?0x3f5863:0x727463);
 if(species==='bloodsucker'){root.userData.gun.visible=false;root.scale.set(1.1,1.18,1.1);for(const x of [-.15,-.05,.05,.15])box(root.userData.body,[.045,.32,.055],[x,1.4,-.28],0x705949);}
 if(species==='controller'){root.userData.gun.visible=false;root.scale.set(1.4,1.22,1.15);sphere(root.userData.body,.32,[0,1.7,0],0x92967a);}
 return root;
}
function label(text,color='#e0edc1'){
 const c=document.createElement('canvas');c.width=512;c.height=96;const ctx=c.getContext('2d');ctx.fillStyle='#10201cdd';ctx.fillRect(0,0,512,96);ctx.fillStyle=color;ctx.textAlign='center';ctx.font='bold 30px monospace';ctx.fillText(text,256,58);
 const texture=new THREE.CanvasTexture(c);const sprite=new THREE.Sprite(new THREE.SpriteMaterial({map:texture,depthTest:true}));sprite.scale.set(3.2,.6,1);return sprite;
}
function strength(p){return p.power+state.items[p.weapon].power+state.items[p.armor].power;}
function connect(){
 if(playing||ws?.readyState===WebSocket.CONNECTING)return;audio??=new AudioContext();audio.resume();$('deploy').disabled=true;
 ws=new WebSocket(`${location.protocol==='https:'?'wss':'ws'}://${location.host}/ws?token=${token}`);
 let failure='Connection lost. Refresh to reconnect; your progression is saved.';
 ws.onmessage=e=>{
  const msg=JSON.parse(e.data);if(msg.kind==='error'){failure=msg.text;notify(failure);return;}
  if(msg.kind==='welcome'){id=msg.id;send({kind:'name',name:$('name').value});try{localStorage.setItem('cordon-name',$('name').value)}catch{}playing=true;$('lobby').hidden=true;$('footer').hidden=true;$('hud').hidden=false;document.body.classList.add('playing');canvas.focus();return;}
  if(msg.kind!=='state')return;state=msg;
  if(lastEvent===null)lastEvent=state.events.at(-1)?.seq||0;
  for(const event of state.events){if(event.seq<=lastEvent)continue;lastEvent=event.seq;if(event.who===id){notify(event.text);if(event.kind==='win')sound(true);}}
 };
 ws.onclose=()=>{playing=false;keys.clear();auto=false;holdTrain=false;notify(failure);$('connection').textContent='DISCONNECTED';$('deploy').disabled=false;};
 ws.onerror=()=>notify('Cannot reach the room.');
}
$('deploy').onclick=connect;
$('train').onclick=()=>{send({kind:'train'});sound();};
$('auto').onclick=()=>{auto=!auto;$('auto').textContent=`AUTO: ${auto?'ON':'OFF'}`;$('auto').classList.toggle('active',auto);};
function toggleInventory(){const panel=$('inventory');panel.hidden=!panel.hidden;keys.clear();holdTrain=false;}
$('inventoryToggle').onclick=toggleInventory;$('closeInventory').onclick=toggleInventory;
$('challenge').onclick=()=>send({kind:'challenge',opponent:selected});
$('promote').onclick=()=>send({kind:'promote'});$('home').onclick=()=>send({kind:'home'});
canvas.addEventListener('contextmenu',e=>e.preventDefault());canvas.addEventListener('mousedown',e=>{canvas.focus();if(e.button===2)dragging=true;if(e.button===0)holdTrain=true;});
document.addEventListener('mouseup',()=>{dragging=false;holdTrain=false;});
document.addEventListener('mousemove',e=>{if(dragging){yaw-=e.movementX*.004;pitch=Math.max(-.1,Math.min(.8,pitch+e.movementY*.003));}});
document.addEventListener('keydown',e=>{
 if(!playing||e.target instanceof HTMLInputElement)return;
 if(e.code==='KeyI'&&!e.repeat)toggleInventory();if(e.code==='Escape'){$('inventory').hidden=true;keys.clear();holdTrain=false;}
 if(!$('inventory').hidden)return;
 if(e.code==='KeyE'&&!e.repeat)send({kind:'challenge',opponent:selected});if(e.code==='KeyB'&&!e.repeat)send({kind:'home'});
 if(['KeyW','KeyA','KeyS','KeyD','ShiftLeft','Space','ArrowLeft','ArrowRight','ArrowUp','ArrowDown'].includes(e.code)){keys.add(e.code);e.preventDefault();}
});
document.addEventListener('keyup',e=>keys.delete(e.code));window.addEventListener('blur',()=>{keys.clear();holdTrain=false;dragging=false;});
function updateInventory(me){
 const p=me.profile;const key=JSON.stringify([p.inventory,p.rubles,p.weapon,p.armor,me.duel]);if(profileKey===key)return;profileKey=key;
 $('equipment').textContent=`WEAPON: ${state.items[p.weapon].name}   /   ARMOR: ${state.items[p.armor].name}`;
 $('items').replaceChildren();state.items.forEach((item,n)=>{
  const equipped=n===p.weapon||n===p.armor,owned=p.inventory.includes(n),card=document.createElement('div');card.className='item'+(equipped?' equipped':'');
  const symbol=document.createElement('span');symbol.className='symbol';symbol.textContent=item.slot==='weapon'?'⌁':'♜';
  const title=document.createElement('b');title.textContent=item.name;const desc=document.createElement('small');desc.textContent=`${item.slot.toUpperCase()} · +${item.power} strength`;
  const button=document.createElement('button');button.textContent=equipped?'EQUIPPED':owned?'EQUIP':`BUY · ₽${item.price}`;button.disabled=equipped||(!owned&&p.rubles<item.price)||me.duel!==null;button.onclick=()=>send({kind:owned?'equip':'buy',item:n});
  card.append(symbol,title,desc,button);$('items').append(card);
 });
}
function updateUI(me){
 const p=me.profile,total=strength(p);$('power').textContent=total.toLocaleString();$('breakdown').textContent=`${p.power} fieldcraft + ${total-p.power} equipment`;
 $('rubles').textContent=p.rubles.toLocaleString();$('wins').textContent=p.wins.toLocaleString();$('rank').textContent=`${state.ranks[p.rank].toUpperCase()} · ×${2**p.rank} TRAINING`;
 const cost=state.promotions[p.rank];$('promote').textContent=cost?`PROMOTE · ${cost} WINS`:'LEGEND';$('promote').disabled=!cost||p.wins<cost||me.duel!==null;
 $('promotionHint').textContent=cost?'Resets fieldcraft. Keeps inventory & rubles. Doubles training gains.':'You reached the highest rank.';
 $('train').textContent=`PRACTICE +${2**p.rank}`;$('train').disabled=me.duel!==null;
 $('opponents').replaceChildren();state.opponents.forEach((o,n)=>{
  const distance=Math.hypot(...o.pos.map((v,i)=>v-me.pos[i]));const button=document.createElement('button');button.className='threat'+(n===selected?' selected':'');
  const text=document.createElement('div'),name=document.createElement('b'),desc=document.createElement('small');name.textContent=o.name;desc.textContent=`${o.power.toLocaleString()} strength · ${Math.round(distance)}m · +${o.wins} wins`;text.append(name,desc);const icon=document.createElement('span');icon.textContent=total>=o.power?'✓':'↑';button.append(text,icon);button.onclick=()=>{selected=n;notify(`Track ${o.name}: approach the marked enemy, then press E.`);};$('opponents').append(button);
 });
 const enemy=state.opponents[selected],distance=Math.hypot(...enemy.pos.map((v,i)=>v-me.pos[i]));$('challenge').disabled=distance>6||me.duel!==null;$('challenge').textContent=distance>6?`${Math.round(distance)}m TO ${enemy.name.toUpperCase()}`:`FIGHT ${enemy.name.toUpperCase()} [E]`;
 $('duelPanel').hidden=me.duel===null;if(me.duel!==null){const o=state.opponents[me.duel];$('duelTitle').textContent=`VS ${o.name.toUpperCase()}`;$('duelBar').style.width=(1-me.duel_remaining/3)*100+'%';$('duelHint').textContent=`Your strength ${total} / Enemy ${o.power}`;}
 $('leaderboard').replaceChildren();[...state.players].sort((a,b)=>strength(b.profile)-strength(a.profile)).forEach(player=>{const row=document.createElement('div');row.className='score';const name=document.createElement('span'),score=document.createElement('b');name.textContent=player.profile.name+(player.id===id?' (you)':'');score.textContent=strength(player.profile).toLocaleString();row.append(name,score);$('leaderboard').append(row);});
 $('connection').textContent=`LIVE · ${state.players.length} STALKERS · AUTOSAVE`;updateInventory(me);
}
function frame(now){
 requestAnimationFrame(frame);const dt=Math.min((now-last)/1000,.05);last=now;
 if(now>noticeUntil)$('notification').textContent='';
 for(let n=tracers.length-1;n>=0;n--)if(now>tracers[n].until){const line=tracers[n].line;scene.remove(line);line.geometry.dispose();line.material.dispose();tracers.splice(n,1);}
 if(playing&&state){
  const me=state.players.find(p=>p.id===id);if(!me)return;
  if(keys.has('ArrowLeft'))yaw+=dt*1.8;if(keys.has('ArrowRight'))yaw-=dt*1.8;if(keys.has('ArrowUp'))pitch=Math.max(-.1,pitch-dt);if(keys.has('ArrowDown'))pitch=Math.min(.8,pitch+dt);
  if(now-inputAt>45){inputAt=now;send({kind:'input',forward:Number(keys.has('KeyW'))-Number(keys.has('KeyS')),right:Number(keys.has('KeyD'))-Number(keys.has('KeyA')),yaw,sprint:keys.has('ShiftLeft')});}
  if((auto||holdTrain||keys.has('Space'))&&now-trainAt>205&&me.duel===null){trainAt=now;send({kind:'train'});}
  const present=new Set();for(const p of state.players){present.add(p.id);let actor=actors.get(p.id);if(!actor){actor=stalker();actors.set(p.id,actor);actor.position.fromArray(p.pos);const tag=label(p.profile.name);tag.position.y=2.15;tag.visible=p.id!==id;actor.add(tag);}
   const moving=actor.position.distanceTo(new THREE.Vector3(...p.pos))>.025;actor.position.lerp(new THREE.Vector3(...p.pos),1-Math.exp(-dt*18));actor.rotation.y=p.yaw;
   const data=actor.userData;data.legs.forEach((leg,n)=>leg.rotation.x=moving?Math.sin(now*.014+n*Math.PI)*.5:0);
   data.arms.forEach((arm,n)=>arm.rotation.x=p.training>0?-1.2+Math.sin(now*.03)*.35:p.duel!==null?-1.2:0);
   data.coat.material.color.setHex(p.profile.armor===7?0x596974:p.profile.armor===6?0x9a9e6b:[0x52634c,0x69754e,0x688b62,0x6b848a,0x8b7d55,0xa49456,0xb9a55b][p.profile.rank]);
   data.gun.scale.z=1+p.profile.weapon*.18;
   if(p.duel!==null){const o=state.opponents[p.duel];actor.rotation.y=Math.atan2(-(o.pos[0]-p.pos[0]),-(o.pos[2]-p.pos[2]));
    if(now-(data.lastShot||0)>300){data.lastShot=now;const geometry=new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(...p.pos).add(new THREE.Vector3(0,1.15,0)),new THREE.Vector3(...o.pos).add(new THREE.Vector3(0,o.species==='dog'?.5:1.1,0))]);const line=new THREE.Line(geometry,new THREE.LineBasicMaterial({color:0xffd786,transparent:true,opacity:.8}));scene.add(line);tracers.push({line,until:now+70});if(p.id===id)sound();}
   }
  }
  for(const [key,actor] of actors)if(!present.has(key)){scene.remove(actor);actor.traverse(o=>{o.geometry?.dispose();if(o.material){o.material.map?.dispose();o.material.dispose();}});actors.delete(key);}
  state.opponents.forEach((o,n)=>{if(!enemies[n]){const actor=mutant(o.species);actor.position.fromArray(o.pos);const tag=label(`${o.name} · ${o.power}`);tag.position.y=o.species==='dog'?1.5:2.3;actor.add(tag);const ring=new THREE.Mesh(new THREE.RingGeometry(.8,1,40),new THREE.MeshBasicMaterial({color:0xd8f071,side:THREE.DoubleSide}));ring.rotation.x=-Math.PI/2;ring.position.y=.06;actor.add(ring);actor.userData.ring=ring;enemies[n]=actor;}
   const enemy=enemies[n];enemy.userData.ring.visible=n===selected;enemy.rotation.y=Math.atan2(-(me.pos[0]-o.pos[0]),-(me.pos[2]-o.pos[2]));const battling=state.players.some(p=>p.duel===n);enemy.position.y=o.pos[1]+(battling?Math.abs(Math.sin(now*.016))*.12:0);
  });
  focus.set(me.pos[0],me.pos[1]+1.4,me.pos[2]);const dir=new THREE.Vector3(Math.sin(yaw)*Math.cos(pitch),Math.sin(pitch),Math.cos(yaw)*Math.cos(pitch));
  if(now-cameraAt>100){cameraAt=now;raycaster.set(focus,dir);raycaster.far=6;const nearby=staticMeshes.filter(m=>m.geometry.boundingSphere.center.distanceTo(focus)<m.geometry.boundingSphere.radius+6);const hits=raycaster.intersectObjects(nearby,false);cameraDistance=hits.length?Math.max(.35,hits[0].distance-.2):6;}
  desired.copy(focus).addScaledVector(dir,cameraDistance);if(camera.position.distanceTo(desired)>15)camera.position.copy(desired);else camera.position.lerp(desired,1-Math.exp(-dt*15));camera.lookAt(focus);actors.get(id).visible=cameraDistance>.8;
  if(now-lastUi>200){lastUi=now;updateUI(me);}
 }else{camera.position.set(-185+Math.sin(now*.00003)*10,4,165);camera.lookAt(-215,-17,122);}
 renderer.render(scene,camera);
}
window.addEventListener('resize',()=>{renderer.setSize(innerWidth,innerHeight);camera.aspect=innerWidth/innerHeight;camera.updateProjectionMatrix();});requestAnimationFrame(frame);

async function loadWorld(){
 const resources=await ResourceCache.open();const info=JSON.parse(new TextDecoder().decode(await resources.get('world.json')));
 $('loading').textContent='Loading Cordon geometry from your local installation…';const data=await resources.get('world.bin');
 const textureJobs=[];for(let index=0;index<info.meshes.length;index++){
  const m=info.meshes[index];let material=materials.get(m.texture);if(!material){material=new THREE.MeshStandardMaterial({color:0xbabbb0,roughness:1,side:THREE.DoubleSide,alphaTest:m.alpha?.45:0});materials.set(m.texture,material);textureJobs.push({material,name:m.texture});}
  const geometry=new THREE.BufferGeometry(),interleaved=new THREE.InterleavedBuffer(new Float32Array(data,m.offset,m.vertices*5),5);
  geometry.setAttribute('position',new THREE.InterleavedBufferAttribute(interleaved,3,0));geometry.setAttribute('uv',new THREE.InterleavedBufferAttribute(interleaved,2,3));geometry.setIndex(new THREE.BufferAttribute(new Uint16Array(data,m.offset+m.vertices*20,m.indices),1));geometry.computeVertexNormals();geometry.computeBoundingSphere();const mesh=new THREE.Mesh(geometry,material);mesh.matrixAutoUpdate=false;mesh.updateMatrix();scene.add(mesh);staticMeshes.push(mesh);
  if(index%400===0){$('loading').textContent=`Building Cordon · ${Math.round(index/info.meshes.length*100)}%`;await new Promise(r=>setTimeout(r,0));}
 }
 let done=0,missing=0;async function worker(){while(textureJobs.length){const job=textureJobs.pop();try{const bytes=await resources.get('textures/'+job.name+'.dds');const url=URL.createObjectURL(new Blob([bytes]));let texture;try{texture=await dds.loadAsync(url);}finally{URL.revokeObjectURL(url);}texture.wrapS=texture.wrapT=THREE.RepeatWrapping;texture.colorSpace=THREE.SRGBColorSpace;texture.anisotropy=2;job.material.map=texture;job.material.color.set(0xffffff);job.material.needsUpdate=true;}catch{missing++;}done++;$('loading').textContent=`Applying local textures · ${done}/${materials.size}`;}}
 await Promise.all(Array.from({length:6},worker));$('loading').textContent=`Actual Anomaly Cordon · ${materials.size-missing} local textures · 1 shared room`;if(missing)$('loading').textContent+=` · ${missing} missing textures use fallback colors`;
 $('loading').textContent+=` · ${resources.hits} cached / ${resources.downloads} downloaded chunks`;
 $('deploy').disabled=false;$('deploy').innerHTML='ENTER THE ZONE <span>↗</span>';$('connection').textContent='CORDON READY';$('lamp').classList.add('live');
}

loadWorld().catch(e=>{$('loading').textContent=e.message;$('connection').textContent='LOAD FAILED';console.error(e);});
