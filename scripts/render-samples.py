#!/usr/bin/env python3
"""GPU previews of isolated decoded contents; does not certify entire tilesets."""
import argparse, hashlib, json, pathlib, subprocess
ROOT=pathlib.Path(__file__).resolve().parents[1]
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--max-samples',type=int,default=16);a=p.parse_args()
 if not 1<=a.max_samples<=100:p.error('max-samples 1..100')
 report=json.loads((ROOT/'tests/samples/report.json').read_text());previews=ROOT/'tests/samples/previews';previews.mkdir(exist_ok=True)
 rows=[];seen=set()
 for r in report['results']:
  for check in r['contentChecks']:
   if check['status']=='passed' and check['result']['stage']=='content-decode' and check['id'] not in seen:
    seen.add(check['id']);key=hashlib.sha256(check['id'].encode()).hexdigest();rows.append(dict(id=check['id'],path=str(ROOT/'.sample-cache'/key),output=str(previews/(key+'.png')),preview='../../tests/samples/previews/'+key+'.png'))
 manifest=ROOT/'.sample-cache/render-manifest.json';manifest.write_text(json.dumps(rows[:a.max_samples]))
 process=subprocess.run([str(ROOT/'target/debug/sample-render-check'),str(manifest)],text=True,capture_output=True,check=True)
 results=[json.loads(s) for s in process.stdout.splitlines()]
 out=dict(commit=report['commit'],scope=__doc__,candidates=len(rows),checked=len(results),results=results)
 (ROOT/'tests/samples/gpu-report.json').write_text(json.dumps(out,indent=2));print(json.dumps({s:sum(r['status']==s for r in results) for s in ['passed','failed']}))
if __name__=='__main__':main()
