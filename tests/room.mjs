import assert from 'node:assert/strict';
import {randomBytes} from 'node:crypto';
const base=process.env.GAME_URL||'http://127.0.0.1:8788';
const sleep=ms=>new Promise(r=>setTimeout(r,ms));
const token=randomBytes(32).toString('hex');
async function client(key){return new Promise((resolve,reject)=>{
 const ws=new WebSocket(base.replace('http','ws')+'/ws?token='+key);let id,state;
 const timer=setTimeout(()=>{ws.close();reject(Error('Handshake timeout'))},10000);
 ws.onerror=reject;ws.onmessage=e=>{const msg=JSON.parse(e.data);if(msg.kind==='error'){clearTimeout(timer);ws.close();reject(Error(msg.text));}if(msg.kind==='welcome')id=msg.id;if(msg.kind==='state'){state=msg;if(id){clearTimeout(timer);resolve({ws,id,get state(){return state},send(v){ws.send(JSON.stringify(v))},get profile(){return state.players.find(p=>p.id===id)?.profile}})}}};
})}
const a=await client(token),b=await client(randomBytes(32).toString('hex'));
try{
 const origin=a.state.players.find(p=>p.id===a.id).pos[1];
 let peak=origin;const jumping=setInterval(()=>{a.send({kind:'input',jump:true});peak=Math.max(peak,a.state.players.find(p=>p.id===a.id).pos[1]);},50);
 await sleep(1600);clearInterval(jumping);assert(peak>origin+1,'jump must rise');
 assert(Math.abs(a.state.players.find(p=>p.id===a.id).pos[1]-origin)<.05,'held jump must land without bouncing');
 a.send({kind:'input',jump:false});await sleep(100);a.send({kind:'input',jump:true});await sleep(150);assert(a.state.players.find(p=>p.id===a.id).pos[1]>origin+.4,'release rearms jump');
 a.send({kind:'home'});await sleep(100);
 await sleep(100);assert(a.state.players.some(p=>p.id===b.id));
 await assert.rejects(client(token),/already connected/);
 a.send({kind:'name',name:'<script>Test'});a.send({kind:'equip',item:4});await sleep(100);assert.equal(a.profile.weapon,0);assert(!a.profile.name.includes('<'));
 const before=a.profile.power;for(let i=0;i<20;i++)a.send({kind:'train'});await sleep(100);assert.equal(a.profile.power,before,'old practice command disabled');
 a.send({kind:'claim_exp',stage:1});await sleep(100);assert.equal(a.profile.exp,0,'locked remote EXP claim rejected');
 const fire=a.state.campfire;
 for(let n=0;n<100;n++){const pos=a.state.players.find(p=>p.id===a.id).pos,dx=fire[0]-pos[0],dz=fire[2]-pos[2];if(Math.hypot(dx,dz)<2.5)break;a.send({kind:'input',forward:1,yaw:Math.atan2(-dx,-dz)});await sleep(70);}
 a.send({kind:'input'});await sleep(100);
 a.send({kind:'sit'});await sleep(1200);assert(a.profile.power>before,'sitting trains automatically');
 while(a.profile.power<20)await sleep(500);
 a.send({kind:'sit'});await sleep(100);const trained=a.profile.power;
 a.send({kind:'home'});await sleep(100);a.send({kind:'challenge'});await sleep(200);assert.equal(a.state.players.find(p=>p.id===a.id).duel,0,'nearest enemy chosen by server');
 await sleep(1000);assert.equal(a.state.players.find(p=>p.id===a.id).duel_health,200,'waiting deals no damage');
 for(let n=0;n<10;n++){a.send({kind:'attack',damage:999999});await sleep(100);}
 await sleep(150);assert(a.profile.trophies.includes(0));assert.equal(a.profile.power,trained);assert.equal(a.profile.exp,0,'victories do not directly grant EXP');
 a.send({kind:'buy',item:4});await sleep(100);assert.equal(a.profile.weapon,0,'trader removed');
 const marker=a.state.opponents[0].reward_pos;
 for(let n=0;n<80;n++){const pos=a.state.players.find(p=>p.id===a.id).pos,dx=marker[0]-pos[0],dz=marker[2]-pos[2];if(Math.hypot(dx,dz)<1.5)break;a.send({kind:'input',forward:1,yaw:Math.atan2(-dx,-dz)});await sleep(70);}
 a.send({kind:'input'});await sleep(100);a.send({kind:'claim_exp',stage:0});await sleep(150);assert.equal(a.profile.exp,1);assert.equal(a.state.players.find(p=>p.id===a.id).pos[0],-205,'EXP returns to camp');
 const saved=structuredClone(a.profile);saved.inventory.sort((a,b)=>a-b);a.ws.close();await sleep(300);
 const resumed=await client(token);try{assert.deepEqual(resumed.profile,saved)}finally{resumed.ws.close()}
 await sleep(2200);
 console.log('PASS shared room, profile identity, campfire training, removed practice/trader, stage gates, nearest-enemy battle, and reconnect persistence.');
}finally{a.ws.close();b.ws.close()}


