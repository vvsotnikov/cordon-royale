import assert from 'node:assert/strict';
const base=process.env.GAME_URL||'http://127.0.0.1:8787';
const sleep=ms=>new Promise(r=>setTimeout(r,ms));
assert.equal((await fetch(base+'/health')).status,200);
function client(){return new Promise((resolve,reject)=>{const ws=new WebSocket(base.replace('http','ws')+'/ws');let state,id;const timer=setTimeout(()=>{ws.close();reject(Error('Handshake timeout'))},15000);ws.onerror=reject;ws.onmessage=e=>{const msg=JSON.parse(e.data);if(msg.kind==='welcome')id=msg.id;if(msg.kind==='state'){state=msg;if(id){clearTimeout(timer);resolve({ws,get state(){return state},id})}}};})}
const a=await client(),b=await client();
const me=()=>a.state.players.find(p=>p.id===a.id);
const send=v=>a.ws.send(JSON.stringify(v));
try{
 assert.notEqual(a.id,b.id);send({kind:'name',name:'<script>Test</script>'});await sleep(200);
 assert(a.state.players.some(p=>p.id===b.id));assert(b.state.players.some(p=>p.id===a.id));assert(!me().name.includes('<'));assert(me().pos.every(Number.isFinite));
 assert.equal(a.state.phase,'waiting','run this test on an empty room');
 const before=[...me().pos];const stash=a.state.stashes.filter(s=>!s.open).sort((x,y)=>Math.hypot(x.pos[0]-before[0],x.pos[2]-before[2])-Math.hypot(y.pos[0]-before[0],y.pos[2]-before[2]))[0];
 for(let i=0;i<30;i++){const p=me().pos,dx=stash.pos[0]-p[0],dz=stash.pos[2]-p[2];if(Math.hypot(dx,dz)<2)break;send({forward:1,yaw:Math.atan2(-dx,-dz)});await sleep(50)}
 assert(Math.hypot(...me().pos.map((x,i)=>x-before[i]))>.2,'player actually moves on imported collision');
 for(let i=0;i<35;i++){send({interact:true});await sleep(50)}
 assert(me().armor>=25,'search awards armor');assert(me().meds>=2,'search awards a medkit');assert(a.state.events.some(e=>e.kind==='loot'&&e.who===a.id));
 const armor=me().armor;for(let i=0;i<30;i++){send({interact:true});await sleep(50)}assert.equal(me().armor,armor,'opened stash cannot award twice');
 const pos=[...me().pos];send({forward:999,right:999});await sleep(500);assert(Math.hypot(...me().pos.map((x,i)=>x-pos[i]))<1,'invalid movement cannot teleport');
 const deadline=Date.now()+20000;while(a.state.phase==='waiting'&&Date.now()<deadline)await sleep(100);
 assert.equal(a.state.phase,'active');const ammo=me().ammo;
 for(let i=0;i<8;i++){send({fire:true,yaw:0,pitch:1.4});await sleep(50)}
 assert(me().ammo<ammo,'server consumes ammunition during combat');assert(a.state.events.some(e=>['shot','hit'].includes(e.kind)&&e.who===a.id));
 const reserve=me().reserve;send({reload:true});await sleep(2000);assert.equal(me().ammo,me().weapon?30:12);assert(me().reserve<reserve,'reload transfers reserve ammunition');
 console.log('PASS two clients, shared state, imported terrain movement, stash rewards, no duplicate loot, invalid input rejection.');
}finally{a.ws.close();b.ws.close()}
await sleep(300);const c=await client();try{assert.equal(c.state.phase,'waiting');assert.equal(c.state.players.find(p=>p.id===c.id).hp,100);assert(c.state.stashes.every(s=>!s.open));console.log('PASS empty room resets inventory and stashes for the next player.')}finally{c.ws.close()}



