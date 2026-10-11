// Converts documented X-Ray v14 chunks to browser buffers. No game files belong in Git.
import fs from 'node:fs';
import path from 'node:path';
import {usesAlphaCutout} from './world-materials.mjs';
const [source,out]=process.argv.slice(2);
if(!source||!out)throw Error('Usage: node tools/import-level.mjs <extracted level folder> <external output>');
fs.mkdirSync(out,{recursive:true});
function chunks(b){let r=new Map();for(let p=0;p<b.length;){const id=b.readUInt32LE(p),n=b.readUInt32LE(p+4);if(id&0x80000000||p+8+n>b.length)throw Error('Unsupported compressed/truncated chunk');r.set(id,b.subarray(p+8,p+8+n));p+=8+n;}return r}
const level=chunks(fs.readFileSync(path.join(source,'level'))),geom=chunks(fs.readFileSync(path.join(source,'level.geom')));
if(level.get(1).readUInt16LE(0)!==14)throw Error('Expected Anomaly level v14');
const shaders=level.get(2).subarray(4).toString().split('\0');
const sizes=[4,8,12,16,4,4,4,8,4,4,4,8,4,4,4,4,8];
const vb=[],ib=[],swi=[];let b=geom.get(9),p=4;
for(let k=0;k<b.readUInt32LE(0);k++){const decl=[];let stride=0;while(b.readUInt16LE(p)!==255){const offset=b.readUInt16LE(p+2),type=b[p+4],usage=b[p+6],index=b[p+7];decl.push({offset,type,usage,index});stride=Math.max(stride,offset+sizes[type]);p+=8;}p+=8;const count=b.readUInt32LE(p);p+=4;vb.push({decl,stride,data:b.subarray(p,p+count*stride)});p+=count*stride;}
b=geom.get(10);p=4;for(let k=0;k<b.readUInt32LE(0);k++){let n=b.readUInt32LE(p);p+=4;ib.push(b.subarray(p,p+n*2));p+=n*2;}
b=geom.get(11);p=4;for(let k=0;k<b.readUInt32LE(0);k++){const n=b.readUInt32LE(p+16);swi.push({offset:b.readUInt32LE(p+20),count:b.readUInt16LE(p+24)*3});p+=20+n*8;}
const bins=[],meshes=[],textures=new Set();let bytes=0,triangles=0;
// Split by visual, letting the browser frustum-cull the actual level sectors.
for(const visual of chunks(level.get(3)).values()){
 const c=chunks(visual),h=c.get(1),gc=c.get(21);if(!gc)continue;
 const type=h[1],shader=shaders[h.readUInt16LE(2)]||'',texture=shader.split('/')[1]?.split(',')[0]?.toLowerCase();if(!texture)continue;
 const v=vb[gc.readUInt32LE(0)],vbase=gc.readUInt32LE(4),vn=gc.readUInt32LE(8),i=ib[gc.readUInt32LE(12)];let ibase=gc.readUInt32LE(16),count=gc.readUInt32LE(20);
 if(type===2){const s=c.get(6);ibase+=s.readUInt32LE(20);count=s.readUInt16LE(24)*3;}
 if(type===11){const s=swi[c.get(20).readUInt32LE(0)];ibase+=s.offset;count=s.count;}
 const uv=v.decl.find(d=>d.usage===5&&d.index===0),tree=c.get(12),mat=tree&&Array.from({length:16},(_,i)=>tree.readFloatLE(i*4));
 // Static vertices are already world-space; tree vertices use per-instance matrices.
 const vertices=Buffer.alloc(vn*20),indices=Buffer.alloc(count*2);
 for(let n=0;n<vn;n++){let a=(vbase+n)*v.stride,x=v.data.readFloatLE(a),y=v.data.readFloatLE(a+4),z=v.data.readFloatLE(a+8);
  if(mat){const px=x,py=y,pz=z;x=px*mat[0]+py*mat[4]+pz*mat[8]+mat[12];y=px*mat[1]+py*mat[5]+pz*mat[9]+mat[13];z=px*mat[2]+py*mat[6]+pz*mat[10]+mat[14];}
  vertices.writeFloatLE(x,n*20);vertices.writeFloatLE(y,n*20+4);vertices.writeFloatLE(-z,n*20+8);
  let u=0,t=0;if(uv){if(uv.type===1||uv.type===3){u=v.data.readFloatLE(a+uv.offset);t=v.data.readFloatLE(a+uv.offset+4);}else{u=v.data.readInt16LE(a+uv.offset)/1024;t=v.data.readInt16LE(a+uv.offset+2)/1024;if(mat){u/=2;t/=2;}else{const tangent=v.decl.find(d=>d.usage===6),binormal=v.decl.find(d=>d.usage===7);if(tangent)u+=v.data[a+tangent.offset+3]/255/1024;if(binormal)t+=v.data[a+binormal.offset+3]/255/1024;}}}
  vertices.writeFloatLE(u,n*20+12);vertices.writeFloatLE(t,n*20+16);
 }
 for(let n=0;n<count;n+=3){const a=i.readUInt16LE((ibase+n)*2),b=i.readUInt16LE((ibase+n+1)*2),d=i.readUInt16LE((ibase+n+2)*2);if(Math.max(a,b,d)>=vn)throw Error('Invalid visual index');indices.writeUInt16LE(a,n*2);indices.writeUInt16LE(d,n*2+2);indices.writeUInt16LE(b,n*2+4);}
 const pad=Buffer.alloc((4-indices.length%4)%4);
 meshes.push({offset:bytes,vertices:vn,indices:count,texture:texture.replaceAll('\\','/'),alpha:usesAlphaCutout(shader)});bins.push(vertices,indices,pad);bytes+=vertices.length+indices.length+pad.length;triangles+=count/3;textures.add(texture);
}
fs.writeFileSync(path.join(out,'world.bin'),Buffer.concat(bins));fs.writeFileSync(path.join(out,'world.json'),JSON.stringify({version:1,meshes,triangles}));
fs.writeFileSync(path.join(out,'textures-needed.txt'),[...textures].sort().join('\n'));
fs.copyFileSync(path.join(source,'level.cform'),path.join(out,'collision.cform'));
console.log(JSON.stringify({meshes:meshes.length,triangles,megabytes:Math.round(bytes/1e6),textures:textures.size}));
