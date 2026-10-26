// Production SDK source with only the WASM GPU boundary replaced; real Response streams.
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import { pathToFileURL } from "node:url";
class Canvas extends EventTarget {
  width = 800;
  height = 600;
  clientHeight = 600;
  getBoundingClientRect() {
    return { width: 800, height: 600, left: 0, top: 0 };
  }
  setPointerCapture() {}
  focus() {}
}
let lastOptions;
let phase = "public",
  requested = false,
  submits = 0,
  cleared = 0,
  layerSets = 0,
  failed = 0;
const raw = {
  advanceTime(){},renderFrame() {},
  resize() {},
  orbit() {},
  zoom() {},
  focusGlobe() {},
  cancelRequests() {},
  cancelTerrainRequests() {},
  setImageryTemplate() {},
  takeImageryRequests() {
    return "[]";
  },
  takeRequests() {
    return "[]";
  },
  setPublicTerrain() {
    phase = "public";
  },
  setTerrariumTerrain() {
    phase = "public";
  },
  setTerrainLayer() {
    phase = "new";
    layerSets++;
  },
  takeTerrainRequests() {
    if (phase === "public" && !requested) {
      requested = true;
      return JSON.stringify([
        {
          id: "1/1/0",
          url: "http://test.invalid/old.terrain",
          accept: "application/octet-stream",
        },
      ]);
    }
    return "[]";
  },
  submitTerrain() {
    submits++;
  },
  failTerrain() {
    failed++;
  },
  clearTerrain() {
    phase = "off";
    cleared++;
  },
  setTerrainOptions(exaggeration,sse) {lastOptions=[exaggeration,sse];},
  sampleHeight() {
    return undefined;
  },
  focusLocation() {},
  lookAtTerrain() {},
  destroy() {},
  free() {},
  lastPick: "null",
  stats: "test",
  imageryCredits: "test",
  terrainCredits: "test terrain",
};
Object.defineProperty(globalThis, "navigator", {
  value: { gpu: {} },
  configurable: true,
});
Object.assign(globalThis, {
  window: { devicePixelRatio: 1 },
  location: { href: "http://test.invalid/" },
  ResizeObserver: class {
    observe() {}
    disconnect() {}
  },
  fakeViewer: raw,
});
const frames = [];
globalThis.requestAnimationFrame = (fn) => {
  frames.push(fn);
  return frames.length;
};
globalThis.cancelAnimationFrame = () => {};
const pending = new Map();
globalThis.fetch = (url, options) =>
  new Promise((resolve) => pending.set(String(url), { resolve, options }));
let source = await fs.readFile(
  new URL("../bindings/web/sdk.js", import.meta.url),
  "utf8",
);
source=source.replace("import {hydrateImages} from './resources.js';",`import {hydrateImages} from ${JSON.stringify(new URL('../bindings/web/resources.js',import.meta.url).href)};`);
source = source.replace(
  "import {attachControls} from './controls.js';",
  `import {attachControls} from ${JSON.stringify(pathToFileURL(new URL("../bindings/web/controls.js", import.meta.url).pathname).href)};`,
);
source = source.replace(
  /^import init,.*$/m,
  'const init=async()=>{};const attach_canvas=async()=>globalThis.fakeViewer;const bingMetadataUrl=()=>"";',
);
const { attachCanvas } = await import(
  "data:text/javascript," + encodeURIComponent(source)
);
const errors = [];
const sdk = await attachCanvas(new Canvas(), {
  onError: (e) => errors.push(e),
});
const root = sdk.loadTerrain("/new/layer.json");
frames.shift()();
assert(pending.has("http://test.invalid/old.terrain"));
assert.equal(
  pending.get("http://test.invalid/old.terrain").options.headers.Accept,
  "application/octet-stream",
);
pending.get("http://test.invalid/new/layer.json").resolve(new Response("{}"));
assert.equal(await root, true);
pending
  .get("http://test.invalid/old.terrain")
  .resolve(new Response(new Uint8Array([1, 2, 3])));
await new Promise((r) => setTimeout(r, 0));
assert.equal(submits, 0, "old terrain must not enter the new provider");
const cancelled = sdk.loadTerrain("/cancel/layer.json");
sdk.clearTerrain();
pending
  .get("http://test.invalid/cancel/layer.json")
  .resolve(new Response("{}"));
assert.equal(await cancelled, false);
assert.equal(layerSets, 1);
assert.equal(cleared, 1);
assert.equal(sdk.sampleHeight(0, 0), null);
sdk.showTerrain();sdk.setTerrainOptions(2,4);sdk.clearTerrain();sdk.showTerrain();assert.deepEqual(lastOptions,[2,4]);
const world = sdk.loadWorldTerrain("user-secret");
await new Promise((r) => setTimeout(r, 0));
const endpoint = [...pending.keys()].find((u) =>
  u.startsWith("https://api.cesium.com/v1/assets/1/endpoint"),
);
assert(endpoint);
pending
  .get(endpoint)
  .resolve(
    new Response(
      JSON.stringify({
        type: "TERRAIN",
        url: "https://terrain.example.test/world/",
        accessToken: "asset-secret",
      }),
    ),
  );
await new Promise((r) => setTimeout(r, 0));
const request = pending.get("https://terrain.example.test/world/layer.json");
assert.equal(request.options.headers.Authorization, "Bearer asset-secret");
assert.equal(request.options.credentials, "omit");
assert.equal(request.options.redirect, "error");
request.resolve(new Response("{}"));
assert.equal(await world, true);
const late = sdk.loadTerrain("/destroy/layer.json");
sdk.destroy();
pending
  .get("http://test.invalid/destroy/layer.json")
  .resolve(new Response("{}"));
assert.equal(await late, false);
assert.equal(layerSets, 2);
assert.equal(failed, 0);
assert.deepEqual(errors, []);
console.log(
  "Terrain lifecycle passed: in-flight old provider isolation, abort/clear/destroy guards, unknown heights, ion endpoint and bearer headers",
);
