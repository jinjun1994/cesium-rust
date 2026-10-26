#!/usr/bin/env python3
"""Pin the upstream gallery inventory; API mentions are evidence, never passing tests."""
import argparse, concurrent.futures, hashlib, json, pathlib, re, time, urllib.request
ROOT = pathlib.Path(__file__).resolve().parents[1]
def fetch(url):
    for attempt in range(4):
        try:
            with urllib.request.urlopen(urllib.request.Request(url, headers={'User-Agent':'cesium-rs-gallery-inventory'}), timeout=25) as r:
                data=r.read(1024*1024+1)
                if len(data)>1024*1024: raise ValueError('source size limit')
                return data
        except Exception:
            if attempt==3: raise
            time.sleep(attempt+1)
def run():
    p=argparse.ArgumentParser();p.add_argument('--tree',type=pathlib.Path);p.add_argument('--commit',required=True);a=p.parse_args()
    if not re.fullmatch(r'[0-9a-f]{40}',a.commit): raise ValueError('full pinned commit SHA required')
    if a.tree: tree=json.loads(a.tree.read_text())
    else:
        package=json.loads(fetch('https://api.github.com/repos/CesiumGS/cesium/contents/packages/sandcastle?ref='+a.commit))
        gallery=next(e for e in package if e['name']=='gallery' and e['type']=='dir')
        tree=json.loads(fetch('https://api.github.com/repos/CesiumGS/cesium/git/trees/'+gallery['sha']+'?recursive=1'))
    if tree.get('truncated'): raise ValueError('truncated gallery tree')
    entries=tree['tree'];prefix=''
    ids=sorted(e['path'][len(prefix):].rsplit('/',1)[0] for e in entries if e['path'].startswith(prefix) and e['path'].endswith('/sandcastle.yaml'))
    out=ROOT/'tests/sandcastle';out.mkdir(parents=True,exist_ok=True)
    def case(id):
        base=f'https://raw.githubusercontent.com/CesiumGS/cesium/{a.commit}/packages/sandcastle/gallery/{id}/'
        yaml=fetch(base+'sandcastle.yaml').decode();source=fetch(base+'main.js').decode()
        def scalar(key):
            m=re.search(r'^'+key+r':\s*(.*)$',yaml,re.M);return m.group(1).strip('"\'') if m else id
        labels=re.findall(r'^\s+-\s+(.+)$',yaml,re.M)
        api=sorted(set(re.findall(r'\bCesium\.([A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*)',source)))
        # Never ship upstream evaluation credentials. Original content digest remains available.
        sanitized=re.sub(r'\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+', '[REDACTED_PUBLIC_TOKEN]',source)
        sanitized=re.sub(r'AIza[A-Za-z0-9_-]{30,}', '[REDACTED_PUBLIC_KEY]',sanitized)
        folder=out/'sources'/id;folder.mkdir(parents=True,exist_ok=True)
        (folder/'main.js').write_text(sanitized);(folder/'sandcastle.yaml').write_text(yaml)
        return dict(id=id,title=scalar('title'),description=scalar('description'),labels=labels,apiMentions=api,
            status='not-verified',reason='Upstream recipe is not executed by the Rust engine. API mentions do not establish compatibility.',
            referenceUrl='https://sandcastle.cesium.com/?id='+id,sourceUrl=base+'main.js',sourceSha256=hashlib.sha256(source.encode()).hexdigest(),sourceRedacted=sanitized!=source)
    cases=[]
    with concurrent.futures.ThreadPoolExecutor(max_workers=20) as pool:
        for f in concurrent.futures.as_completed([pool.submit(case,id) for id in ids]):
            cases.append(f.result())
            if len(cases)%50==0: print(f'{len(cases)}/{len(ids)} fetched',flush=True)
    catalog=dict(schemaVersion=1,release='1.146',commit=a.commit,caseCount=len(cases),scope='All gallery metadata entries at this pinned release, including -dev recipes',coverageRule='Only explicit rendering and interaction evidence can mark a full recipe passed.',cases=sorted(cases,key=lambda x:x['id']))
    (out/'catalog.json').write_text(json.dumps(catalog,ensure_ascii=False,indent=2)+'\n');print('Saved',len(cases),'recipes')
if __name__=='__main__':run()
