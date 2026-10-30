#!/usr/bin/env python3
"""Promote complete case contracts with matching source/data and real GPU/browser evidence."""
import hashlib,json,pathlib,re
root=pathlib.Path(__file__).resolve().parents[1]
manifest=json.loads((root/'tests/sandcastle/full-recipes.json').read_text())
browser=json.loads((root/'docs/full-recipes-browser.json').read_text())
gpu=(root/'docs/full-recipes-gpu-validation.log').read_text()
counts=re.findall(r"test result: ok\. (\d+) passed; 0 failed",gpu)
assert sum(map(int,counts))>=len(manifest['cases'])
assert "test wide_line_pixel_width_materials_and_picking ... ok" in gpu
assert len(browser['cases'])==len(manifest['cases'])
assert browser['destroy']=='passed'
for c in manifest['cases']:
    test_name = c['id'].replace('-', '_').replace('.', '_')
    assert f"test complete_{test_name} ... ok" in gpu
    b=next(b for b in browser['cases'] if b['id']==c['id'])
    source=(root/f"tests/sandcastle/sources/{c['id']}/main.js").read_bytes()
    data=(root/f"tests/sandcastle/full/{c['id']}.json").read_bytes()
    assert hashlib.sha256(source).hexdigest()==c['sourceSha256']==b['sourceSha256']
    assert hashlib.sha256(data).hexdigest()==b['fixtureSha256']
    actual_input=(root/'bindings/web'/c['path']).resolve().read_bytes()
    assert hashlib.sha256(actual_input).hexdigest()==b['inputSha256']
    for key in ['htmlSha256','assetSha256']:
        if key in c:assert c[key]==b[key]
    assert b['status']=='passed' and b['entities']==c['entityCount']
    smoke=(root/f"docs/full-{c['id']}-smoke.log").read_text()
    assert '90 frames' in smoke and 'errors 0' in smoke
    if c['format']=='variants':
        for i in range(1,c['variantCount']):
            extra=(root/f"docs/full-{c['id']}-variant-{i}-smoke.log").read_text();assert '90 frames' in extra and 'errors 0' in extra
    c['status']='functional-passed'
    c['validation']={'source':'passed','gpu':'passed','browser':'passed','nativeSmoke':'passed','pixelExactReference':'not-claimed','browserReport':'../../docs/full-recipes-browser.json','nativeScope':('local Natural Earth image files and shared globe' if c['id']=='offline' else 'shared globe/projection kernel only; browser verifies DOM and resolution') if c['format']=='scene' else 'actual source geometry and rendering'}
manifest['webSceneNativeScope']='HTML Overlays, Resolution Scaling and Hello World native smokes cover the shared globe/projection kernel; their DOM/DPR behavior is browser-only. Offline native smoke loads real local TMS images.'
manifest['acceptanceScope']='All case-specific source statements, inputs, geometry/entities, visibility groups, browser DOM/events/resolution controls and camera behavior in the fixed sources; not a pixel-exact Cesium renderer or implementation of all Viewer widgets.'
(root/'tests/sandcastle/full-recipes.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
print(f"Recorded {len(manifest['cases'])} fully implemented case contracts: source, actual GPU, browser WebGPU and native smoke passed.")
