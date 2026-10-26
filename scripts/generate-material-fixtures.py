#!/usr/bin/env python3
"""Small deterministic GLBs testing all core texture channels; no external downloads."""
from pathlib import Path
import json,struct,io,base64
from PIL import Image
root=Path(__file__).resolve().parents[1]
data=(root/'tests/fixtures/triangle.glb').read_bytes();length=struct.unpack_from('<I',data,12)[0]
original=json.loads(data[20:20+length]);bin_offset=20+length;bin_length=struct.unpack_from('<I',data,bin_offset)[0];binary=data[bin_offset+8:bin_offset+8+bin_length]
def generate(uri=False):
 doc=json.loads(json.dumps(original));blob=bytearray(binary)
 def view(payload):
  while len(blob)%4:blob.append(0)
  offset=len(blob);blob.extend(payload);doc['bufferViews'].append({'buffer':0,'byteOffset':offset,'byteLength':len(payload)});return len(doc['bufferViews'])-1
 def attr(values,kind):
  n=len(values[0]);v=view(struct.pack('<'+'f'*(len(values)*n),*(x for row in values for x in row)))
  doc['accessors'].append({'bufferView':v,'componentType':5126,'count':len(values),'type':kind});return len(doc['accessors'])-1
 attrs=doc['meshes'][0]['primitives'][0]['attributes'];attrs['TEXCOORD_0']=attr([[.1,.1],[.9,.1],[.5,.9]],'VEC2');attrs['TEXCOORD_1']=attr([[.9,.9],[.1,.9],[.5,.1]],'VEC2');attrs['COLOR_0']=attr([[1,1,1,1]]*3,'VEC4');attrs['TANGENT']=attr([[1,0,0,1]]*3,'VEC4')
 doc['images']=[];doc['textures']=[];doc['samplers']=[{'magFilter':9728,'minFilter':9987,'wrapS':33648,'wrapT':33071}]
 for color in [(20,220,35,255),(0,180,0,255),(128,128,255,255),(120,0,0,255),(25,10,0,255)]:
  im=Image.new('RGBA',(4,4),color);b=io.BytesIO();im.save(b,format='PNG');png=b.getvalue()
  doc['images'].append({'uri':'data:image/png;base64,'+base64.b64encode(png).decode()} if uri else {'bufferView':view(png),'mimeType':'image/png'})
  doc['textures'].append({'source':len(doc['images'])-1,'sampler':0})
 mat=doc['materials'][0];mat['pbrMetallicRoughness']={'baseColorFactor':[1,1,1,1],'metallicFactor':1,'roughnessFactor':1,'baseColorTexture':{'index':0,'texCoord':1},'metallicRoughnessTexture':{'index':1}}
 mat.update(normalTexture={'index':2,'scale':.6},occlusionTexture={'index':3,'strength':.5},emissiveTexture={'index':4},emissiveFactor=[.3,.1,0],alphaMode='MASK',alphaCutoff=.5)
 doc['buffers'][0]['byteLength']=len(blob);j=json.dumps(doc,separators=(',',':')).encode();j+=b' '*(-len(j)%4);blob+=b'\0'*(-len(blob)%4)
 return b'glTF'+struct.pack('<II',2,12+8+len(j)+8+len(blob))+struct.pack('<I4s',len(j),b'JSON')+j+struct.pack('<I4s',len(blob),b'BIN\0')+blob
for name,uri in [('textured.glb',False),('textured-data-uri.glb',True)]:
 (root/'tests/fixtures'/name).write_bytes(generate(uri))
print('Material fixtures generated')
