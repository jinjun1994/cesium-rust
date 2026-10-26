import {attachControls} from './controls.js';
import {hydrateImages} from './resources.js';
import init, { attach_canvas, bingMetadataUrl } from './pkg/cesium_viewer.js';
const MAX_BYTES = 64 * 1024 * 1024;
async function boundedFetch(url, signal, maxBytes = MAX_BYTES, headers = {}) {
  const response = await fetch(url, { signal, credentials: 'omit', redirect: 'error', headers });
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  if (Number(response.headers.get('content-length')) > maxBytes) throw new Error('Input byte limit');
  if (!response.body) throw new Error('Streaming response body unavailable');
  const reader = response.body.getReader(); const chunks = []; let size = 0;
  try {
    while (true) {
      const {done, value} = await reader.read(); if (done) break;
      size += value.byteLength; if (size > maxBytes) throw new Error('Input byte limit');
      chunks.push(value);
    }
  } catch (e) { await reader.cancel().catch(() => {}); throw e; }
  finally { reader.releaseLock(); }
  const bytes = new Uint8Array(size); let offset = 0;
  for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.byteLength; }
  return bytes;
}
/** Explicit-tree baseline. Decoding is currently on the main thread, not in Workers. */
export async function attachCanvas(canvas, {onPick = () => {}, onStatus = () => {}, onError = () => {}, onRenderError = () => {}, onCredits = () => {}, ionToken = '', terrain = true} = {}) {
  if (!navigator.gpu) throw new Error('WebGPU requires a supported browser and a secure context');
  await init(); let viewer = await attach_canvas(canvas);
  let epoch = 0, active = 0, imageryActive = 0, raf = 0, destroyed = false, pick = [-1,-1], lastPick = 'null';
  let resolutionScale=1,useBrowserRecommendedResolution=false;const preRenderCallbacks=new Set();
  let geoActive=0,czmlActive=0,entityImports=0,entityRevision=0,entityController=new AbortController();
  let imageActive=0;const imageWaiters=[];
  let terrainActive=0,terrainEpoch=0,terrainController=new AbortController();
  let terrainConfig=terrain ? {kind:"public"} : null;
  let terrainCompleted=0,terrainFailed=0;
  let terrainOptions={exaggeration:1,sse:8};
  let tileRequested=0,tileCompleted=0,tileFailed=0,tileErrors=[];
  let controller = new AbortController(); const cleanups = [];
  function invalidateEntities(){entityRevision++;entityController.abort();entityController=new AbortController();}
  async function imageFetch(url,signal,maxBytes){
    if(signal.aborted)throw new DOMException('Aborted','AbortError');
    await new Promise((resolve,reject)=>{
      const waiter={resolve,reject,signal,abort:null};
      const start=()=>{signal.removeEventListener('abort',waiter.abort);imageActive++;resolve();};waiter.start=start;
      waiter.abort=()=>{const i=imageWaiters.indexOf(waiter);if(i>=0)imageWaiters.splice(i,1);reject(new DOMException('Aborted','AbortError'));};
      if(imageActive<4)start();else{if(imageWaiters.length>=64){reject(Error('image request queue limit'));return;}imageWaiters.push(waiter);signal.addEventListener('abort',waiter.abort,{once:true});}
    });
    try{return await boundedFetch(url,signal,maxBytes);}finally{imageActive--;while(imageWaiters.length&&imageActive<4){const next=imageWaiters.shift();if(!next.signal.aborted)next.start();else next.abort();}}
  }
  async function importEntities(value,base,upsert,czml=false){
    if(destroyed)throw Error('viewer destroyed');if(entityImports+czmlActive>=4)throw Error('entity import concurrency limit');entityImports++;
    const generation=epoch,revision=entityRevision,signal=entityController.signal;
    try{
      if(typeof value==='string'){base=new URL(value,location.href).href;value=JSON.parse(new TextDecoder().decode(await boundedFetch(base,signal,8*1024*1024)));}
      const data=await hydrateImages(value,base||location.href,imageFetch,signal);
      if(destroyed||generation!==epoch||revision!==entityRevision)return false;
      const bytes=new TextEncoder().encode(JSON.stringify(data));if(czml)viewer.loadCzml(bytes);else if(upsert)viewer.upsertEntities(bytes);else viewer.setEntities(bytes);return true;
    }catch(e){if(destroyed||generation!==epoch||revision!==entityRevision||e?.name==='AbortError')return false;throw e;}finally{entityImports--;}
  }
  async function importPrimitives(value,base=location.href){
    if(destroyed)throw Error('viewer destroyed');if(entityImports+czmlActive>=4)throw Error('primitive import concurrency limit');entityImports++;
    const generation=epoch,revision=entityRevision,signal=entityController.signal;
    try{if(typeof value==='string'){base=new URL(value,location.href).href;value=JSON.parse(new TextDecoder().decode(await boundedFetch(base,signal,8*1024*1024)));}
      value=await hydrateImages(value,base,imageFetch,signal);
      if(destroyed||generation!==epoch||revision!==entityRevision)return false;
      const bytes=new TextEncoder().encode(JSON.stringify(value));if(bytes.length>8*1024*1024)throw Error('primitive input limit');viewer.setPrimitiveInstances(bytes);return true;
    }catch(e){if(destroyed||generation!==epoch||revision!==entityRevision||e?.name==='AbortError')return false;throw e;}finally{entityImports--;}
  }
  function resize() {
    if (destroyed) return;
    const rect = canvas.getBoundingClientRect(); const dpr = useBrowserRecommendedResolution?1:(window.devicePixelRatio || 1);
    const w = Math.max(1,Math.round(rect.width * dpr * resolutionScale)), h = Math.max(1,Math.round(rect.height * dpr * resolutionScale));
    if (canvas.width !== w || canvas.height !== h) { canvas.width = w; canvas.height = h; viewer.resize(w,h); pick = [-1,-1]; }
  }
  const observer = new ResizeObserver(resize); observer.observe(canvas); cleanups.push(() => observer.disconnect()); resize();
  cleanups.push(attachControls(canvas, {
    orbit:(dx,dy)=>viewer.orbit(dx,dy),zoom:delta=>viewer.zoom(delta),
    pick:(x,y)=>{pick=[x,y];},reset:()=>viewer.focusGlobe(),
  }));
  function stream() {
    for (const request of JSON.parse(viewer.takeRequests(4-active))) {
      const generation=epoch; const signal=controller.signal; active++;tileRequested++;
      boundedFetch(request.url,signal).then(bytes => {
        if (!destroyed && generation===epoch) {viewer.submitTile(request.id,bytes);tileCompleted++;}
      }).catch(error => {
        if (!destroyed && generation===epoch && error.name !== 'AbortError') {tileFailed++;tileErrors.push({id:request.id,error:error.message});if(tileErrors.length>64)tileErrors.shift();viewer.failTile(request.id); onError(`Tile ${request.id}: ${error.message}`); }
      }).finally(() => { if (generation===epoch) active--; });
    }
  }
  function streamImagery() {
    for (const request of JSON.parse(viewer.takeImageryRequests(4-imageryActive))) {
      const generation=epoch; const signal=controller.signal; imageryActive++;
      boundedFetch(request.url,signal,4*1024*1024).then(bytes => {
        if (!destroyed && generation===epoch) viewer.submitImagery(request.id,bytes);
      }).catch(error => {
        if (!destroyed && generation===epoch && error.name!=='AbortError') {
          viewer.failImagery(request.id); onError(`Imagery ${request.id}: request/decode failed`);
        }
      }).finally(() => {if(generation===epoch)imageryActive--;});
    }
  }
  function credits(){onCredits([viewer.imageryCredits,viewer.terrainCredits||''].filter(Boolean).join(' · '));}
  function invalidateTerrain(){viewer.cancelTerrainRequests();terrainEpoch++;terrainController.abort();terrainController=new AbortController();terrainActive=0;terrainCompleted=0;terrainFailed=0;}
  function applyTerrain(){
    if(!terrainConfig){viewer.clearTerrain();return;}
    if(terrainConfig.kind==='public')viewer.setPublicTerrain();
    else if(terrainConfig.kind==='terrarium')viewer.setTerrariumTerrain(terrainConfig.template,terrainConfig.maxLevel);
    else viewer.setTerrainLayer(terrainConfig.bytes,terrainConfig.source,terrainConfig.authorization||'');
    viewer.setTerrainOptions(terrainOptions.exaggeration,terrainOptions.sse);credits();
  }
  function streamTerrain(){
    for(const request of JSON.parse(viewer.takeTerrainRequests(4-terrainActive))){
      const generation=terrainEpoch,sourceGeneration=epoch,signal=terrainController.signal;terrainActive++;
      const headers={Accept:request.accept};if(request.authorization)headers.Authorization=request.authorization;
      boundedFetch(request.url,signal,8*1024*1024,headers).then(bytes=>{
        if(!destroyed&&generation===terrainEpoch&&sourceGeneration===epoch){viewer.submitTerrain(request.id,bytes);terrainCompleted++;}
      }).catch(error=>{
        if(!destroyed&&generation===terrainEpoch&&sourceGeneration===epoch&&error.name!=='AbortError'){terrainFailed++;viewer.failTerrain(request.id);onError(`Terrain ${request.id}: ${error.message}`);}
      }).finally(()=>{if(generation===terrainEpoch)terrainActive--;});
    }
  }
  async function loadTerrain(url){
    if(destroyed)throw Error('viewer destroyed');invalidateTerrain();const generation=terrainEpoch;
    const absolute=new URL(url,location.href).href;
    try{const bytes=await boundedFetch(absolute,terrainController.signal,1024*1024,{Accept:'application/json'});
      if(destroyed||generation!==terrainEpoch)return false;
      invalidateTerrain();viewer.setTerrainLayer(bytes,absolute,'');viewer.setTerrainOptions(terrainOptions.exaggeration,terrainOptions.sse);terrainConfig={kind:'layer',bytes,source:absolute};credits();return true;
    }catch(error){if(destroyed||generation!==terrainEpoch||error.name==='AbortError')return false;throw error;}
  }
  async function loadWorldTerrain(token){
    if(!token?.trim())throw Error('需要 Cesium ion access token');if(destroyed)throw Error('viewer destroyed');
    invalidateTerrain();const generation=terrainEpoch,signal=terrainController.signal;
    try{const url=new URL('https://api.cesium.com/v1/assets/1/endpoint');url.searchParams.set('access_token',token);
      const endpoint=JSON.parse(new TextDecoder().decode(await boundedFetch(url.href,signal,1024*1024)));
      if(destroyed||generation!==terrainEpoch)return false;
      if(endpoint.type!=='TERRAIN'||typeof endpoint.accessToken!=='string'||!endpoint.accessToken||endpoint.accessToken.length>8192||/[\r\n]/.test(endpoint.accessToken))throw Error('terrain endpoint');
      const base=new URL(endpoint.url);if(!['https:','http:'].includes(base.protocol)||base.username||base.password)throw Error('terrain endpoint');
      const source=new URL('layer.json',base).href,authorization='Bearer '+endpoint.accessToken;
      const bytes=await boundedFetch(source,signal,1024*1024,{Accept:'application/json',Authorization:authorization});
      if(destroyed||generation!==terrainEpoch)return false;
      invalidateTerrain();viewer.setTerrainLayer(bytes,source,authorization);viewer.setTerrainOptions(terrainOptions.exaggeration,terrainOptions.sse);terrainConfig={kind:'layer',bytes,source,authorization};credits();return true;
    }catch(error){if(destroyed||generation!==terrainEpoch||error.name==='AbortError')return false;throw Error('全球地形初始化失败，请检查 ion token、地形权限和网络');}
  }
  function setNaturalEarth() {
    viewer.setImageryTemplate('https://cesium.com/downloads/cesiumjs/releases/1.146/Build/Cesium/Assets/Textures/NaturalEarthII/{z}/{x}/{reverseY}.jpg',true,2,'Natural Earth II · Cesium sample imagery');
    credits();
  }
  async function loadWorldImagery(token) {
    if (!token?.trim()) throw new Error('需要 Cesium ion access token');
    invalidate(); viewer.showGlobe();applyTerrain(); const generation=epoch, signal=controller.signal;
    try {
      const url=new URL('https://api.cesium.com/v1/assets/2/endpoint');url.searchParams.set('access_token',token);
      const endpoint=new TextDecoder().decode(await boundedFetch(url.href,signal,1024*1024));
      if(destroyed || generation!==epoch)return;
      const metadata=new TextDecoder().decode(await boundedFetch(bingMetadataUrl(endpoint),signal,1024*1024));
      if(destroyed || generation!==epoch)return;
      viewer.setWorldImagery(endpoint,metadata);credits();
    } catch(error) {
      if(error.name==='AbortError' || generation!==epoch || destroyed)return;
      throw new Error('默认全球影像初始化失败，请检查 ion token、访问权限和网络');
    }
  }
  setNaturalEarth();applyTerrain();
  if(ionToken)loadWorldImagery(ionToken).catch(error=>onError(error.message));
  let previousFrame;
  function frame(timestamp) {
    raf=0;
    if (destroyed) return;
    try {
      const elapsed=Number.isFinite(timestamp)&&Number.isFinite(previousFrame)?Math.min(1,Math.max(0,(timestamp-previousFrame)/1000)):0;previousFrame=timestamp;
      viewer.advanceTime(elapsed);
      resize(); for(const callback of [...preRenderCallbacks])callback(scene,JSON.parse(viewer.clockState).currentTime); viewer.renderFrame(...pick); pick=[-1,-1]; stream(); streamImagery(); streamTerrain();
      const result=viewer.lastPick; if (result!==lastPick) { lastPick=result; onPick(JSON.parse(result)); }
      onStatus(viewer.stats); raf=requestAnimationFrame(frame);
    } catch (e) { onError(e?.message || String(e)); onRenderError(e?.message || String(e)); }
  }
  raf=requestAnimationFrame(frame);
  function invalidate() { viewer.cameraCommand?.(JSON.stringify({type:"cancelFlight"}));viewer.cameraCommand?.(JSON.stringify({type:"stopTracking"}));invalidateEntities();invalidateTerrain();epoch++;viewer.cancelRequests(); controller.abort(); controller=new AbortController(); active=0;imageryActive=0;tileRequested=0;tileCompleted=0;tileFailed=0;tileErrors=[];pick=[-1,-1]; }
  const scene={
    preRender:{addEventListener(callback){if(destroyed)throw Error('viewer destroyed');if(typeof callback!=='function')throw Error('callback required');preRenderCallbacks.add(callback);return()=>preRenderCallbacks.delete(callback);},removeEventListener(callback){return preRenderCallbacks.delete(callback);},get numberOfListeners(){return preRenderCallbacks.size;}},
    cartesianToCanvasCoordinates(position,result={}){if(destroyed)return undefined;const p=Array.isArray(position)?position:[position.x,position.y,position.z];const xy=JSON.parse(viewer.projectCartesian(JSON.stringify(p)));if(!xy)return undefined;const rect=canvas.getBoundingClientRect();result.x=xy[0]*rect.width/canvas.width;result.y=xy[1]*rect.height/canvas.height;return result;}
  };
  return {
    scene,
    setImageryTemplate(template,geographic=true,maxLevel=2,credit='Configured imagery') {
      if(destroyed)throw Error('viewer destroyed');const absolute=new URL(template,location.href);
      if(!['http:','https:'].includes(absolute.protocol)||absolute.username||absolute.password||absolute.href.length>4096)throw Error('imagery URL');
      invalidate();if(typeof geographic!=='boolean'||!Number.isInteger(maxLevel)||maxLevel<0||maxLevel>20)throw Error('imagery options');
      const resolved=absolute.href.replace(/%7B(z|x|y|reverseY|quadkey|subdomain|culture)%7D/g,'{$1}');viewer.setImageryTemplate(resolved,geographic,maxLevel,credit);credits();
    },
    useNaturalEarthImagery(){if(destroyed)throw Error('viewer destroyed');invalidate();setNaturalEarth();},
    get resolutionScale(){return resolutionScale;},set resolutionScale(value){if(destroyed)throw Error('viewer destroyed');if(!Number.isFinite(value)||value<0.1||value>2)throw Error('resolutionScale must be finite in 0.1..2');resolutionScale=value;resize();},
    get useBrowserRecommendedResolution(){return useBrowserRecommendedResolution;},set useBrowserRecommendedResolution(value){if(destroyed)throw Error('viewer destroyed');if(typeof value!=='boolean')throw Error('boolean resolution mode required');useBrowserRecommendedResolution=value;resize();},
    async loadGroupedEntities(value){
      if(destroyed)throw Error('viewer destroyed');if(entityImports+czmlActive>=4)throw Error('grouped import concurrency limit');entityImports++;
      const generation=epoch,revision=entityRevision,signal=entityController.signal;
      try{if(typeof value==='string')value=JSON.parse(new TextDecoder().decode(await boundedFetch(new URL(value,location.href).href,signal,8*1024*1024)));
        if(destroyed||generation!==epoch||revision!==entityRevision)return false;
        const bytes=new TextEncoder().encode(JSON.stringify(value));if(bytes.length>8*1024*1024)throw Error('grouped input limit');viewer.setGroupedEntities(bytes);return true;
      }catch(e){if(destroyed||generation!==epoch||revision!==entityRevision||e?.name==='AbortError')return false;throw e;}finally{entityImports--;}
    },
    get entityGroups(){return destroyed?[]:JSON.parse(viewer.entityGroups);},showEntityGroup(id,show=true){return !destroyed&&viewer.showEntityGroup(id,show);},
    get cameraState() {return destroyed?null:JSON.parse(viewer.cameraState);},
    zoomToEntities(ids=[]) {if(destroyed)throw Error('viewer destroyed');viewer.zoomToEntities(JSON.stringify(ids));return true;},
    setView(view) {if(destroyed)throw Error('viewer destroyed');viewer.cameraCommand(JSON.stringify({type:'setView',view}));},
    flyTo(view,{duration=3}={}) {if(destroyed)throw Error('viewer destroyed');viewer.cameraCommand(JSON.stringify({type:'flyTo',view,duration}));return JSON.parse(viewer.cameraState).flightId;},
    cancelFlight() {if(destroyed)return false;const active=JSON.parse(viewer.cameraState).flightStatus==='flying';viewer.cameraCommand(JSON.stringify({type:'cancelFlight'}));return active;},
    lookAt(targetDegrees,offset={}) {if(destroyed)throw Error('viewer destroyed');viewer.cameraCommand(JSON.stringify({type:'lookAt',targetDegrees,offset}));},
    trackEntity(id,offset={}) {if(destroyed)throw Error('viewer destroyed');viewer.cameraCommand(JSON.stringify({type:'trackEntity',id,offset}));},
    stopTracking() {if(destroyed)return false;const active=!!JSON.parse(viewer.cameraState).trackedEntity;viewer.cameraCommand(JSON.stringify({type:'stopTracking'}));return active;},
    setEntities(entities) {if(destroyed)throw Error('viewer destroyed');invalidateEntities();const bytes=new TextEncoder().encode(JSON.stringify(entities));if(bytes.length>8*1024*1024)throw Error('entity input limit');viewer.setEntities(bytes);},
    upsertEntities(entities) {if(destroyed)throw Error('viewer destroyed');invalidateEntities();const bytes=new TextEncoder().encode(JSON.stringify(entities));if(bytes.length>8*1024*1024)throw Error('entity input limit');viewer.upsertEntities(bytes);},
    cancelEntityLoads(){if(!destroyed)invalidateEntities();},
    loadCzmlData(value,{baseUrl=location.href}={}) {return importEntities(value,baseUrl,false,true);},
    get maximumAliasedLineWidth(){return destroyed?0:viewer.maximumAliasedLineWidth;},
    loadPrimitiveInstances(value,{baseUrl=location.href}={}){return importPrimitives(value,baseUrl);},
    loadEntities(value,{baseUrl=location.href,upsert=false}={}) {return importEntities(value,baseUrl,upsert);},
    get entities() {return destroyed?[]:JSON.parse(viewer.entitySpecs);},
    showEntity(id,show=true) {return !destroyed&&viewer.showEntity(id,show);},
    loadCzmlBytes(bytes) {if(destroyed)throw Error('viewer destroyed');invalidateEntities();if(bytes.byteLength>8*1024*1024)throw Error('CZML input limit');viewer.loadCzml(bytes);},
    async loadCzml(url) {
      if(destroyed)throw Error('viewer destroyed');if(czmlActive+entityImports>=4)throw Error('CZML load concurrency limit');
      const generation=epoch,revision=entityRevision,signal=entityController.signal;czmlActive++;
      try{const base=new URL(url,location.href).href;const bytes=await boundedFetch(base,signal,8*1024*1024);if(destroyed||generation!==epoch||revision!==entityRevision)return false;const data=await hydrateImages(JSON.parse(new TextDecoder().decode(bytes)),base,imageFetch,signal);if(destroyed||generation!==epoch||revision!==entityRevision)return false;viewer.loadCzml(new TextEncoder().encode(JSON.stringify(data)));return true;}
      catch(error){if(destroyed||generation!==epoch||revision!==entityRevision||error?.name==='AbortError')return false;throw error;}
      finally{czmlActive--;}
    },
    get clock() {return JSON.parse(viewer.clockState);},
    setClock(options) {if(destroyed)throw Error('viewer destroyed');viewer.setClock(JSON.stringify({...JSON.parse(viewer.clockState),...options}));},
    removeEntity(id) {return viewer.removeEntity(id);},
    clearEntities() {if(!destroyed){invalidateEntities();viewer.clearEntities();}},
    focusEntity(id,range=100000) {return viewer.focusEntity(id,range);},
    async load(url) {
      invalidate(); const generation=epoch; const absolute=new URL(url,location.href).href;
      const bytes=await boundedFetch(absolute,controller.signal); if (destroyed || epoch!==generation) return;
      const magic=String.fromCharCode(...bytes.subarray(0,4));
      if (magic==='glTF' || magic==='b3dm') {invalidate();viewer.loadBytes(bytes);}
      else {const json=JSON.parse(new TextDecoder().decode(bytes));if(Array.isArray(json)) {const revision=entityRevision;const data=await hydrateImages(json,absolute,imageFetch,entityController.signal);if(destroyed||generation!==epoch||revision!==entityRevision)return;viewer.loadCzml(new TextEncoder().encode(JSON.stringify(data)));} else if(json.type) {viewer.addGeoJson(bytes,'{}');} else {invalidate();viewer.loadTileset(bytes,absolute);applyTerrain();}}
      if (!raf) raf=requestAnimationFrame(frame);
    },
    loadBytes(bytes) { invalidate(); if (bytes.byteLength>MAX_BYTES) throw new Error('Input byte limit'); viewer.loadBytes(bytes); },
    async loadGeoJson(url, style = {}) {
      if(destroyed)throw new Error('viewer destroyed');
      if(geoActive>=4)throw new Error('GeoJSON load concurrency limit');
      const generation=epoch, signal=controller.signal;geoActive++;
      try {
        const bytes=await boundedFetch(new URL(url,location.href).href,signal,8*1024*1024);
        if(destroyed || generation!==epoch)return null;
        return viewer.addGeoJson(bytes,JSON.stringify(style));
      } catch(error) {if(destroyed || generation!==epoch || error?.name==='AbortError')return null;throw error;}
      finally {geoActive--;}
    },
    addGeoJson(value, style = {}) {
      if(destroyed)throw new Error('viewer destroyed');
      const bytes=value instanceof Uint8Array ? value : new TextEncoder().encode(JSON.stringify(value));
      if(bytes.byteLength>8*1024*1024)throw new Error('GeoJSON input byte limit');
      return viewer.addGeoJson(bytes,JSON.stringify(style));
    },
    removeGeoJson(id) {return !destroyed && viewer.removeGeoJson(id);},
    clearGeoJson() {if(!destroyed){invalidate();viewer.clearGeoJson();}},
    focusGeoJson(id) {return !destroyed && viewer.focusGeoJson(id);},
    loadWorldImagery,
    loadTerrain,
    loadWorldTerrain,
    showTerrain(){if(destroyed)return;invalidateTerrain();const previous=terrainConfig;terrainConfig={kind:'public'};try{applyTerrain();}catch(error){terrainConfig=previous;throw error;}},
    setTerrariumTerrain(template,maxLevel=15){if(destroyed)return;invalidateTerrain();viewer.setTerrariumTerrain(template,maxLevel);viewer.setTerrainOptions(terrainOptions.exaggeration,terrainOptions.sse);terrainConfig={kind:'terrarium',template,maxLevel};credits();},
    clearTerrain(){if(destroyed)return;invalidateTerrain();terrainConfig=null;viewer.clearTerrain();credits();},
    setTerrainOptions(exaggeration=1,sse=8){if(!destroyed){viewer.setTerrainOptions(exaggeration,sse);terrainOptions={exaggeration,sse};}},
    sampleHeight(longitude,latitude){return destroyed?null:viewer.sampleHeight(longitude,latitude)??null;},
    lookAtTerrain(longitude,latitude,range=20000){if(!destroyed)viewer.lookAtTerrain(longitude,latitude,range);},
    focusLocation(longitude,latitude,height=20000){if(!destroyed)viewer.focusLocation(longitude,latitude,height);},

    showGlobe() { invalidate();viewer.showGlobe();setNaturalEarth();applyTerrain();if(!raf)raf=requestAnimationFrame(frame); },
    zoom(delta) { if(!destroyed)viewer.zoom(delta); },
    resetView() { if(!destroyed)viewer.focusGlobe(); },
    get terrainCredits(){return destroyed?'':viewer.terrainCredits;},
    get imageryCredits() {return destroyed ? '' : viewer.imageryCredits;},
    resize,
    destroy() { if (destroyed) return; destroyed=true;preRenderCallbacks.clear();invalidate();cancelAnimationFrame(raf);for(const cleanup of cleanups) cleanup();viewer.destroy();viewer.free();viewer=null; },
    get diagnostics() {return {epoch,tileRequested,tileCompleted,tileFailed,tilePending:active,imageryPending:imageryActive,errors:tileErrors.map(e=>({...e})),terrainPending:terrainActive,terrainCompleted,terrainFailed,destroyed};},
    get stats() { return destroyed ? 'destroyed' : viewer.stats; },
  };
}
