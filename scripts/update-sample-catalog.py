#!/usr/bin/env python3
"""Refresh the complete bertt tileset inventory and pin URLs to the resolved commit."""
import argparse,json,pathlib,urllib.request
ROOT=pathlib.Path(__file__).resolve().parents[1]
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--ref',default='master');a=p.parse_args()
 url='https://api.github.com/repos/bertt/cesium_3dtiles_samples/git/trees/'+urllib.parse.quote(a.ref,safe='')+'?recursive=1'
 with urllib.request.urlopen(urllib.request.Request(url,headers={'User-Agent':'cesium-rs-validator'}),timeout=30) as r:data=r.read(32*1024*1024+1)
 if len(data)>32*1024*1024:raise RuntimeError('Repository tree exceeds byte limit')
 tree=json.loads(data)
 if tree.get('truncated'):raise RuntimeError('GitHub returned an incomplete tree; catalog was not changed')
 rows=[]
 for item in tree['tree']:
  path=item['path']
  if item['type']=='blob' and path.lower().endswith('.json') and 'tileset' in path.lower():rows.append(dict(id=path,path=path,url=f'https://raw.githubusercontent.com/bertt/cesium_3dtiles_samples/{tree["sha"]}/{path}',size=item['size'],subtree='/pointcloud/' in path and path!='samples/pointcloud/schiphol/3dtiles/schiphol-colored-3857/tileset.json'))
 target=ROOT/'tests/samples/catalog.json';target.parent.mkdir(parents=True,exist_ok=True);target.write_text(json.dumps(dict(repository='https://github.com/bertt/cesium_3dtiles_samples',commit=tree['sha'],samples=rows),ensure_ascii=False,indent=2));print(f'{len(rows)} tileset JSONs pinned to {tree["sha"]}; rerun validation for this commit')
