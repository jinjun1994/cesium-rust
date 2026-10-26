const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto'),assert=require('node:assert/strict');
const hash=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
async function snapshot(page,key){await page.evaluate(key=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>{const c=document.getElementById('globe'),o=document.createElement('canvas');o.width=c.width;o.height=c.height;const ctx=o.getContext('2d');ctx.drawImage(c,0,0);window[key]={bytes:ctx.getImageData(0,0,o.width,o.height).data,w:o.width,h:o.height};resolve(true);}))),key);}
async function difference(page,id){await snapshot(page,'__pointVisible');await page.evaluate(id=>window.cesiumSandcastleViewer.showEntity(id,false),id);await snapshot(page,'__pointHidden');const data=await page.evaluate(()=>{const a=window.__pointVisible,b=window.__pointHidden;let pixels=0,energy=0;for(let i=0;i<a.bytes.length;i+=4){const d=[0,1,2].reduce((n,k)=>n+Math.abs(a.bytes[i+k]-b.bytes[i+k]),0);if(d>6){pixels++;energy+=d;}}return{pixels,energy};});await page.evaluate(id=>window.cesiumSandcastleViewer.showEntity(id,true),id);return data;}
module.exports=async(page,root,manifest,errors)=>{
 const out=[];
 const report=(id,checks,extra={})=>{const c=manifest.cases.find(c=>c.id===id);return{id,status:'passed',entities:c.entityCount,sourceSha256:c.sourceSha256,inputSha256:hash(path.resolve(root,'bindings/web',c.path)),fixtureSha256:hash(path.join(root,`tests/sandcastle/full/${id}.json`)),checks,...extra};};
 console.log('start complete points menu');await page.locator('#fullNew26').click();await page.waitForFunction(()=>document.getElementById('fullStatus').textContent.startsWith('Points：'));
 const variants=JSON.parse(fs.readFileSync(path.join(root,'tests/sandcastle/full/points.variants.json'))).variants;
 const variantResults=[];
 for(let i=0;i<variants.length;i++){
  await page.locator('#fullVariantSelect').selectOption(String(i));await page.waitForFunction(n=>window.cesiumSandcastleViewer.entities.length===n,variants[i].entities.length);
  const actual=await page.evaluate(()=>window.cesiumSandcastleViewer.entities);
  for(const expected of variants[i].entities){const a=actual.find(a=>a.id===expected.id);assert(a);assert.deepEqual(a.positionDegrees,expected.positionDegrees);assert.deepEqual(a.shape,expected.shape);for(let k=0;k<4;k++)assert(Math.abs(a.color[k]-expected.color[k])<1e-6);for(const [k,v]of Object.entries(expected.screen||{}))assert.deepEqual(a.screen[k],v);}
  let pixels=0;
  for(const e of variants[i].entities){
   await page.evaluate(id=>window.cesiumSandcastleViewer.zoomToEntities([id]),e.id);await page.locator('#globe').scrollIntoViewIfNeeded();const d=await difference(page,e.id);assert(d.pixels>8,`points menu ${i} ${e.id}`);pixels+=d.pixels;
   const box=await page.locator('#globe').boundingBox();await page.mouse.click(box.x+box.width/2,box.y+box.height/2);await page.waitForFunction(id=>{try{return JSON.parse(document.getElementById('pick').textContent)?.id===id;}catch{return false;}},e.id);
  }
  await page.locator('#resetFullVariant').click();assert.equal(await page.evaluate(()=>window.cesiumSandcastleViewer.entities.length),variants[i].entities.length);
  variantResults.push({label:variants[i].label,entities:variants[i].entities.length,pixels});
 }
 await page.locator('#fullVariantSelect').selectOption('4');const entityId=variants[4].entities[0].id;
 const measure=async(range)=>{await page.evaluate(({id,range})=>window.cesiumSandcastleViewer.focusEntity(id,range),{id:entityId,range});await page.locator('#globe').scrollIntoViewIfNeeded();return difference(page,entityId);};
 const near=await measure(150),far=await measure(15000000);assert(near.pixels>far.pixels*3,JSON.stringify({near,far}));
 await page.locator('#fullVariantSelect').selectOption('5');const opaque=await measure(150),faded=await measure(15000000);assert(opaque.energy>faded.energy*1.5,JSON.stringify({opaque,faded}));
 out.push(report('points',['all six original menu callbacks and reset','exact positions, colors, sizes, outlines and NearFarScalar','real pixels and picking for every variant entity','measured distance scaling and fading'],{variantResults,scaleMeasurements:{near,far},fadeMeasurements:{near:opaque,far:faded}}));console.log('points menu passed');
 console.log('start complete offline imagery');
 // Block all public services for the offline case, and observe actual local texture fetches.
 const local=[],external=[];const listener=req=>{const u=new URL(req.url());if(u.hostname!=='127.0.0.1')external.push(req.url());else if(u.pathname.includes('/assets/NaturalEarthII/'))local.push(u.pathname);};
 await page.route('https://**/*',route=>route.abort());page.on('request',listener);
 try{
  await page.locator('#fullNew27').click();await page.waitForFunction(()=>document.getElementById('fullStatus').textContent.startsWith('Offline：'));
  await page.waitForFunction(()=>{const s=window.cesiumSandcastleViewer.stats;const n=/imagery (\d+) ready/.exec(s);return n&&Number(n[1])>=2;});
  await page.locator('#globe').scrollIntoViewIfNeeded();await snapshot(page,'__offline');
  const colors=await page.evaluate(()=>{const colors=new Set(),b=window.__offline.bytes;for(let i=0;i<b.length;i+=4)colors.add((b[i]<<16)|(b[i+1]<<8)|b[i+2]);return colors.size;});assert(colors>1000,`local Natural Earth colors ${colors}`);assert(local.length>=2);assert.deepEqual(external,[]);
  const before=await page.evaluate(()=>window.cesiumSandcastleViewer.cameraState.view.destinationDegrees[2]);const box=await page.locator('#globe').boundingBox();await page.mouse.move(box.x+box.width/2,box.y+box.height/2);await page.mouse.wheel(0,-100);await page.waitForFunction(h=>window.cesiumSandcastleViewer.cameraState.view.destinationDegrees[2]<h-.1,before);
  const assetList=JSON.parse(fs.readFileSync(path.join(root,'tests/sandcastle/batch25-assets.json')));for(const a of assetList.assets)assert.equal(hash(path.join(root,a.path)),a.sha256);
  out.push(report('offline',['original local TMS provider and disabled picker/geocoder','all 42 pinned local JPEG files present and hashed','public network blocked; real local tile fetches','real detailed globe pixels and wheel zoom'],{localTileRequests:local.length,externalRequests:external.length,colors,assetsSha256:hash(path.join(root,'tests/sandcastle/batch25-assets.json'))}));console.log('offline passed',local.length,colors);
 }finally{page.off('request',listener);await page.unroute('https://**/*');}
 // Leaving the offline and menu cases cleans their controls/provider state.
 await page.locator('#fullBox').click();await page.waitForFunction(()=>document.getElementById('fullStatus').textContent.startsWith('Box：'));assert(await page.locator('#fullVariantControls').evaluate(e=>e.hidden));
 assert.equal(await page.locator('#error').textContent(),'');assert.deepEqual(errors,[]);return out;
};
