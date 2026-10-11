// Reads locally extracted Anomaly OGF files; all generated assets stay external.
import fs from 'node:fs';
import path from 'node:path';
const [source,out]=process.argv.slice(2);
if(!source||!out)throw Error('Usage: node tools/import-characters.mjs <extracted Anomaly> <external web folder>');
const repo=path.resolve(import.meta.dirname,'..');
if(path.resolve(out).toLowerCase().startsWith(repo.toLowerCase()+path.sep)||path.resolve(out).toLowerCase()===repo.toLowerCase())throw Error('Assets must stay outside the repository');
function chunks(b){const m=new Map();for(let p=0;p<b.length;){const id=b.readUInt32LE(p),n=b.readUInt32LE(p+4);if(id&0x80000000||p+8+n>b.length)throw Error('Unsupported OGF chunk');m.set(id,b.subarray(p+8,p+8+n));p+=8+n;}return m;}
function reader(b){return {b,p:0,u(){const v=b.readUInt32LE(this.p);this.p+=4;return v;},f(){const v=b.readFloatLE(this.p);this.p+=4;return v;},s(){const end=b.indexOf(0,this.p);if(end<0)throw Error('Missing string terminator');const v=b.toString('utf8',this.p,end);this.p=end+1;return v;}};}
const models={stalker:'actors/stalker_neutral/st_nt_1',bandit:'actors/stalker_bandit/stalker_bandit2a',mercenary:'actors/stalker_merc/stalker_merc_0',dog:'monsters/dog/dog',bloodsucker:'monsters/krovosos/krovosos',controller:'monsters/controller/controller_1'};
Object.assign(models,{player_0:'actors/stalker_hero_coc/sviter_3',player_1:'actors/stalker_neutral/stalker_neutral0a',player_2:'actors/stalker_neutral/stalker_neutral1a',player_3:'actors/stalker_neutral/stalker_neutral3a',player_4:'actors/stalker_neutral/stalker_neutral4a',player_5:'actors/stalker_dolg/stalker_dolg4nosorog'});
const result={},textures=new Set();
for(const [name,file] of Object.entries(models)){
 const c=chunks(fs.readFileSync(path.join(source,'meshes',file+'.ogf'))),r=reader(c.get(13)),count=r.u(),bones=[];
 for(let i=0;i<count;i++){bones.push({name:r.s(),parent:r.s()});r.p+=60;}
 const ik=reader(c.get(16));for(const bone of bones){const version=ik.u();ik.s();ik.p+=112+(version?76:72);bone.rotation=[ik.f(),ik.f(),ik.f()];bone.position=[ik.f(),ik.f(),ik.f()];ik.p+=16;}
 if(ik.p!==ik.b.length)throw Error('Unexpected bone layout '+name);
 const meshes=[];
 for(const child of chunks(c.get(9)).values()){
  const m=chunks(child),v=m.get(3),n=v.readUInt32LE(4),stride=(v.length-8)/n;
  if(![60,64,70,76].includes(stride))throw Error('Unsupported vertex stride '+stride);
  const links={60:1,64:2,70:3,76:4}[stride],positions=[],uv=[],indices=[],joints=[],weights=[];
  for(let i=0;i<n;i++){
   const a=8+i*stride,p=a+(links===1?0:links*2);
   positions.push(v.readFloatLE(p),v.readFloatLE(p+4),-v.readFloatLE(p+8));
   uv.push(v.readFloatLE(a+stride-(links===1?12:8)),v.readFloatLE(a+stride-(links===1?8:4)));
   const js=links===1?[v.readUInt32LE(a+56)]:Array.from({length:links},(_,j)=>v.readUInt16LE(a+j*2));
   let ws=links===1?[1]:Array.from({length:links-1},(_,j)=>v.readFloatLE(p+48+j*4));
   if(links===2)ws=[1-ws[0],ws[0]];else if(links>2)ws.push(1-ws.reduce((a,b)=>a+b,0));
   if(js.some(j=>j>=bones.length))throw Error('Invalid bone index');
   joints.push(...js,...Array(4-links).fill(0));weights.push(...ws,...Array(4-links).fill(0));
  }
  const ib=m.get(4),sw=m.get(6),start=sw?sw.readUInt32LE(20):0,total=sw?sw.readUInt16LE(24)*3:ib.readUInt32LE(0);
  for(let i=0;i<total;i+=3)indices.push(ib.readUInt16LE(4+(start+i)*2),ib.readUInt16LE(4+(start+i+2)*2),ib.readUInt16LE(4+(start+i+1)*2));
  const texture=m.get(2).toString().split('\0')[0].replaceAll('\\','/').toLowerCase();textures.add(texture);
  meshes.push({positions,uv,indices,joints,weights,texture});
 }
 result[name]={bones,meshes};
}
const terrain=['terrain/terrain_escape_mask','detail/detail_grnd_grass','detail/detail_asfalt_det1','detail/detail_earth_det1','detail/detail_grnd_pebbles'];
for(const name of [...textures,...terrain]){const target=path.join(out,'textures',name+'.dds');fs.mkdirSync(path.dirname(target),{recursive:true});fs.copyFileSync(path.join(source,'textures',name+'.dds'),target);}
fs.writeFileSync(path.join(out,'characters.json'),JSON.stringify(result));
fs.writeFileSync(path.join(out,'extra-textures.json'),JSON.stringify([...textures,...terrain]));
console.log('Imported '+Object.keys(result).join(', '));
