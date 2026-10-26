// Run with npm run test:web after scripts/build-web.py.
const fs = require('node:fs');
const path = require('node:path');
const http = require('node:http');
const assert = require('node:assert/strict');
const {chromium} = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const root = path.resolve(__dirname, '..');
const mime = {'.html':'text/html','.js':'text/javascript','.wasm':'application/wasm','.json':'application/json'};
(async () => {
  const server = http.createServer((request,response) => {
    const url = new URL(request.url, 'http://localhost');
    let file = path.resolve(root, '.' + decodeURIComponent(url.pathname));
    if (!file.startsWith(root + path.sep)) {response.writeHead(403).end();return;}
    if (fs.existsSync(file) && fs.statSync(file).isDirectory()) file=path.join(file,'index.html');
    fs.readFile(file,(error,bytes) => {
      if (error) {response.writeHead(404).end();return;}
      response.writeHead(200,{'content-type':mime[path.extname(file)] || 'application/octet-stream'});response.end(bytes);
    });
  });
  await new Promise(resolve => server.listen(0,'127.0.0.1',resolve));
  let browser;
  try {
    browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH || undefined,headless:true,
      args:['--no-sandbox','--no-zygote','--enable-unsafe-webgpu','--enable-unsafe-swiftshader','--use-angle=swiftshader']});
    const page=await browser.newPage({viewport:{width:1280,height:800},deviceScaleFactor:1});
    const errors=[];page.on('pageerror',error=>errors.push(error.message));
    if(process.env.DEBUG_WEB) page.on('console',message=>console.log('browser:',message.type(),message.text()));
    await page.goto(`http://127.0.0.1:${server.address().port}/bindings/web/`);
    try { await page.waitForFunction(() => window.cesiumViewer && document.getElementById('status').textContent.includes('Earth yes'),null,{timeout:15000}); } catch(error) { console.error(await page.evaluate(() => ({status:document.getElementById('status').textContent,error:document.getElementById('error').textContent,gpu:!!navigator.gpu})),errors); throw error; }
    const base=await page.evaluate(() => location.origin);
    await page.evaluate(url => window.cesiumViewer.load(url),base+'/tests/fixtures/tileset.json');
    await page.waitForFunction(() => document.getElementById('status').textContent.includes('resident 1'),null,{timeout:15000});
    const center=async()=>{const b=await page.locator('canvas').boundingBox();await page.mouse.click(b.x+b.width/2,b.y+b.height/2);};await center();
    try {await page.waitForFunction(() => document.getElementById('properties').textContent.includes('fixture triangle'),null,{timeout:10000});} catch(error) {console.error(await page.evaluate(() => ({properties:document.getElementById('properties').textContent,error:document.getElementById('error').textContent,status:document.getElementById('status').textContent}))); if(process.env.DEBUG_SCREENSHOT) await page.screenshot({path:process.env.DEBUG_SCREENSHOT});throw error;}
    await page.evaluate(url => window.cesiumViewer.load(url),base+'/tests/fixtures/tileset.json');
    await page.waitForFunction(() => document.getElementById('status').textContent.includes('resident 1'),null,{timeout:15000});
    await page.setViewportSize({width:1024,height:768});
    await page.waitForFunction(() => document.querySelector('canvas').width===Math.round(document.querySelector('canvas').getBoundingClientRect().width));
    await center();
    await page.waitForFunction(() => document.getElementById('properties').textContent.includes('fixture triangle'));
    const properties=await page.locator('#properties').textContent();
    assert.equal(JSON.parse(properties).properties.name,'fixture triangle');
    await page.evaluate(url => window.cesiumViewer.load(url),base+'/tests/fixtures/triangle.glb');
    await page.waitForFunction(() => document.getElementById('status').textContent.includes('resident 1'));
    assert.equal(await page.locator('#error').textContent(),'');assert.deepEqual(errors,[]);
    await page.evaluate(() => window.cesiumViewer.destroy());
    assert.equal(await page.evaluate(() => window.cesiumViewer.stats),'destroyed');
    await page.goto(base+'/bindings/web/sandcastle.html');
    await page.waitForFunction(()=>!document.getElementById('shapes').disabled,null,{timeout:15000});
    assert.equal(await page.locator('#cases button').count(),309);
    await page.locator('#shapes').click();
    await page.waitForFunction(()=>document.getElementById('status').textContent.includes('Entities 5'));
    await center();await page.waitForFunction(()=>document.getElementById('pick').textContent.includes('ellipsoid'));
    await page.locator('#motion').click();
    await page.waitForFunction(()=>document.getElementById('status').textContent.includes('Entities 1'));
    await page.locator('#pause').click();
    await page.locator('#seek').evaluate(e=>{e.value='30';e.dispatchEvent(new Event('input'));});
    await page.waitForFunction(()=>document.getElementById('time').textContent==='30.00s');
    await page.locator('#play').click();
    await page.waitForFunction(()=>parseFloat(document.getElementById('time').textContent)>30.1);
    assert.equal(await page.locator('#error').textContent(),'');assert.deepEqual(errors,[]);
    await page.locator('#geographic').click();
    await page.waitForFunction(()=>document.getElementById('status').textContent.includes('Entities 6'));
    await page.locator('#entitySelect').selectOption('ellipse');await page.locator('#focusEntity').click();
    await center();await page.waitForFunction(()=>document.getElementById('pick').textContent.includes('ellipse'));
    await page.locator('#hideEntity').click();await center();await page.waitForFunction(()=>document.getElementById('pick').textContent==='null');
    await page.locator('#showEntity').click();await center();await page.waitForFunction(()=>document.getElementById('pick').textContent.includes('ellipse'));
    await page.locator('#geographicCzml').click();
    await page.waitForFunction(()=>document.getElementById('status').textContent.includes('Entities 4'));
    assert.equal(await page.locator('#error').textContent(),'');
    await page.locator('#materials').click();
    await page.waitForFunction(()=>document.getElementById('status').textContent.includes('Entities 5'));
    await page.locator('#materialsCzml').click();
    await page.waitForFunction(()=>document.getElementById('editor').value.includes('checkerboard'));
    await page.locator('#imageOutline').click();
    await page.waitForFunction(()=>document.getElementById('status').textContent.includes('Entities 5'));
    await page.locator('#imageOutlineCzml').click();
    await page.waitForFunction(()=>document.getElementById('status').textContent.includes('Entities 2'));
    assert.equal(await page.locator('#error').textContent(),'');
    await page.locator('#screen').click();
    await page.waitForFunction(()=>document.getElementById('status').textContent.includes('Entities 3'));
    await page.locator('#entitySelect').selectOption('point');await page.locator('#focusEntity').click();await center();
    await page.waitForFunction(()=>document.getElementById('pick').textContent.includes('point'));
    await page.locator('#screenCzml').click();
    await page.waitForFunction(()=>document.getElementById('status').textContent.includes('Entities 3'));
    await page.locator('#labels').click();
    await page.waitForFunction(()=>document.getElementById('status').textContent.includes('Entities 3'));
    await page.locator('#distanceDisplay').click();
    await page.waitForFunction(()=>document.getElementById('editor').value.includes('distanceDisplayCondition'));
    await page.locator('#distanceNear').click();await center();
    await page.waitForFunction(()=>document.getElementById('pick').textContent.includes('distance-display'));
    await page.locator('#distanceMid').click();await center();
    await page.waitForFunction(()=>document.getElementById('pick').textContent==='null');
    await page.locator('#distanceScale').click();
    await page.waitForFunction(()=>document.getElementById('editor').value.includes('scaleByDistance'));
    await page.locator('#distanceAlpha').click();
    await page.waitForFunction(()=>document.getElementById('editor').value.includes('translucencyByDistance'));
    await page.locator('#screenOrigins').click();
    await page.waitForFunction(()=>document.getElementById('editor').value.includes('horizontalOrigin'));
    await page.locator('#labelDistanceCzml').click();
    await page.waitForFunction(()=>document.getElementById('status').textContent.includes('Entities 5'));
    for(const [button,property]of [['labelBackground','showBackground'],['labelOutline','FILL_AND_OUTLINE'],['billboardRotation','rotation'],['pixelOffsetDistance','pixelOffsetScaleByDistance'],['depthTestDistance','disableDepthTestDistance']]) {
      await page.locator('#'+button).click();
      await page.waitForFunction(property=>document.getElementById('editor').value.includes(property),property);
      assert.equal(await page.locator('#error').textContent(),'');
    }
    await page.locator('#screenEffectsCzml').click();
    await page.waitForFunction(()=>document.getElementById('status').textContent.includes('Entities 6'));
    for(const [button,mode] of [['cameraSetView','相机 view'],['cameraFlyTo','相机 flying'],['cameraTrack','相机 tracking'],['cameraLookAt','相机 lookAt']]) {
      await page.locator('#'+button).click();
      await page.waitForFunction(mode=>document.getElementById('cameraStatus').textContent.includes(mode),mode);
      assert.equal(await page.locator('#error').textContent(),'');
      if(button==='cameraFlyTo') {await page.locator('#cameraCancel').click();await page.waitForFunction(()=>document.getElementById('cameraStatus').textContent.includes('cancelled'));}
      if(button==='cameraTrack') {await page.locator('#cameraStop').click();await page.waitForFunction(()=>document.getElementById('cameraStatus').textContent.includes('跟随 无'));}
    }
    await page.locator('#originalWall').click();
    await page.waitForFunction(()=>document.getElementById('status').textContent.includes('Entities 1'));
    assert.equal(await page.locator('#error').textContent(),'');
    console.log('browser smoke passed: WebGPU init, metadata pick, HTTP tileset/GLB, resize/destroy, Sandcastle entity rendering/pick and CZML clock');
  } finally {if(browser) await browser.close();await new Promise(resolve=>server.close(resolve));}
})().catch(error=>{console.error(error);process.exitCode=1;});
