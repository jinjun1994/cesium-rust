// Production UI and real DOM; the SDK boundary is substituted because GPU is separate.
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import { JSDOM } from "jsdom";
const root = new URL("../", import.meta.url),
  html = await fs.readFile(new URL("bindings/web/index.html", root), "utf8");
const catalog = JSON.parse(
  await fs.readFile(new URL("tests/samples/catalog.json", root), "utf8"),
);
let source = await fs.readFile(
  new URL("bindings/web/workbench.js", root),
  "utf8",
);
source = source.replace(
  /await import\(['"]\.\/sdk\.js['"]\)/,
  "globalThis.testSdk",
);
const response = {
  commit: catalog.commit,
  summary: { passed: 1, failed: 1 },
  results: [
    {
      id: catalog.samples[0].id,
      status: "passed",
      result: { nodes: 2 },
      contentChecks: [{ id: "content-preview" }],
      renderStatus: "not-tested",
    },
    {
      id: catalog.samples[1].id,
      status: "failed",
      error: "required extension",
    },
  ],
};
let hooks;
const gpuFixture = {
  commit: catalog.commit,
  results: [
    {
      id: "content-preview",
      status: "passed",
      result: {
        visiblePixelsObserved: true,
        preview: "../../tests/samples/previews/test.png",
      },
    },
  ],
};
const calls = [],
  downloads = [];
const sdk = {
  showTerrain() {
    calls.push(["terrain-public"]);
  },
  clearTerrain() {
    calls.push(["terrain-off"]);
  },
  async loadWorldTerrain(token) {
    calls.push(["terrain-world", token]);
    return true;
  },
  async loadTerrain(url) {
    calls.push(["terrain-layer", url]);
    return true;
  },
  setTerrariumTerrain(template) {
    calls.push(["terrarium", template]);
  },
  setTerrainOptions(scale, sse) {
    calls.push(["terrain-options", scale, sse]);
  },
  sampleHeight(lon, lat) {
    calls.push(["height", lon, lat]);
    return 8848;
  },
  focusLocation(lon, lat, height) {
    calls.push(["location", lon, lat, height]);
  },
  lookAtTerrain(lon, lat, range) {
    calls.push(["tilt", lon, lat, range]);
  },
  diagnostics: { tileCompleted: 1, tileFailed: 0, tilePending: 0 },
  stats: "resident 1 | draws 1",
  async load(url) {
    calls.push(["load", url]);
    if (url.includes("3dcitydb_ingolstadt"))
      throw Error("Unsupported test content");
  },
  zoom(d) {
    calls.push(["zoom", d]);
  },
  resetView() {
    calls.push(["reset"]);
  },
  showGlobe() {
    calls.push(["earth"]);
  },
  async loadWorldImagery(t) {
    calls.push(["world", t]);
  },
  async loadGeoJson(u) {
    calls.push(["geo", u]);
    return 7;
  },
  focusGeoJson(id) {
    calls.push(["focus", id]);
  },
  clearGeoJson() {
    calls.push(["clear"]);
  },
  loadBytes() {
    calls.push(["bytes"]);
  },
  destroy() {
    calls.push(["destroy"]);
  },
};
async function waitFor(fn) {
  const start = Date.now();
  while (!fn()) {
    if (Date.now() - start > 4000) throw Error("UI wait timed out");
    await new Promise((r) => setTimeout(r, 10));
  }
}
async function mount(fail = false) {
  const dom = new JSDOM(html, {
    url: "http://localhost:8000/bindings/web/",
    pretendToBeVisual: true,
  });
  Object.assign(globalThis, {
    window: dom.window,
    document: dom.window.document,
    DOMParser: dom.window.DOMParser,
  });
  globalThis.testSdk = {
    async attachCanvas(canvas, options) {
      if (fail) throw Error("GPU unavailable");
      hooks = options;
      options.onStatus("Earth yes");
      return sdk;
    },
  };
  globalThis.fetch = async (url) =>
    new Response(
      JSON.stringify(
        String(url).endsWith("catalog.json")
          ? catalog
          : String(url).endsWith("gpu-report.json")
            ? gpuFixture
            : response,
      ),
    );
  URL.createObjectURL = (blob) => {
    downloads.push(blob);
    return "blob:test";
  };
  URL.revokeObjectURL = () => {};
  dom.window.HTMLAnchorElement.prototype.click = function () {};
  await import(
    "data:text/javascript," + encodeURIComponent(source) + "#" + Math.random()
  );
  await waitFor(
    () =>
      document.querySelector(".sample") &&
      document.getElementById("gpuState").textContent !== "初始化中" &&
      document.getElementById("validation").textContent.includes("summary"),
  );
  return dom;
}
let dom = await mount();
const $ = (id) => document.getElementById(id),
  click = (id) => $(id).click();
assert.match($("catalogStatus").textContent, /94 \/ 3721/);
assert.equal($("loadSelected").disabled, false);
$("search").value = "3dbag_mulitplefeatureids";
$("search").dispatchEvent(new window.Event("input"));
assert.equal(document.querySelectorAll(".sample").length, 1);
document.querySelector(".sample").click();
assert.match($("validation").textContent, /JSON 解析通过/);
assert.equal($("gpuPreview").hidden, false);
assert.match($("previewImage").src, /previews\/test.png/);
click("visualPass");
assert.match($("error").textContent, /先加载/);
click("loadSelected");
await waitFor(() => $("liveState").textContent.includes("根已解析"));
click("visualPass");
assert.match($("visualState").textContent, /通过/);
click("zoomIn");
click("zoomOut");
assert.deepEqual(
  calls.filter((c) => c[0] === "zoom"),
  [
    ["zoom", 2],
    ["zoom", -2],
  ],
);
click("geoDemo");
await waitFor(() => calls.some((c) => c[0] === "geo"));
click("geoFocus");
click("geoClear");
assert(calls.some((c) => c[0] === "focus" && c[1] === 7));
assert(calls.some((c) => c[0] === "clear"));
click("runQueue");
click("stopQueue");
click("runQueue");
await waitFor(() => $("queueStatus").textContent.includes("轮测完成"));
click("export");
const exported = JSON.parse(await downloads.at(-1).text());
assert.equal(exported.commit, catalog.commit);
assert.equal(exported.sessions[0].load, "content-decoded");
assert.equal(exported.sessions[0].visual, "未验证");
$("search").value = "3dcitydb_ingolstadt";
$("search").dispatchEvent(new window.Event("input"));
document.querySelector(".sample").click();
click("loadSelected");
await waitFor(() => $("liveState").textContent === "根加载失败");
assert.match($("error").textContent, /Unsupported test content/);
$("search").value = "";
$("search").dispatchEvent(new window.Event("input"));
$("subtrees").checked = true;
$("subtrees").dispatchEvent(new window.Event("input"));
assert.match($("catalogStatus").textContent, /3721 \/ 3721/);
assert.equal(document.querySelectorAll(".sample").length, 300);
$("filter").value = "failed";
$("filter").dispatchEvent(new window.Event("input"));
assert.equal(document.querySelectorAll(".sample").length, 1);
click("terrainPublic");
assert(calls.some((c) => c[0] === "terrain-public"));
$("terrainScale").value = "2";
click("terrainOptions");
assert(
  calls.some((c) => c[0] === "terrain-options" && c[1] === 2 && c[2] === 8),
);
click("terrainFocus");
click("terrainTilt");
assert(calls.some((c) => c[0] === "location" && c[1] === 86.92528));
assert(calls.some((c) => c[0] === "tilt"));
click("terrainSample");
assert.match($("terrainHeight").textContent, /8848/);
$("terrainUrl").value = "https://example.test/layer.json";
click("terrainLoad");
await waitFor(() => calls.some((c) => c[0] === "terrain-layer"));
click("terrainOff");
assert.match($("terrainStatus").textContent, /关闭/);
hooks.onRenderError("Device lost");
assert.equal($("loadSelected").disabled, true);
assert.match($("gpuState").textContent, /渲染已停止/);
window.dispatchEvent(new window.Event("pagehide"));
dom.window.close();
dom = await mount(true);
assert.equal($("loadSelected").disabled, true);
assert.match($("gpuState").textContent, /不可用/);
document.querySelector(".sample").click();
assert.match($("selection").textContent, /samples\//);
click("export");
assert(downloads.length === 2);
dom.window.close();
console.log(
  "Workbench passed: 3721 catalog, filtering, selection, SDK actions, queue/export, manual visual guard, load failure, GPU-unavailable browsing",
);
