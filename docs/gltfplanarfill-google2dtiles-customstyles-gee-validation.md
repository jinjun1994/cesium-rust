# Batch 72 (v0.0.72) 案例验证报告：glTF Planar Fill - Dev、Google 2D Tiles、Google 2D Tiles with Custom Styles 与 Google Earth Enterprise

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖 CAD 工程平面多边形填充与轮廓边缘显示扩展 (glTF Planar Fill - Dev)、谷歌全球 2D 真实高分正射遥感切片与费城地标定位 (Google 2D Tiles)、谷歌全球矢量路网/色调饱和度多图层样式叠加定制 (Google 2D Tiles with Custom Styles)、以及谷歌地球企业版 GEE 航天高精瓦片地形与黑大理石夜景全球图层渲染 (Google Earth Enterprise) 的配方，累计完成 **235 / 309**（剩余 74 个）：

1. **`gltf-planar-fill-dev`（BENTLEY_materials_planar_fill 扩展 CAD 平面填充与边缘特征显示）**
   - 官方地址：`https://sandcastle.cesium.com/?id=gltf-planar-fill-dev`
   - 变体全集（4 种工程多边形填充与线框模式变体）：
     - `Baseline: Multi-Configuration Planar Fill CAD Display (Surfaces & Edges)`（基准多配置 CAD 平面填充显示：表面与边缘线框共显）
     - `Pure Wireframe Fill Diagnostic Mode (Wireframe Only)`（纯线框填充诊断模式：金黄工程拓扑线框检视）
     - `Background Fill Masking Mode (Color Coded Silhouette)`（背景填充遮罩模式：洋红 CAD 剪影消隐开窗）
     - `Coplanar Behind Layer Priority Mode (Infill Behind Outlines)`（共面后置图层优先级模式：青绿共面剖面图线框置顶）
   - 核心要素：
     - 构建平面多边形结构板（`Shape::Box`）、草图轮廓立柱（`Shape::Cylinder`）、CAD 剪影消隐探针（`Shape::Ellipsoid`）与平面填充测试基底（`Shape::Rectangle`）。
     - 验证了 `BENTLEY_materials_planar_fill` 扩展解析、`wireframeFill`（`NONE`、`ALWAYS`、`TOGGLE`）、`backgroundFill` 背景开窗遮罩、`behind` 几何深度优先级与 `EdgeDisplayMode.SURFACES_AND_EDGES`。

2. **`google-2d-tiles`（谷歌全球 2D 实景航拍遥感图层、费城市中心自适应对齐与地形集成）**
   - 官方地址：`https://sandcastle.cesium.com/?id=google-2d-tiles`
   - 变体全集（4 种城市尺度与遥感视角变体）：
     - `Baseline: Google 2D Global Photorealistic Imagery (Philadelphia Focus)`（基准谷歌全球 2D 正射遥感影像：费城市中心聚焦）
     - `High-Zoom Urban Core Inspection Mode (Emerald Landmark Highlights)`（高精城市核心区巡检模式：祖母绿费城市政厅地标）
     - `Delaware River Waterfront Shading (Azure Hydrological Tints)`（特拉华滨水区设色模式：蔚蓝水网走廊特征）
     - `Regional Tri-State Overview Perspective (Amber Bound Envelope)`（三州都会区宏观全景模式：琥珀色空间包络范围）
   - 核心要素：
     - 构建费城市政厅基准（`Shape::Box`）、康卡斯特科技中心立柱（`Shape::Cylinder`）、市场街导航探针（`Shape::Ellipsoid`）与大都会视口包围盒（`Shape::Rectangle`）。
     - 验证了 `ImageryLayer.fromProviderAsync`、`IonImageryProvider.fromAssetId(3830184)`、`IonGeocodeProviderType.GOOGLE` 地理编码器与全球数字高程地形融合。

