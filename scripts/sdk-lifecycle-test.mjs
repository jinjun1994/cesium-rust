// Exercise the production SDK request guards with a fake GPU boundary and real Response streams.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {pathToFileURL} from 'node:url';
class Canvas extends EventTarget {
 width=800;height=600;clientHeight=600;
 getBoundingClientRect(){return {width:800,height:600,left:0,top:0};}
 setPointerCapture(){} focus(){}
}
let primitiveLoads=0,primitiveInput,imageryTemplate;
const elapsedFrames=[];let czmlLoads=0,entityLoads=0,upserts=0;let clockState={startTime:0,stopTime:60,currentTime:0,multiplier:1,shouldAnimate:false,range:"LOOP_STOP"};
const callbacks=[];let submits=0,adds=0,clears=0,phase='old',oldRequested=false;
const viewer={maximumAliasedLineWidth:1,setPrimitiveInstances(bytes){primitiveLoads++;primitiveInput=JSON.parse(new TextDecoder().decode(bytes));},advanceTime(dt){elapsedFrames.push(dt);},
 loadCzml(){czmlLoads++;},setEntities(){entityLoads++;},upsertEntities(){upserts++;},entitySpecs:'[{"id":"box","show":true}]',showEntity(id,show){return id==='box';},get clockState(){return JSON.stringify(clockState);},setClock(text){clockState=JSON.parse(text);},removeEntity(){return true;},clearEntities(){},focusEntity(){return true;},renderFrame(){},resize(){},orbit(){},zoom(){},focusGlobe(){},cancelRequests(){},
 setImageryTemplate(value){imageryTemplate=value;},takeImageryRequests(){return '[]';},loadTileset(){phase='new';},loadBytes(){phase='new';},
 takeRequests(){if(phase==='old' && !oldRequested){oldRequested=true;return JSON.stringify([{id:0,url:'http://test.invalid/old-tile'}]);}return '[]';},
 submitTile(){submits++;},failTile(){},addGeoJson(){adds++;return 7;},removeGeoJson(id){return id===7;},clearGeoJson(){clears++;},focusGeoJson(id){return id===7;},
 setTerrainOptions(){},setPublicTerrain(){},clearTerrain(){},cancelTerrainRequests(){},takeTerrainRequests(){return '[]';},terrainCredits:'test terrain',destroy(){},free(){},lastPick:'null',stats:'test',imageryCredits:'test'};
