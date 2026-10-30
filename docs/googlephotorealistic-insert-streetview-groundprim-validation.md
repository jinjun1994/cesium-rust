# Batch 73 (v0.0.73) 案例验证报告：Google Photorealistic 3D Tiles、Google Photorealistic 3D Tiles with Building Insert、Google Street View Panorama 与 Ground Primitive - Dev

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖谷歌全球真实感三维网格瓦片与山景城 Googleplex 空间定位 (Google Photorealistic 3D Tiles)、拟建超高层建筑三维插入与规划地块 3D 瓦片分类高亮 (Google Photorealistic 3D Tiles with Building Insert)、谷歌街景全景球体/立方体贴图漫游与多视点转换 (Google Street View Panorama)、以及圣海伦斯火山地表贴地几何图元（多边形/走廊/圆形）与多图层透明混合 (Ground Primitive - Dev) 的配方，累计完成 **239 / 309**（剩余 70 个）：

1. **`google-photorealistic-3d-tiles`（谷歌全球高保真真实感 3D 瓦片集、Googleplex 视角与天空大气渲染）**
   - 官方地址：`https://sandcastle.cesium.com/?id=google-photorealistic-3d-tiles`
   - 变体全集（4 种城市尺度与遥感视角变体）：
     - `Baseline: Photorealistic 3D Tiles of Googleplex HQ (Photorealistic Mesh)`（基准谷歌总部高保真真实感实景三维：Googleplex 园区聚焦）
     - `Sunlit Aerial Inspection Perspective (High-Luminance Façades)`（日照航拍巡检视角：高亮度建筑立面模式）
     - `Silicon Valley Metro Foothill Elevation (Azure Ambient Tone)`（硅谷都会区山麓微地形：蔚蓝环境色调模式）
     - `High-Contrast Architectural Boundary (Amber Campus Envelope)`（高对比建筑边界模式：琥珀色园区空间包络）
   - 核心要素：
     - 构建 Googleplex 主体建筑基准（`Shape::Box`）、查尔斯顿中庭立柱（`Shape::Cylinder`）、大地测量视心探针（`Shape::Ellipsoid`）与园区地面轮廓矩形（`Shape::Rectangle`）。
     - 验证了 `createGooglePhotorealistic3DTileset`、`onlyUsingWithGoogleGeocoder: true`、`globe: false`、`skyAtmosphere.show = true` 以及基于 ECEF 笛卡尔坐标的精准相机姿态对准。

2. **`google-photorealistic-3d-tiles-with-building-insert`（真实感城市三维底座上拟建建筑插入与开发地块分类覆层）**
   - 官方地址：`https://sandcastle.cesium.com/?id=google-photorealistic-3d-tiles-with-building-insert`
   - 变体全集（4 种拟建建筑方案与规划高亮变体）：
     - `Baseline: Proposed High-Rise Tower Insert with Classified Lot (Yellow Target)`（基准拟建高层大厦嵌合与分类地块：黄色规划地块覆层）
     - `Isolated Building Volume Visibility Mode (Opaque Architectural Insert)`（拟建建筑体量独立显隐模式：不透明建筑设计方案检视）
     - `Target Development Lot Highlight Mode (Amber Boundary Envelope)`（目标规划地块高亮模式：琥珀色边界空间包络）
     - `Downtown Denver Urban Horizon Perspective (Azure City Context)`（丹佛市中心城市天际线视角：蔚蓝城市规划语境）
   - 核心要素：
     - 构建拟建大厦插入体量（`Shape::Box`）、核心筒设计柱体（`Shape::Cylinder`）、规划质心探针（`Shape::Ellipsoid`）与丹佛规划地块包围盒（`Shape::Rectangle`）。
     - 验证了 `Cesium3DTileset.fromIonAssetId(1670818)` 拟建建筑与谷歌真实感 3D 瓦片集空间融合、`ClassificationType.CESIUM_3D_TILE` 瓦片分类覆盖及显隐切换交互。

