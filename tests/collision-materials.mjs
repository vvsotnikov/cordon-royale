import assert from 'node:assert/strict';
import {solidCollision} from '../tools/import-collision.mjs';
const u=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);return b;};
const chunk=(id,b)=>Buffer.concat([u(id),u(b.length),b]);
const material=(id,flags)=>Buffer.concat([chunk(0x1000,Buffer.concat([u(id),Buffer.from('material\0')])),chunk(0x1001,u(flags))]);
const library=Buffer.concat([chunk(0x1000,Buffer.from([1,0])),chunk(0x1002,Buffer.concat([chunk(0,material(37,128)),chunk(1,material(39,0))]))]);
const cform=Buffer.alloc(36+36+48);cform.writeUInt32LE(4);cform.writeUInt32LE(3,4);cform.writeUInt32LE(3,8);
for(let i=0;i<3;i++){const p=72+i*16;cform.writeUInt32LE(1,p+4);cform.writeUInt32LE(2,p+8);cform.writeUInt32LE(([37,39,123][i])|0x1234c000,p+12);}
const result=solidCollision(cform,library);
assert.equal(result.removed,1);assert.equal(result.bytes.readUInt32LE(8),2);
assert.deepEqual(result.bytes.subarray(12,72),cform.subarray(12,72));
assert.deepEqual(result.bytes.subarray(72),cform.subarray(88),'solid trunk and unknown material must stay intact, including sector/flags');
assert.throws(()=>solidCollision(cform.subarray(0,80),library),/length/);
console.log('PASS passable material ID filtering, solid/unknown preservation, triangle metadata and truncated input rejection.');
