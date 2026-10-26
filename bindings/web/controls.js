/** Normalize DOM pixel/line/page wheel units into shared camera zoom steps. */
export function wheelZoomDelta(event, height = 800) {
  const unit=event.deltaMode===1 ? 16 : event.deltaMode===2 ? height : 1;
  return Number.isFinite(event.deltaY) ? Math.max(-8,Math.min(8,-event.deltaY*unit/50)) : 0;
}
/** Input can be tested without initializing WebGPU. Returns a listener cleanup. */
export function attachControls(canvas, {orbit, zoom, pick, reset}) {
  const cleanup=[];const points=new Map();let dragged=0;
  const listen=(name,fn,options)=>{canvas.addEventListener(name,fn,options);cleanup.push(()=>canvas.removeEventListener(name,fn,options));};
  const separation=()=>{const [a,b]=[...points.values()];return a && b ? Math.hypot(a.x-b.x,a.y-b.y) : 0;};
  canvas.tabIndex=0;
  listen('pointerdown',e=>{
    if(e.button!==0)return;
    if(points.size===0)dragged=0;else dragged=Infinity;
    points.set(e.pointerId,{x:e.clientX,y:e.clientY});
    canvas.setPointerCapture(e.pointerId);canvas.focus({preventScroll:true});
  });
  listen('pointermove',e=>{
    const previous=points.get(e.pointerId);if(!previous)return;
    const oldDistance=separation();const dx=e.clientX-previous.x,dy=e.clientY-previous.y;
    points.set(e.pointerId,{x:e.clientX,y:e.clientY});
    if(points.size>=2) {
      const distance=separation();if(oldDistance>0 && distance>0)zoom(Math.log(distance/oldDistance)*10);
      dragged=Infinity;
    } else {dragged+=Math.abs(dx)+Math.abs(dy);orbit(dx,dy);}
  });
  listen('pointerup',e=>{
    if(!points.has(e.pointerId))return;
    if(points.size===1 && dragged<4) {
      const rect=canvas.getBoundingClientRect();
      if(rect.width>0 && rect.height>0)pick(Math.floor((e.clientX-rect.left)*canvas.width/rect.width),Math.floor((e.clientY-rect.top)*canvas.height/rect.height));
    }
    points.delete(e.pointerId);
  });
  listen('pointercancel',e=>{points.delete(e.pointerId);dragged=Infinity;});
  listen('lostpointercapture',e=>{points.delete(e.pointerId);});
  listen('wheel',e=>{e.preventDefault();zoom(wheelZoomDelta(e,canvas.clientHeight || 800));},{passive:false});
  listen('keydown',e=>{
    if(e.key==='+' || e.key==='=') {e.preventDefault();zoom(2);}
    else if(e.key==='-' || e.key==='_') {e.preventDefault();zoom(-2);}
    else if(e.key==='Home') {e.preventDefault();reset();}
  });
  return ()=>{points.clear();for(const remove of cleanup)remove();};
}
