const $ = (id) => document.getElementById(id),
  labels = {
    passed: "JSON 解析通过",
    failed: "解析失败",
    "download-failed": "下载失败",
    "not-tested": "未验证",
  };
let catalog = [],
  report = new Map(),
  selected = null,
  viewer = null,
  lastGeo = null,
  queueStopped = true,
  queueEpoch = 0,
  runId = 0,
  currentSample = null,
  commit = "",
  gpuReport = new Map();
const sessions = new Map();
const log = (m) => {
  $("error").textContent = (
    `${new Date().toLocaleTimeString()} ${m}\n` + $("error").textContent
  ).slice(0, 12000);
};
const session = (id) => {
  if (!sessions.has(id)) sessions.set(id, { id, visual: "未验证" });
  return sessions.get(id);
};
function filtered() {
  return catalog.filter(
    (r) =>
      ($("subtrees").checked || !r.subtree) &&
      r.path.toLowerCase().includes($("search").value.toLowerCase()) &&
      ($("filter").value === "all" ||
        (report.get(r.id)?.status || "not-tested") === $("filter").value),
  );
}
function render() {
  const rows = filtered();
  $("catalogStatus").textContent =
    `${rows.length} / ${catalog.length} 个 tileset · 固定版本`;
  $("catalogStatus").title = commit;
  const fragment = document.createDocumentFragment();
  for (const r of rows.slice(0, 300)) {
    const button = document.createElement("button");
    button.className = "sample" + (selected?.id === r.id ? " active" : "");
    const title = document.createElement("strong"),
      badge = document.createElement("small"),
      state = report.get(r.id)?.status || "not-tested";
    title.textContent = r.path.replace(/^samples\//, "");
    badge.className = state;
    badge.textContent =
      labels[state] + " · 视觉 " + (sessions.get(r.id)?.visual || "未验证");
    button.append(title, badge);
    button.onclick = () => select(r);
    fragment.append(button);
  }
  if (rows.length > 300) {
    const p = document.createElement("p");
    p.textContent = "显示前 300 项，请搜索更精确的路径。轮测使用全部筛选结果。";
    fragment.append(p);
  }
  $("catalog").replaceChildren(fragment);
}
function select(row) {
  selected = row;
  $("selection").textContent = row.path;
  $("source").href = row.url;
  $("url").value = row.url;
  const r = report.get(row.id);
  $("validation").textContent = r
    ? JSON.stringify(
        {
          JSON: labels[r.status],
          error: r.error,
          nodes: r.result?.nodes,
          optionalExtensions: r.optionalExtensions,
          contentCoverage: r.contentCoverage,
          contentChecks: r.contentChecks,
          renderStatus: r.renderStatus,
        },
        null,
        2,
      )
    : "未执行批量验证";
  const check = r?.contentChecks
    ?.map((c) => gpuReport.get(c.id))
    .find((c) => c?.status === "passed" && c.result?.visiblePixelsObserved);
  $("gpuPreview").hidden = !check;
  if (check) $("previewImage").src = check.result.preview;
  $("visualState").textContent =
    "视觉状态：" + (sessions.get(row.id)?.visual || "未验证");
  render();
}
async function loadSample(row) {
  if (!viewer) throw Error("三维渲染器尚未就绪");
  const operation = ++runId;
  currentSample = row.id;
  select(row);
  $("currentName").textContent = row.path.replace(/^samples\//, "");
  $("liveState").textContent = "正在加载根 tileset…";
  const r = session(row.id);
  r.startedAt = new Date().toISOString();
  r.load = "loading";
  r.visual = "未验证";
  delete r.error;
  try {
    await viewer.load(row.url);
    if (operation !== runId) return false;
    r.load = "root-loaded";
    $("liveState").textContent = "根已解析，正在请求内容；视觉尚未确认";
    return true;
  } catch (e) {
    if (operation === runId) {
      r.load = "failed";
      r.error = e.message;
      $("liveState").textContent = "根加载失败";
    }
    throw e;
  }
}
async function action(fn) {
  try {
    await fn();
  } catch (e) {
    log(e?.message || String(e));
  }
}
function gpuAction(id, fn, changesScene = false) {
  $(id).onclick = () =>
    action(() => {
      queueStopped = true;
      ++queueEpoch;
      if (changesScene) {
        ++runId;
        currentSample = null;
      }
      if (!viewer) throw Error("WebGPU 不可用");
      return fn();
    });
}
for (const id of ["search", "filter", "subtrees"])
  $(id).addEventListener("input", render);
$("loadSelected").onclick = () =>
  action(async () => {
    queueStopped = true;
    ++queueEpoch;
    if (!selected) throw Error("请先选择样本");
    await loadSample(selected);
  });
$("stopQueue").onclick = () => {
  queueStopped = true;
  ++queueEpoch;
  $("queueStatus").textContent = "已停止轮测";
};
$("runQueue").onclick = () =>
  action(async () => {
    if (!viewer) throw Error("WebGPU 不可用");
    if (!queueStopped) return;
    const rows = filtered();
    queueStopped = false;
    const token = ++queueEpoch;
    for (
      let i = 0;
      i < rows.length && !queueStopped && token === queueEpoch;
      i++
    ) {
      const row = rows[i];
      $("queueStatus").textContent =
        `轮测 ${i + 1}/${rows.length} · 每项观察最多 10 秒`;
      try {
        if (!(await loadSample(row))) break;
      } catch (e) {
        log(row.path + ": " + e.message);
        continue;
      }
      const operation = runId,
        deadline = performance.now() + 10000;
      let observed = -1,
        quietSince = performance.now();
      while (
        !queueStopped &&
        token === queueEpoch &&
        operation === runId &&
        performance.now() < deadline
      ) {
        await new Promise((r) => setTimeout(r, 200));
        const d = viewer.diagnostics,
          count = d.tileCompleted + d.tileFailed;
        if (count !== observed) {
          observed = count;
          quietSince = performance.now();
        }
        if (
          count > 0 &&
          d.tilePending === 0 &&
          performance.now() - quietSince >= 1000
        )
          break;
      }
      if (operation !== runId || token !== queueEpoch) break;
      const r = session(row.id);
      r.stats = viewer.stats;
      r.diagnostics = viewer.diagnostics;
      r.load = r.diagnostics.tileFailed
        ? "content-failed"
        : r.diagnostics.tileCompleted
          ? "content-decoded"
          : "no-content-observed";
    }
    if (token !== queueEpoch) return;
    if (!queueStopped)
      $("queueStatus").textContent =
        "轮测完成；可导出请求和驻留结果。视觉仍需人工确认。";
    queueStopped = true;
    ++queueEpoch;
  });
for (const [id, value] of [
  ["visualPass", "通过（人工确认）"],
  ["visualFail", "异常（人工确认）"],
])
  $(id).onclick = () => {
    if (
      !selected ||
      currentSample !== selected.id ||
      ["loading", "failed", undefined].includes(session(selected.id).load)
    ) {
      log("请先加载当前样本，再确认视觉结果");
      return;
    }
    session(selected.id).visual = value;
    $("visualState").textContent = "视觉状态：" + value;
    render();
  };
$("export").onclick = () => {
  const url = URL.createObjectURL(
      new Blob(
        [
          JSON.stringify(
            {
              commit,
              exportedAt: new Date().toISOString(),
              scope:
                "Parser/decode report only; visual results require manual confirmation.",
              batch: [...report.values()],
              isolatedGpu: [...gpuReport.values()],
              sessions: [...sessions.values()],
            },
            null,
            2,
          ),
        ],
        { type: "application/json" },
      ),
    ),
    a = document.createElement("a");
  a.href = url;
  a.download = "cesium-sample-test-results.json";
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
};
$("clearLog").onclick = () => {
  $("error").textContent = "";
};
gpuAction("zoomIn", () => viewer.zoom(2));
gpuAction("zoomOut", () => viewer.zoom(-2));
gpuAction("resetView", () => viewer.resetView());
gpuAction("load", () => viewer.load($("url").value), true);
gpuAction(
  "earth",
  () => {
    viewer.showGlobe();
    $("currentName").textContent = "全球渐进影像";
  },
  true,
);
gpuAction("world", () => viewer.loadWorldImagery($("ionToken").value), true);
gpuAction(
  "fixture",
  () => viewer.load("../../tests/fixtures/tileset.json"),
  true,
);
gpuAction("geoDemo", async () => {
  lastGeo = await viewer.loadGeoJson(
    "../../tests/fixtures/geojson/demo.geojson",
    { point_radius: 100000, line_width: 50000, surface_offset: 10000 },
  );
});
gpuAction("geoLoad", async () => {
  lastGeo = await viewer.loadGeoJson($("url").value);
});
gpuAction("geoFocus", () => {
  if (lastGeo !== null) viewer.focusGeoJson(lastGeo);
});
gpuAction("geoClear", () => {
  viewer.clearGeoJson();
  lastGeo = null;
});
gpuAction("fileButton", () => $("file").click());
$("file").onchange = () =>
  action(async () => {
    queueStopped = true;
    ++queueEpoch;
    ++runId;
    currentSample = null;
    const f = $("file").files[0];
    if (!f) return;
    if (f.size > 64 * 1024 * 1024) throw Error("文件超过 64 MiB");
    viewer.loadBytes(new Uint8Array(await f.arrayBuffer()));
  });
gpuAction("terrainPublic", () => {
  viewer.showTerrain();
  $("terrainStatus").textContent = "公开 Terrarium 全球地形 · 极区椭球回退";
});
gpuAction("terrainWorld", async () => {
  if (await viewer.loadWorldTerrain($("ionToken").value))
    $("terrainStatus").textContent = "Cesium World Terrain 已启用";
});
gpuAction("terrainOff", () => {
  viewer.clearTerrain();
  $("terrainStatus").textContent = "地形已关闭 · 椭球影像";
  $("terrainHeight").textContent = "地形已关闭";
});
gpuAction("terrainLoad", async () => {
  const url = $("terrainUrl").value;
  if (url.includes("{z}")) {
    viewer.setTerrariumTerrain(url, 15);
    $("terrainStatus").textContent = "自定义 Terrarium 地形";
  } else if (await viewer.loadTerrain(url))
    $("terrainStatus").textContent = "自定义 quantized-mesh 地形";
});
gpuAction("terrainDemo", async () => {
  viewer.showGlobe();
  if (await viewer.loadTerrain("../../tests/fixtures/terrain/demo/layer.json"))
    $("terrainStatus").textContent = "离线合成地形（非真实高程），仅用于测试";
});
gpuAction("terrainOptions", () =>
  viewer.setTerrainOptions(
    Number($("terrainScale").value),
    Number($("terrainSse").value),
  ),
);
function terrainPoint() {
  const lon = Number($("terrainLon").value),
    lat = Number($("terrainLat").value);
  if (!Number.isFinite(lon) || !Number.isFinite(lat) || Math.abs(lat) > 90)
    throw Error("经纬度无效");
  return [lon, lat];
}
gpuAction("terrainFocus", () => viewer.focusLocation(...terrainPoint(), 20000));
gpuAction("terrainTilt", () => viewer.lookAtTerrain(...terrainPoint(), 20000));
gpuAction("terrainSample", () => {
  const height = viewer.sampleHeight(...terrainPoint());
  $("terrainHeight").textContent =
    height === null
      ? "此位置尚未加载高程"
      : "已加载高程：" +
        height.toFixed(2) +
        " m（数据源原始高度基准；未乘高度倍率）";
});
async function loadCatalog() {
  try {
    const response = await fetch("../../tests/samples/catalog.json");
    if (!response.ok) throw Error("目录 HTTP " + response.status);
    const data = await response.json();
    catalog = data.samples;
    commit = data.commit;
    render();
    try {
      const response = await fetch("../../tests/samples/report.json");
      if (!response.ok)
        throw Error("暂无报告，请运行 scripts/validate-samples.py");
      const data = await response.json();
      if (data.commit !== commit) throw Error("报告与目录版本不一致");
      report = new Map(data.results.map((r) => [r.id, r]));
      try {
        const gpu = await fetch("../../tests/samples/gpu-report.json");
        if (gpu.ok) {
          const g = await gpu.json();
          if (g.commit === commit)
            gpuReport = new Map(g.results.map((r) => [r.id, r]));
        }
      } catch {}
      render();
      if (selected) select(selected);
      else
        $("validation").textContent = JSON.stringify(
          {
            summary: data.summary,
            contentSummary: data.contentSummary,
            scope: data.scope,
          },
          null,
          2,
        );
    } catch (e) {
      $("validation").textContent = e.message;
    }
  } catch (e) {
    log(e.message);
    $("catalogStatus").textContent = "目录读取失败";
  }
}
async function initialize() {
  document.querySelectorAll("[data-gpu]").forEach((b) => (b.disabled = true));
  try {
    const { attachCanvas } = await import("./sdk.js");
    viewer = await attachCanvas($("globe"), {
      onPick: (p) => ($("properties").textContent = JSON.stringify(p, null, 2)),
      onStatus: (s) => ($("status").textContent = s),
      onError: log,
      onRenderError: () => {
        queueStopped = true;
        ++queueEpoch;
        $("gpuState").textContent = "渲染已停止";
        $("emptyState").hidden = false;
        $("emptyState").textContent =
          "渲染发生错误。目录、报告和原生预览仍可使用，详细原因见运行日志。";
        document
          .querySelectorAll("[data-gpu]")
          .forEach((b) => (b.disabled = true));
      },
      onCredits: (c) =>
        ($("credits").textContent = new DOMParser().parseFromString(
          c,
          "text/html",
        ).body.textContent),
    });
    window.cesiumViewer = viewer;
    $("gpuState").textContent = "WebGPU 就绪";
    $("emptyState").hidden = true;
    document
      .querySelectorAll("[data-gpu]")
      .forEach((b) => (b.disabled = false));
  } catch (e) {
    $("gpuState").textContent = "WebGPU 不可用";
    $("status").textContent = "渲染器初始化失败";
    $("emptyState").textContent =
      "三维渲染不可用。可继续浏览样本和验证报告。请使用支持 WebGPU 的浏览器。";
    log(e.message);
  }
}
loadCatalog();
initialize();
window.addEventListener(
  "pagehide",
  () => {
    queueStopped = true;
    ++queueEpoch;
    viewer?.destroy();
  },
  { once: true },
);
