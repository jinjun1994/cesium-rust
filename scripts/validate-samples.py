#!/usr/bin/env python3
"""Validate all pinned bertt tileset JSONs with production Rust parsers.
Content decoding is bounded to N direct contents per principal sample.
This report does not verify GPU rendering or optional extension semantics.
"""
import argparse, concurrent.futures, datetime, hashlib, json, pathlib, subprocess, urllib.request
ROOT=pathlib.Path(__file__).resolve().parents[1]
CACHE=ROOT/'.sample-cache'
def download(url,maximum=64*1024*1024):
    path=CACHE/hashlib.sha256(url.encode()).hexdigest()
    if path.exists() and path.stat().st_size<=maximum: return path
    with urllib.request.urlopen(urllib.request.Request(url,headers={'User-Agent':'cesium-rs-validator'}),timeout=30) as response: data=response.read(maximum+1)
    if len(data)>maximum: raise ValueError('Download byte limit exceeded')
    path.write_bytes(data);return path

def inspect(rows,executable):
    if not rows:return []
    p=subprocess.run([str(executable),'batch'],input=''.join(json.dumps(r)+'\n' for r in rows),text=True,capture_output=True,check=True)
    out=[json.loads(line) for line in p.stdout.splitlines()]
    if len(out)!=len(rows):raise RuntimeError('Inspector response count mismatch')
    return out

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--workers',type=int,default=24);p.add_argument('--content-count',type=int,default=2);p.add_argument('--inspector',type=pathlib.Path,default=ROOT/'target/debug/asset-inspector');a=p.parse_args()
    if not a.inspector.exists():p.error('Build first: cargo build --locked -p asset-inspector')
    if not 1<=a.workers<=64 or not 0<=a.content_count<=16:p.error('workers 1..64, content-count 0..16')
    catalog=json.loads((ROOT/'tests/samples/catalog.json').read_text());CACHE.mkdir(exist_ok=True);rows=[];results={};warnings={}
    def fetch(row):
        try:
            path=download(row['url'],8*1024*1024);value=json.loads(path.read_bytes());names=set()
            def walk(v):
                if isinstance(v,dict):
                    if isinstance(v.get('extensions'),dict):names.update(v['extensions'])
                    for z in v.values():walk(z)
                elif isinstance(v,list):
                    for z in v:walk(z)
            walk(value);return dict(id=row['id'],path=str(path),url=row['url']),sorted(names),None
        except Exception as e:return None,[],str(e)
    with concurrent.futures.ThreadPoolExecutor(a.workers) as pool:
        tasks={pool.submit(fetch,r):r for r in catalog['samples']}
        for n,future in enumerate(concurrent.futures.as_completed(tasks),1):
            row=tasks[future];local,optional,error=future.result();warnings[row['id']]=optional
            if local:rows.append(local)
            else:results[row['id']]={'id':row['id'],'status':'download-failed','error':error}
            if n%250==0:print(f'Downloaded {n}/{len(tasks)}',flush=True)
    for r in inspect(rows,a.inspector):results[r['id']]=r
    urls=[];owners={}
    for row in catalog['samples']:
        r=results[row['id']];r.update(optionalExtensions=warnings.get(row['id'],[]),renderStatus='not-tested',contentCoverage='not-tested',contentChecks=[])
        if not row['subtree'] and r['status']=='passed':
            contents=r['result']['contents'];r['contentCoverage']=f'first {min(a.content_count,len(contents))} of {len(contents)} direct contents; external trees not traversed'
            for url in contents[:a.content_count]:
                if url not in owners:urls.append(url);owners[url]=[]
                owners[url].append(row['id'])
    def fetch_content(url):
        try:return dict(id=url,path=str(download(url)),url=url),None
        except Exception as e:return None,dict(id=url,status='download-failed',error=str(e))
    checks=[];pending=[]
    with concurrent.futures.ThreadPoolExecutor(min(a.workers,8)) as pool:
        for local,error in pool.map(fetch_content,urls):
            if local:pending.append(local)
            else:checks.append(error)
    checks+=inspect(pending,a.inspector)
    for check in checks:
        for id in owners[check['id']]:results[id]['contentChecks'].append(check)
    summary={}
    for r in results.values():summary[r['status']]=summary.get(r['status'],0)+1
    report=dict(repository=catalog['repository'],commit=catalog['commit'],generatedAt=datetime.datetime.now(datetime.timezone.utc).isoformat(),scope=__doc__,summary=summary,contentSummary={s:sum(c['status']==s for c in checks) for s in ('passed','failed','download-failed')},results=[results[r['id']] for r in catalog['samples']])
    (ROOT/'tests/samples/report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2));print(json.dumps({'tilesets':summary,'contents':report['contentSummary']}),flush=True)
if __name__=='__main__':main()
