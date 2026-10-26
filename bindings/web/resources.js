// Resolve only declared image materials, never arbitrary metadata image fields.
export async function hydrateImages(value, base, read, signal) {
  const encoded=JSON.stringify(value);if(new TextEncoder().encode(encoded).byteLength>8*1024*1024)throw Error('input byte limit');
  const copy=JSON.parse(encoded);if(!Array.isArray(copy)&&!Array.isArray(copy?.primitives))throw Error('entity/CZML array or primitives required');
  const refs=[];
  for(const p of copy.primitives||[])if(p?.appearance?.material?.type==='Image')refs.push(p.appearance.material);
  for(const item of Array.isArray(copy)?copy:[]){
    if(item?.shape && item.material?.type==='image')refs.push(item.material);
    if(item?.shape?.type==='billboard')refs.push(item.shape);
    if(item?.billboard)refs.push(item.billboard);
    for(const key of ['box','ellipsoid','cylinder','plane','ellipse','rectangle','polygon','wall']){
      const material=item?.[key]?.material?.image;if(material)refs.push(material);
    }
  }
  if(refs.length>256)throw Error('image material count limit');
  const cache=new Map();
  const dataUri=bytes=>{
    const png=bytes.length>=8&&[137,80,78,71,13,10,26,10].every((v,i)=>bytes[i]===v);
    const jpg=bytes.length>=3&&bytes[0]===255&&bytes[1]===216&&bytes[2]===255;
    if(!png&&!jpg)throw Error('PNG/JPEG image required');
    let binary='';for(let i=0;i<bytes.length;i+=8192)binary+=String.fromCharCode(...bytes.subarray(i,i+8192));
    return `data:image/${png?'png':'jpeg'};base64,${btoa(binary)}`;
  };
  const urls=[];
  for(const ref of refs){if(ref.image && typeof ref.image==='object' && (Object.keys(ref.image).length!==1 || !Object.hasOwn(ref.image,'uri')))throw Error('constant image URI object required');const raw=typeof ref.image==='string'?ref.image:ref.image?.uri;
    if(typeof raw!=='string'||raw.length>6*1024*1024)throw Error('constant image URI required');
    if(raw.startsWith('data:')){if(!/^data:image\/(png|jpeg|jpg);base64,/.test(raw))throw Error('PNG/JPEG base64 image required');continue;}
    const url=new URL(raw,base);if(!['http:','https:'].includes(url.protocol)||url.username||url.password)throw Error('image URL scheme/credentials');
    const key=url.href;if(!cache.has(key)){if(cache.size>=16)throw Error('unique image resource limit');cache.set(key,null);urls.push(key);}ref.image=key;
  }
  let next=0;const active=new AbortController();const abort=()=>active.abort();if(signal?.aborted)abort();signal?.addEventListener('abort',abort,{once:true});
  try{await Promise.all(Array.from({length:Math.min(4,urls.length)},async()=>{
    while(next<urls.length){const url=urls[next++];if(active.signal.aborted)throw new DOMException('Aborted','AbortError');cache.set(url,dataUri(await read(url,active.signal,4*1024*1024)));}
  }));}catch(e){active.abort();throw e;}finally{signal?.removeEventListener('abort',abort);}
  for(const ref of refs)if(cache.has(ref.image))ref.image=cache.get(ref.image);
  if(new TextEncoder().encode(JSON.stringify(copy)).byteLength>8*1024*1024)throw Error('hydrated input byte limit');return copy;
}
