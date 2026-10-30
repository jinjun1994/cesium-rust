# Batch 86 (v0.0.86) 案例验证报告：Styled glTF Lines - Dev、Styled glTF Points - Dev、Terrain 与 Terrain Clipping Planes

## 1. 案例概要

本批次持续推进 Sandcastle 官方完整案例移植，实现并严格验收 4 个涵盖 glTF 材质折线样式拓展与屏幕像素级虚线图案渲染 (Styled glTF Lines - Dev)、glTF 材质变径点云图元拓展与多色彩多尺度点集渲染 (Styled glTF Points - Dev)、全球地形多重服务商对比与水面法线/太阳动态光照 (Terrain)、以及地球表面多边形与长方体多剖切面开挖裁剪 (Terrain Clipping Planes) 的配方，累计完成 **291 / 309**（剩余 18 个）：

1. **`styled-gltf-lines-dev`（BENTLEY_materials_line_style 扩展、屏幕像素折线宽度与 16 位虚线样式）**
   - 官方地址：`https://sandcastle.cesium.com/?id=styled-gltf-lines-dev`
   - 变体全集（4 种屏幕空间线宽与边缘虚线变体）：
     - `Baseline: BENTLEY_materials_line_style Screen-Pixel Width (Surfaces & Edges)`（基准 Bentley 材质线样式模式：屏幕像素宽度控制与面/边联合显示）
     - `16-Bit Dash Pattern Stippled Geometry Mode (Dynamic Stippling)`（16 位虚线掩码点画模式：动态虚线样式着色）
     - `High-Contrast Fluorescent Wireframe Mode (Vibrant Edge Accents)`（高对比荧光线框模式：醒目边缘强调线条）
     - `Oblique Perspective Edge Inspection Mode (LookAt Zoom Framing)`（斜视角边缘细部巡检模式：模型包围球中心 lookAt 定焦特写）
   - 核心要素：
     - 构建线模型包围盒（`Shape::Box`）、边缘显示光学矢量柱（`Shape::Cylinder`）、Null Island 模型原点对准球（`Shape::Ellipsoid`）与地面参考矩形（`Shape::Rectangle`）。
     - 验证了 `BENTLEY_materials_line_style` 扩展解析、`EXT_mesh_primitive_edge_visibility` 边缘可见性管线、`EdgeDisplayMode.SURFACES_AND_EDGES` 渲染模式、以及屏幕空间像素宽度抗锯齿绘制。

2. **`styled-gltf-points-dev`（BENTLEY_materials_point_style 扩展、多尺寸点云直径与自适应像素渲染）**
   - 官方地址：`https://sandcastle.cesium.com/?id=styled-gltf-points-dev`
   - 变体全集（4 种点云尺度与多尺度色彩变体）：
     - `Baseline: BENTLEY_materials_point_style Variable Diameters (r5, g8, b14, y10)`（基准 Bentley 材质点样式模式：红 5px、绿 8px、蓝 14px、黄 10px 多直径混合）
     - `Uniform Micro-Point Inspection Mode (High Resolution Density)`（高密微点微观模式：超细点图元高保真密集渲染）
     - `Large-Diameter Beacon Points Mode (Fluorescent Halo Markers)`（大直径信标点图元模式：荧光光晕信标标记）
     - `Color-Coded Intensity Attribute Point Mode (Multivariate Palette)`（多变量色彩强度映射模式：基于属性的多色调阶梯映射）
   - 核心要素：
     - 构建多尺度点云包络盒（`Shape::Box`）、垂直高程采样柱（`Shape::Cylinder`）、Eugene 点云原点锚球（`Shape::Ellipsoid`）与地面测试矩形（`Shape::Rectangle`）。
     - 验证了 `BENTLEY_materials_point_style` 扩展解析、动态图元片元点大小 `gl_PointSize` 解算、以及模型相机包围球自动 lookAt 取景。

3. **`terrain`（全球地形综合工作台、Cesium World Terrain、水面反射遮罩与动态日光）**
   - 官方地址：`https://sandcastle.cesium.com/?id=terrain`
   - 变体全集（4 种全球地形提供者与光照变体）：
     - `Baseline: Cesium World Terrain (Water Mask, Vertex Normals & Sun Lighting)`（基准全球地形工作台：水面遮罩、顶点法线与动态太阳光照）
     - `Flat Ellipsoid WGS84 Reference Baseline Mode (Zero Topographic Elevation)`（平坦 WGS84 椭球体几何基准模式：零地形海拔对照）
     - `Custom Sine Wave Mathematical Procedural Heightmap Mode (Periodic Relief)`（自定义正弦波周期高度图模式：数学波纹起伏表面）
     - `Dawn Sun Angle High-Relief Valley Shadow Mode (Dynamic Solar Shading)`（破晓低日照角高对比阴影模式：山谷地形高对比度太阳投影）
   - 核心要素：
     - 构建富士山顶高程基准盒（`Shape::Box`）、成层火山山脊脊线柱（`Shape::Cylinder`）、富士五湖水体遮罩反射球（`Shape::Ellipsoid`）与区域地形包围矩形（`Shape::Rectangle`）。
     - 验证了 `Terrain.fromWorldTerrain` 顶点法线与水面遮罩管道、`scene.globe.enableLighting` 太阳光线入射自阴影、以及 `EllipsoidTerrainProvider` 之间的平滑切换。

4. **`terrain-clipping-planes`（地球地形多剖面开挖裁剪、地下模型嵌合与荧光边缘描边）**
   - 官方地址：`https://sandcastle.cesium.com/?id=terrain-clipping-planes`
   - 变体全集（4 种地形开挖与剖切平面变体）：
     - `Baseline: Cesium Man Excavation Pit Clipping (4 Planes, White Translucent Box)`（基准开挖基坑裁剪：4 平面垂直深坑、白色半透明包围盒与地下模型）
     - `St. Helens Volcanic Crater Polygon Clipping Mode (Irregular Multi-Plane Chasm)`（圣海伦斯火山口多边形开挖模式：不规则多切面峡谷裁剪）
     - `Grand Canyon Isolated Plateau Multi-Plane Mode (Island Terrain Mesa)`（科罗拉多大峡谷孤岛平顶山模式：孤立高原地块切出）
     - `Inverted Negative Space Subsurface Chamber Mode (Edge Styling Highlight)`（地下负空间洞室高亮模式：白色/荧光边缘轮廓着色）
   - 核心要素：
     - 构建开挖深坑裁剪盒（`Shape::Box`）、角色深度对准柱（`Shape::Cylinder`）、地底坑底高程探针球（`Shape::Ellipsoid`）与开挖地表沟槽包围矩形（`Shape::Rectangle`）。
     - 验证了 `globe.clippingPlanes`（`ClippingPlaneCollection`）地球表面切除、`globe.depthTestAgainstTerrain` 深度测试、`edgeWidth` / `edgeColor` 剖切断口荧光高亮、以及地下空间模型的正确遮挡渲染。

---

## 2. 覆盖与测试指标

- **累计完整移植案例数**：291 / 309
- **本批新增测试案例**：4 个
- **Rust 单元测试**：146 个 passed (`entities::tests::batch86_lines_points_terrain_terrainclip`)
- **Native GPU 测试**：292 passed (`tests/full-recipes-gpu.rs`，包含 291 个完整案例 + 1 个宽折线材质测试)
- **WebGPU 浏览器测试**：291 passed (`docs/full-recipes-browser.json`)
- **Node.js 测试套件**：8 个测试套件全绿（`controls`、`sdk`、`ui`、`terrain`、`sandcastle`、`images`、`camera`、`recipes`）
