// Run after importing resources. Generated data stays outside the repository.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
const root=path.resolve(process.argv[2]||'D:/GameAssets/cordon-royale/web');
const repo=path.resolve(import.meta.dirname,'..');
if(root===repo||root.startsWith(repo+path.sep))throw Error('Resource output must be outside the repository');
const hash=b=>createHash('sha256').update(b).digest('hex');
const world=JSON.parse(await fs.readFile(path.join(root,'world.json'),'utf8'));
const names=['world.json','world.bin',...new Set(world.meshes.map(m=>'textures/'+m.texture+'.dds'))];
names.push('characters.json',...JSON.parse(await fs.readFile(path.join(root,'extra-textures.json'),'utf8')).map(n=>'textures/'+n+'.dds'));
const resources={};
await fs.mkdir(path.join(root,'cache'),{recursive:true});
for(const name of names){
 let data;try{data=await fs.readFile(path.join(root,name));}catch(e){if(e.code==='ENOENT'&&name.startsWith('textures/'))continue;throw e;}
 const parts=[];
 for(let offset=0;offset<data.length;offset+=16*1024*1024){
  const part=data.subarray(offset,Math.min(offset+16*1024*1024,data.length));
  const digest=hash(part),url='cache/'+digest+'.bin';
  try{await fs.writeFile(path.join(root,url),part,{flag:'wx'});}catch(e){if(e.code!=='EEXIST')throw e;}
  parts.push({url,hash:digest,size:part.length});
 }
 resources[name]={size:data.length,parts};
}
await fs.writeFile(path.join(root,'cache-manifest.json.tmp'),JSON.stringify({version:1,resources}));
await fs.rename(path.join(root,'cache-manifest.json.tmp'),path.join(root,'cache-manifest.json'));
console.log(`Cache manifest: ${Object.keys(resources).length} resources, content-addressed chunks up to 16 MiB.`);
