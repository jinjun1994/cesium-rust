struct GlobalUniform { projection:mat4x4<f32>, sun_direction_eye:vec4<f32>, viewport_exposure:vec4<f32> };
struct ObjectUniform { model_view_relative:mat4x4<f32>, normal_col0:vec4<f32>, normal_col1:vec4<f32>, normal_col2:vec4<f32>, feature_params:vec4<u32>, screen_params:vec4<f32> };
struct MaterialUniform {base_color:vec4<f32>,factors:vec4<f32>,emissive_normal:vec4<f32>,params:vec4<f32>,uv_sets:vec4<u32>,extras:vec4<u32>,secondary:vec4<f32>,pattern:vec4<f32>};
@group(0) @binding(0) var<uniform> global:GlobalUniform;
@group(1) @binding(0) var<uniform> object:ObjectUniform;
@group(2) @binding(0) var<uniform> material:MaterialUniform;
@group(2) @binding(1) var base_color_texture:texture_2d<f32>;
@group(2) @binding(2) var color_sampler:sampler;
@group(2) @binding(3) var mr_texture:texture_2d<f32>;
@group(2) @binding(4) var mr_sampler:sampler;
@group(2) @binding(5) var normal_texture:texture_2d<f32>;
@group(2) @binding(6) var normal_sampler:sampler;
@group(2) @binding(7) var occlusion_texture:texture_2d<f32>;
@group(2) @binding(8) var occlusion_sampler:sampler;
@group(2) @binding(9) var emissive_texture:texture_2d<f32>;
@group(2) @binding(10) var emissive_sampler:sampler;
struct VertexInput {
 @location(0) position:vec3<f32>,@location(1) normal:vec3<f32>,@location(2) uv:vec2<f32>,@location(3) feature_id:u32,
 @location(4) uv1:vec2<f32>,@location(5) color:vec4<f32>,@location(6) tangent:vec4<f32>,
};
struct VertexOutput {
 @builtin(position) position:vec4<f32>,@location(0) normal:vec3<f32>,@location(1) eye_position:vec3<f32>,
 @location(2) @interpolate(flat) feature_id:u32,@location(3) uv:vec2<f32>,@location(4) uv1:vec2<f32>,@location(5) color:vec4<f32>,@location(6) tangent:vec4<f32>,@location(7) @interpolate(flat) line:vec4<f32>,
};
fn vertex_main(v:VertexInput)->VertexOutput {
 var out:VertexOutput;var eye=object.model_view_relative*vec4<f32>(v.position,1.0);
 if object.feature_params.w==2u {eye=object.model_view_relative*vec4<f32>(0.0,0.0,0.0,1.0);}
 out.position=global.projection*eye;
 if object.feature_params.w==2u {
  let local=(v.position.xy-object.screen_params.xy)*object.normal_col0.w;
  let rotated=vec2<f32>(local.x*object.screen_params.z-local.y*object.screen_params.w,local.x*object.screen_params.w+local.y*object.screen_params.z);
  out.position=vec4<f32>(out.position.xy+(rotated+object.screen_params.xy*object.normal_col2.w)*2.0/global.viewport_exposure.xy*out.position.w,out.position.zw);
 }
 // WebGPU disallows fixed depth bias for lines. A small relative Reverse-Z offset
 // stabilizes coplanar outlines while preserving normal scene occlusion.
 if object.feature_params.w==1u {out.position.z=min(out.position.z*1.000001,out.position.w);}
 out.line=vec4<f32>(0.0);
 if object.feature_params.w==3u {
  let a=global.projection*object.model_view_relative*vec4<f32>(v.normal,1.0);
  let b=global.projection*object.model_view_relative*vec4<f32>(v.tangent.xyz,1.0);
  let center=out.position.xy/max(out.position.w,0.0001)*global.viewport_exposure.xy;
  let before=a.xy/max(a.w,0.0001)*global.viewport_exposure.xy;
  let after=b.xy/max(b.w,0.0001)*global.viewport_exposure.xy;
  let d0=normalize(center-before+vec2<f32>(0.000001,0.0));
  let d1=normalize(after-center+vec2<f32>(0.000001,0.0));
  let n0=vec2<f32>(-d0.y,d0.x);let n1=vec2<f32>(-d1.y,d1.x);
  let m=normalize(n0+n1+vec2<f32>(0.000001,0.0));
  let scale=min(4.0,1.0/max(abs(dot(m,n1)),0.25));
  out.position=vec4<f32>(out.position.xy+m*scale*v.uv1.x*v.uv1.y/global.viewport_exposure.xy*out.position.w,out.position.zw);
  out.line=vec4<f32>(d1.x,-d1.y,v.uv1.x,0.0);
 }
 out.eye_position=eye.xyz;
 out.normal=mat3x3<f32>(object.normal_col0.xyz,object.normal_col1.xyz,object.normal_col2.xyz)*v.normal;
 let sign=select(-1.0,1.0,determinant(object.model_view_relative)>0.0);
 out.tangent=vec4<f32>((object.model_view_relative*vec4<f32>(v.tangent.xyz,0.0)).xyz,v.tangent.w*sign);
 out.uv=v.uv;out.uv1=v.uv1;out.color=v.color;
 if object.feature_params.w==2u {out.color.a*=object.normal_col1.w;}
 out.feature_id=v.feature_id;return out;
}
@vertex fn vs_main(v:VertexInput)->VertexOutput { return vertex_main(v); }
// WebGPU point-list primitives have fixed size; expand each point as an instanced quad.
@vertex fn vs_point(v:VertexInput,@builtin(vertex_index) index:u32)->VertexOutput {
 var out=vertex_main(v);
 // WebGL clips a point by its center before expanding its screen-space footprint.
 if out.position.w<=0.0 || abs(out.position.x)>out.position.w || abs(out.position.y)>out.position.w || out.position.z<0.0 || out.position.z>out.position.w {
  out.position=vec4<f32>(2.0,2.0,0.0,1.0);
  return out;
 }
 // Cesium keeps points with eye-space normal.z >= 0; without normals culling is disabled.
 if (u32(object.screen_params.w) & 2u)!=0u && (material.extras.y & 8u)!=0u && out.normal.z<0.0 {
  out.position=vec4<f32>(2.0,2.0,0.0,1.0);
  return out;
 }
 let corners=array<vec2<f32>,6>(vec2<f32>(-1.0,-1.0),vec2<f32>(1.0,-1.0),vec2<f32>(1.0,1.0),vec2<f32>(-1.0,-1.0),vec2<f32>(1.0,1.0),vec2<f32>(-1.0,1.0));
 var size=object.screen_params.x;
 if object.screen_params.y>=0.0 {
  let depth=max(-out.eye_position.z,0.000001);
  let multiplier=global.viewport_exposure.y*global.projection[1][1]*0.5;
  size=clamp(object.screen_params.y*multiplier/depth,1.0,object.screen_params.z);
 }
 out.position=vec4<f32>(out.position.xy+corners[index]*size/global.viewport_exposure.xy*out.position.w,out.position.zw);
 return out;
}
fn tex_uv(v:VertexOutput,index:u32)->vec2<f32> {return select(v.uv,v.uv1,index==1u);}
fn base_color(v:VertexOutput)->vec4<f32> {
 let sample_uv=tex_uv(v,material.uv_sets.x)*select(vec2<f32>(1.0),material.pattern.xy,material.extras.w==4u);
 var texel=textureSample(base_color_texture,color_sampler,sample_uv);
 if material.extras.w==4u && material.params.w<0.5 {texel.a=1.0;}
 let uv=v.uv;var color=material.base_color;
 if material.extras.w==1u {
  let coord=select(uv.y,uv.x,material.pattern.w>0.5);
  let phase=fract((coord-material.pattern.z)*material.pattern.x*0.5);
  color=select(material.base_color,material.secondary,phase>=0.5);
 } else if material.extras.w==2u {
  let tile=floor(uv*material.pattern.xy);let odd=(i32(tile.x+tile.y)&1)==1;
  color=select(material.base_color,material.secondary,odd);
 } else if material.extras.w==3u {
  let grid=uv*material.pattern.xy;let distance=min(fract(grid),1.0-fract(grid));
  let width=max(fwidth(grid),vec2<f32>(0.00001))*material.params.yz*0.5;
  let lines=(1.0-smoothstep(width,width+max(fwidth(grid),vec2<f32>(0.00001)),distance))*select(vec2<f32>(0.0),vec2<f32>(1.0),material.params.yz>vec2<f32>(0.0));
  color.a*=mix(material.params.w,1.0,max(lines.x,lines.y));
 }
 if material.extras.w==6u {
  let b=smoothstep(0.3,0.32,length(fract(uv*material.pattern.xy)-vec2<f32>(0.5)));
  color=mix(material.base_color,material.secondary,b);
 }
 if material.extras.w==5u {
  let radius=length((uv-vec2<f32>(0.5))*2.0);
  color=select(color,material.secondary,radius>material.pattern.x);
  if radius>1.0 {color.a=0.0;}
 }
 if material.extras.w==7u {
  var glow=material.pattern.x/max(abs(uv.y-0.5),0.00001)-material.pattern.x/0.5;
  if material.pattern.y<=0.99999 {glow*=min(1.0,material.pattern.y/max(0.5-uv.x*0.5,0.00001)-material.pattern.y/0.5);}
  color=vec4<f32>(max(vec3<f32>(glow-1.0)+color.rgb,color.rgb),clamp(glow,0.0,1.0)*color.a);
 } else if material.extras.w==8u {
  let base=1.0-abs(fwidth(uv.x))*10.0;
  let half=select(0.15,0.5*(1.0-uv.x)/max(1.0-base,0.00001),uv.x>=base);
  color.a*=1.0-smoothstep(half-max(fwidth(uv.y),0.0001),half+max(fwidth(uv.y),0.0001),abs(uv.y-0.5));
 } else if material.extras.w==9u {
  let half=0.5*(material.pattern.x-material.pattern.y)/material.pattern.x;
  let mixv=smoothstep(half-max(fwidth(uv.y),0.0001),half+max(fwidth(uv.y),0.0001),abs(uv.y-0.5));
  color=mix(color,material.secondary,mixv);
 } else if material.extras.w==10u {
  let along=dot(v.position.xy,v.line.xy);
  let bit=u32(floor(fract(along/material.pattern.x)*16.0));
  color=select(material.secondary,color,(u32(material.pattern.y)&(1u<<bit))!=0u);
 } else if material.extras.w==11u {
  color=mix(color,material.secondary,clamp(uv.x,0.0,1.0));
 }
 return texel*color*v.color;
}
fn surface_normal(v:VertexOutput,sample:vec3<f32>)->vec3<f32> {
 let n=normalize(v.normal);let uv=tex_uv(v,material.uv_sets.z);
 let p1=dpdx(v.eye_position);let p2=dpdy(v.eye_position);let u1=dpdx(uv);let u2=dpdy(uv);
 if (material.extras.y & 1u)==0u {return n;}
 var tangent=v.tangent.xyz;var handedness=v.tangent.w;
 if length(tangent)<0.000001 || material.uv_sets.z!=0u {
  let det=u1.x*u2.y-u1.y*u2.x;
  if abs(det)<0.00000001 {return n;}
  tangent=(p1*u2.y-p2*u1.y)/det;
  let bitangent=(p2*u1.x-p1*u2.x)/det;
  handedness=select(-1.0,1.0,dot(cross(n,tangent),bitangent)>=0.0);
 }
 tangent=tangent-n*dot(n,tangent);
 if length(tangent)<0.000001 {return n;}
 let t=normalize(tangent);let b=cross(n,t)*handedness;
 let mapped=vec3<f32>(sample.xy*material.emissive_normal.w,sample.z);
 if length(mapped)<0.000001 {return n;}
 return normalize(mat3x3<f32>(t,b,n)*mapped);
}
@fragment fn fs_color(v:VertexOutput,@builtin(front_facing) front:bool)->@location(0) vec4<f32> {
 let base=base_color(v);
 if (object.feature_params.w==2u || object.feature_params.w==3u) && base.a<0.0039215686 {discard;}
 if material.extras.z!=0u {
  if material.factors.z>=0.0 && base.a<material.factors.z {discard;}
  var color=base.rgb;
  if (material.extras.y & 8u)!=0u && (u32(object.screen_params.w) & 1u)!=0u {
   color*=max(dot(normalize(v.normal),normalize(global.sun_direction_eye.xyz)),0.4);
  }
  if object.feature_params.y==v.feature_id {color=mix(color,vec3<f32>(1.0,0.7,0.05),0.7);}
  return vec4<f32>(color*global.viewport_exposure.z,select(1.0,base.a,material.factors.z>=0.0 || material.factors.z== -2.0));
 }
 let mr=textureSample(mr_texture,mr_sampler,tex_uv(v,material.uv_sets.y));
 let norm=textureSample(normal_texture,normal_sampler,tex_uv(v,material.uv_sets.z)).xyz*2.0-1.0;
 let occ=textureSample(occlusion_texture,occlusion_sampler,tex_uv(v,material.uv_sets.w)).r;
 let emissive=textureSample(emissive_texture,emissive_sampler,tex_uv(v,material.extras.x)).rgb*material.emissive_normal.rgb;
 let n=surface_normal(v,norm)*select(1.0,select(-1.0,1.0,front),(material.extras.y & 2u)!=0u);
 if (material.extras.y & 4u)!=0u {
  let diffuse=max(dot(n,normalize(global.sun_direction_eye.xyz)),0.0);
  var color=base.rgb*(0.2+diffuse);
  if object.feature_params.y==v.feature_id {color=mix(color,vec3<f32>(1.0,0.7,0.05),0.7);}
  return vec4<f32>(color*global.viewport_exposure.z,select(1.0,base.a,material.factors.z== -2.0));
 }
 if material.factors.z>=0.0 && base.a<material.factors.z {discard;}
 let l=normalize(global.sun_direction_eye.xyz);let view=normalize(-v.eye_position);let h=normalize(l+view);
 let nl=max(dot(n,l),0.0);let nv=max(dot(n,view),0.001);let nh=max(dot(n,h),0.0);let vh=max(dot(view,h),0.0);
 let rough=clamp(material.factors.y*mr.g,0.045,1.0);let metal=clamp(material.factors.x*mr.b,0.0,1.0);
 let a2=rough*rough*rough*rough;let denom=nh*nh*(a2-1.0)+1.0;
 let distribution=a2/max(3.14159265*denom*denom,0.00001);let k=(rough+1.0)*(rough+1.0)/8.0;
 let visibility=(nl/(nl*(1.0-k)+k))*(nv/(nv*(1.0-k)+k));
 let f0=mix(vec3<f32>(0.04),base.rgb,vec3<f32>(metal));let fresnel=f0+(vec3<f32>(1.0)-f0)*pow(1.0-vh,5.0);
 let specular=distribution*visibility*fresnel/max(4.0*nl*nv,0.0001);
 let diffuse=(vec3<f32>(1.0)-fresnel)*(1.0-metal)*base.rgb/3.14159265;
 let ao=mix(1.0,occ,material.params.x);
 var color: vec3<f32>;
 if material.factors.w>0.5 {
  let fresnel=1.0-nv;
  let haze=vec3<f32>(0.06,0.16,0.34)*pow(fresnel,1.6)+vec3<f32>(0.18,0.45,0.88)*pow(fresnel,4.0);
  color=base.rgb*1.15+haze+emissive;
 } else {
  color=(diffuse+specular)*nl*2.0+base.rgb*0.35*ao+emissive;
 }
 if object.feature_params.y==v.feature_id {color=mix(color,vec3<f32>(1.0,0.7,0.05),0.7);}
 return vec4<f32>(color*global.viewport_exposure.z,select(1.0,base.a,material.factors.z>=0.0 || material.factors.z== -2.0));
}
@fragment fn fs_pick(v:VertexOutput)->@location(0) u32 {
 let base=base_color(v);if (object.feature_params.w==2u || object.feature_params.w==3u) && base.a<0.0039215686 {discard;}if material.factors.z== -2.0 && base.a<=0.0 {discard;}if material.factors.z>=0.0 && base.a<material.factors.z {discard;}
 if object.feature_params.z==0u {return 0u;}return object.feature_params.x+v.feature_id;
}
