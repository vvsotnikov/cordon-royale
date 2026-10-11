import assert from 'node:assert/strict';
import {DDSLoader} from '../public/vendor/DDSLoader.js';
import {RGBA_S3TC_DXT1_Format,RGBAFormat} from '../public/vendor/three.module.js';
// A transparent DXT1 block: color0 <= color1 and all pixels use index 3.
function fixture(cube=false){const b=new ArrayBuffer(128+(cube?6:1)*8),h=new Uint32Array(b,0,32);h[0]=0x20534444;h[1]=124;h[2]=0x1007;h[3]=h[4]=4;h[19]=32;h[20]=4;h[21]=0x31545844;h[27]=0x1000;h[28]=cube?0xfe00:0;for(let p=128;p<b.byteLength;p+=8){new DataView(b).setUint16(p+2,0xffff,true);new DataView(b).setUint32(p+4,0xffffffff,true);}return b;}
const loader=new DDSLoader(),leaf=loader.parse(fixture(),true);
assert.equal(leaf.format,RGBA_S3TC_DXT1_Format,'GPU must receive alpha-capable DXT1');
assert.equal(leaf.mipmaps[0].data[7],255);
const cube=loader.parse(fixture(true),true);assert(cube.isCubemap);assert.equal(cube.mipmaps.length,6);assert(cube.mipmaps.every(m=>m.width===4&&m.height===4&&m.data.length===8));
console.log('PASS DXT1 cutout alpha and six-face cubemap parsing');

const tiny=fixture(),header=new Uint32Array(tiny,0,32);header[3]=header[4]=2;
const decoded=loader.parse(tiny,true);assert.equal(decoded.format,RGBAFormat);assert.equal(decoded.mipmaps[0].data.length,16);assert(decoded.mipmaps[0].data.every(v=>v===0),'transparent placeholders must remain invisible');
new DataView(tiny).setUint32(132,0x55555555,true);const white=loader.parse(tiny,true);assert(white.mipmaps[0].data.every(v=>v===255),'opaque pixels must remain opaque');
console.log('PASS tiny DXT1 RGBA fallback preserves transparent and opaque pixels');