Object.defineProperty(globalThis,'navigator',{value:{gpu:{}},configurable:true});
globalThis.window={devicePixelRatio:1};globalThis.location={href:'http://test.invalid/'};
globalThis.ResizeObserver=class {observe(){} disconnect(){}};
globalThis.requestAnimationFrame=callback=>{callbacks.push(callback);return callbacks.length;};globalThis.cancelAnimationFrame=()=>{};
globalThis.fakeViewer=viewer;
const pending=new Map();globalThis.fetch=(url)=>new Promise(resolve=>pending.set(String(url),resolve));
const file=new URL('../bindings/web/sdk.js',import.meta.url);
let source=await fs.readFile(file,'utf8');
source=source.replace("import {hydrateImages} from './resources.js';",`import {hydrateImages} from ${JSON.stringify(new URL('../bindings/web/resources.js',import.meta.url).href)};`);
source=source.replace("import {attachControls} from './controls.js';",`import {attachControls} from ${JSON.stringify(pathToFileURL(new URL('../bindings/web/controls.js',import.meta.url).pathname).href)};`);
source=source.replace(/^import init,.*$/m,'const init=async()=>{};const attach_canvas=async()=>globalThis.fakeViewer;const bingMetadataUrl=()=>"";');
const {attachCanvas}=await import('data:text/javascript,'+encodeURIComponent(source));
const canvas = new Canvas();
const errors=[];const sdk=await attachCanvas(canvas,{onError:error=>errors.push(error)});
const modelLoad=sdk.load('/root.json');callbacks.shift()();
assert(pending.has('http://test.invalid/old-tile'));
pending.get('http://test.invalid/root.json')(new Response(JSON.stringify({asset:{version:'1.1'},root:{}})));
await modelLoad;
pending.get('http://test.invalid/old-tile')(new Response(new Uint8Array([1,2,3])));
await new Promise(resolve=>setTimeout(resolve,0));assert.equal(submits,0,'old source bytes cannot be submitted into replacement scene');
const geoLoad=sdk.loadGeoJson('/geo.json');sdk.clearGeoJson();
pending.get('http://test.invalid/geo.json')(new Response(JSON.stringify({type:'Point',coordinates:[110,25]})));
assert.equal(await geoLoad,null);assert.equal(adds,0);assert.equal(clears,1);
assert.equal(sdk.addGeoJson({type:'Point',coordinates:[110,25]}),7);assert(sdk.focusGeoJson(7));assert(sdk.removeGeoJson(7));
sdk.setEntities([{id:'box'}]);assert.equal(entityLoads,1);sdk.upsertEntities([{id:'box'}]);assert.equal(upserts,1);assert.equal(sdk.entities[0].id,'box');assert(sdk.showEntity('box',false));
sdk.setClock({shouldAnimate:true,multiplier:-2});assert.equal(sdk.clock.multiplier,-2);
callbacks.shift()(1000);callbacks.shift()(1500);assert.equal(elapsedFrames.at(-1),0.5,'RAF milliseconds are converted to clock seconds');
callbacks.shift()(5000);assert.equal(elapsedFrames.at(-1),1,'background tab time is bounded');
const staleCzml=sdk.loadCzml('/stale.czml');sdk.loadBytes(new Uint8Array([1]));
pending.get('http://test.invalid/stale.czml')(new Response('[]'));assert.equal(await staleCzml,false);assert.equal(czmlLoads,0);
const imports=Array.from({length:4},(_,i)=>sdk.loadCzml(`/active-${i}.czml`));await assert.rejects(sdk.loadCzml('/fifth.czml'),/concurrency/);
for(let i=0;i<4;i++)pending.get(`http://test.invalid/active-${i}.czml`)(new Response('[]'));
assert((await Promise.all(imports)).every(Boolean));assert.equal(czmlLoads,4);
assert.equal(sdk.maximumAliasedLineWidth,1);
const primitiveFixture=JSON.parse(await fs.readFile(new URL('../tests/sandcastle/full/box-outline-dev.primitive.json',import.meta.url),'utf8'));
assert.equal(await sdk.loadPrimitiveInstances(primitiveFixture),true);assert.deepEqual(primitiveInput,primitiveFixture);
const stalePrimitive=sdk.loadPrimitiveInstances('/stale-primitive.json');sdk.setEntities([]);pending.get('http://test.invalid/stale-primitive.json')(new Response(JSON.stringify(primitiveFixture)));assert.equal(await stalePrimitive,false);assert.equal(primitiveLoads,1);
const primitiveImports=Array.from({length:4},(_,i)=>sdk.loadPrimitiveInstances(`/primitive-${i}.json`));await assert.rejects(sdk.loadPrimitiveInstances('/fifth-primitive.json'),/concurrency/);
for(let i=0;i<4;i++)pending.get(`http://test.invalid/primitive-${i}.json`)(new Response(JSON.stringify(primitiveFixture)));assert((await Promise.all(primitiveImports)).every(Boolean));assert.equal(primitiveLoads,5);
await assert.rejects(sdk.loadPrimitiveInstances({payload:'x'.repeat(8*1024*1024)}),/limit/);
window.devicePixelRatio=2;
sdk.resolutionScale=0.25;assert.equal(canvas.width,400);assert.equal(canvas.height,300);
sdk.useBrowserRecommendedResolution=true;assert.equal(canvas.width,200);assert.equal(canvas.height,150);
assert.throws(()=>sdk.resolutionScale=0,/bound|range|resolution/i);
viewer.projectCartesian=()=>JSON.stringify([100,75]);
const result={};assert.equal(sdk.scene.cartesianToCanvasCoordinates([1,2,3],result),result);assert.deepEqual(result,{x:400,y:300});
viewer.projectCartesian=()=>"null";assert.equal(sdk.scene.cartesianToCanvasCoordinates([1,2,3]),undefined);
let preRenders=0;const unsubscribe=sdk.scene.preRender.addEventListener(()=>preRenders++);
callbacks.shift()(6000);assert.equal(preRenders,1);unsubscribe();callbacks.shift()(6100);assert.equal(preRenders,1);
let groupsInput;viewer.setGroupedEntities=b=>{groupsInput=JSON.parse(new TextDecoder().decode(b));};
viewer.entityGroups='[{"id":"folder","show":true,"entityIds":[]}]';viewer.showEntityGroup=(id,show)=>id==='folder'&&!show;
assert(await sdk.loadGroupedEntities({entities:[],groups:[]}));assert.deepEqual(groupsInput,{entities:[],groups:[]});assert(sdk.showEntityGroup('folder',false));assert.equal(sdk.entityGroups.length,1);
const staleGroups=sdk.loadGroupedEntities('/stale-groups.json');sdk.clearEntities();pending.get('http://test.invalid/stale-groups.json')(new Response('{"entities":[],"groups":[]}'));assert.equal(await staleGroups,false);
sdk.setImageryTemplate('/local/{z}/{x}/{reverseY}.jpg');assert.equal(imageryTemplate,'http://test.invalid/local/{z}/{x}/{reverseY}.jpg');assert.throws(()=>sdk.setImageryTemplate('file:///tmp/tiles/{z}.jpg'),/URL/);
sdk.scene.preRender.addEventListener(()=>{});
sdk.destroy();assert.equal(sdk.scene.preRender.numberOfListeners,0);await assert.rejects(sdk.loadCzml('/destroyed.czml'),/destroyed/);await assert.rejects(sdk.loadPrimitiveInstances(primitiveFixture),/destroyed/);assert.equal(sdk.maximumAliasedLineWidth,0);assert.equal(sdk.stats,'destroyed');assert.deepEqual(sdk.entities,[]);assert.equal(sdk.showEntity('box'),false);assert.deepEqual(errors,[]);
console.log('SDK lifecycle passed: source replacement drops stale tile bytes; clear cancels GeoJSON import; layer add/focus/remove/destroy');
