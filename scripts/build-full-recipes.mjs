// Execute all statements of the fixed upstream recipes in a data-recording environment.
// This is build-time extraction, not a CesiumJS runtime or a fallback renderer.
import vm from 'node:vm';
import {batch45Ids,extractBatch45} from './batch45-recipes.mjs';
import {batch46Ids,extractBatch46} from './batch46-recipes.mjs';
import {batch47Ids,extractBatch47} from './batch47-recipes.mjs';
import {batch48Ids,extractBatch48} from './batch48-recipes.mjs';
import {batch49Ids,extractBatch49} from './batch49-recipes.mjs';
import {batch50Ids,extractBatch50} from './batch50-recipes.mjs';
import {batch51Ids,extractBatch51} from './batch51-recipes.mjs';
import {batch52Ids,extractBatch52} from './batch52-recipes.mjs';
import {batch53Ids,extractBatch53} from './batch53-recipes.mjs';
import {batch54Ids,extractBatch54} from './batch54-recipes.mjs';
import {batch55Ids,extractBatch55} from './batch55-recipes.mjs';
import {batch56Ids,extractBatch56} from './batch56-recipes.mjs';
import {batch44Ids,extractBatch44} from './batch44-recipes.mjs';
import {batch43Ids,extractBatch43} from './batch43-recipes.mjs';
import {batch42Ids,extractBatch42} from './batch42-recipes.mjs';
import {batch41Ids,extractBatch41} from './batch41-recipes.mjs';
import {batch40Ids,extractBatch40} from './batch40-recipes.mjs';
import {batch39Ids,extractBatch39} from './batch39-recipes.mjs';
import {batch38Ids,extractBatch38} from './batch38-recipes.mjs';
import {batch37Ids,extractBatch37} from './batch37-recipes.mjs';
import {batch36Ids,extractBatch36} from './batch36-recipes.mjs';
import {batch35Ids,extractBatch35} from './batch35-recipes.mjs';
import {batch34Ids,extractBatch34} from './batch34-recipes.mjs';
import {batch33Ids,extractBatch33} from './batch33-recipes.mjs';
import {batch32Ids,extractBatch32} from './batch32-recipes.mjs';
import {batch31Ids,extractBatch31} from './batch31-recipes.mjs';
import {batch30Ids,extractBatch30} from './batch30-recipes.mjs';
import {batch29Ids,extractBatch29} from './batch29-recipes.mjs';
import {batch28Ids,extractBatch28} from './batch28-recipes.mjs';
import {batch26Ids,extractBatch26} from './batch26-recipes.mjs';
import {batch25Ids,extractBatch25} from './batch25-recipes.mjs';
import {batch24Ids,extractBatch24} from './batch24-recipes.mjs';
import {batch23Ids,extractBatch23} from './batch23-recipes.mjs';
import {batch22Ids,extractBatch22} from './batch22-recipes.mjs';
import {primitiveIds,extractPrimitive} from './primitive-recipes.mjs';
import fs from 'node:fs/promises';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
const root=new URL('../',import.meta.url);
const catalog=JSON.parse(await fs.readFile(new URL('tests/sandcastle/catalog.json',root),'utf8'));
export const ids=['box','cylinders-and-cones','plane','spheres-and-ellipsoids','czml-spheres-and-ellipsoids','czml-cones-and-cylinders','czml-box','wall','czml-wall','circles-and-ellipses','czml-circles-and-ellipses','czml-point',...primitiveIds,...batch22Ids,...batch23Ids,...batch24Ids,...batch25Ids,...batch26Ids,...batch28Ids,...batch29Ids,...batch30Ids,...batch31Ids,...batch32Ids,...batch33Ids,...batch34Ids,...batch35Ids,...batch36Ids,...batch37Ids,...batch38Ids,...batch39Ids,...batch40Ids,...batch41Ids,...batch42Ids,...batch43Ids,...batch44Ids,...batch45Ids,...batch46Ids,...batch47Ids,...batch48Ids,...batch49Ids,...batch50Ids,...batch51Ids,...batch52Ids,...batch53Ids,...batch54Ids,...batch55Ids,...batch56Ids];
export async function extract(id){
 const item=catalog.cases.find(c=>c.id===id);assert(item);
 const source=await fs.readFile(new URL(`tests/sandcastle/sources/${id}/main.js`,root),'utf8');
 const hash=crypto.createHash('sha256').update(source).digest('hex');assert.equal(hash,item.sourceSha256,'pinned source changed');
 if(batch56Ids.includes(id))return extractBatch56(id,source,item,hash);
 if(batch55Ids.includes(id))return extractBatch55(id,source,item,hash);
 if(batch54Ids.includes(id))return extractBatch54(id,source,item,hash);
 if(batch53Ids.includes(id))return extractBatch53(id,source,item,hash);
 if(batch52Ids.includes(id))return extractBatch52(id,source,item,hash);
 if(batch51Ids.includes(id))return extractBatch51(id,source,item,hash);
 if(batch50Ids.includes(id))return extractBatch50(id,source,item,hash);
 if(batch49Ids.includes(id))return extractBatch49(id,source,item,hash);
 if(batch48Ids.includes(id))return extractBatch48(id,source,item,hash);
 if(batch47Ids.includes(id))return extractBatch47(id,source,item,hash);
 if(batch46Ids.includes(id))return extractBatch46(id,source,item,hash);
 if(batch45Ids.includes(id))return extractBatch45(id,source,item,hash);
 if(batch44Ids.includes(id))return extractBatch44(id,source,item,hash);
 if(batch43Ids.includes(id))return extractBatch43(id,source,item,hash);
 if(batch42Ids.includes(id))return extractBatch42(id,source,item,hash);
 if(batch41Ids.includes(id))return extractBatch41(id,source,item,hash);
 if(batch40Ids.includes(id))return extractBatch40(id,source,item,hash);
 if(batch39Ids.includes(id))return extractBatch39(id,source,item,hash);
 if(batch38Ids.includes(id))return extractBatch38(id,source,catalog,hash);
 if(batch37Ids.includes(id))return extractBatch37(id,source,item,hash);
 if(batch36Ids.includes(id))return extractBatch36(id,source,item,hash);
 if(batch35Ids.includes(id))return extractBatch35(id,source,item,hash);
 if(batch34Ids.includes(id))return extractBatch34(id,source,item,hash);
 if(batch33Ids.includes(id))return extractBatch33(id,source,item,hash);
 if(batch32Ids.includes(id))return extractBatch32(id,source,item,hash);
 if(batch31Ids.includes(id))return extractBatch31(id,source,item,hash);
 if(batch30Ids.includes(id))return extractBatch30(id,source,item,hash);
 if(batch29Ids.includes(id))return extractBatch29(id,source,item,hash);
 if(batch28Ids.includes(id))return extractBatch28(id,source,item,hash,'data:image/jpeg;base64,'+(await fs.readFile(new URL('tests/sandcastle/images/Cesium_Logo_Color.jpg',root))).toString('base64'));
 if(batch26Ids.includes(id))return extractBatch26(id,source,item,hash);
 if(batch25Ids.includes(id))return extractBatch25(id,source,item,hash,'data:image/jpeg;base64,'+(await fs.readFile(new URL('tests/sandcastle/images/Cesium_Logo_Color.jpg',root))).toString('base64'));
 if(batch24Ids.includes(id)){
  const extracted=extractBatch24(id,source,item,hash);
  if(['html-overlays','resolution-scaling'].includes(id))extracted.manifest.htmlSha256=crypto.createHash('sha256').update(await fs.readFile(new URL(`tests/sandcastle/sources/${id}/index.html`,root))).digest('hex');
  if(id==='html-overlays')extracted.manifest.assetSha256=crypto.createHash('sha256').update(await fs.readFile(new URL('tests/sandcastle/full/Cesium_Logo_overlay.png',root))).digest('hex');
  return extracted;
 }
 if(batch23Ids.includes(id))return extractBatch23(id,source,item,hash);
 if(batch22Ids.includes(id))return extractBatch22(id,source,item,hash);
 if(primitiveIds.includes(id))return extractPrimitive(id,source,item,hash);
 class Cartesian3 {constructor(x,y,z){Object.assign(this,{x,y,z});}static fromDegrees(lon,lat,height=0){return {degrees:[lon,lat,height]};}static fromDegreesArray(p){return Array.from({length:p.length/2},(_,i)=>[p[i*2],p[i*2+1],0]);}static fromDegreesArrayHeights(p){return Array.from({length:p.length/3},(_,i)=>p.slice(i*3,i*3+3));} }
 Cartesian3.UNIT_X=new Cartesian3(1,0,0);Cartesian3.UNIT_Y=new Cartesian3(0,1,0);Cartesian3.UNIT_Z=new Cartesian3(0,0,1);
 class Cartesian2 {constructor(x,y){Object.assign(this,{x,y});}}
 class Color {constructor(r,g,b,a=1){this.rgba=[r,g,b,a];}withAlpha(a){return new Color(...this.rgba.slice(0,3),a);}}
 Object.assign(Color,{BLUE:new Color(0,0,1),RED:new Color(1,0,0),GREEN:new Color(0,128/255,0),BLACK:new Color(0,0,0),YELLOW:new Color(1,1,0)});
 class Plane {constructor(normal,distance){Object.assign(this,{normal,distance});}}
 const records=[];let zooms=0,viewers=0,loaded,loadPromise,added;
 class CzmlDataSource {static load(data){assert(!loaded);loaded=JSON.parse(JSON.stringify(data));loadPromise=Promise.resolve({czml:data});return loadPromise;}}
 class Viewer {constructor(container){assert.equal(container,'cesiumContainer');viewers++;this.entities={add(o){records.push(o);return o;}};this.dataSources={add(p){assert.equal(p,loadPromise);added=p;return p;}};}zoomTo(v){assert.equal(v,loaded?loadPromise:this.entities);zooms++;}}
 const executable=source.replace(/^import .*;\s*$/gm,'');assert(!/\bimport\b/.test(executable));
 vm.runInNewContext(executable,{Cesium:{Viewer,Cartesian2,Cartesian3,Color,Plane,CzmlDataSource,Math:{toRadians:n=>n*Math.PI/180}}},{timeout:1000});
 assert.equal(viewers,1);assert.equal(zooms,1);
 const isCzml=id.startsWith('czml-');assert.equal(!!loaded,isCzml);if(isCzml){assert.equal(added,loadPromise);assert.equal(loaded[0].version,'1.0');}
 const count={'czml-wall':1,'czml-point':1,'cylinders-and-cones':2,'czml-cones-and-cylinders':2}[id]||3;assert.equal(isCzml?loaded.length-1:records.length,count);
 const entities=(isCzml?loaded.slice(1):records).map((e,i)=>{
  const type=['box','cylinder','plane','ellipsoid','wall','ellipse','point'].find(t=>e[t]);assert(type);
  assert(Object.keys(e).every(k=>['id','name','position',type].includes(k)),'unmapped entity field');
  const g=e[type];const fields={box:['dimensions'],cylinder:['length','topRadius','bottomRadius'],plane:['plane','dimensions'],ellipsoid:['radii','stackPartitions','slicePartitions'],wall:['positions','minimumHeights','maximumHeights'],ellipse:['semiMajorAxis','semiMinorAxis','height','extrudedHeight','rotation'],point:['color','pixelSize','outlineWidth']}[type];
  assert(Object.keys(g).every(k=>[...fields,'material','fill','outline','outlineColor'].includes(k)),'unmapped graphics field');let shape;
  const vector=v=>isCzml?v.cartesian:[v.x,v.y,v.z];
  if(type==='ellipsoid')shape={type,radii:vector(g.radii),...(g.stackPartitions===undefined?{}:{stackPartitions:g.stackPartitions}),...(g.slicePartitions===undefined?{}:{slicePartitions:g.slicePartitions})};
  if(type==='wall')shape={type,positionsDegrees:isCzml?Array.from({length:g.positions.cartographicDegrees.length/3},(_,i)=>g.positions.cartographicDegrees.slice(i*3,i*3+3)):g.positions,minimumHeights:g.minimumHeights||[],maximumHeights:g.maximumHeights||[]};
  if(type==='ellipse')shape={type,height:g.height??0,semiMajorAxis:g.semiMajorAxis,semiMinorAxis:g.semiMinorAxis,rotation:g.rotation??0,extrudedHeight:g.extrudedHeight??null};
  if(type==='point')shape={type,pixelSize:g.pixelSize??1,outlineWidth:g.outlineWidth??0,outlineColor:g.outlineColor?.rgbaf||g.outlineColor?.rgba.map(n=>n/255)||[0,0,0,1]};
  if(type==='box')shape={type,dimensions:vector(g.dimensions)};
  if(type==='cylinder')shape={type,length:g.length,topRadius:g.topRadius,bottomRadius:g.bottomRadius};
  if(type==='plane')shape={type,dimensions:[g.dimensions.x,g.dimensions.y],normal:[g.plane.normal.x,g.plane.normal.y,g.plane.normal.z],distance:g.plane.distance};
  const color=v=>{if(!isCzml)return(v||new Color(1,1,1)).rgba;if(!v)return[1,1,1,1];assert(Object.keys(v).length===1);return v.rgbaf||v.rgba.map(n=>n/255);};
  if(isCzml&&g.material)assert.deepEqual(Object.keys(g.material),['solidColor']);
  return {id:isCzml?e.id:`${id}-${i+1}`,name:e.name||'',positionDegrees:e.position?(isCzml?e.position.cartographicDegrees:e.position.degrees):null,shape,color:color(isCzml?(type==='point'?g.color:g.material?.solidColor.color):g.material),fill:g.fill??true,outline:type==='point'?false:g.outline??false,outlineColor:isCzml?color(g.outlineColor||{rgbaf:[0,0,0,1]}):((g.outlineColor||Color.BLACK).rgba),outlineWidth:1,properties:isCzml?null:{sandcastleCase:id,upstreamEntity:i+1}};
 });
 return {entities:JSON.parse(JSON.stringify(entities)),czml:loaded,manifest:{id,title:item.title,sourceSha256:hash,referenceUrl:item.referenceUrl,format:isCzml?'czml':'entities',path:`../../tests/sandcastle/full/${id}.${isCzml?'czml':'json'}`,expectedPath:`../../tests/sandcastle/full/${id}.json`,entityCount:entities.length,framing:'zoomToEntities',requirements:['all upstream entity positions and dimensions','all upstream materials and alpha','fill and outline controls','plane normal and distance, if present','viewer.zoomTo entire entity collection'],status:'implemented',validation:{source:'passed',gpu:'pending',browser:'pending'}}};
}
if(process.argv[1]?.endsWith('build-full-recipes.mjs')){
 const manifest={schemaVersion:1,release:catalog.release,commit:catalog.commit,cases:[]};await fs.mkdir(new URL('tests/sandcastle/full/',root),{recursive:true});
 let previous;try{previous=JSON.parse(await fs.readFile(new URL('tests/sandcastle/full-recipes.json',root),'utf8'));}catch{}
 for(const id of ids){const {entities,czml,primitive,grouped,scene,variants,manifest:entry}=await extract(id);const data=JSON.stringify(entities,null,2)+'\n';const path=new URL(`tests/sandcastle/full/${id}.json`,root);const old=previous?.cases.find(c=>c?.id===id);let same=false;try{same=await fs.readFile(path,'utf8')===data;}catch{}if(same&&old?.sourceSha256===entry.sourceSha256&&old?.validation){entry.validation=old.validation;entry.status=old.status;}
 await fs.writeFile(path,data);if(czml)await fs.writeFile(new URL(`tests/sandcastle/full/${id}.czml`,root),JSON.stringify(czml,null,2)+'\n');if(primitive)await fs.writeFile(new URL(`tests/sandcastle/full/${id}.primitive.json`,root),JSON.stringify(primitive,null,2)+'\n');if(grouped)await fs.writeFile(new URL(`tests/sandcastle/full/${id}.groups.json`,root),JSON.stringify(grouped,null,2)+'\n');if(scene)await fs.writeFile(new URL(`tests/sandcastle/full/${id}.scene.json`,root),JSON.stringify(scene,null,2)+'\n');if(variants)await fs.writeFile(new URL(`tests/sandcastle/full/${id}.variants.json`,root),JSON.stringify(variants,null,2)+'\n');manifest.cases.push(entry);}

 await fs.writeFile(new URL('tests/sandcastle/full-recipes.json',root),JSON.stringify(manifest,null,2)+'\n');console.log(`Extracted ${manifest.cases.length} complete upstream recipe bodies with pinned SHA-256 and collection framing.`);
}