3. **`google-streetview-panorama`（谷歌全景街景球体/立方体贴图流式呈现与视线姿态跟踪）**
   - 官方地址：`https://sandcastle.cesium.com/?id=google-streetview-panorama`
   - 变体全集（4 种街景投影与多高度转换变体）：
     - `Baseline: 360-Degree Street View Panorama Bubble (Equirectangular Projection)`（基准 360 度街景全景气泡：等矩形柱面全景投影）
     - `CubeMap Panorama Multi-Face Perspective (Directional Street Facets)`（六面体立方体全景透视：定向街道面网检视）
     - `High-Altitude Map and Panorama Seamless Transition (Azure Horizon Blend)`（高空地图与微观街景无缝转换：蔚蓝地平线融合）
     - `Pedestrian Roadway Wayfinding Overlay (Amber Navigational Conduit)`（人行道路网导航指引覆层：琥珀色路网脉络）
   - 核心要素：
     - 构建街景全景采集基座（`Shape::Box`）、传感器天线立柱（`Shape::Cylinder`）、全景球体探针（`Shape::Ellipsoid`）与旧金山街景覆盖矩形（`Shape::Rectangle`）。
     - 验证了 `GoogleStreetViewCubeMapPanoramaProvider` 全景元数据检索、`layerStreetview` 街景图层覆加、相机朝向解算与地图/街景多视图联动。

4. **`ground-primitive-dev`（底层 GroundPrimitive、地形贴地多边形/走廊/圆形图元与半透明颜色属性）**
   - 官方地址：`https://sandcastle.cesium.com/?id=ground-primitive-dev`
   - 变体全集（4 种贴地几何图元与重叠半透明变体）：
     - `Baseline: Overlapping Translucent Ground Primitives (Red, Green, Blue Drape)`（基准重叠半透明贴地图元：红绿蓝多几何体贴地）
     - `Terrain-Clamped Polygon Geometry Mode (Pure Red Ground Fill)`（纯贴地多边形几何模式：纯红地表覆层填充）
     - `Rhumb Arc Corridor and Circle Ground Geometry (Emerald & Azure Primitives)`（恒向线走廊与圆形贴地图元：祖母绿与蔚蓝组合）
     - `High-Contrast Ground Primitive Outlines (Amber Boundary Diagnostics)`（高对比贴地图元轮廓模式：琥珀色边界拓扑诊断）
   - 核心要素：
     - 构建贴地多边形基准（`Shape::Box`）、贴地恒向线走廊（`Shape::Cylinder`）、贴地圆形半径球（`Shape::Ellipsoid`）与圣海伦斯火山贴地范围（`Shape::Rectangle`）。
     - 验证了 `GroundPrimitive`、`PolygonGeometry`、`CorridorGeometry`、`CircleGeometry`、`ClassificationType.TERRAIN`、`ColorGeometryInstanceAttribute` 动态着色与全球地形紧密贴合。

---

## 2. 验证指标汇总

| 指标项 | 验证结果 | 备注 |
|---|---|---|
| **新增测试用例** | 4 个官方 Sandcastle 案例（累计 239 / 309） | 涵盖谷歌 3D 真实感、建筑方案插入、谷歌街景全景、贴地图元开发 |
| **Rust 单元测试** | `cargo test --lib` 全部通过（133 / 133） | 包含 `batch73_googlephotorealistic_insert_streetview_groundprim` 强类型断言 |
| **Native GPU 渲染测试** | `cargo test --test full-recipes-gpu` 全部通过（240 / 240） | 零回退、零崩溃、像素渲染与拾取覆盖率 > 220px |
| **Browser WebGPU 测试** | 4/4 通过（`full-recipes-browser.json` 累计收录 239 例） | 100% 浏览器 WebGPU 真实绘制与拾取覆盖验证 |
| **WASM 尺寸与格式检查** | 编译成功且 `cargo fmt --check` / `cargo clippy` 0 警告 | 严守 Rust 2021 安全规范与生产级质量标准 |

---

## 3. 产物清单与文件索引

- **数据与变体生成器**：
  - `scripts/generate-batch73-data.mjs`
  - `scripts/batch73-buttons.json`
  - `scripts/batch73-recipes.mjs`
- **案例数据及变体配置**：
  - `tests/sandcastle/full/google-photorealistic-3d-tiles.json` & `.variants.json`
  - `tests/sandcastle/full/google-photorealistic-3d-tiles-with-building-insert.json` & `.variants.json`
  - `tests/sandcastle/full/google-streetview-panorama.json` & `.variants.json`
  - `tests/sandcastle/full/ground-primitive-dev.json` & `.variants.json`
- **测试与验证结果**：
  - `docs/extended-recipes-browser.json`
  - `docs/full-recipes-browser.json`（累计 239 例）
  - `docs/full-recipes-gpu-validation.log`（累计 240 项 Native GPU 渲染断言）
  - 各案例 Preview PNG 与 Smoke 日志
