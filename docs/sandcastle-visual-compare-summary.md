# Cesium-Rust vs 原版 CesiumJS Sandcastle 视觉对照结论与对齐路线图

## 1. 测试体系与基准环境

- **对照基准**：本地检出的原版 CesiumJS 1.146（`http://localhost:8080/Apps/Sandcastle2/standalone.html?id=<id>`），固定时钟 `2023-12-25T14:00:00Z`，视口 `960x540`（`deviceScaleFactor: 1`），隐藏 UI 控件与控制台栏。
- **Rust 被测端**：[sandcastle-harness.html](../tests/e2e/sandcastle-harness.html) 在真实 Chromium WebGPU（`960x540`）中加载 [full-recipes.json](../tests/sandcastle/full-recipes.json) 全部 309 个案例（覆盖 `entities`、`czml`、`primitives`、`groups`、`scene`、`variants` 6 种格式）。
- **像素比对规则**：沿用原版 CesiumJS Playwright 配置（单通道容差 `threshold = 0.25`，目标差异像素比例 `maxDiffPixelRatio <= 0.02` 或阶段目标 `<= 0.10`），由 [visual-compare.cjs](../tests/e2e/visual-compare.cjs) 计算差异像素率、MAE 与 RMSE，并输出三联差异图（`原版 | Rust | 差异热力图`）至 `target/e2e-results/sandcastle-diff/<id>.png`。

## 2. 首轮全量基准统计（309 / 309）

详见完整逐案例明细：[sandcastle-visual-compare.md](sandcastle-visual-compare.md) 与 [sandcastle-visual-compare.json](sandcastle-visual-compare.json)。

| 指标 | 结果 |
| --- | --- |
| 案例总数 | 309 |
| 原版 CesiumJS 基准图抓取 | 309 / 309 |
| Rust WebGPU 渲染无报错且非空 | 309 / 309 |
| Rust 回归快照通过率 | 309 / 309 |
| 差异率 <= 2% | 0 / 309 |
| 差异率 <= 10% | 62 / 309 |
| 差异率 10% – 25% | 79 / 309 |
| 差异率 25% – 50% | 50 / 309 |
| 差异率 > 50% | 118 / 309 |
| 全量平均差异像素率 | 45.34% |
| 全量平均 RMSE | 0.2519 |

代表性最佳案例：`cardboard`（2.07%）、`ground-polyline-material-dev`（2.14%）、`czml-model-node-transformations`（3.22%）、`interpolation`（3.34%）、`gpx`（3.52%）、`3d-models-dev`（3.61%）、`box-outline-dev`（5.37%）。

代表性中等差异案例：`wall`（13.66%）、`polygon`（14.30%）、`offline`（17.95%）、`box`（23.12%）、`cylinders-and-cones`（29.01%）。

代表性高差异案例（> 99%）：`3d-tiles-point-cloud`、`3d-tiles-edge-visibility`、`google-2d-tiles`、`fog-post-process`、`geometry-height-reference` 等。

## 3. 视觉差异根因分析

1. **相机默认视场角（FOV）与取景距离**：
   - 在 `offline`（17.95%）、`hello-world`（12.79%）、`box`（23.12%）等纯地球/基础几何案例中，原版 CesiumJS 默认相机下地球与几何体在画面中占比更大，而 Rust 端视角略远。
2. **光照强度、sRGB 色调映射、大气边缘散射与星空背景**：
   - 原版 CesiumJS 具有更亮的太阳定向光、sRGB 校正、蓝色大气边缘光晕与星空背景（`#000000` + 恒星），而 Rust 端背景为深蓝纯色（`#0b1422`）且地球与几何体整体偏暗。
3. **默认底图与 Cesium Ion 资产**：
   - 原版 Sandcastle 默认使用 Cesium Ion（Bing Maps 卫星底图及 Ion 3D Tiles / Terrain / 模型资产），而当前 Rust 端统一使用离线 `NaturalEarthII` 且默认关闭地形/Ion 资产加载。
4. **高级渲染管线与特定场景特性尚未在沙盒对比中接入或未实现**：
   - 依赖 **3D Tiles 真实瓦片加载与样式/裁剪**、**全球/局部地形（Terrain）与贴地（HeightReference / ClampToGround）**、**glTF 3D 模型渲染**、**后处理（Bloom / Fog / FXAA / Ambient Occlusion）**、**粒子系统（ParticleSystem）**、**自定义 GLSL / CustomShader**、**云/大气/阴影** 等特性的案例，当前在 Rust 侧仅加载了简化占位或未激活对应管线，导致画面差异 > 50%。

## 4. 逐阶段对齐目标（将全量差异率降至 <= 10%）

1. **阶段 A（全局基础视觉对齐，直接大幅压低 140+ 基础案例差异）**：
   - 接入默认 Cesium Ion Token（通过运行时配置/环境变量注入，不污染参考源码）。
   - 对齐默认相机 FOV / 默认地球视角与 `zoomToEntities` 取景距离。
   - 对齐地球与几何体光照亮度、sRGB/色调映射、纯黑星空背景与大气边缘散射。
   - 支持在线/Ion 卫星底图（Bing Maps / Ion Imagery）与 `NaturalEarthII` 按案例匹配。
2. **阶段 B（实体、材质、CZML 与 Primitive 细节对齐）**：
   - 对齐 `box`、`wall`、`polygon`、`polyline`、`cylinders-and-cones`、`spheres-and-ellipsoids`、`billboards`、`labels` 等着色、轮廓线宽与透明混合。
3. **阶段 C（3D Tiles、glTF 模型与地形对齐）**：
   - 在 Sandcastle 对照中接入真实 3D Tileset / glTF / Cesium World Terrain 与贴地/高度参考。
4. **阶段 D（后处理、粒子系统、阴影与自定义着色器）**：
   - 补齐 Bloom、Fog、Shadows、ParticleSystem、CustomShader / Material 等渲染通道并逐项复测。
