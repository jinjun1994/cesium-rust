// Execute fixed Primitive recipes against recording geometry/instance classes.
import vm from 'node:vm';
import assert from 'node:assert/strict';
export const primitiveIds=['box-outline-dev','cylinder-outline-dev','sphere-outline-dev','ellipsoid-outline-dev','rectangle-outline-dev','wall-outline-dev','box-dev','cylinder-dev','sphere-dev','ellipsoid-dev','wall-dev','ellipse-dev'];
const keys=(value,allowed)=>assert(Object.keys(value).every(k=>allowed.includes(k)),'unmapped Primitive option');
export function extractPrimitive(id,source,item,hash){
 class Cartesian3 {constructor(x,y,z){Object.assign(this,{x,y,z});}toJSON(){return [this.x,this.y,this.z];}static fromDegrees(lon,lat,height=0){return {degrees:[lon,lat,height],toJSON(){return this.degrees;}};}static fromDegreesArray(p){assert.equal(p.length%2,0);return Array.from({length:p.length/2},(_,i)=>[p[2*i],p[2*i+1],0]);}}
 class Matrix4 {constructor(){this.values=[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1];}toJSON(){return this.values;}static multiplyByTranslation(m,t,result){result.values=m.values.slice();for(let i=0;i<3;i++)result.values[12+i]+=m.values[i]*t.x+m.values[4+i]*t.y+m.values[8+i]*t.z;return result;}}
 const Transforms={eastNorthUpToFixedFrame(p){const [lon,lat,h]=p.degrees;const l=lon*Math.PI/180,f=lat*Math.PI/180,sl=Math.sin(l),cl=Math.cos(l),sf=Math.sin(f),cf=Math.cos(f);const n=6378137/Math.sqrt(1-(1/298.257223563)*(2-1/298.257223563)*sf*sf),m=new Matrix4();m.values=[-sl,cl,0,0,-sf*cl,-sf*sl,cf,0,cf*cl,cf*sl,sf,0,(n+h)*cf*cl,(n+h)*cf*sl,(n*(1-(1/298.257223563)*(2-1/298.257223563))+h)*sf,1];return m;}};
 const geometry=(type,allowed)=>class {constructor(o){keys(o,allowed);Object.assign(this,{type},o);}};
 const BoxOutlineGeometry=geometry('BoxOutlineGeometry',['dimensions']);BoxOutlineGeometry.fromDimensions=o=>new BoxOutlineGeometry(o);
 const CylinderOutlineGeometry=geometry('CylinderOutlineGeometry',['length','topRadius','bottomRadius','numberOfVerticalLines']);
 const SphereOutlineGeometry=geometry('SphereOutlineGeometry',['radius','stackPartitions','slicePartitions']);
 const EllipsoidOutlineGeometry=geometry('EllipsoidOutlineGeometry',['radii','stackPartitions','slicePartitions']);
 const RectangleOutlineGeometry=geometry('RectangleOutlineGeometry',['rectangle']);
 const WallOutlineGeometry=geometry('WallOutlineGeometry',['positions','maximumHeight','minimumHeight']);WallOutlineGeometry.fromConstantHeights=o=>new WallOutlineGeometry(o);
 const BoxGeometry=geometry('BoxGeometry',['dimensions','vertexFormat']);BoxGeometry.fromDimensions=o=>new BoxGeometry(o);
 const CylinderGeometry=geometry('CylinderGeometry',['length','topRadius','bottomRadius','vertexFormat']);
 const SphereGeometry=geometry('SphereGeometry',['radius','vertexFormat']);
 const EllipsoidGeometry=geometry('EllipsoidGeometry',['radii','vertexFormat']);
 const WallGeometry=geometry('WallGeometry',['positions','minimumHeight','maximumHeight','minimumHeights','maximumHeights','vertexFormat']);WallGeometry.fromConstantHeights=o=>new WallGeometry(o);
 const EllipseGeometry=geometry('EllipseGeometry',['center','semiMinorAxis','semiMajorAxis','height','rotation','extrudedHeight','vertexFormat']);
 class Color{constructor(r,g,b,a=1){this.rgba=[r,g,b,a];}toJSON(){return this.rgba;}}
 Object.assign(Color,{WHITE:new Color(1,1,1),RED:new Color(1,0,0),GREEN:new Color(0,128/255,0),BLUE:new Color(0,0,1)});
 class GeometryInstance {constructor(o){keys(o,['geometry','modelMatrix','attributes','id']);keys(o.attributes,['color']);Object.assign(this,o);}}
 class PerInstanceColorAppearance {constructor(o={}){keys(o,['flat','renderState','closed','translucent']);if(o.renderState)keys(o.renderState,['lineWidth']);Object.assign(this,{type:'PerInstanceColorAppearance'},o);}}
 PerInstanceColorAppearance.VERTEX_FORMAT='POSITION_NORMAL';
 class Primitive {constructor(o){keys(o,['geometryInstances','appearance']);Object.assign(this,o);}}
 let viewers=0;const primitives=[];
 class Viewer {constructor(container){assert.equal(container,'cesiumContainer');viewers++;this.scene={maximumAliasedLineWidth:1,primitives:{add(p){assert(p instanceof Primitive);primitives.push(p);return p;}}};}}
 const Cesium={Viewer,Cartesian3,Matrix4,Transforms,BoxOutlineGeometry,CylinderOutlineGeometry,SphereOutlineGeometry,EllipsoidOutlineGeometry,RectangleOutlineGeometry,WallOutlineGeometry,BoxGeometry,CylinderGeometry,SphereGeometry,EllipsoidGeometry,WallGeometry,EllipseGeometry,Math:{toRadians:n=>n*Math.PI/180},GeometryInstance,PerInstanceColorAppearance,Primitive,Rectangle:{fromDegrees:(...p)=>p},Color,ColorGeometryInstanceAttribute:{fromColor:c=>c.rgba.map(v=>(v===1?255:Math.floor(v*256))/255)}};
 const executable=source.replace(/^import .*;\s*$/gm,'');assert(!/\bimport\b/.test(executable));vm.runInNewContext(executable,{Cesium,Math},{timeout:1000});assert.equal(viewers,1);assert.equal(primitives.length,1);
 const raw=JSON.parse(JSON.stringify({primitives}));
 const entities=raw.primitives.flatMap((p,pi)=>[p.geometryInstances].flat().map((instance,ii)=>{
  const outline=instance.geometry.type.includes('Outline');if(outline){assert.equal(p.appearance.flat,true);assert.equal(p.appearance.renderState.lineWidth,1);}const g=instance.geometry;let shape;let position=null;
  if(g.type==='BoxOutlineGeometry')shape={type:'box',dimensions:g.dimensions};
  if(g.type==='CylinderOutlineGeometry')shape={type:'cylinder',length:g.length,topRadius:g.topRadius,bottomRadius:g.bottomRadius};
  if(g.type==='SphereOutlineGeometry'||g.type==='EllipsoidOutlineGeometry')shape={type:'ellipsoid',radii:g.radii||[g.radius,g.radius,g.radius],stackPartitions:g.stackPartitions,slicePartitions:g.slicePartitions};
  if(g.type==='RectangleOutlineGeometry')shape={type:'rectangle',coordinatesDegrees:g.rectangle,height:0,extrudedHeight:null};
  if(g.type==='WallOutlineGeometry')shape={type:'wall',positionsDegrees:g.positions,minimumHeights:g.positions.map(()=>g.minimumHeight??0),maximumHeights:g.positions.map(()=>g.maximumHeight)};
  if(!outline){
   assert.equal(g.vertexFormat,'POSITION_NORMAL');
   if(g.type==='BoxGeometry')shape={type:'box',dimensions:g.dimensions};
   if(g.type==='CylinderGeometry')shape={type:'cylinder',length:g.length,topRadius:g.topRadius,bottomRadius:g.bottomRadius};
   if(g.type==='SphereGeometry'||g.type==='EllipsoidGeometry')shape={type:'ellipsoid',radii:g.radii||[g.radius,g.radius,g.radius]};
   if(g.type==='WallGeometry')shape={type:'wall',positionsDegrees:g.positions,minimumHeights:g.minimumHeights||g.positions.map(()=>g.minimumHeight??0),maximumHeights:g.maximumHeights||g.positions.map(()=>g.maximumHeight)};
   if(g.type==='EllipseGeometry'){position=g.center;shape={type:'ellipse',semiMajorAxis:g.semiMajorAxis,semiMinorAxis:g.semiMinorAxis,height:g.height??0,rotation:g.rotation??0,extrudedHeight:g.extrudedHeight??null};}
  }
  assert(shape);return {id:instance.id||`primitive-${pi}-instance-${ii}`,name:'',positionDegrees:position,...(!outline?{surfaceAppearance:{flat:p.appearance.flat??false,closed:p.appearance.closed??false,translucent:p.appearance.translucent??true}}:{}),...(instance.modelMatrix?{modelMatrix:instance.modelMatrix}:{}),shape,color:outline?[1,1,1,1]:instance.attributes.color,fill:!outline,outline,outlineColor:outline?instance.attributes.color:[0,0,0,1],outlineWidth:1,properties:{primitiveIndex:pi,instanceIndex:ii}};
 }));
 return {entities,primitive:raw,manifest:{id,title:item.title,sourceSha256:hash,referenceUrl:item.referenceUrl,format:'primitives',path:`../../tests/sandcastle/full/${id}.primitive.json`,expectedPath:`../../tests/sandcastle/full/${id}.json`,entityCount:entities.length,framing:'viewer-default',requirements:['all original GeometryInstance options','original full modelMatrix in ECEF','all per-instance colors, quantization, flat/closed/translucent appearance','lineWidth=min(2, maximumAliasedLineWidth=1)','all fill/outline geometry; no case-specific camera calls'],status:'implemented',validation:{source:'passed',gpu:'pending',browser:'pending'}}};
}