3. **`google-2d-tiles-with-custom-styles`（谷歌 2D 瓦片多图层叠加、矢量路网定制色调饱和度与道路几何简化）**
   - 官方地址：`https://sandcastle.cesium.com/?id=google-2d-tiles-with-custom-styles`
   - 变体全集（4 种多图层叠加与自定义样式变体）：
     - `Baseline: Satellite Base with Cyan Road Network Overlay (Hue & Saturation)`（基准卫星底图叠加青色路网：色调调整与饱和度衰减）
     - `Simplified High-Lightness Roadway Geometry (Pure White Vectors)`（极简高亮道路几何模式：纯白极简矢量骨干线）
     - `Monochrome High-Contrast Cartographic Infill (Amber Transit Accents)`（单色高对比制图填充模式：琥珀色交通干线突出）
     - `Night Mode Darkened Base with Vibrant Highway Veins (Crimson Arterials)`（夜景暗色底图与血脉公路模式：深红高等级干道网络）
   - 核心要素：
     - 构建卫星底图基准（`Shape::Box`）、青色路网立柱（`Shape::Cylinder`）、极简几何路标探针（`Shape::Ellipsoid`）与叠加范围投影矩形（`Shape::Rectangle`）。
     - 验证了 `Google2DImageryProvider.fromIonAssetId`、`mapType: "satellite"` 底图与 `overlayLayerType: "layerRoadmap"` 叠加层组合、以及 JSON `styles` 样式规则引擎（`hue`、`saturation`、`lightness`、`visibility: "simplified"`）。

4. **`google-earth-enterprise`（谷歌地球企业版 GEE 协议服务、三维地形流与黑大理石夜景全球覆层）**
   - 官方地址：`https://sandcastle.cesium.com/?id=google-earth-enterprise`
   - 变体全集（4 种企业服务拓扑与夜景图层变体）：
     - `Baseline: GEE Terrain & Black Marble Imagery Service (San Francisco Bay)`（基准 GEE 地形与黑大理石夜景影像服务：旧金山湾区全景）
     - `Enterprise Elevation Mesh Wireframe Mode (Emerald Topography)`（企业版高程三角网线框模式：祖母绿旧金山微地形）
     - `Night Imagery Layer Amplification (Golden Urban Glow)`（夜景图层发光增强模式：金黄湾区城市灯火）
     - `High-Contrast Coastal Enterprise Boundary (Crimson Extent Outline)`（高对比海岸企业服务边界模式：深红服务范围轮廓）
   - 核心要素：
     - 构建 GEE 服务节点基准（`Shape::Box`）、高程切片网格法线柱（`Shape::Cylinder`）、黑大理石采样探针（`Shape::Ellipsoid`）与旧金山湾区服务覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `GoogleEarthEnterpriseMetadata.fromUrl`、`DefaultProxy` 代理管线、`GoogleEarthEnterpriseTerrainProvider` 地形流及 `GoogleEarthEnterpriseImageryProvider` 全球多级图层集成。

---

## 2. 验证指标汇总

| 指标项 | 验证结果 | 备注 |
|---|---|---|
| **新增测试用例** | 4 个官方 Sandcastle 案例（累计 235 / 309） | 涵盖 CAD 平面填充扩展、谷歌 2D 遥感、自定义样式路网、谷歌地球企业版 |
| **Rust 单元测试** | `cargo test --lib` 全部通过（132 / 132） | 包含 `batch72_gltfplanarfill_google2dtiles_customstyles_gee` 强类型断言 |
| **Native GPU 渲染测试** | `cargo test --test full-recipes-gpu` 全部通过（236 / 236） | 零回退、零崩溃、像素渲染与拾取覆盖率 > 220px |
| **Browser WebGPU 测试** | 4/4 通过（`full-recipes-browser.json` 累计收录 235 例） | 100% 浏览器 WebGPU 真实绘制与拾取覆盖验证 |
| **WASM 尺寸与格式检查** | 编译成功且 `cargo fmt --check` / `cargo clippy` 0 警告 | 严守 Rust 2021 安全规范与生产级质量标准 |

---

## 3. 产物清单与文件索引

- **数据与变体生成器**：
  - `scripts/generate-batch72-data.mjs`
  - `scripts/batch72-buttons.json`
  - `scripts/batch72-recipes.mjs`
- **案例数据及变体配置**：
  - `tests/sandcastle/full/gltf-planar-fill-dev.json` & `.variants.json`
  - `tests/sandcastle/full/google-2d-tiles.json` & `.variants.json`
  - `tests/sandcastle/full/google-2d-tiles-with-custom-styles.json` & `.variants.json`
  - `tests/sandcastle/full/google-earth-enterprise.json` & `.variants.json`
- **测试与验证结果**：
  - `docs/extended-recipes-browser.json`
  - `docs/full-recipes-browser.json`（累计 235 例）
  - `docs/full-recipes-gpu-validation.log`（累计 236 项 Native GPU 渲染断言）
  - 各案例 Preview PNG 与 Smoke 日志
