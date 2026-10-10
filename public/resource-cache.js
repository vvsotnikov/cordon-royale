// IndexedDB also works on plain HTTP LAN origins, where service workers do not.
export class ResourceCache {
  constructor(manifest, db) {
    this.manifest = manifest;
    this.db = db;
    this.hits = 0;
    this.downloads = 0;
  }
  static async open() {
    const response = await fetch('/assets/cache-manifest.json', {cache:'no-store'});
    if (!response.ok) throw Error('Run node tools/prepare-cache.mjs for the imported assets.');
    const manifest = await response.json();
    if (manifest.version !== 1) throw Error('Unsupported resource cache manifest');
    let db = null;
    try {
      db = await new Promise((resolve, reject) => {
        const request = indexedDB.open('cordon-resources', 1);
        request.onupgradeneeded = () => request.result.createObjectStore('chunks');
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
        request.onblocked = () => reject(Error('Cache upgrade blocked'));
      });
    } catch { /* Storage restrictions must not prevent playing. */ }
    const cache = new ResourceCache(manifest, db);
    try { await cache.prune(); } catch { /* Old entries can be evicted by the browser. */ }
    return cache;
  }
  async prune() {
    if (!this.db) return;
    const live = new Set(Object.values(this.manifest.resources).flatMap(r => r.parts.map(p => p.hash)));
    await new Promise((resolve, reject) => {
      const tx = this.db.transaction('chunks', 'readwrite');
      const cursor = tx.objectStore('chunks').openKeyCursor();
      cursor.onsuccess = () => {
        const entry = cursor.result;
        if (!entry) return;
        if (!live.has(entry.key)) tx.objectStore('chunks').delete(entry.key);
        entry.continue();
      };
      tx.oncomplete = resolve;
      tx.onabort = tx.onerror = () => reject(tx.error);
    });
  }
  async stored(hash) {
    if (!this.db) return null;
    return new Promise((resolve, reject) => {
      const tx = this.db.transaction('chunks', 'readonly');
      const request = tx.objectStore('chunks').get(hash);
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  }
  async save(hash, bytes) {
    if (!this.db) return;
    await new Promise((resolve, reject) => {
      const tx = this.db.transaction('chunks', 'readwrite');
      tx.objectStore('chunks').put(bytes, hash);
      tx.oncomplete = resolve;
      tx.onerror = () => reject(tx.error);
      tx.onabort = () => reject(tx.error);
    });
  }
  async chunk(part) {
    try {
      const cached = await this.stored(part.hash);
      if (cached instanceof ArrayBuffer && cached.byteLength === part.size) {
        this.hits++;
        return cached;
      }
    } catch { this.db = null; }
    const response = await fetch('/assets/'+part.url, {cache:'force-cache'});
    if (!response.ok) throw Error(`Resource download failed (${response.status})`);
    const bytes = await response.arrayBuffer();
    if (bytes.byteLength !== part.size) throw Error('Incomplete resource download; refresh to retry.');
    this.downloads++;
    try { await this.save(part.hash, bytes); } catch { this.db = null; }
    return bytes;
  }
  async get(name) {
    const resource = this.manifest.resources[name];
    if (!resource) throw Error(`Resource unavailable: ${name}`);
    if (resource.parts.length === 1) return this.chunk(resource.parts[0]);
    const result = new Uint8Array(resource.size);
    let offset = 0;
    for (const part of resource.parts) {
      const bytes = await this.chunk(part);
      result.set(new Uint8Array(bytes), offset);
      offset += bytes.byteLength;
    }
    return result.buffer;
  }
}
