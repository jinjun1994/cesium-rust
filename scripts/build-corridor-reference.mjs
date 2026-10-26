// Optional build-time reference generator. Cesium core is never imported by the viewer.
// npm install --prefix .reference-tools @cesium/core@0.1.0 mersenne-twister earcut
// node scripts/build-corridor-reference.mjs
import fs from 'node:fs';import path from 'node:path';import {pathToFileURL} from 'node:url';import crypto from 'node:crypto';import assert from 'node:assert/strict';
const sourceDir=path.resolve(process.argv[2]||'.reference-tools/node_modules/@cesium/core/Source');
const get=async name=>(await import(pathToFileURL(path.join(sourceDir,name+'.js')).href)).default;
const [Cartesian3,CorridorOutlineGeometry,Ellipsoid,CornerType]=await Promise.all(['Cartesian3','CorridorOutlineGeometry','Ellipsoid','CornerType'].map(get));
const hash=crypto.createHash('sha256').update(fs.readFileSync(path.join(sourceDir,'CorridorGeometryLibrary.js'))).digest('hex');assert.equal(hash,'d4e040e851045cff862e8f69a74b0907878e41cf70cc6be5aed9c931e8ec25d8','pinned core geometry changed');
const cases=[];
for(const cornerType of ['ROUNDED','MITERED','BEVELED']){
 const shape={positionsDegrees:[[-100,40,0],[-105,40,0],[-105,35,0]],width:200000,height:0,extrudedHeight:null,cornerType,type:'corridor'};
 const g=CorridorOutlineGeometry.createGeometry(new CorridorOutlineGeometry({positions:shape.positionsDegrees.map(p=>Cartesian3.fromDegrees(...p)),width:shape.width,cornerType:CornerType[cornerType]}));
 const values=g.attributes.position.values;const positions=Array.from({length:values.length/3},(_,i)=>Array.from(values.slice(i*3,i*3+3)));cases.push({shape,positions,indices:Array.from(g.indices)});
}
const d={commit:'b8d3a36fe98a3e432eb89253c95d5f20e605e0f1',package:'@cesium/core@0.1.0',corridorLibrarySha256:hash,cases};
fs.writeFileSync(new URL('../tests/sandcastle/full/corridor-reference.json',import.meta.url),JSON.stringify(d,null,2)+'\n');console.log('Independent Cesium reference',cases.map(c=>[c.shape.cornerType,c.positions.length]));
