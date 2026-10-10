import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';

function chunks(bytes){
 const result=new Map();
 for(let p=0;p<bytes.length;){
  if(p+8>bytes.length)throw Error('Truncated material chunk');
  const id=bytes.readUInt32LE(p),size=bytes.readUInt32LE(p+4);
  if(id&0x80000000||p+8+size>bytes.length)throw Error('Invalid material chunk');
  result.set(id,bytes.subarray(p+8,p+8+size));p+=8+size;
 }
 return result;
}
export function solidCollision(cform,library){
 const materials=chunks(library);
 if(materials.get(0x1000)?.readUInt16LE(0)!==1)throw Error('Unsupported material library');
 const passable=new Set();
 for(const material of chunks(materials.get(0x1002)).values()){
  const fields=chunks(material),id=fields.get(0x1000).readUInt32LE(0),flags=fields.get(0x1001).readUInt32LE(0);
  // CFORM stores persistent material IDs, not positions in the library array.
  if(flags&128)passable.add(id);
 }
 if(cform.length<36||cform.readUInt32LE(0)!==4)throw Error('Expected collision v4');
 const start=36+cform.readUInt32LE(4)*12,count=cform.readUInt32LE(8);
 if(cform.length!==start+count*16)throw Error('Invalid collision length');
 const output=Buffer.alloc(cform.length);cform.copy(output,0,0,start);let kept=0;
 for(let i=0;i<count;i++){
  const p=start+i*16,id=cform.readUInt32LE(p+12)&0x3fff;
  if(!passable.has(id)){cform.copy(output,start+kept*16,p,p+16);kept++;}
 }
 output.writeUInt32LE(kept,8);
 return {bytes:output.subarray(0,start+kept*16),removed:count-kept};
}
if(process.argv[1]&&import.meta.url===pathToFileURL(path.resolve(process.argv[1])).href){
 const [source,library,destination]=process.argv.slice(2);
 if(!source||!library||!destination)throw Error('Usage: node tools/import-collision.mjs <level.cform> <gamemtl.xr> <external collision.cform>');
 const repo=path.resolve(import.meta.dirname,'..').toLowerCase(),out=path.resolve(destination).toLowerCase();
 if(out.startsWith(repo+path.sep))throw Error('Game resources must remain outside the repository');
 const result=solidCollision(fs.readFileSync(source),fs.readFileSync(library));
 fs.writeFileSync(destination,result.bytes);
 console.log(`Excluded ${result.removed} passable collision triangles; solid surfaces retained.`);
}
