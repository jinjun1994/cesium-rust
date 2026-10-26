#!/usr/bin/env python3
"""Run every actual full recipe input for 90 frames; requires a display and Vulkan/Metal/DX12."""
import argparse,json,os,pathlib,subprocess
root=pathlib.Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('--binary',default=str(root/'target/debug/desktop-viewer'));p.add_argument('--case',action='append',default=[]);args=p.parse_args()
for c in json.loads((root/'tests/sandcastle/full-recipes.json').read_text())['cases']:
    if args.case and c['id'] not in args.case:continue
    env=os.environ.copy()
    for key in ['CESIUM_ENTITIES','CESIUM_CZML','CESIUM_PRIMITIVES','CESIUM_GROUPS','CESIUM_SCENE','CESIUM_VARIANTS','CESIUM_VARIANT_INDEX','CESIUM_ZOOM_ENTITIES','CESIUM_VIEW_LOCATION']:env.pop(key,None)
    env.update(CESIUM_OFFLINE='1',CESIUM_TERRAIN='0')
    key={'entities':'CESIUM_ENTITIES','czml':'CESIUM_CZML','primitives':'CESIUM_PRIMITIVES','groups':'CESIUM_GROUPS','scene':'CESIUM_SCENE','variants':'CESIUM_VARIANTS'}[c['format']]
    env[key]=str((root/'bindings/web'/c['path']).resolve())
    if c['framing']=='zoomToEntities':env['CESIUM_ZOOM_ENTITIES']='1'
    else:env['CESIUM_VIEW_LOCATION']='-100,40,17500000'
    log=root/f"docs/full-{c['id']}-smoke.log"
    with log.open('w') as f:result=subprocess.run([args.binary,'--smoke-frames=90'],cwd=root,env=env,stdout=f,stderr=subprocess.STDOUT,timeout=60)
    text=log.read_text();assert result.returncode==0 and '90 frames' in text and 'errors 0' in text,(c['id'],text[-2000:])
    print(c['id'],'90 frames errors 0',flush=True)

    if c['format']=='variants':
        variants=json.loads(pathlib.Path(env[key]).read_text())['variants']
        for i in range(1,len(variants)):
            env['CESIUM_VARIANT_INDEX']=str(i)
            extra=root/f"docs/full-{c['id']}-variant-{i}-smoke.log"
            with extra.open('w') as f:result=subprocess.run([args.binary,'--smoke-frames=90'],cwd=root,env=env,stdout=f,stderr=subprocess.STDOUT,timeout=60)
            text=extra.read_text();assert result.returncode==0 and '90 frames' in text and 'errors 0' in text,(c['id'],i,text[-2000:])
            print(c['id'],'variant',i,'90 frames errors 0',flush=True)
