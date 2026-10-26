import assert from 'node:assert/strict';
import {attachControls,wheelZoomDelta} from '../bindings/web/controls.js';
class Canvas extends EventTarget {
  width=1600;height=1200;clientHeight=600;tabIndex=-1;
  setPointerCapture() {}
  focus() {this.focused=true;}
  getBoundingClientRect() {return {left:0,top:0,width:800,height:600};}
}
function dispatch(canvas,type,properties) {
  const event=new Event(type,{cancelable:true});Object.assign(event,properties);canvas.dispatchEvent(event);return event;
}
assert.equal(wheelZoomDelta({deltaY:48,deltaMode:0}),wheelZoomDelta({deltaY:3,deltaMode:1}));
assert.equal(wheelZoomDelta({deltaY:1,deltaMode:2},100),wheelZoomDelta({deltaY:100,deltaMode:0}));
assert.equal(wheelZoomDelta({deltaY:NaN}),0);
const canvas=new Canvas(),zooms=[],orbits=[],picks=[];let resets=0;
const detach=attachControls(canvas,{zoom:d=>zooms.push(d),orbit:(...args)=>orbits.push(args),pick:(...args)=>picks.push(args),reset:()=>resets++});
const wheel=dispatch(canvas,'wheel',{deltaY:-100,deltaMode:0});assert(wheel.defaultPrevented);assert(zooms.at(-1)>0);
dispatch(canvas,'wheel',{deltaY:100,deltaMode:0});assert(zooms.at(-1)<0);
dispatch(canvas,'keydown',{key:'+'});assert(zooms.at(-1)>0);dispatch(canvas,'keydown',{key:'-'});assert(zooms.at(-1)<0);dispatch(canvas,'keydown',{key:'Home'});assert.equal(resets,1);
dispatch(canvas,'pointerdown',{button:0,pointerId:1,clientX:100,clientY:100});
dispatch(canvas,'pointerdown',{button:0,pointerId:2,clientX:200,clientY:100});
dispatch(canvas,'pointermove',{pointerId:2,clientX:300,clientY:100});assert(zooms.at(-1)>0);
dispatch(canvas,'pointermove',{pointerId:2,clientX:150,clientY:100});assert(zooms.at(-1)<0);
dispatch(canvas,'pointerup',{pointerId:2,clientX:150,clientY:100});dispatch(canvas,'pointerup',{pointerId:1,clientX:100,clientY:100});assert.equal(picks.length,0);assert.equal(orbits.length,0);
dispatch(canvas,'pointerdown',{button:0,pointerId:3,clientX:20,clientY:30});dispatch(canvas,'pointerup',{pointerId:3,clientX:20,clientY:30});assert.deepEqual(picks,[[40,60]]);assert(canvas.focused);
dispatch(canvas,'pointerdown',{button:0,pointerId:4,clientX:20,clientY:30});dispatch(canvas,'pointercancel',{pointerId:4});dispatch(canvas,'pointermove',{pointerId:4,clientX:40,clientY:60});assert.equal(orbits.length,0);
const count=zooms.length;detach();dispatch(canvas,'wheel',{deltaY:-100,deltaMode:0});assert.equal(zooms.length,count);
console.log('Controls passed: wheel units/direction, keyboard, pinch both directions, no pick after pinch, cancel, cleanup');
