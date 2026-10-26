import vm from 'node:vm';
import assert from 'node:assert/strict';
export const batch23Ids=['circle-dev','corridor','corridor-dev','corridor-outline-dev'];
export function extractBatch23(id,source,item,hash){
 const keys=(o,a)=>assert(Object.keys(o).every(k=>a.includes(k)),`unmapped option ${Object.keys(o)}`);
 class Color{constructor(r,g,b,a=1){this.rgba=[r,g,b,a];}withAlpha(a){return new Color(...this.rgba.slice(0,3),a);}}
 Object.assign(Color,{RED:new Color(1,0,0),GREEN:new Color(0,128/255,0),BLUE:new Color(0,0,1),WHITE:new Color(1,1,1),BLACK:new Color(0,0,0)});
 const Cartesian3={fromDegrees:(x,y,h=0)=>[x,y,h],fromDegreesArray:p=>Array.from({length:p.length/2},(_,i)=>[p[2*i],p[2*i+1],0])};
 const geometry=(type,allowed)=>class{constructor(o){keys(o,allowed);Object.assign(this,{type},o);}};
 const CircleGeometry=geometry('CircleGeometry',['center','radius','vertexFormat','height','extrudedHeight']);
 const CorridorGeometry=geometry('CorridorGeometry',['positions','width','height','extrudedHeight','cornerType','vertexFormat']);
 const CorridorOutlineGeometry=geometry('CorridorOutlineGeometry',['positions','width','height','extrudedHeight','cornerType']);
 class GeometryInstance{constructor(o){keys(o,['geometry','attributes']);keys(o.attributes,['color']);Object.assign(this,o);}}
 class PerInstanceColorAppearance{constructor(o={}){keys(o,['flat','closed','translucent','renderState']);Object.assign(this,{type:'PerInstanceColorAppearance'},o);}}
 PerInstanceColorAppearance.VERTEX_FORMAT='POSITION_NORMAL';
 class Primitive{constructor(o){keys(o,['geometryInstances','appearance']);Object.assign(this,o);}}
 const records=[],primitives=[];let viewers=0,zooms=0;
 class Viewer{constructor(container){assert.equal(container,'cesiumContainer');viewers++;this.entities={add:o=>{records.push(o);return o;}};this.scene={maximumAliasedLineWidth:1,primitives:{add:p=>{assert(p instanceof Primitive);primitives.push(p);return p;}}};}zoomTo(v){assert.equal(v,this.entities);zooms++;}}
 const Cesium={Viewer,Cartesian3,Color,CircleGeometry,CorridorGeometry,CorridorOutlineGeometry,GeometryInstance,PerInstanceColorAppearance,Primitive,CornerType:{ROUNDED:'ROUNDED',MITERED:'MITERED',BEVELED:'BEVELED'},ColorGeometryInstanceAttribute:{fromColor:c=>c.rgba.map(v=>(v===1?255:Math.floor(v*256))/255)}};
 const code=source.replace(/^import .*;\s*$/gm,'');assert(!/\bimport\b/.test(code));vm.runInNewContext(code,{Cesium,Math},{timeout:1000});assert.equal(viewers,1);
 const isEntity=id==='corridor';assert.equal(zooms,isEntity?1:0);assert.equal(records.length,isEntity?3:0);
 const shape=g=>g.type==='CircleGeometry'?{type:'ellipse',height:g.height??0,semiMajorAxis:g.radius,semiMinorAxis:g.radius,rotation:0,extrudedHeight:g.extrudedHeight??null}:{type:'corridor',positionsDegrees:g.positions,width:g.width,height:g.height??0,extrudedHeight:g.extrudedHeight??null,cornerType:g.cornerType??'ROUNDED'};
 let entities;
 if(isEntity)entities=records.map((e,i)=>{keys(e,['name','corridor']);const g=e.corridor;keys(g,['positions','width','height','extrudedHeight','cornerType','material','outline','outlineColor']);return{id:`${id}-${i+1}`,name:e.name,positionDegrees:null,shape:shape(g),color:g.material.rgba,fill:true,outline:g.outline??false,outlineColor:g.outlineColor?.rgba||[0,0,0,1],outlineWidth:1,properties:{sandcastleCase:id,upstreamEntity:i+1}};});
 else entities=primitives.flatMap((p,pi)=>[p.geometryInstances].flat().map((ins,ii)=>{const g=ins.geometry,outline=g.type==='CorridorOutlineGeometry',a=p.appearance;return{id:`primitive-${pi}-instance-${ii}`,name:'',positionDegrees:g.type==='CircleGeometry'?g.center:null,shape:shape(g),color:outline?[1,1,1,1]:ins.attributes.color,fill:!outline,outline,outlineColor:outline?ins.attributes.color:[0,0,0,1],outlineWidth:1,...(!outline?{surfaceAppearance:{flat:a.flat??false,closed:a.closed??false,translucent:a.translucent??true}}:{}),properties:{primitiveIndex:pi,instanceIndex:ii}};}));
 assert.equal(entities.length,id==='corridor-outline-dev'?2:3);
 return{entities,primitive:isEntity?undefined:JSON.parse(JSON.stringify({primitives})),manifest:{id,title:item.title,sourceSha256:hash,referenceUrl:item.referenceUrl,format:isEntity?'entities':'primitives',path:`../../tests/sandcastle/full/${id}.${isEntity?'json':'primitive.json'}`,expectedPath:`../../tests/sandcastle/full/${id}.json`,entityCount:entities.length,framing:isEntity?'zoomToEntities':'viewer-default',shouldAnimate:false,requirements:['all original instances and colors/alpha','rounded/mitered/beveled corners and rounded/flat end caps','source width, height, extrusion, fill and outline','source appearance state and camera calls'],status:'implemented',validation:{source:'passed',gpu:'pending',browser:'pending'}}};
}
