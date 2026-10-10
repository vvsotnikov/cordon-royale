import assert from 'node:assert/strict';
import {ResourceCache} from '../public/resource-cache.js';
const bytes = new Uint8Array([1,2,3]).buffer;
const originalFetch=globalThis.fetch;
const part={url:'cache/new.bin',hash:'new',size:3};
const manifest={version:1,resources:{test:{size:3,parts:[part]}}};
let requests=0, stored=0;
try {
 globalThis.fetch=async()=>{requests++;return new Response(bytes)};
 const warm=new ResourceCache(manifest,null);
 warm.stored=async hash=>hash==='new'?bytes:null;
 assert.deepEqual(await warm.get('test'),bytes);assert.equal(requests,0);
 const changed=new ResourceCache(manifest,null);
 changed.stored=async hash=>hash==='old'?bytes:null;
 changed.save=async()=>{stored++};
 assert.deepEqual(await changed.get('test'),bytes);assert.equal(requests,1);assert.equal(stored,1);
 const quota=new ResourceCache(manifest,null);
 quota.save=async()=>{throw Error('QuotaExceededError')};
 assert.deepEqual(await quota.get('test'),bytes);
 const partial=new ResourceCache(manifest,null);
 partial.save=async()=>{throw Error('must not cache truncated data')};
 globalThis.fetch=async()=>new Response(new Uint8Array([1]));
 await assert.rejects(partial.get('test'),/Incomplete resource/);
 globalThis.fetch=async()=>new Response(bytes);
 const split=new ResourceCache({resources:{test:{size:6,parts:[part,part]}}},null);
 assert.deepEqual(new Uint8Array(await split.get('test')),new Uint8Array([1,2,3,1,2,3]));
 console.log('PASS cache hit avoids fetch; changed digest downloads; quota failure falls back; truncated data rejected; geometry chunks reassembled.');
}finally{globalThis.fetch=originalFetch}
