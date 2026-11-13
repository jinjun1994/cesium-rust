# Sandcastle 全量 309 案例同场景视觉对照基线结论

- **生成时间**：2026-11-13
- **对照基准**：本地原版 CesiumJS 1.146（`http://localhost:8080/Apps/Sandcastle2/standalone.html?id=<id>`，固定时钟 `2023-12-25T14:00:00Z`，视口 `960x540`，隐藏 UI 控件与控制台栏）
- **Rust 运行环境**：`cesium-rust` WASM + Chrome WebGPU（`960x540`，本地 `NaturalEarthII` 底图，无地形，离线拦截外网）

## 1. 测试体系与产出

- 原版基准抓取脚本：[scripts/capture-sandcastle-references.mjs](../../scripts/capture-sandcastle-references.mjs)，已抓取 **309 / 309** 案例至 `tests/e2e/sandcastle-reference/<id>.png`。
- Rust WebGPU 承载页：[tests/e2e/sandcastle-harness.html](../../tests/e2e/sandcastle-harness.html)，支持 `full-recipes.json` 全部 6 种格式（`entities`、`czml`、`primitives`、`groups`、`scene`、`variants`）。
- 像素对照工具：[tests/e2e/visual-compare.cjs](../../tests/e2e/visual-compare.cjs)，计算差异像素率（`threshold=0.25`）、`MAE`、`RMSE` 并输出三联对比图（`target/e2e-results/sandcastle-diff/<id>.png`）。
- 汇总报告：[docs/sandcastle-visual-compare.md](sandcastle-visual-compare.md) 与 [docs/sandcastle-visual-compare.json](sandcastle-visual-compare.json)。

## 2. 基线实测结果

- **原有 3D Tiles 端到端测试**（`npm run test:e2e`）：`2 / 2` 通过（`batched`、`point-cloud`）。
- **全量 Sandcastle 端到端测试**（`REGRESSION_SNAPSHOTS=1 npm run test:e2e:sandcastle`）：`309 / 309` 全部无报错、画面非空，且通过 Rust 回归快照校验。
- **与原版 CesiumJS 像素对照（309 / 309）**：
  - 差异率 `<= 2%`：`0 / 309`
  - 差异率 `<= 10%`：`62 / 309`（例如 `cardboard: 2.07%`、`ground-polyline-material-dev: 2.14%`、`box-outline-dev: 5.37%`）
  - 差异率 `10% – 25%`：`79 / 309`（例如 `wall: 13.66%`、`polygon: 14.30%`、`offline: 17.95%`、`box: 23.12%`）
  - 差异率 `25% – 50%`：`50 / 309`
  - 差异率 `> 50%`：`118 / 309`（例如 `3d-tiles-point-cloud: 99.99%`、`google-2d-tiles: 100.00%`、`fog-post-process: 99.96%`）
  - **平均像素差异率（threshold=0.25）**：`45.34%`
  - **平均 RMSE**：`0.2519`

## 3. 主要视觉差异根因

1. **底图与在线 Ion 资产**：基线测试中 Rust 侧默认使用离线 `NaturalEarthII` 且拦截了外部网络请求，未启用 Cesium Ion 默认底图、地形及在线 3D Tileset。
2. **默认相机 FOV / 视距与默认视角**：部分 `viewer-default` 与 `zoomToEntities` 场景的相机视距与原版存在偏差（如 `offline` 地球在画面中明显偏小）。
3. **光照、sRGB 色调与大气/星空**：原版 CesiumJS 具有太阳定向光、sRGB 亮度曲线、大气边缘散射与星空背景，Rust 侧整体偏暗且背景为纯深蓝黑。
4. **高级渲染特性与场景语义还原**：依赖 3D Tiles、地形、后处理（Bloom/DOF/Fog/AO）、粒子系统、自定义 GLSL、模型加载（glTF/GLB）及动态时钟/相机预设的案例尚未在画面上与原版对齐。

## 4. 优化目标

使用提供的默认 Cesium Ion Token 并系统补全/对齐渲染管线与场景参数，将全量 309 个 Sandcastle 案例的同场景像素差异率逐步压低至合理阈值（`<= 10%`）以下。
